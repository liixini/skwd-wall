#!/usr/bin/env python3
import argparse
import fcntl
import hashlib
import json
import os
import signal
import socket
import stat
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PREFIX = Path.home() / ".local" / "lib" / "skwd-suite"
MANIFEST_NAME = ".skwd-suite.json"
WALL_NAME = "skwd-wall"
PAPER_NAME = "skwd-paper"
DAEMON_NAME = "skwd-walld"


class LaunchError(RuntimeError):
    pass


def parse_arguments(argv=None):
    configured_prefix = os.environ.get("SKWD_SUITE_PREFIX") or DEFAULT_PREFIX
    parser = argparse.ArgumentParser(
        description="Build, activate, and launch the current local Skwd suite."
    )
    parser.add_argument(
        "--stage-only",
        action="store_true",
        help="activate existing release artifacts without running Cargo",
    )
    parser.add_argument(
        "--deploy-only",
        action="store_true",
        help="build, stage, and activate without opening Wall",
    )
    parser.add_argument(
        "--theme-audition",
        action="store_true",
        help="open the compact colour-profile preview instead of the wallpaper selector",
    )
    parser.add_argument(
        "--prefix",
        type=Path,
        default=Path(configured_prefix),
        help="suite root containing bin (default: ~/.local/lib/skwd-suite)",
    )
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument(
        "--product",
        action="append",
        choices=("wall", "deck", "paper", "lens"),
        dest="products",
        help="refresh one product and preserve the others; repeat as needed (default: wall)",
    )
    selection.add_argument(
        "--all-products",
        action="store_true",
        help="refresh Wall, Deck, Paper, and Lens together",
    )
    return parser.parse_args(argv)


def open_lock(path):
    flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags, 0o600)
    except OSError as error:
        raise LaunchError(f"cannot open launch lock {path}: {error}") from error
    metadata = os.fstat(descriptor)
    if not stat.S_ISREG(metadata.st_mode):
        os.close(descriptor)
        raise LaunchError(f"launch lock is not a regular file: {path}")
    return os.fdopen(descriptor, "a+b")


def validate_prefix(prefix):
    current = Path(prefix.anchor)
    for component in prefix.parts[1:]:
        current /= component
        if current.exists() or current.is_symlink():
            metadata = current.lstat()
            if current.is_symlink():
                raise LaunchError(f"suite path has a symlinked ancestor: {current}")
            if current == prefix and not stat.S_ISDIR(metadata.st_mode):
                raise LaunchError(f"suite prefix is not a regular directory: {prefix}")
    if not prefix.exists():
        prefix.mkdir(parents=True, mode=0o755)


def read_manifest(bin_dir):
    path = bin_dir / MANIFEST_NAME
    try:
        payload = path.read_bytes()
        manifest = json.loads(payload)
    except (OSError, json.JSONDecodeError) as error:
        raise LaunchError(f"cannot read suite manifest {path}: {error}") from error
    if manifest.get("format") != 1 or not isinstance(manifest.get("artifacts"), list):
        raise LaunchError(f"unsupported suite manifest: {path}")
    products = {}
    for artifact in manifest["artifacts"]:
        name = artifact.get("name")
        product = artifact.get("product")
        if not isinstance(name, str) or not isinstance(product, str) or name in products:
            raise LaunchError(f"invalid artifact entry in {path}")
        products[name] = product
    return products


def hash_file(path):
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise LaunchError(f"cannot open staged artifact {path}: {error}") from error
    metadata = os.fstat(descriptor)
    if not stat.S_ISREG(metadata.st_mode):
        os.close(descriptor)
        raise LaunchError(f"staged artifact is not a regular file: {path}")
    digest = hashlib.sha256()
    try:
        with os.fdopen(descriptor, "rb", closefd=False) as reader:
            while chunk := reader.read(1024 * 1024):
                digest.update(chunk)
    finally:
        os.close(descriptor)
    return digest.hexdigest()


def suite_hashes(bin_dir, products):
    return {name: hash_file(bin_dir / name) for name in products}


def changed_artifacts(before, after):
    return {name for name, digest in after.items() if before.get(name) != digest} | {
        name for name in before if name not in after
    }


def activation_plan(changed, daemon_current):
    replaced = bool(changed) or not daemon_current
    return {
        "close_wall": replaced,
        "stop_paper": replaced,
        "restart_daemon": replaced,
    }


def owned_processes(executable):
    try:
        expected = executable.stat()
    except OSError:
        return []
    found = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        try:
            actual = (entry / "exe").stat()
        except OSError:
            continue
        if actual.st_dev == expected.st_dev and actual.st_ino == expected.st_ino:
            found.append(int(entry.name))
    return sorted(found)


def invoked_processes(executable, proc_root=Path("/proc")):
    expected = os.fsencode(executable)
    found = []
    for entry in proc_root.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            invoked = (entry / "cmdline").read_bytes().split(b"\0", 1)[0]
        except OSError:
            continue
        if invoked == expected:
            found.append(int(entry.name))
    return sorted(found)


def suite_processes(executable):
    return sorted(set(owned_processes(executable)) | set(invoked_processes(executable)))


def process_alive(pid):
    try:
        state = Path(f"/proc/{pid}/stat").read_text()
    except OSError:
        return False
    tail = state[state.rfind(")") + 2 :].split()
    return bool(tail) and tail[0] != "Z"


def wait_processes(pids, timeout):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        remaining = [pid for pid in pids if process_alive(pid)]
        if not remaining:
            return []
        time.sleep(0.05)
    return [pid for pid in pids if process_alive(pid)]


def terminate_remaining(pids):
    for pid in pids:
        try:
            os.kill(pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    remaining = wait_processes(pids, 3.0)
    if remaining:
        raise LaunchError(f"updated suite processes did not exit after SIGTERM: {remaining}")


def stop_wall(bin_dir, pids):
    if not pids:
        return
    subprocess.run(
        [str(bin_dir / "skwd-helm"), "ui", "hide"],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=5,
    )
    terminate_remaining(wait_processes(pids, 3.0))


def stop_paper(bin_dir, pids):
    if not pids:
        return
    subprocess.run(
        [str(bin_dir / PAPER_NAME), "stop", "--all"],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=5,
    )
    terminate_remaining(wait_processes(pids, 3.0))


def daemon_socket_ready():
    configured = os.environ.get("SKWD_WALL_V2_SOCK")
    runtime = os.environ.get("XDG_RUNTIME_DIR")
    default = (
        Path(runtime) / "skwd-wall-v2" / "wall.sock"
        if runtime
        else Path("/tmp/skwd-wall-v2/wall.sock")
    )
    path = Path(configured) if configured else default
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        client.settimeout(0.2)
        client.connect(str(path))
        return True
    except OSError:
        return False
    finally:
        client.close()


def service_property(name):
    return subprocess.run(
        ["systemctl", "--user", "show", "skwd-walld.service", "-p", name, "--value"],
        check=True,
        capture_output=True,
        text=True,
        timeout=10,
    ).stdout.strip()


def daemon_is_current(bin_dir):
    try:
        pid = int(service_property("MainPID"))
        active = service_property("ActiveState")
        configured = service_property("ExecStart")
    except (OSError, ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return False
    expected = str(bin_dir / DAEMON_NAME)
    return (
        active == "active"
        and pid > 0
        and f"path={expected} " in configured
        and process_alive(pid)
        and pid in owned_processes(bin_dir / DAEMON_NAME)
        and daemon_socket_ready()
    )


def restart_daemon(bin_dir, previous_pids):
    subprocess.run(
        ["systemctl", "--user", "stop", "skwd-walld.service"],
        check=True,
        timeout=30,
    )
    terminate_remaining(wait_processes(previous_pids, 3.0))
    subprocess.run(
        ["systemctl", "--user", "start", "skwd-walld.service"],
        check=True,
        timeout=30,
    )

    deadline = time.monotonic() + 5.0
    while time.monotonic() < deadline:
        if daemon_is_current(bin_dir):
            return
        time.sleep(0.05)
    raise LaunchError("staged skwd-walld did not retain the daemon socket")


def selected_products(arguments):
    if arguments.all_products:
        return ()
    return tuple(dict.fromkeys(arguments.products or ("wall",)))


def run_stager(prefix, stage_only, products):
    command = [
        sys.executable,
        str(ROOT / "scripts/stage-local-suite.py"),
        "--if-changed",
        "--prefix",
        str(prefix),
    ]
    if stage_only:
        command.append("--stage-only")
    for product in products:
        command.extend(("--product", product))
    subprocess.run(command, check=True)


def main(argv=None):
    arguments = parse_arguments(argv)
    selected = selected_products(arguments)
    prefix = Path(os.path.abspath(os.fspath(arguments.prefix.expanduser())))
    if prefix == Path("/") or prefix == Path.home():
        print(f"launch-local-suite: refusing unsafe suite prefix: {prefix}", file=sys.stderr)
        return 1
    try:
        validate_prefix(prefix)
        with open_lock(prefix / ".launch.lock") as lock:
            try:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise LaunchError("another suite build or launch is already in progress") from error
            bin_dir = prefix / "bin"
            before_products = read_manifest(bin_dir) if bin_dir.is_dir() else {}
            before_hashes = suite_hashes(bin_dir, before_products) if before_products else {}
            wall_pids = suite_processes(bin_dir / WALL_NAME)
            paper_pids = suite_processes(bin_dir / PAPER_NAME)
            daemon_pids = suite_processes(bin_dir / DAEMON_NAME)
            run_stager(prefix, arguments.stage_only, selected)
            products = read_manifest(bin_dir)
            after_hashes = suite_hashes(bin_dir, products)
            changed = changed_artifacts(before_hashes, after_hashes)
            plan = activation_plan(changed, daemon_is_current(bin_dir))
            if plan["close_wall"]:
                stop_wall(bin_dir, wall_pids)
            if plan["stop_paper"]:
                stop_paper(bin_dir, paper_pids)
            if plan["restart_daemon"]:
                restart_daemon(bin_dir, daemon_pids)
            if arguments.deploy_only:
                print(
                    "local suite is current"
                    if not changed
                    else f"activated updated suite artifacts: {', '.join(sorted(changed))}"
                )
                return 0
            environment = os.environ.copy()
            if arguments.theme_audition:
                environment["SKWD_WALL_START"] = "theme-audition"
            os.execve(bin_dir / WALL_NAME, [WALL_NAME], environment)
    except (LaunchError, OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        print(f"launch-local-suite: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
