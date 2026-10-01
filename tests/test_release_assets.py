"""Offline contracts for the exact filenames uploaded to GitHub Releases."""
import importlib.util
import json
import pathlib
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
        self.assertEqual(release.verify_assets(downloaded, self.version), self.record)
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
                    release.verify_assets(self.output, self.version)
                changed.rename(original)

    def test_duplicate_or_missing_checksum_is_rejected(self):
        path = self.output / 'SHA256SUMS'
        original = path.read_text()
        for content in [original + original.splitlines()[0] + '\n', '\n'.join(original.splitlines()[1:]) + '\n']:
            with self.subTest(content=content):
                path.write_text(content)
                with self.assertRaises(ValueError):
                    release.verify_assets(self.output, self.version)

    def test_corrupted_or_empty_asset_is_rejected(self):
        path = self.output / self.record['assets'][0]['name']
        for data in [b'corrupted', b'']:
            with self.subTest(data=data):
                path.write_bytes(data)
                with self.assertRaises(ValueError):
                    release.verify_assets(self.output, self.version)

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
            release.verify_assets(self.output, self.version)

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


if __name__ == '__main__':
    unittest.main()
