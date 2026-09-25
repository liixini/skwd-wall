import re
import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCES = {
    "iced_layershell": (
        "https://github.com/liixini/skwd-iced-layershell.git",
        "0329bd09e66ff4cbd4d354f6925543341710e394",
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
    def test_manifest_uses_only_full_immutable_pins(self):
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

    def test_forgejo_workflow_resolves_immutable_fork_sources(self):
        workflow = (ROOT / ".forgejo" / "workflows" / "verify.yml").read_text()
        for name, (url, revision) in SOURCES.items():
            self.assertIn(url, (ROOT / "Cargo.toml").read_text())
            self.assertIn(revision, (ROOT / "Cargo.toml").read_text())
            self.assertNotIn(f"repository: liixini/skwd-{name.replace('_', '-')}", workflow)
        self.assertNotIn("GIT_CONFIG_COUNT", workflow)
        verify_revision = "8142b83b05cb927fe10d027b214c7ea517efc48d"
        self.assertIn("repository: liixini/skwd-verify", workflow)
        self.assertIn(f"ref: {verify_revision}", workflow)
        verifier_step = workflow.split("      - name: Verify system-test repository\n", 1)[1]
        verifier_step = verifier_step.split("\n      - name: ", 1)[0]
        for required in (
            "working-directory: suite/skwd-verify",
            "continue-on-error: true",
            "--suite verify.repository --target linux-x86_64",
            '-- env "PATH=$SKWD_VERIFY_TEST_ENV/bin:$PATH" scripts/test-all.sh',
        ):
            self.assertIn(required, verifier_step)
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
