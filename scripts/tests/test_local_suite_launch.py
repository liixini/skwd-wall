import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]


def load_launch_module():
    module_path = ROOT / "scripts/launch-local-suite.py"
    spec = importlib.util.spec_from_file_location("launch_local_suite", module_path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class LocalSuiteLaunchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.launch = load_launch_module()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def write_suite(self, prefix, payloads):
        bin_dir = prefix / "bin"
        bin_dir.mkdir(parents=True, exist_ok=True)
        artifacts = []
        for name, (product, payload) in payloads.items():
            path = bin_dir / name
            path.write_bytes(payload)
            path.chmod(0o755)
            artifacts.append({"name": name, "product": product})
        (bin_dir / ".skwd-suite.json").write_text(
            json.dumps({"format": 1, "artifacts": artifacts}), encoding="utf-8"
        )
        return bin_dir

    def test_any_artifact_replacement_recycles_the_suite_processes(self):
        before = {"skwd-wall": "old", "skwd-walld": "same", "skwd-lens": "old"}
        after = {"skwd-wall": "new", "skwd-walld": "same", "skwd-lens": "new"}
        changed = self.launch.changed_artifacts(before, after)
        plan = self.launch.activation_plan(changed, daemon_current=True)
        self.assertEqual(changed, {"skwd-wall", "skwd-lens"})
        self.assertEqual(
            plan, {"close_wall": True, "stop_paper": True, "restart_daemon": True}
        )

    def test_paper_or_deck_replacement_recycles_the_suite_processes(self):
        self.assertEqual(
            self.launch.activation_plan({"skwd-wall-vk"}, daemon_current=True),
            {"close_wall": True, "stop_paper": True, "restart_daemon": True},
        )
        self.assertEqual(
            self.launch.activation_plan({"skwd-helm"}, daemon_current=True),
            {"close_wall": True, "stop_paper": True, "restart_daemon": True},
        )

    def test_no_artifact_replacement_is_a_process_noop(self):
        self.assertEqual(
            self.launch.activation_plan(set(), daemon_current=True),
            {"close_wall": False, "stop_paper": False, "restart_daemon": False},
        )

    def test_stale_daemon_recycles_an_identical_suite(self):
        self.assertEqual(
            self.launch.activation_plan(set(), daemon_current=False),
            {"close_wall": True, "stop_paper": True, "restart_daemon": True},
        )

    def test_invoked_processes_find_a_deleted_suite_executable_by_argv_zero(self):
        proc = self.root / "proc"
        (proc / "41").mkdir(parents=True)
        (proc / "42").mkdir()
        executable = self.root / "suite" / "bin" / "skwd-walld"
        (proc / "41" / "cmdline").write_bytes(os.fsencode(executable) + b"\0--wait\0")
        (proc / "42" / "cmdline").write_bytes(b"/usr/bin/other\0")
        self.assertEqual(self.launch.invoked_processes(executable, proc), [41])

    def test_theme_audition_launch_mode_is_explicit(self):
        arguments = self.launch.parse_arguments(["--stage-only", "--theme-audition"])
        self.assertTrue(arguments.stage_only)
        self.assertTrue(arguments.theme_audition)

    def test_launcher_refreshes_only_wall_by_default(self):
        arguments = self.launch.parse_arguments(["--deploy-only"])
        self.assertEqual(self.launch.selected_products(arguments), ("wall",))

    def test_launcher_requires_explicit_all_products_selection(self):
        arguments = self.launch.parse_arguments(["--deploy-only", "--all-products"])
        self.assertEqual(self.launch.selected_products(arguments), ())

    def test_launcher_passes_selected_products_to_stager(self):
        prefix = self.root / "suite"
        with mock.patch.object(self.launch.subprocess, "run") as run:
            self.launch.run_stager(prefix, True, ("wall", "paper"))
        command = run.call_args.args[0]
        self.assertIn("--stage-only", command)
        self.assertEqual(command[-4:], ["--product", "wall", "--product", "paper"])

    def test_daemon_socket_probe_passes_a_string_path_to_the_socket(self):
        runtime = self.root / "runtime"
        client = mock.Mock()
        with mock.patch.dict(
            self.launch.os.environ,
            {"XDG_RUNTIME_DIR": str(runtime)},
            clear=True,
        ), mock.patch.object(self.launch.socket, "socket", return_value=client):
            self.assertTrue(self.launch.daemon_socket_ready())
            client.connect.assert_called_once_with(
                str(runtime / "skwd-wall-v2" / "wall.sock")
            )
        client.close.assert_called_once_with()

    def test_current_daemon_requires_service_process_binary_and_socket(self):
        bin_dir = self.root / "bin"
        expected = str(bin_dir / "skwd-walld")
        properties = ["41", "active", f"{{ path={expected} ; argv[]={expected} ; }}"]
        with mock.patch.object(
            self.launch,
            "service_property",
            side_effect=properties,
        ), mock.patch.object(
            self.launch,
            "process_alive",
            return_value=True,
        ), mock.patch.object(
            self.launch,
            "owned_processes",
            return_value=[41],
        ), mock.patch.object(
            self.launch,
            "daemon_socket_ready",
            return_value=True,
        ):
            self.assertTrue(self.launch.daemon_is_current(bin_dir))

    def test_deploy_only_activates_a_changed_wall_without_launching(self):
        prefix = self.root / "suite"
        bin_dir = self.write_suite(
            prefix,
            {
                "skwd-wall": ("wall", b"old-wall"),
                "skwd-walld": ("deck", b"daemon"),
            },
        )

        def stage(_prefix, stage_only, products):
            self.assertTrue(stage_only)
            self.assertEqual(products, ("wall",))
            (bin_dir / "skwd-wall").write_bytes(b"new-wall")

        def owned(path):
            return {"skwd-wall": [77], "skwd-paper": [88], "skwd-walld": [99]}[path.name]

        with mock.patch.object(self.launch, "run_stager", side_effect=stage), mock.patch.object(
            self.launch, "suite_processes", side_effect=owned
        ), mock.patch.object(
            self.launch, "daemon_is_current", return_value=False
        ), mock.patch.object(self.launch, "stop_wall") as stop_wall, mock.patch.object(
            self.launch, "stop_paper"
        ) as stop_paper, mock.patch.object(self.launch, "restart_daemon") as restart_daemon:
            result = self.launch.main(
                ["--stage-only", "--deploy-only", "--prefix", str(prefix)]
            )

        self.assertEqual(result, 0)
        stop_wall.assert_called_once_with(bin_dir, [77])
        stop_paper.assert_called_once_with(bin_dir, [88])
        restart_daemon.assert_called_once_with(bin_dir, [99])

    def test_identical_deploy_is_a_process_noop(self):
        prefix = self.root / "suite"
        self.write_suite(
            prefix,
            {
                "skwd-wall": ("wall", b"wall"),
                "skwd-walld": ("deck", b"daemon"),
            },
        )
        with mock.patch.object(self.launch, "run_stager"), mock.patch.object(
            self.launch, "suite_processes", return_value=[]
        ), mock.patch.object(
            self.launch, "daemon_is_current", return_value=True
        ), mock.patch.object(self.launch, "stop_wall") as stop_wall, mock.patch.object(
            self.launch, "stop_paper"
        ) as stop_paper, mock.patch.object(self.launch, "restart_daemon") as restart_daemon:
            result = self.launch.main(
                ["--stage-only", "--deploy-only", "--prefix", str(prefix)]
            )
        self.assertEqual(result, 0)
        stop_wall.assert_not_called()
        stop_paper.assert_not_called()
        restart_daemon.assert_not_called()

    def test_symlinked_prefix_is_rejected_before_lock_creation(self):
        actual = self.root / "actual"
        actual.mkdir()
        linked = self.root / "linked"
        linked.symlink_to(actual, target_is_directory=True)
        result = self.launch.main(["--deploy-only", "--prefix", str(linked / "suite")])
        self.assertEqual(result, 1)
        self.assertEqual(list(actual.iterdir()), [])

    def test_daemon_restart_verifies_systemd_owned_state(self):
        bin_dir = self.root / "bin"
        bin_dir.mkdir()
        responses = [mock.Mock(stdout=""), mock.Mock(stdout="")]
        with mock.patch.object(
            self.launch.subprocess, "run", side_effect=responses
        ) as run, mock.patch.object(
            self.launch, "daemon_is_current", return_value=True
        ):
            self.launch.restart_daemon(bin_dir, [])
        self.assertEqual(run.call_count, 2)


if __name__ == "__main__":
    unittest.main()
