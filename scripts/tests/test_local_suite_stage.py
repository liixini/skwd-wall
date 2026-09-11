import importlib.util
import json
import stat
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[2]


def load_stage_module():
    module_path = ROOT / "scripts/stage-local-suite.py"
    spec = importlib.util.spec_from_file_location("stage_local_suite", module_path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class LocalSuiteStageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.stage = load_stage_module()

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.products = {}
        for product_name, names in self.stage.PRODUCT_BINARIES.items():
            product_root = self.root / product_name
            bin_dir = product_root / "target/release"
            bin_dir.mkdir(parents=True)
            (product_root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
            for name in names:
                path = bin_dir / name
                path.write_bytes(f"{product_name}:{name}:one\n".encode())
                path.chmod(0o755 if name in self.stage.EXECUTABLES else 0o644)
            self.products[product_name] = self.stage.Product(
                name=product_name,
                root=product_root,
                bin_dir=bin_dir,
            )
        self.prefix = self.root / "prefix"

    def tearDown(self):
        self.temporary.cleanup()

    def test_stages_regular_copies_with_exact_names_and_modes(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        expected = {
            name for names in self.stage.PRODUCT_BINARIES.values() for name in names
        }
        actual = {path.name for path in destination.iterdir() if not path.name.startswith(".")}
        self.assertEqual(actual, expected)
        for name in expected:
            installed = destination / name
            self.assertFalse(installed.is_symlink())
            self.assertTrue(installed.is_file())
            self.assertEqual(stat.S_IMODE(installed.stat().st_mode), 0o755)
        manifest = json.loads((destination / ".skwd-suite.json").read_text())
        self.assertEqual(manifest["format"], 1)
        self.assertEqual(manifest["build_mode"], "stage-only")
        self.assertEqual({item["name"] for item in manifest["artifacts"]}, expected)

    def test_paper_stages_one_public_cli_private_tinier_and_compatibility_workers(self):
        self.assertEqual(
            self.stage.PRODUCT_BINARIES["paper"],
            ("skwd-paper", "skwd-paper-tinier", "skwd-wall-vk", "skwd-wall-still"),
        )
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        manifest = json.loads((destination / ".skwd-suite.json").read_text())
        paper_artifacts = {
            item["name"] for item in manifest["artifacts"] if item["product"] == "paper"
        }
        self.assertEqual(paper_artifacts, set(self.stage.PRODUCT_BINARIES["paper"]))

    def test_builds_in_dependency_order_into_each_release_directory(self):
        with mock.patch.object(self.stage.subprocess, "run") as run, mock.patch("builtins.print"):
            self.stage.build_suite(self.products)
        commands = [call.args[0] for call in run.call_args_list]
        manifests = [
            Path(command[command.index("--manifest-path") + 1]).parent.name
            for command in commands
        ]
        self.assertEqual(manifests, ["paper", "lens", "deck", "wall"])
        for command, product_name in zip(commands, manifests, strict=True):
            self.assertIn("--locked", command)
            self.assertIn("--release", command)
            target = Path(command[command.index("--target-dir") + 1])
            self.assertEqual(target, self.products[product_name].root / "target")
        self.assertIn("--workspace", commands[0])
        self.assertIn("--workspace", commands[1])
        self.assertIn("--workspace", commands[2])
        self.assertNotIn("--workspace", commands[3])
        self.assertIn("skwd-wall", commands[3])

    def test_product_scoped_build_only_builds_the_selected_product(self):
        with mock.patch.object(self.stage.subprocess, "run") as run, mock.patch("builtins.print"):
            self.stage.build_suite(self.products, ("wall",))
        self.assertEqual(len(run.call_args_list), 1)
        command = run.call_args.args[0]
        self.assertEqual(
            Path(command[command.index("--manifest-path") + 1]).parent,
            self.products["wall"].root,
        )

    def test_product_scoped_refresh_preserves_other_artifacts_and_provenance(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        before_manifest = json.loads((destination / ".skwd-suite.json").read_text())
        before_paper = {
            name: (destination / name).read_bytes()
            for name in self.stage.PRODUCT_BINARIES["paper"]
        }
        wall_source = self.products["wall"].bin_dir / "skwd-wall"
        wall_source.write_text("wall:new\n", encoding="utf-8")
        wall_source.chmod(0o755)

        current = self.stage.publish_suite(
            self.products,
            self.prefix,
            "built",
            selected_products=("wall",),
        )

        after_manifest = json.loads((current / ".skwd-suite.json").read_text())
        self.assertEqual(after_manifest["build_mode"], "built:wall")
        self.assertEqual((current / "skwd-wall").read_text(), "wall:new\n")
        self.assertEqual(
            {
                name: (current / name).read_bytes()
                for name in self.stage.PRODUCT_BINARIES["paper"]
            },
            before_paper,
        )
        self.assertEqual(
            after_manifest["repositories"]["paper"],
            before_manifest["repositories"]["paper"],
        )
        before_artifacts = {
            item["name"]: item for item in before_manifest["artifacts"]
        }
        after_artifacts = {
            item["name"]: item for item in after_manifest["artifacts"]
        }
        for name in self.stage.PRODUCT_BINARIES["paper"]:
            self.assertEqual(after_artifacts[name], before_artifacts[name])

    def test_product_scoped_refresh_requires_an_installed_suite(self):
        with self.assertRaisesRegex(
            self.stage.StageError, "requires an installed complete suite"
        ):
            self.stage.publish_suite(
                self.products,
                self.prefix,
                "built",
                selected_products=("wall",),
            )

    def test_product_scoped_refresh_rejects_changed_preserved_artifact(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        before_wall = (destination / "skwd-wall").read_bytes()
        (destination / "skwd-wall-vk").write_bytes(b"changed outside the manifest")
        with self.assertRaisesRegex(
            self.stage.StageError, "does not match its manifest"
        ):
            self.stage.publish_suite(
                self.products,
                self.prefix,
                "built",
                selected_products=("wall",),
            )
        self.assertEqual((destination / "skwd-wall").read_bytes(), before_wall)
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])

    def test_refresh_replaces_the_complete_bin_directory(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        original_inode = destination.stat().st_ino
        wall_source = self.products["wall"].bin_dir / "skwd-wall"
        wall_source.write_text("wall:new\n", encoding="utf-8")
        wall_source.chmod(0o755)
        refreshed = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertEqual(refreshed, destination)
        self.assertNotEqual(refreshed.stat().st_ino, original_inode)
        self.assertEqual((refreshed / "skwd-wall").read_text(), "wall:new\n")
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])

    def test_if_changed_keeps_identical_published_directory(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        original_inode = destination.stat().st_ino
        original_manifest = (destination / ".skwd-suite.json").read_bytes()

        with mock.patch.object(
            self.stage, "copy_artifact", wraps=self.stage.copy_artifact
        ) as copy_artifact:
            current = self.stage.publish_suite(
                self.products, self.prefix, "built", if_changed=True
            )

        self.assertEqual(current.stat().st_ino, original_inode)
        self.assertEqual((current / ".skwd-suite.json").read_bytes(), original_manifest)
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])
        copy_artifact.assert_not_called()

    def test_if_changed_replaces_one_changed_artifact(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        original_inode = destination.stat().st_ino
        source = self.products["paper"].bin_dir / "skwd-wall-vk"
        source.write_text("paper:skwd-wall-vk:two\n", encoding="utf-8")
        source.chmod(0o755)

        current = self.stage.publish_suite(
            self.products, self.prefix, "built", if_changed=True
        )

        self.assertNotEqual(current.stat().st_ino, original_inode)
        self.assertEqual((current / "skwd-wall-vk").read_text(), "paper:skwd-wall-vk:two\n")

    def test_refresh_migrates_the_exact_pre_cli_suite(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        original_inode = destination.stat().st_ino
        (destination / "skwd-paper").unlink()
        (destination / "skwd-paper-tinier").unlink()
        manifest_path = destination / ".skwd-suite.json"
        manifest = json.loads(manifest_path.read_text())
        manifest["artifacts"] = [
            item
            for item in manifest["artifacts"]
            if item["name"] not in {"skwd-paper", "skwd-paper-tinier"}
        ]
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

        refreshed = self.stage.publish_suite(self.products, self.prefix, "stage-only")

        self.assertNotEqual(refreshed.stat().st_ino, original_inode)
        self.assertTrue((refreshed / "skwd-paper").is_file())
        self.assertTrue((refreshed / "skwd-paper-tinier").is_file())
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])

    def test_refresh_migrates_the_exact_pre_tinier_suite(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        original_inode = destination.stat().st_ino
        (destination / "skwd-paper-tinier").unlink()

        refreshed = self.stage.publish_suite(self.products, self.prefix, "stage-only")

        self.assertNotEqual(refreshed.stat().st_ino, original_inode)
        self.assertTrue((refreshed / "skwd-paper-tinier").is_file())
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])

    def test_other_incomplete_suite_is_not_accepted_as_legacy(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        (destination / "skwd-wall-vk").unlink()

        with self.assertRaisesRegex(self.stage.StageError, "missing artifacts: skwd-wall-vk"):
            self.stage.publish_suite(self.products, self.prefix, "stage-only")

        self.assertFalse((destination / "skwd-wall-vk").exists())
        self.assertTrue((destination / "skwd-paper").is_file())
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])

    def test_unexpected_file_refuses_refresh_and_leaves_destination_intact(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        installed = destination / "skwd-wall"
        before = installed.read_bytes()
        unexpected = destination / "local-note"
        unexpected.write_text("keep", encoding="utf-8")
        source = self.products["wall"].bin_dir / "skwd-wall"
        source.write_text("wall:new\n", encoding="utf-8")
        source.chmod(0o755)
        with self.assertRaisesRegex(self.stage.StageError, "unexpected entries"):
            self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertEqual(installed.read_bytes(), before)
        self.assertEqual(unexpected.read_text(), "keep")

    def test_unexpected_directory_refuses_refresh_and_leaves_destination_intact(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        before = (destination / ".skwd-suite.json").read_bytes()
        unexpected = destination / "local-directory"
        unexpected.mkdir()
        with self.assertRaisesRegex(self.stage.StageError, "unexpected entries"):
            self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertEqual((destination / ".skwd-suite.json").read_bytes(), before)
        self.assertTrue(unexpected.is_dir())

    def test_symlink_refuses_refresh_and_leaves_destination_intact(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        manifest = destination / ".skwd-suite.json"
        manifest.unlink()
        target = self.root / "external-manifest"
        target.write_text("external", encoding="utf-8")
        manifest.symlink_to(target)
        before = (destination / "skwd-wall").read_bytes()
        with self.assertRaisesRegex(self.stage.StageError, "contains a symlink"):
            self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertEqual((destination / "skwd-wall").read_bytes(), before)
        self.assertTrue(manifest.is_symlink())
        self.assertEqual(target.read_text(), "external")

    def test_symlinked_prefix_ancestor_is_rejected(self):
        real = self.root / "real-prefix-parent"
        real.mkdir()
        linked = self.root / "linked-prefix-parent"
        linked.symlink_to(real, target_is_directory=True)
        with self.assertRaisesRegex(self.stage.StageError, "symlinked ancestor"):
            self.stage.publish_suite(self.products, linked / "suite", "stage-only")
        self.assertEqual(list(real.iterdir()), [])

    def test_post_exchange_change_rolls_back_without_deleting_old_directory(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        installed = destination / "skwd-wall"
        before = installed.read_bytes()
        source = self.products["wall"].bin_dir / "skwd-wall"
        source.write_text("wall:new\n", encoding="utf-8")
        source.chmod(0o755)
        exchange = self.stage.exchange_directories
        call_count = 0

        def exchange_and_change(first, second):
            nonlocal call_count
            exchange(first, second)
            call_count += 1
            if call_count == 1:
                (first / "concurrent-file").write_text("keep", encoding="utf-8")

        with mock.patch.object(self.stage, "exchange_directories", exchange_and_change):
            with self.assertRaisesRegex(self.stage.StageError, "unexpected entries"):
                self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertEqual(call_count, 2)
        self.assertEqual(installed.read_bytes(), before)
        self.assertEqual((destination / "concurrent-file").read_text(), "keep")
        self.assertEqual(list(self.prefix.glob(".bin.stage-*")), [])

    def test_rejects_symlinked_source_without_replacing_current_suite(self):
        destination = self.stage.publish_suite(self.products, self.prefix, "stage-only")
        before = (destination / ".skwd-suite.json").read_bytes()
        source = self.products["lens"].bin_dir / "skwd-lens"
        target = source.with_name("skwd-lens-real")
        source.rename(target)
        source.symlink_to(target.name)
        with self.assertRaises(self.stage.StageError):
            self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertEqual((destination / ".skwd-suite.json").read_bytes(), before)
        self.assertTrue((destination / "skwd-lens").is_file())

    def test_rejects_non_executable_source(self):
        source = self.products["paper"].bin_dir / "skwd-paper"
        source.chmod(0o644)
        with self.assertRaisesRegex(self.stage.StageError, "not executable"):
            self.stage.publish_suite(self.products, self.prefix, "stage-only")
        self.assertFalse((self.prefix / "bin").exists())


if __name__ == "__main__":
    unittest.main()
