#!/usr/bin/env python3
import argparse
import ctypes
import errno
import fcntl
import hashlib
import json
import os
import stat
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PREFIX = Path.home() / ".local" / "lib" / "skwd-suite"
PRODUCT_BINARIES = {
    "wall": ("skwd-wall",),
    "deck": (
        "skwd-walld",
        "skwd-helm",
        "skwd-wall-scan",
        "skwd-wall-effects",
        "skwd-steam",
        "libsteam_api.so",
    ),
    "paper": ("skwd-paper", "skwd-paper-tinier", "skwd-wall-vk", "skwd-wall-still"),
    "lens": ("skwd-lens",),
}
EXECUTABLES = frozenset(
    name for names in PRODUCT_BINARIES.values() for name in names if name != "libsteam_api.so"
)
ARTIFACT_NAMES = frozenset(name for names in PRODUCT_BINARIES.values() for name in names)
LEGACY_ARTIFACT_NAMES = frozenset({"skwd-lens-tagger"})
MIGRATION_ARTIFACT_SETS = frozenset({
    ARTIFACT_NAMES - {"skwd-paper-tinier"},
    ARTIFACT_NAMES - {"skwd-paper", "skwd-paper-tinier"},
    ARTIFACT_NAMES | LEGACY_ARTIFACT_NAMES,
    (ARTIFACT_NAMES - {"skwd-paper-tinier"}) | LEGACY_ARTIFACT_NAMES,
    (ARTIFACT_NAMES - {"skwd-paper", "skwd-paper-tinier"}) | LEGACY_ARTIFACT_NAMES,
})
MANIFEST_NAME = ".skwd-suite.json"
PUBLISHED_NAMES = ARTIFACT_NAMES | LEGACY_ARTIFACT_NAMES | {MANIFEST_NAME}
BUILD_ORDER = ("paper", "lens", "deck", "wall")


class StageError(RuntimeError):
    pass


@dataclass(frozen=True)
class Product:
    name: str
    root: Path
    bin_dir: Path


def parse_arguments(argv=None):
    configured_prefix = os.environ.get("SKWD_SUITE_PREFIX") or DEFAULT_PREFIX
    parser = argparse.ArgumentParser(
        description="Build and atomically stage the local Skwd suite runtime."
    )
    parser.add_argument(
        "--stage-only",
        action="store_true",
        help="publish existing release artifacts without running Cargo",
    )
    parser.add_argument(
        "--if-changed",
        action="store_true",
        help="keep the published directory when every artifact is byte-identical",
    )
    parser.add_argument(
        "--prefix",
        type=Path,
        default=Path(configured_prefix),
        help="suite root containing bin (default: ~/.local/lib/skwd-suite)",
    )
    parser.add_argument(
        "--product",
        action="append",
        choices=PRODUCT_BINARIES,
        dest="products",
        help="refresh one product and preserve the others from the installed suite; repeat as needed",
    )
    return parser.parse_args(argv)


def product_roots():
    parent = ROOT.parent
    roots = {
        "wall": ROOT,
        "deck": parent / "skwd-deck",
        "paper": parent / "skwd-paper",
        "lens": parent / "skwd-lens",
    }
    products = {}
    for name, root in roots.items():
        override = os.environ.get(f"SKWD_{name.upper()}_BIN_DIR")
        bin_dir = Path(override).expanduser() if override else root / "target" / "release"
        products[name] = Product(name=name, root=root, bin_dir=bin_dir)
    return products


def validate_products(products, selected_products=None):
    if set(products) != set(PRODUCT_BINARIES):
        raise StageError("product set does not match the suite manifest")
    selected = set(selected_products or PRODUCT_BINARIES)
    for name, product in products.items():
        if product.name != name:
            raise StageError(f"product key/name mismatch: {name}/{product.name}")
        if name not in selected:
            continue
        manifest = product.root / "Cargo.toml"
        if not manifest.is_file() or manifest.is_symlink():
            raise StageError(f"missing regular Cargo manifest for {name}: {manifest}")


def build_suite(products, selected_products=None):
    selected = set(selected_products or BUILD_ORDER)
    for name in BUILD_ORDER:
        if name not in selected:
            continue
        product = products[name]
        command = [
            "cargo",
            "build",
            "--locked",
            "--release",
            "--manifest-path",
            str(product.root / "Cargo.toml"),
            "--target-dir",
            str(product.root / "target"),
        ]
        if name != "wall":
            command.append("--workspace")
        else:
            command.extend(("--package", "skwd-wall", "--bin", "skwd-wall"))
        print(f"== building {name} ==", flush=True)
        subprocess.run(command, check=True)


def git_state(root):
    try:
        revision = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        dirty = bool(
            subprocess.run(
                ["git", "status", "--porcelain"],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
        )
    except (OSError, subprocess.CalledProcessError):
        return {"revision": None, "dirty": None}
    return {"revision": revision, "dirty": dirty}


def open_source(path, executable):
    if path.name not in EXECUTABLES and path.name != "libsteam_api.so":
        raise StageError(f"unexpected suite artifact name: {path.name}")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
        raise StageError(f"cannot open regular source artifact {path}: {error}") from error
    metadata = os.fstat(descriptor)
    if not stat.S_ISREG(metadata.st_mode):
        os.close(descriptor)
        raise StageError(f"source artifact is not a regular file: {path}")
    if executable and metadata.st_mode & 0o111 == 0:
        os.close(descriptor)
        raise StageError(f"source artifact is not executable: {path}")
    return descriptor, metadata


def copy_artifact(source, destination, executable):
    source_descriptor, source_metadata = open_source(source, executable)
    digest = hashlib.sha256()
    destination_descriptor = None
    try:
        destination_descriptor = os.open(
            destination,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_CLOEXEC", 0),
            0o755,
        )
        with os.fdopen(source_descriptor, "rb", closefd=False) as reader:
            while chunk := reader.read(1024 * 1024):
                digest.update(chunk)
                view = memoryview(chunk)
                while view:
                    written = os.write(destination_descriptor, view)
                    if written == 0:
                        raise OSError(errno.EIO, "short artifact write")
                    view = view[written:]
        os.fchmod(destination_descriptor, 0o755)
        os.fsync(destination_descriptor)
    except OSError as error:
        raise StageError(f"cannot copy {source} to {destination}: {error}") from error
    finally:
        os.close(source_descriptor)
        if destination_descriptor is not None:
            os.close(destination_descriptor)

    installed = destination.lstat()
    if destination.is_symlink() or not stat.S_ISREG(installed.st_mode):
        raise StageError(f"staged artifact is not a regular copy: {destination}")
    if stat.S_IMODE(installed.st_mode) != 0o755:
        raise StageError(f"staged artifact has mode {stat.S_IMODE(installed.st_mode):04o}: {destination}")
    if installed.st_size != source_metadata.st_size:
        raise StageError(f"staged artifact size changed while copying: {source}")
    if installed.st_ino == source_metadata.st_ino and installed.st_dev == source_metadata.st_dev:
        raise StageError(f"staged artifact aliases its source instead of copying it: {destination}")
    return {
        "name": source.name,
        "product": None,
        "sha256": digest.hexdigest(),
        "size": installed.st_size,
        "source": str(source),
        "source_mode": f"{stat.S_IMODE(source_metadata.st_mode):04o}",
        "installed_mode": "0755",
        "executable": executable,
    }


def fsync_directory(path):
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def exchange_directories(first, second):
    library = ctypes.CDLL(None, use_errno=True)
    try:
        renameat2 = library.renameat2
    except AttributeError as error:
        raise StageError("atomic directory exchange is unavailable on this system") from error
    renameat2.argtypes = (
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    )
    renameat2.restype = ctypes.c_int
    result = renameat2(-100, os.fsencode(first), -100, os.fsencode(second), 2)
    if result != 0:
        error_number = ctypes.get_errno()
        raise StageError(
            f"atomic directory exchange failed: {os.strerror(error_number)}"
        ) from OSError(error_number, os.strerror(error_number))


def validate_prefix(prefix):
    prefix = Path(os.path.abspath(os.fspath(prefix.expanduser())))
    if prefix == Path("/") or prefix == Path.home():
        raise StageError(f"refusing unsafe suite prefix: {prefix}")
    current = Path(prefix.anchor)
    for component in prefix.parts[1:]:
        current /= component
        if current.exists() or current.is_symlink():
            if current.is_symlink():
                raise StageError(f"suite path has a symlinked ancestor: {current}")
    if prefix.exists() or prefix.is_symlink():
        metadata = prefix.lstat()
        if prefix.is_symlink() or not stat.S_ISDIR(metadata.st_mode):
            raise StageError(f"suite prefix is not a regular directory: {prefix}")
    else:
        prefix.mkdir(parents=True, mode=0o755)
    destination = prefix / "bin"
    if destination.exists() or destination.is_symlink():
        metadata = destination.lstat()
        if destination.is_symlink() or not stat.S_ISDIR(metadata.st_mode):
            raise StageError(f"suite bin is not a regular directory: {destination}")
    return prefix, destination


def validate_suite_directory(
    directory, require_complete, require_manifest, allow_legacy=False
):
    if not directory.exists() and not directory.is_symlink():
        raise StageError(f"suite directory does not exist: {directory}")
    metadata = directory.lstat()
    if directory.is_symlink() or not stat.S_ISDIR(metadata.st_mode):
        raise StageError(f"suite directory is not a regular directory: {directory}")
    entries = list(directory.iterdir())
    names = {entry.name for entry in entries}
    unexpected = names - PUBLISHED_NAMES
    if unexpected:
        listed = ", ".join(sorted(unexpected))
        raise StageError(f"suite directory contains unexpected entries: {listed}")
    if require_complete:
        present_artifacts = names - {MANIFEST_NAME}
        legacy_complete = allow_legacy and present_artifacts in MIGRATION_ARTIFACT_SETS
        if present_artifacts != ARTIFACT_NAMES and not legacy_complete:
            missing = ARTIFACT_NAMES - names
            listed = ", ".join(sorted(missing))
            raise StageError(f"suite directory is missing artifacts: {listed}")
    if require_manifest and MANIFEST_NAME not in names:
        raise StageError(f"suite directory is missing {MANIFEST_NAME}")
    for entry in entries:
        entry_metadata = entry.lstat()
        if entry.is_symlink():
            raise StageError(f"suite directory contains a symlink: {entry}")
        if not stat.S_ISREG(entry_metadata.st_mode):
            raise StageError(f"suite directory contains a non-file entry: {entry}")
    return entries


def remove_validated_stage(stage, prefix, require_complete, allow_legacy=False):
    if stage.parent != prefix or not stage.name.startswith(".bin.stage-"):
        raise StageError(f"refusing to remove unexpected stage path: {stage}")
    if not stage.exists() and not stage.is_symlink():
        return
    entries = validate_suite_directory(stage, require_complete, False, allow_legacy)
    for entry in entries:
        entry_metadata = entry.lstat()
        if entry.is_symlink() or not stat.S_ISREG(entry_metadata.st_mode):
            raise StageError(f"refusing to unlink changed stage entry: {entry}")
        entry.unlink()
    stage.rmdir()


def open_stage_lock(lock_path):
    flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(lock_path, flags, 0o600)
    except OSError as error:
        raise StageError(f"cannot open regular stage lock {lock_path}: {error}") from error
    metadata = os.fstat(descriptor)
    if not stat.S_ISREG(metadata.st_mode):
        os.close(descriptor)
        raise StageError(f"stage lock is not a regular file: {lock_path}")
    return os.fdopen(descriptor, "a+b")


def artifact_equal(first, second, executable):
    first_descriptor, first_metadata = open_source(first, executable)
    second_descriptor, second_metadata = open_source(second, executable)
    try:
        if first_metadata.st_size != second_metadata.st_size:
            return False
        with os.fdopen(first_descriptor, "rb", closefd=False) as first_reader:
            with os.fdopen(second_descriptor, "rb", closefd=False) as second_reader:
                while True:
                    first_chunk = first_reader.read(1024 * 1024)
                    second_chunk = second_reader.read(1024 * 1024)
                    if first_chunk != second_chunk:
                        return False
                    if not first_chunk:
                        break
    finally:
        os.close(first_descriptor)
        os.close(second_descriptor)
    return True


def artifacts_equal(first, second):
    return all(
        artifact_equal(first / name, second / name, name in EXECUTABLES)
        for name in ARTIFACT_NAMES
    )


def sources_equal_destination(products, destination, selected_products=None):
    selected = set(selected_products or PRODUCT_BINARIES)
    names = {entry.name for entry in destination.iterdir()} - {MANIFEST_NAME}
    return names == ARTIFACT_NAMES and all(
        artifact_equal(
            products[product_name].bin_dir / name,
            destination / name,
            name in EXECUTABLES,
        )
        for product_name, names in PRODUCT_BINARIES.items()
        if product_name in selected
        for name in names
    )


def read_installed_manifest(destination):
    path = destination / MANIFEST_NAME
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise StageError(f"cannot read installed suite manifest {path}: {error}") from error
    if manifest.get("format") != 1:
        raise StageError(f"unsupported installed suite manifest: {path}")
    repositories = manifest.get("repositories")
    artifacts = manifest.get("artifacts")
    if not isinstance(repositories, dict) or not isinstance(artifacts, list):
        raise StageError(f"invalid installed suite manifest: {path}")
    by_name = {}
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            raise StageError(f"invalid artifact entry in installed suite manifest: {path}")
        name = artifact.get("name")
        product = artifact.get("product")
        if (
            name not in ARTIFACT_NAMES
            or product not in PRODUCT_BINARIES
            or name not in PRODUCT_BINARIES[product]
            or name in by_name
        ):
            raise StageError(f"invalid artifact entry in installed suite manifest: {path}")
        by_name[name] = artifact
    if set(by_name) != ARTIFACT_NAMES or set(repositories) != set(PRODUCT_BINARIES):
        raise StageError(f"incomplete installed suite manifest: {path}")
    return repositories, by_name


def preserved_artifact(source, destination, previous):
    artifact = copy_artifact(source, destination, source.name in EXECUTABLES)
    if artifact["sha256"] != previous.get("sha256"):
        raise StageError(f"installed artifact does not match its manifest: {source}")
    for field in ("source", "source_mode"):
        if field in previous:
            artifact[field] = previous[field]
    return artifact


def publish_suite(
    products, prefix, build_mode, if_changed=False, selected_products=None
):
    selected = set(selected_products or PRODUCT_BINARIES)
    validate_products(products, selected)
    prefix, destination = validate_prefix(prefix)
    partial = selected != set(PRODUCT_BINARIES)
    lock_path = prefix / ".stage.lock"
    with open_stage_lock(lock_path) as lock:
        try:
            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise StageError(f"another suite refresh holds {lock_path}") from error
        previous_repositories = None
        previous_artifacts = None
        if destination.exists() or destination.is_symlink():
            validate_suite_directory(destination, True, partial, not partial)
            if partial:
                previous_repositories, previous_artifacts = read_installed_manifest(destination)
            if (
                if_changed
                and not partial
                and sources_equal_destination(products, destination, selected)
            ):
                return destination
        elif partial:
            raise StageError("product-scoped refresh requires an installed complete suite")

        stage = Path(tempfile.mkdtemp(prefix=".bin.stage-", dir=prefix))
        stage_safe_to_remove = True
        stage_requires_complete = False
        stage_allows_legacy = False
        try:
            artifacts = []
            repositories = {}
            for product_name in ("wall", "deck", "paper", "lens"):
                product = products[product_name]
                if product_name in selected:
                    repositories[product_name] = {
                        "root": str(product.root),
                        **git_state(product.root),
                    }
                else:
                    repositories[product_name] = previous_repositories[product_name]
                for name in PRODUCT_BINARIES[product_name]:
                    if product_name in selected:
                        source = product.bin_dir / name
                        artifact = copy_artifact(source, stage / name, name in EXECUTABLES)
                    else:
                        source = destination / name
                        artifact = preserved_artifact(
                            source, stage / name, previous_artifacts[name]
                        )
                    artifact["product"] = product_name
                    artifacts.append(artifact)

            manifest = {
                "format": 1,
                "generated_at": datetime.now(timezone.utc).isoformat(),
                "build_mode": (
                    build_mode
                    if not partial
                    else f"{build_mode}:{','.join(sorted(selected))}"
                ),
                "repositories": repositories,
                "artifacts": artifacts,
            }
            manifest_path = stage / MANIFEST_NAME
            descriptor = os.open(
                manifest_path,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_CLOEXEC", 0),
                0o644,
            )
            try:
                payload = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()
                view = memoryview(payload)
                while view:
                    written = os.write(descriptor, view)
                    if written == 0:
                        raise OSError(errno.EIO, "short manifest write")
                    view = view[written:]
                os.fchmod(descriptor, 0o644)
                os.fsync(descriptor)
            finally:
                os.close(descriptor)

            fsync_directory(stage)
            validate_suite_directory(stage, True, True)
            stage_requires_complete = True
            if destination.exists() or destination.is_symlink():
                validate_suite_directory(destination, True, False, True)
                if if_changed and artifacts_equal(stage, destination):
                    return destination
                exchange_directories(stage, destination)
                stage_safe_to_remove = False
                try:
                    validate_suite_directory(stage, True, False, True)
                except StageError:
                    try:
                        exchange_directories(stage, destination)
                    except StageError as rollback_error:
                        raise StageError(
                            f"old suite changed during exchange; retained at {stage}; "
                            f"rollback failed: {rollback_error}"
                        ) from rollback_error
                    stage_safe_to_remove = True
                    stage_requires_complete = True
                    raise
                stage_safe_to_remove = True
                stage_requires_complete = True
                stage_allows_legacy = True
            else:
                os.rename(stage, destination)
                stage_safe_to_remove = False
            fsync_directory(prefix)
        finally:
            if stage_safe_to_remove:
                remove_validated_stage(
                    stage, prefix, stage_requires_complete, stage_allows_legacy
                )
    return destination


def main(argv=None):
    arguments = parse_arguments(argv)
    products = product_roots()
    selected = tuple(dict.fromkeys(arguments.products or PRODUCT_BINARIES))
    try:
        validate_products(products, selected)
        if not arguments.stage_only:
            build_suite(products, selected)
        mode = "stage-only" if arguments.stage_only else "built"
        destination = publish_suite(
            products,
            arguments.prefix,
            mode,
            arguments.if_changed,
            selected,
        )
    except (StageError, OSError, subprocess.CalledProcessError) as error:
        print(f"stage-local-suite: {error}", file=sys.stderr)
        return 1
    print(f"staged Skwd suite at {destination}")
    print(f"manifest: {destination / '.skwd-suite.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
