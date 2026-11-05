import hashlib
import io
import json
import os
import re
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
FULL_REVISION = re.compile(r"^[0-9a-f]{40}$")
PUBLISHER_FINGERPRINT = "A" * 40
FIXTURE_NOW = 1_800_000_100


class SupplyChainInputs(unittest.TestCase):
    def run_installer(
        self,
        *,
        corrupt_archive=False,
        cross_owner_component=False,
        bad_signature=False,
        bad_fingerprint=False,
        cross_origin_asset=False,
        expired_manifest=False,
        fail_download=False,
        existing_binary=False,
        fail_commit=False,
        future_manifest=False,
        inconsistent_versions=False,
        insecure_token_transport=False,
        machine="x86_64",
        missing_gpgv=False,
        previous_version=None,
        release_repository="liixini/skwd-release",
        token=False,
        unsafe_archive=False,
    ):
        with tempfile.TemporaryDirectory(prefix="skwd-installer-test-") as root_name:
            root = Path(root_name)
            payloads = {name: root / f"payload-{name}" for name in ("wall", "deck", "paper", "lens")}
            binary = payloads["wall"] / "usr" / "bin" / "skwd-wall-v2"
            binary.parent.mkdir(parents=True)
            binary.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            binary.chmod(0o755)

            deck_bin = payloads["deck"] / "usr/bin"
            deck_bin.mkdir(parents=True)
            for name in ("skwd-walld", "skwd-wall-scan", "skwd-wall-effects", "skwd-wall-draw", "skwd-steam", "skwd-helm"):
                path = deck_bin / name
                path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
                path.chmod(0o755)
            service = payloads["deck"] / "usr/lib/systemd/user/skwd-walld.service"
            service.parent.mkdir(parents=True)
            service.write_text(
                "[Service]\nExecStart=/usr/bin/skwd-walld --wait-for-session\n",
                encoding="utf-8",
            )

            paper_bin = payloads["paper"] / "usr/bin"
            paper_bin.mkdir(parents=True)
            for name in ("skwd-paper-v2", "skwd-wall-still", "skwd-wall-vk"):
                path = paper_bin / name
                path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
                path.chmod(0o755)

            lens_bin = payloads["lens"] / "usr/bin/skwd-lens"
            lens_bin.parent.mkdir(parents=True)
            lens_bin.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            lens_bin.chmod(0o755)
            for name, payload in payloads.items():
                license_name = "skwd-wall-v2" if name == "wall" else f"skwd-{name}"
                licenses = payload / "usr/share/licenses" / license_name
                (licenses / "third-party").mkdir(parents=True)
                (licenses / "LICENSE").write_text("fixture license\n", encoding="utf-8")
                (licenses / "third-party/fixture.txt").write_text(
                    "fixture dependency license\n", encoding="utf-8"
                )

            archives = {}
            for name, payload in payloads.items():
                version = "2.0.0" if inconsistent_versions and name == "lens" else "1.0.0"
                archive_name = f"skwd-{name}_{version}_linux-x86_64.tar.xz"
                archive = root / archive_name
                with tarfile.open(archive, "w:xz") as package:
                    package.add(payload / "usr", arcname="usr")
                    if unsafe_archive and name == "wall":
                        content = b"must not escape\n"
                        member = tarfile.TarInfo("../escape")
                        member.size = len(content)
                        package.addfile(member, io.BytesIO(content))
                archives[name] = archive
            issued_at = FIXTURE_NOW + 301 if future_manifest else FIXTURE_NOW - 100
            expires_at = FIXTURE_NOW - 1 if expired_manifest else FIXTURE_NOW + 3600
            revisions = dict(zip(("wall", "deck", "paper", "lens"), "abcd", strict=True))
            release_owner = release_repository.split("/", 1)[0]

            def installer_manifest(version, *, changed=False):
                lines = [
                    "SKWD-INSTALL-MANIFEST\t1",
                    "contract\tskwd-portable-suite-v1",
                    "architecture\tx86_64",
                    f"version\t{version}",
                    f"tag\tv{version}",
                    f"issued-at\t{issued_at - int(changed)}",
                    f"expires-at\t{expires_at}",
                    f"release\t{release_repository}\t{'f' * 40}",
                ]
                for component in ("wall", "deck", "paper", "lens"):
                    archive = archives[component]
                    archive_version = archive.name.split("_", 2)[1]
                    archive_name = (
                        archive.name
                        if version == "1.0.0"
                        else f"skwd-{component}_{version}_linux-x86_64.tar.xz"
                    )
                    lines.append(
                        "\t".join(
                            (
                                "component",
                                component,
                                (
                                    f"hostile/skwd-{component}"
                                    if cross_owner_component and component == "lens"
                                    else f"{release_owner}/skwd-{component}"
                                ),
                                revisions[component] * 40,
                                archive_name,
                                hashlib.sha256(archive.read_bytes()).hexdigest(),
                                str(archive.stat().st_size),
                            )
                        )
                    )
                    self.assertEqual(archive_version, "2.0.0" if inconsistent_versions and component == "lens" else "1.0.0")
                return "\n".join(lines) + "\n"

            (root / "SKWD-INSTALL-MANIFEST").write_text(
                installer_manifest("1.0.0"), encoding="utf-8"
            )
            (root / "SKWD-INSTALL-MANIFEST.asc").write_text(
                "fixture signature\n", encoding="utf-8"
            )
            (root / "skwd-release-keyring.gpg").write_text(
                "fixture keyring\n", encoding="utf-8"
            )
            if corrupt_archive:
                archive = archives["wall"]
                archive.write_bytes(archive.read_bytes() + b"corrupt")

            base_url = "https://forgejo.invalid/assets"
            (root / "release.json").write_text(
                json.dumps(
                    {
                        "tag_name": "v1.0.0",
                        "assets": [
                            *[
                                {
                                    "browser_download_url": (
                                        f"https://hostile.invalid/{archive.name}"
                                        if cross_origin_asset and archive == archives["wall"]
                                        else f"{base_url}/{archive.name}"
                                    )
                                }
                                for archive in archives.values()
                            ],
                            {"browser_download_url": f"{base_url}/SKWD-INSTALL-MANIFEST"},
                            {"browser_download_url": f"{base_url}/SKWD-INSTALL-MANIFEST.asc"},
                            {"browser_download_url": f"{base_url}/skwd-release-keyring.gpg"},
                        ]
                    }
                ),
                encoding="utf-8",
            )
            tools = root / "tools"
            tools.mkdir()
            fake_dirname = tools / "dirname"
            fake_dirname.write_text(
                "#!/bin/sh\nexec /usr/bin/dirname \"$@\"\n", encoding="utf-8"
            )
            fake_dirname.chmod(0o755)
            fake_curl = tools / "curl"
            fake_curl.write_text(
                """#!/bin/sh
set -eu
destination=-
url=
while [ "$#" -gt 0 ]; do
    case "$1" in
        -o) destination=$2; shift 2 ;;
        -H) shift 2 ;;
        --config) shift 2 ;;
        -*) shift ;;
        *) url=$1; shift ;;
    esac
done
case "$url" in
    */releases/latest) source=$SKWD_INSTALLER_FIXTURES/release.json ;;
    */releases/tags/v1.0.0) source=$SKWD_INSTALLER_FIXTURES/release.json ;;
    */SKWD-INSTALL-MANIFEST) source=$SKWD_INSTALLER_FIXTURES/SKWD-INSTALL-MANIFEST ;;
    */SKWD-INSTALL-MANIFEST.asc) source=$SKWD_INSTALLER_FIXTURES/SKWD-INSTALL-MANIFEST.asc ;;
    */skwd-release-keyring.gpg) source=$SKWD_INSTALLER_FIXTURES/skwd-release-keyring.gpg ;;
    */skwd-*_linux-*.tar.xz) source=$SKWD_INSTALLER_FIXTURES/${url##*/} ;;
    *) exit 64 ;;
esac
if [ "$destination" = - ]; then
    exec /usr/bin/cat "$source"
fi
exec /usr/bin/cp "$source" "$destination"
""",
                encoding="utf-8",
            )
            fake_curl.chmod(0o755)
            fake_gpgv = tools / "gpgv"
            if not missing_gpgv:
                fake_gpgv.write_text(
                    "#!/bin/sh\n"
                    "[ \"$1\" = --status-fd ]\n"
                    "[ \"$2\" = 1 ]\n"
                    "[ \"$3\" = --keyring ]\n"
                    "[ -f \"$4\" ]\n"
                    "[ -f \"$5\" ]\n"
                    "[ -f \"$6\" ]\n"
                    "[ \"${SKWD_INSTALLER_SIGNATURE_VALID:-0}\" = 1 ] || exit 1\n"
                    "printf '[GNUPG:] VALIDSIG %s 20260827 1800000000 0 4 0 1 10 00 %s\\n' "
                    "\"$SKWD_INSTALLER_SIGNING_FINGERPRINT\" "
                    "\"$SKWD_INSTALLER_SIGNING_FINGERPRINT\"\n",
                    encoding="utf-8",
                )
                fake_gpgv.chmod(0o755)
            fake_date = tools / "date"
            fake_date.write_text(
                "#!/bin/sh\nprintf '%s\\n' \"$SKWD_INSTALLER_NOW\"\n",
                encoding="utf-8",
            )
            fake_date.chmod(0o755)
            fake_uname = tools / "uname"
            fake_uname.write_text(f"#!/bin/sh\nprintf '%s\\n' '{machine}'\n", encoding="utf-8")
            fake_uname.chmod(0o755)
            fake_mv = tools / "mv"
            fake_mv.write_text(
                "#!/bin/sh\n"
                "set -eu\n"
                "count_file=$SKWD_INSTALLER_FIXTURES/mv-count\n"
                "count=0\n"
                "[ ! -f \"$count_file\" ] || count=$(/usr/bin/cat \"$count_file\")\n"
                "count=$((count + 1))\n"
                "printf '%s\\n' \"$count\" > \"$count_file\"\n"
                "if [ \"${SKWD_INSTALLER_FAIL_COMMIT:-0}\" = 1 ] && [ \"$count\" -eq 2 ]; then exit 74; fi\n"
                "exec /usr/bin/mv \"$@\"\n",
                encoding="utf-8",
            )
            fake_mv.chmod(0o755)
            if fail_download:
                fake_curl.write_text(
                    fake_curl.read_text(encoding="utf-8").replace(
                        "exec /usr/bin/cp \"$source\" \"$destination\"",
                        "case \"$url\" in */skwd-paper_*) exit 74 ;; esac\n"
                        "exec /usr/bin/cp \"$source\" \"$destination\"",
                    ),
                    encoding="utf-8",
                )

            bindir = root / "bin"
            if existing_binary:
                bindir.mkdir()
                (bindir / "skwd-wall-v2").write_text(
                    "old working install\n", encoding="utf-8"
                )
            state_home = root / "state"
            if previous_version is not None:
                state = state_home / "skwd-wall-v2"
                state.mkdir(parents=True)
                (state / "portable-release.manifest").write_text(
                    installer_manifest(
                        previous_version.removesuffix("-changed"),
                        changed=previous_version.endswith("-changed"),
                    ),
                    encoding="utf-8",
                )
                (state / "portable-release.manifest.asc").write_text(
                    "fixture previous signature\n", encoding="utf-8"
                )
            environment = os.environ.copy()
            environment.update(
                {
                    "HOME": str(root / "home"),
                    "PATH": f"{tools}:{environment['PATH']}",
                    "SKWD_BINDIR": str(bindir),
                    "SKWD_FORGEJO_URL": (
                        "http://forgejo.invalid"
                        if insecure_token_transport
                        else "https://forgejo.invalid"
                    ),
                    "SKWD_RELEASE_KEY_FINGERPRINT": PUBLISHER_FINGERPRINT,
                    "SKWD_RELEASE_REPOSITORY": release_repository,
                    "SKWD_INSTALLER_FIXTURES": str(root),
                    "SKWD_INSTALLER_SIGNATURE_VALID": "0" if bad_signature else "1",
                    "SKWD_INSTALLER_SIGNING_FINGERPRINT": (
                        "B" * 40 if bad_fingerprint else PUBLISHER_FINGERPRINT
                    ),
                    "SKWD_INSTALLER_NOW": str(FIXTURE_NOW),
                    "SKWD_INSTALLER_FAIL_COMMIT": "1" if fail_commit else "0",
                    "XDG_DATA_HOME": str(root / "data"),
                    "XDG_STATE_HOME": str(state_home),
                }
            )
            if token or insecure_token_transport:
                environment["SKWD_FORGEJO_TOKEN"] = "fixture-secret"
            if missing_gpgv:
                environment["PATH"] = str(tools)
            result = subprocess.run(
                [str(ROOT / "install.sh")],
                cwd=ROOT,
                env=environment,
                capture_output=True,
                text=True,
                check=False,
            )
            installed = bindir / "skwd-wall-v2"
            content = installed.read_text(encoding="utf-8") if installed.is_file() else ""
            return result, installed.exists(), content

    def test_forgejo_actions_use_full_commit_ids(self):
        workflows = sorted((ROOT / ".forgejo" / "workflows").glob("*.yml"))
        self.assertTrue(workflows)
        for workflow in workflows:
            for line in workflow.read_text(encoding="utf-8").splitlines():
                stripped = line.strip()
                if not stripped.startswith("uses:"):
                    continue
                reference = stripped.rsplit("@", 1)[-1]
                self.assertRegex(reference, FULL_REVISION, f"mutable action in {workflow}")

    def test_installer_verifies_the_selected_release_archive(self):
        installer = (ROOT / "install.sh").read_text(encoding="utf-8")
        for contract in [
            'download "$release_manifest_url" "$release_manifest"',
            'download "$release_signature_url" "$release_signature"',
            'download "$release_keyring_url" "$downloaded_keyring"',
            "gpgv --status-fd 1 --keyring",
            'validate_manifest_shape "$release_manifest"',
            'releases/tags/$release_tag',
            'component_api_tag',
            'sha256sum --check --status',
            'refusing signed release rollback',
            'prepare_file "$release_manifest"',
        ]:
            self.assertIn(contract, installer)
        self.assertLess(
            installer.index("sha256sum --check --status"),
            installer.index("tar --no-same-owner"),
        )

    def test_installer_accepts_a_matching_release_checksum(self):
        result, installed, content = self.run_installer()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(installed)
        self.assertIn("#!/bin/sh", content)

    def test_installer_accepts_a_signed_alternate_self_host_namespace(self):
        result, installed, content = self.run_installer(
            release_repository="mirror/skwd-release"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(installed)
        self.assertIn("#!/bin/sh", content)

    def test_installer_rejects_a_cross_owner_component_manifest(self):
        result, installed, _ = self.run_installer(cross_owner_component=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid coordinated interface", result.stderr)
        self.assertFalse(installed)

    def test_installer_upgrades_an_existing_install(self):
        result, installed, content = self.run_installer(existing_binary=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(installed)
        self.assertIn("#!/bin/sh", content)
        self.assertNotIn("old working install", content)

    def test_installer_rejects_a_mismatched_release_checksum(self):
        result, installed, _ = self.run_installer(corrupt_archive=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 verification failed", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_an_invalid_release_signature(self):
        result, installed, _ = self.run_installer(bad_signature=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("signature verification failed", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_a_signature_from_an_unreviewed_key(self):
        result, installed, _ = self.run_installer(bad_fingerprint=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("reviewed publisher key", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_an_expired_signed_manifest(self):
        result, installed, _ = self.run_installer(expired_manifest=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("manifest has expired", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_a_signed_manifest_from_the_future(self):
        result, installed, _ = self.run_installer(future_manifest=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("manifest is not valid yet", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_cross_origin_assets_before_authenticated_download(self):
        result, installed, _ = self.run_installer(cross_origin_asset=True, token=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the configured Forgejo origin", result.stderr)
        self.assertNotIn("hostile.invalid", result.stdout)
        self.assertFalse(installed)

    def test_installer_refuses_a_token_over_plain_http(self):
        result, installed, _ = self.run_installer(insecure_token_transport=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refusing to send SKWD_FORGEJO_TOKEN", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_mixed_component_versions(self):
        result, installed, _ = self.run_installer(inconsistent_versions=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid coordinated interface", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_a_signed_version_rollback(self):
        result, installed, _ = self.run_installer(previous_version="2.0.0")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("release rollback from 2.0.0 to 1.0.0", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_same_version_manifest_equivocation(self):
        result, installed, _ = self.run_installer(previous_version="1.0.0-changed")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("reuses version 1.0.0 with different content", result.stderr)
        self.assertFalse(installed)

    def test_installer_rejects_an_unsafe_archive_path(self):
        result, installed, _ = self.run_installer(unsafe_archive=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unsafe path", result.stderr)
        self.assertFalse(installed)

    def test_installer_reports_a_missing_signature_verifier(self):
        result, installed, _ = self.run_installer(missing_gpgv=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("gpgv is required", result.stderr)
        self.assertFalse(installed)

    def test_interrupted_download_leaves_no_partial_install(self):
        result, installed, content = self.run_installer(
            fail_download=True, existing_binary=True
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(installed)
        self.assertEqual(content, "old working install\n")

    def test_commit_failure_rolls_back_the_previous_install(self):
        result, installed, content = self.run_installer(
            existing_binary=True, fail_commit=True
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(installed)
        self.assertEqual(content, "old working install\n")

    def test_unsupported_architecture_fails_before_download(self):
        result, installed, content = self.run_installer(
            existing_binary=True, machine="aarch64"
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Linux x86_64 only", result.stderr)
        self.assertTrue(installed)
        self.assertEqual(content, "old working install\n")

    def test_repository_downloads_use_the_reviewed_installer_boundary(self):
        candidates = [ROOT / "install.sh"]
        candidates.extend((ROOT / ".forgejo" / "workflows").glob("*.yml"))
        candidates.extend((ROOT / "packaging").rglob("*.sh"))
        for path in candidates:
            text = path.read_text(encoding="utf-8")
            if path.name == "install.sh":
                commands = [
                    line.strip()
                    for line in text.splitlines()
                    if re.match(r"^curl(?:\s|$)", line.strip())
                ]
                self.assertEqual(len(commands), 4)
                self.assertNotRegex(text, r"^\s*wget(?:\s|$)", re.MULTILINE)
                continue
            self.assertNotRegex(text, r"\b(?:curl|wget)\s", f"unreviewed download in {path}")


if __name__ == "__main__":
    unittest.main()
