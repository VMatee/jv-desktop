"""Offline contracts for the exact filenames uploaded to GitHub Releases."""
import importlib.util
import json
import pathlib
import re
import shutil
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('release_assets', ROOT / 'scripts/release-assets.py')
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseAssetsTests(unittest.TestCase):
    def setUp(self):
        cache = ROOT / '.cache/release-tests'
        cache.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=cache)
        self.addCleanup(self.temporary.cleanup)
        self.root = pathlib.Path(self.temporary.name)
        self.version = '0.1.1'
        self.bundle = self.root / 'bundle'
        self.output = self.root / 'release'
        for index, (relative, _) in enumerate(release.asset_names(self.version)):
            path = self.bundle / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(bytes([index + 1]) * 100_001)
        self.record = release.collect_assets(self.bundle, self.output, self.version, {'commit': 'a' * 40})

    def test_download_round_trip_uses_only_canonical_names(self):
        downloaded = self.root / 'download'
        shutil.copytree(self.output, downloaded)
        # Verification must not rely on the source Tauri bundle names or paths.
        self.bundle.rename(self.root / 'unavailable-source')
        self.assertEqual(release.verify_assets(downloaded, self.version, 'linux'), self.record)
        self.assertEqual((downloaded / 'SHA256SUMS').read_bytes(), (self.output / 'SHA256SUMS').read_bytes())
        self.assertEqual({asset['name'] for asset in self.record['assets']}, {
            'JV-Desktop_0.1.1_amd64.deb', 'JV-Desktop-0.1.1-1.x86_64.rpm', 'JV-Desktop_0.1.1_amd64.AppImage'})
        if shutil.which('sha256sum'):
            subprocess.run(['sha256sum', '-c', 'SHA256SUMS'], cwd=downloaded, check=True, capture_output=True)

    def test_renamed_unsafe_or_extra_asset_is_rejected(self):
        for name in ['JV Desktop_0.1.1_amd64.deb', 'JV.Desktop_0.1.1_amd64.deb', 'unexpected.deb']:
            with self.subTest(name=name):
                original = self.output / self.record['assets'][0]['name']
                changed = original.with_name(name)
                original.rename(changed)
                with self.assertRaises(ValueError):
                    release.verify_assets(self.output, self.version, 'linux')
                changed.rename(original)

    def test_duplicate_or_missing_checksum_is_rejected(self):
        path = self.output / 'SHA256SUMS'
        original = path.read_text()
        for content in [original + original.splitlines()[0] + '\n', '\n'.join(original.splitlines()[1:]) + '\n']:
            with self.subTest(content=content):
                path.write_text(content)
                with self.assertRaises(ValueError):
                    release.verify_assets(self.output, self.version, 'linux')

    def test_corrupted_or_empty_asset_is_rejected(self):
        path = self.output / self.record['assets'][0]['name']
        for data in [b'corrupted', b'']:
            with self.subTest(data=data):
                path.write_bytes(data)
                with self.assertRaises(ValueError):
                    release.verify_assets(self.output, self.version, 'linux')

    def test_build_info_hash_and_size_must_agree(self):
        path = self.output / 'BUILD-INFO.json'
        record = json.loads(path.read_text())
        record['assets'][0]['bytes'] += 1
        path.write_text(json.dumps(record))
        checksums = self.output / 'SHA256SUMS'
        lines = checksums.read_text().splitlines()
        lines[-1] = f'{release.digest(path)}  BUILD-INFO.json'
        checksums.write_text('\n'.join(lines) + '\n')
        with self.assertRaises(ValueError):
            release.verify_assets(self.output, self.version, 'linux')

    def test_old_output_is_not_silently_reused(self):
        with self.assertRaises(ValueError):
            release.collect_assets(self.bundle, self.output, self.version, {'commit': 'a' * 40})

    def test_filename_contract_is_version_independent(self):
        names = release.asset_names('2.34.5')
        self.assertEqual(names[0][1], 'JV-Desktop_2.34.5_amd64.deb')
        for _, name in names:
            self.assertNotIn(' ', name)
        with self.assertRaises(ValueError):
            release.asset_names('../0.1.1')

    def windows(self, commit='a' * 40):
        bundle = self.root / 'windows-bundle'
        for relative, _ in release.asset_names(self.version, 'windows'):
            path = bundle / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'MZ' + b'w' * 100_001)
        output = self.root / 'windows'
        release.collect_assets(bundle, output, self.version, {'commit': commit}, 'windows')
        return output

    def test_windows_roundtrip_uses_canonical_installer_name(self):
        output = self.windows()
        record = release.verify_assets(output, self.version, 'windows')
        self.assertEqual(record['assets'][0]['name'], 'JV-Desktop_0.1.1_x64-setup.exe')
        self.assertEqual(record['signing'], 'unsigned')

    def test_combined_public_download_covers_both_platforms(self):
        output = self.root / 'combined'
        release.combine_assets(self.output, self.windows(), output, self.version, 'a' * 40)
        downloaded = self.root / 'public-download'
        shutil.copytree(output, downloaded)
        self.bundle.rename(self.root / 'source-no-longer-available')
        record = release.verify_assets(downloaded, self.version)
        self.assertEqual(len(record['assets']), 4)
        self.assertEqual(set(record['builds']), {'linux', 'windows'})
        self.assertEqual(len(list(downloaded.iterdir())), 6)

    def test_mixed_source_commits_cannot_be_published(self):
        with self.assertRaises(ValueError):
            release.combine_assets(self.output, self.windows('b' * 40), self.root / 'combined', self.version, 'a' * 40)

    def test_partial_inventory_cannot_pass_public_release_verification(self):
        with self.assertRaises(ValueError):
            release.verify_assets(self.output, self.version)

    def test_tampered_native_build_record_is_rejected_even_with_updated_checksum(self):
        output = self.root / 'combined'
        record = release.combine_assets(self.output, self.windows(), output, self.version, 'a' * 40)
        record['builds']['windows']['commit'] = 'b' * 40
        release.write_metadata(output, record)
        with self.assertRaises(ValueError):
            release.verify_assets(output, self.version)

    def test_missing_or_corrupt_windows_installer_prevents_combination(self):
        windows = self.windows()
        installer = windows / release.asset_names(self.version, 'windows')[0][1]
        installer.write_bytes(b'corrupted')
        with self.assertRaises(ValueError):
            release.combine_assets(self.output, windows, self.root / 'combined', self.version, 'a' * 40)


class ReleaseVersionTests(unittest.TestCase):
    def test_all_desktop_version_metadata_and_release_notes_agree(self):
        version = json.loads((ROOT / 'package.json').read_text())['version']
        lock = json.loads((ROOT / 'package-lock.json').read_text())
        self.assertEqual(lock['version'], version)
        self.assertEqual(lock['packages']['']['version'], version)
        self.assertEqual(json.loads((ROOT / 'src-tauri/tauri.conf.json').read_text())['version'], version)
        cargo = (ROOT / 'Cargo.toml').read_text()
        self.assertRegex(cargo, rf'(?m)^version = "{re.escape(version)}"$')
        cargo_lock = (ROOT / 'Cargo.lock').read_text()
        for package in ['jv-desktop', 'jv-desktop-process']:
            self.assertIn(f'name = "{package}"\nversion = "{version}"', cargo_lock)
        self.assertTrue((ROOT / 'docs/releases' / f'{version}.md').is_file())


if __name__ == '__main__':
    unittest.main()
