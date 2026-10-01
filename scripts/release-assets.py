#!/usr/bin/env python3
"""Collect only production Linux bundles and record their source/tool provenance."""
import hashlib
import json
import pathlib
import shutil
import subprocess

root = pathlib.Path(__file__).resolve().parent.parent
version = json.loads((root / 'package.json').read_text())['version']
assert json.loads((root / 'src-tauri/tauri.conf.json').read_text())['version'] == version
assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=root).strip(), 'Release requires a clean tree'
sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
out = root / 'target/release-assets'
out.mkdir(exist_ok=True)
names = [f'deb/JV Desktop_{version}_amd64.deb', f'rpm/JV Desktop-{version}-1.x86_64.rpm', f'appimage/JV Desktop_{version}_amd64.AppImage']
assets = []
for relative in names:
    source = root / 'target/release/bundle' / relative
    assert source.is_file() and source.stat().st_size > 100_000
    target = out / source.name
    shutil.copy2(source, target)
    assets.append({'name': target.name, 'bytes': target.stat().st_size, 'sha256': hashlib.sha256(target.read_bytes()).hexdigest()})
(out / 'SHA256SUMS').write_text(''.join(f"{a['sha256']}  {a['name']}\n" for a in assets))
def command(*args):
    return subprocess.check_output(args, cwd=root, text=True).strip()
record = {'version': version, 'commit': sha, 'assets': assets,
          'node': command('node', '--version'), 'npm': command('npm', '--version'),
          'rust': command('node', 'scripts/cargo.mjs', '--version'),
          'distribution': pathlib.Path('/etc/os-release').read_text(),
          'reproducible_build_certification': False}
(out / 'BUILD-INFO.json').write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(record, indent=2))
