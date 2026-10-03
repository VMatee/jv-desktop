#!/usr/bin/env python3
"""Collect native bundles, combine same-commit builds, and verify public bytes."""
import argparse
import hashlib
import json
import os
import pathlib
import platform as host
import re
import shutil
import subprocess


def asset_names(version, platform='linux'):
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version):
        raise ValueError('Release version must be major.minor.patch')
    linux = [
        (f'deb/JV Desktop_{version}_amd64.deb', f'JV-Desktop_{version}_amd64.deb'),
        (f'rpm/JV Desktop-{version}-1.x86_64.rpm', f'JV-Desktop-{version}-1.x86_64.rpm'),
        (f'appimage/JV Desktop_{version}_amd64.AppImage', f'JV-Desktop_{version}_amd64.AppImage'),
    ]
    windows = [(f'nsis/JV Desktop_{version}_x64-setup.exe', f'JV-Desktop_{version}_x64-setup.exe')]
    if platform == 'linux':
        return linux
    if platform == 'windows':
        return windows
    if platform == 'all':
        return linux + windows
    raise ValueError('Unsupported release platform')


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def verify_assets(directory, version, platform='all'):
    """Require exactly the intended upload inventory and checksum every other file."""
    names = {name for _, name in asset_names(version, platform)}
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
    if (record['version'] != version or record.get('platform') != platform
            or not re.fullmatch(r'[0-9a-f]{40}', record['commit'])):
        raise ValueError('Invalid release provenance')
    assets = record['assets']
    if len(assets) != len(names) or {asset['name'] for asset in assets} != names:
        raise ValueError('BUILD-INFO asset inventory mismatch')
    for asset in assets:
        if asset['sha256'] != checksums[asset['name']] or asset['bytes'] != (directory / asset['name']).stat().st_size:
            raise ValueError('BUILD-INFO size/hash mismatch')
    if platform == 'all':
        builds = record.get('builds', {})
        if set(builds) != {'linux', 'windows'}:
            raise ValueError('Both native build records are required')
        for name, build in builds.items():
            if build.get('commit') != record['commit'] or build.get('version') != version or build.get('platform') != name:
                raise ValueError('Native builds must represent the same source/version')
            native_names = {filename for _, filename in asset_names(version, name)}
            if build.get('assets') != [asset for asset in assets if asset['name'] in native_names]:
                raise ValueError('Native and combined asset records disagree')
        if builds['windows'].get('signing') != 'unsigned':
            raise ValueError('This release requires explicitly unsigned Windows metadata')
    return record


def write_metadata(output, record):
    (output / 'BUILD-INFO.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8', newline='\n')
    checksummed = [asset['name'] for asset in record['assets']] + ['BUILD-INFO.json']
    (output / 'SHA256SUMS').write_text(''.join(f'{digest(output / name)}  {name}\n' for name in checksummed), encoding='utf-8', newline='\n')


def empty_output(output):
    output.mkdir(parents=True, exist_ok=True)
    if any(output.iterdir()):
        raise ValueError('Collection requires an empty version-specific directory')


def collect_assets(bundle, output, version, provenance, platform='linux'):
    if platform == 'all':
        raise ValueError('Collect each platform natively; combine verified outputs afterward')
    specs = asset_names(version, platform)
    empty_output(output)
    assets = []
    for relative, name in specs:
        source = bundle / relative
        if source.is_symlink() or not source.is_file() or source.stat().st_size <= 100_000:
            raise ValueError(f'Missing or incomplete production bundle: {relative}')
        target = output / name
        shutil.copy2(source, target)
        assets.append({'name': name, 'bytes': target.stat().st_size, 'sha256': digest(target)})
    record = {**provenance, 'version': version, 'platform': platform, 'assets': assets,
              'signing': 'unsigned', 'reproducible_build_certification': False}
    write_metadata(output, record)
    return verify_assets(output, version, platform)


def combine_assets(linux, windows, output, version, expected_commit):
    builds = {name: verify_assets(path, version, name) for name, path in [('linux', linux), ('windows', windows)]}
    if any(build['commit'] != expected_commit for build in builds.values()):
        raise ValueError('Refuse mixed or unexpected source commits')
    empty_output(output)
    assets = []
    for name, directory in [('linux', linux), ('windows', windows)]:
        for asset in builds[name]['assets']:
            shutil.copy2(directory / asset['name'], output / asset['name'])
            assets.append(asset)
    record = {'version': version, 'commit': expected_commit, 'platform': 'all',
              'builds': builds, 'assets': assets, 'reproducible_build_certification': False}
    write_metadata(output, record)
    return verify_assets(output, version)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verify', type=pathlib.Path, metavar='DIRECTORY', help='check final upload/download inventory without collecting')
    parser.add_argument('--platform', choices=['linux', 'windows', 'all'])
    parser.add_argument('--combine', nargs=2, type=pathlib.Path, metavar=('LINUX', 'WINDOWS'))
    parser.add_argument('--output', type=pathlib.Path)
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parent.parent
    version = json.loads((root / 'package.json').read_text(encoding='utf-8'))['version']
    if args.verify:
        record = verify_assets(args.verify, version, args.platform or 'all')
    elif args.combine:
        commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
        record = combine_assets(*args.combine, args.output or root / 'target/release-assets' / version, version, commit)
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
        platform = args.platform or ('windows' if os.name == 'nt' else 'linux')
        if platform != ('windows' if os.name == 'nt' else 'linux'):
            raise ValueError('Production bundles must be collected on their native platform')
        npm_version = command('node', str(pathlib.Path(shutil.which('node')).parent / 'node_modules/npm/bin/npm-cli.js'), '--version') if os.name == 'nt' else command('npm', '--version')
        provenance = {'commit': command('git', 'rev-parse', 'HEAD'),
                      'node': command('node', '--version'), 'npm': npm_version,
                      'rust': rustc, 'cargo': command('node', 'scripts/cargo.mjs', '--version'),
                      'tauri': command('node', 'scripts/tauri.mjs', '--version'),
                      'distribution': host.system() + ' ' + host.release(),
                      'target': 'x86_64-pc-windows-msvc' if platform == 'windows' else 'x86_64-unknown-linux-gnu',
                      'runner': {key: os.environ.get(key) for key in ('RUNNER_OS', 'RUNNER_ARCH', 'ImageOS', 'ImageVersion')}}
        record = collect_assets(root / 'target/release/bundle', args.output or root / 'target/release-assets' / f'{version}-{platform}', version, provenance, platform)
    print(json.dumps(record, indent=2))


if __name__ == '__main__':
    main()
