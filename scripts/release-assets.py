#!/usr/bin/env python3
"""Collect canonical Linux release assets; verify the same contract after download."""
import argparse
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess


def asset_names(version):
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version):
        raise ValueError('Release version must be major.minor.patch')
    return [
        (f'deb/JV Desktop_{version}_amd64.deb', f'JV-Desktop_{version}_amd64.deb'),
        (f'rpm/JV Desktop-{version}-1.x86_64.rpm', f'JV-Desktop-{version}-1.x86_64.rpm'),
        (f'appimage/JV Desktop_{version}_amd64.AppImage', f'JV-Desktop_{version}_amd64.AppImage'),
    ]


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def verify_assets(directory, version):
    """Require exactly the intended upload inventory and checksum every other file."""
    names = {name for _, name in asset_names(version)}
    expected = names | {'BUILD-INFO.json', 'SHA256SUMS'}
    if {path.name for path in directory.iterdir()} != expected:
        raise ValueError('Release directory must contain only the expected assets')
    for path in directory.iterdir():
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', path.name):
            raise ValueError('Unsafe release filename')
        if path.is_symlink() or not path.is_file() or not path.stat().st_size:
            raise ValueError('Release assets must be nonempty regular files')
    checksums = {}
    for line in (directory / 'SHA256SUMS').read_text(encoding='utf-8').splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9][A-Za-z0-9_.-]*)', line)
        if not match or match[2] in checksums:
            raise ValueError('Invalid or duplicate checksum entry')
        checksums[match[2]] = match[1]
    # A checksum listing cannot checksum itself; BUILD-INFO is included.
    if set(checksums) != expected - {'SHA256SUMS'}:
        raise ValueError('Each release asset must have exactly one checksum')
    if any(digest(directory / name) != value for name, value in checksums.items()):
        raise ValueError('Release checksum mismatch')
    record = json.loads((directory / 'BUILD-INFO.json').read_text(encoding='utf-8'))
    if record['version'] != version or not re.fullmatch(r'[0-9a-f]{40}', record['commit']):
        raise ValueError('Invalid release provenance')
    assets = record['assets']
    if len(assets) != len(names) or {asset['name'] for asset in assets} != names:
        raise ValueError('BUILD-INFO asset inventory mismatch')
    for asset in assets:
        if asset['sha256'] != checksums[asset['name']] or asset['bytes'] != (directory / asset['name']).stat().st_size:
            raise ValueError('BUILD-INFO size/hash mismatch')
    return record


def collect_assets(bundle, output, version, provenance):
    specs = asset_names(version)
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        raise ValueError('Collection requires an empty version-specific directory')
    assets = []
    for relative, name in specs:
        source = bundle / relative
        if source.is_symlink() or not source.is_file() or source.stat().st_size <= 100_000:
            raise ValueError(f'Missing or incomplete production bundle: {relative}')
        target = output / name
        shutil.copy2(source, target)
        assets.append({'name': name, 'bytes': target.stat().st_size, 'sha256': digest(target)})
    record = {**provenance, 'version': version, 'assets': assets, 'reproducible_build_certification': False}
    (output / 'BUILD-INFO.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    checksummed = [asset['name'] for asset in assets] + ['BUILD-INFO.json']
    (output / 'SHA256SUMS').write_text(''.join(f'{digest(output / name)}  {name}\n' for name in checksummed), encoding='utf-8')
    return verify_assets(output, version)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify', type=pathlib.Path, metavar='DIRECTORY', help='check final upload/download inventory without collecting')
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parent.parent
    version = json.loads((root / 'package.json').read_text(encoding='utf-8'))['version']
    if args.verify:
        record = verify_assets(args.verify, version)
    else:
        if json.loads((root / 'src-tauri/tauri.conf.json').read_text(encoding='utf-8'))['version'] != version:
            raise ValueError('Tauri/package version mismatch')
        def command(*argv):
            return subprocess.check_output(argv, cwd=root, text=True).strip()
        if command('git', 'status', '--porcelain'):
            raise ValueError('Release requires a clean tree')
        rustc = command('node', '--input-type=module', '-e',
                        "import {spawnSync} from 'node:child_process'; import {toolEnv} from './scripts/tool-env.mjs'; "
                        "const r=spawnSync('rustc',['--version'],{env:toolEnv(),encoding:'utf8',shell:false}); "
                        "if(r.status!==0)process.exit(1); process.stdout.write(r.stdout)")
        provenance = {'commit': command('git', 'rev-parse', 'HEAD'),
                      'node': command('node', '--version'), 'npm': command('npm', '--version'),
                      'rust': rustc, 'cargo': command('node', 'scripts/cargo.mjs', '--version'),
                      'tauri': command('node', 'scripts/tauri.mjs', '--version'),
                      'distribution': pathlib.Path('/etc/os-release').read_text(encoding='utf-8'),
                      'runner': {key: os.environ.get(key) for key in ('RUNNER_OS', 'RUNNER_ARCH', 'ImageOS', 'ImageVersion')}}
        record = collect_assets(root / 'target/release/bundle', root / 'target/release-assets' / version, version, provenance)
    print(json.dumps(record, indent=2))


if __name__ == '__main__':
    main()
