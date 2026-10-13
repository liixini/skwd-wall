import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
VERIFY_ROOT = Path(os.environ.get("SKWD_VERIFY_ROOT", ROOT.parent / "skwd-verify"))
EXPECTED = [
    "verify.repository",
    "wall.format",
    "wall.clippy",
    "wall.tests",
    "wall.allocations",
    "wall.python",
    "wall.unsafe",
    "wall.release",
]


class ForgejoRequiredGateTests(unittest.TestCase):
    def test_storage_precedes_checkout(self):
        workflow = (ROOT / ".forgejo" / "workflows" / "verify.yml").read_text(
            encoding="utf-8"
        )
        self.assertLess(
            workflow.index("- name: Prepare temporary storage"),
            workflow.index("- name: Check out Wall"),
        )
        self.assertIn("working-directory: ${{ github.workspace }}", workflow)
        for name in ("TMPDIR", "TMP", "TEMP", "RUNNER_TEMP"):
            self.assertIn(f"{name}: /var/tmp/skwd-verify-tmp-", workflow)
        self.assertIn('findmnt -n -o FSTYPE -T "$TMPDIR"', workflow)
        self.assertIn('""|tmpfs|ramfs) exit 1', workflow)
        self.assertIn('df -h "$TMPDIR"', workflow)

    def test_workflow_emits_and_aggregates_every_required_suite(self):
        workflow = (ROOT / ".forgejo" / "workflows" / "verify.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("push:\n    branches: [v2]", workflow)
        self.assertNotIn("push:\n    branches: [main]", workflow)
        manifest = [
            line
            for raw in (ROOT / "scripts" / "ci-required-manifest.txt").read_text(
                encoding="utf-8"
            ).splitlines()
            if (line := raw.strip()) and not line.startswith("#")
        ]
        self.assertEqual(manifest, EXPECTED)
        for suite in EXPECTED:
            self.assertEqual(workflow.count(f"--suite {suite} "), 1, suite)
        self.assertIn("SKWD_VERIFY_ROOT: ../skwd-verify", workflow)
        self.assertIn(
            'if: always()\n        run: python3 "$SKWD_VERIFY_ROOT/scripts/ci-report.py" aggregate',
            workflow,
        )
        self.assertIn("--strict-extra", workflow)
        self.assertIn("retention-days: 14", workflow)
        self.assertIn("actions/upload-artifact@c6a3b2bd78b3985e4b2f15397fec357f0fd808de", workflow)

    def test_provenance_records_revision_image_and_toolchains(self):
        with tempfile.TemporaryDirectory(prefix="skwd-ci-provenance-") as directory:
            destination = Path(directory) / "provenance.json"
            environment = os.environ.copy()
            environment.update(
                {
                    "GITHUB_REPOSITORY": "liixini/skwd-wall",
                    "GITHUB_SHA": "a" * 40,
                    "GITHUB_REF": "refs/heads/main",
                    "GITHUB_EVENT_NAME": "push",
                    "RUNNER_NAME": "fixture",
                    "RUNNER_OS": "Linux",
                    "RUNNER_ARCH": "X64",
                }
            )
            subprocess.run(
                [
                    sys.executable,
                    str(VERIFY_ROOT / "scripts" / "ci-provenance.py"),
                    "--out",
                    str(destination),
                    "--runner-image",
                    "sha256:" + "b" * 64,
                    "--toolchains",
                ],
                cwd=ROOT,
                env=environment,
                check=True,
            )
            provenance = json.loads(destination.read_text(encoding="utf-8"))
            self.assertEqual(provenance["schema"], 1)
            self.assertEqual(provenance["revision"], "a" * 40)
            self.assertEqual(provenance["runner_image"], "sha256:" + "b" * 64)
            self.assertNotIn("runnerImage", provenance)
            self.assertEqual(set(provenance["toolchains"]), {"rustc", "cargo", "python"})


if __name__ == "__main__":
    unittest.main()
