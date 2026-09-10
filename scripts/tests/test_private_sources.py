import re
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCES = {
    "iced_layershell": (
        "https://github.com/liixini/skwd-iced-layershell.git",
        "b528b4850d9f7f6a26bea56ca62c684cd412743d",
    ),
    "iced_wgpu": (
        "https://github.com/liixini/skwd-iced-wgpu.git",
        "2ab8bbebec0e31ee24fb919b97cfcbab94ab9ad8",
    ),
}
PRIVATE_CHECKOUTS = {
    "liixini/skwd-deck",
    "liixini/skwd-lens",
    "liixini/skwd-verify",
}
READ_TOKEN = "token: ${{ secrets.SKWD_SUITE_READ_TOKEN }}"


def load_toml(path):
    with path.open("rb") as source:
        return tomllib.load(source)


class PrivateSourcePolicyTests(unittest.TestCase):
    def test_manifest_uses_only_full_immutable_public_pins(self):
        patches = load_toml(ROOT / "Cargo.toml")["patch"]["crates-io"]
        self.assertEqual(set(patches), set(SOURCES))
        for package, (url, revision) in SOURCES.items():
            dependency = patches[package]
            self.assertEqual(dependency, {"git": url, "rev": revision})
            self.assertRegex(revision, r"^[0-9a-f]{40}$")

    def test_lock_matches_every_manifest_pin(self):
        packages = load_toml(ROOT / "Cargo.lock")["package"]
        locked = {package["name"]: package for package in packages}
        for name, (url, revision) in SOURCES.items():
            self.assertEqual(
                locked[name].get("source"),
                f"git+{url}?rev={revision}#{revision}",
            )
            self.assertNotIn("checksum", locked[name])

    def test_source_allowlist_is_exact(self):
        allowed = load_toml(ROOT / "deny.toml")["sources"]["allow-git"]
        self.assertEqual(set(allowed), {url for url, _ in SOURCES.values()})
        self.assertEqual(len(allowed), len(SOURCES))

    def test_git_fetch_uses_the_system_ssh_client(self):
        config = load_toml(ROOT / ".cargo" / "config.toml")
        self.assertIs(config["net"]["git-fetch-with-cli"], True)

    def test_vendored_forks_cannot_return(self):
        self.assertFalse((ROOT / "vendor").exists())

    def test_forgejo_workflow_uses_public_fork_sources(self):
        workflow = (ROOT / ".forgejo" / "workflows" / "verify.yml").read_text()
        for name, (url, revision) in SOURCES.items():
            self.assertIn(url, (ROOT / "Cargo.toml").read_text())
            self.assertIn(revision, (ROOT / "Cargo.toml").read_text())
            self.assertNotIn(f"repository: liixini/skwd-{name.replace('_', '-')}", workflow)
        verify_revision = "bf9f7f4b86c2e5ac8fcef7f2abcdc3a6e5b2cc15"
        self.assertIn("repository: liixini/skwd-verify", workflow)
        self.assertIn(f"ref: {verify_revision}", workflow)
        self.assertRegex(
            workflow,
            r"working-directory: suite/skwd-verify\n"
            r"\s+continue-on-error: true\n"
            r"\s+run: >-\n"
            r"(?:\s+.*\n)*?"
            r"\s+--suite verify\.repository .*\n"
            r"(?:\s+.*\n)*?"
            r"\s+-- scripts/test-all\.sh",
        )
        self.assertFalse((ROOT / ".github" / "workflows" / "ci.yml").exists())

    def test_every_secondary_private_checkout_uses_the_read_token_ephemerally(self):
        workflow = (ROOT / ".forgejo" / "workflows" / "verify.yml").read_text()
        checkouts = {}
        for block in workflow.split("\n      - name: "):
            if "actions/checkout@" not in block:
                continue
            repository = re.search(r"^\s+repository: (\S+)$", block, re.MULTILINE)
            if repository:
                checkouts[repository.group(1)] = block

        self.assertEqual(set(checkouts), PRIVATE_CHECKOUTS)
        for repository, block in checkouts.items():
            with self.subTest(repository=repository):
                self.assertEqual(block.count(READ_TOKEN), 1)
                self.assertEqual(block.count("persist-credentials: false"), 1)

    def test_every_sibling_path_dependency_is_checked_out(self):
        manifest = load_toml(ROOT / "Cargo.toml")
        siblings = {
            f"liixini/{Path(spec['path']).parts[1]}"
            for spec in manifest["dependencies"].values()
            if isinstance(spec, dict)
            and "path" in spec
            and Path(spec["path"]).parts[:1] == ("..",)
        }
        self.assertTrue(siblings)
        self.assertLessEqual(siblings, PRIVATE_CHECKOUTS)


if __name__ == "__main__":
    unittest.main()
