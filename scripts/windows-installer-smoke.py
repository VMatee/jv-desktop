#!/usr/bin/env python3
"""Offline NSIS install/reinstall/WebView/uninstall checks on a clean user profile.

No CLI installation, authentication, project changes, or production requests.
Native picker/title-bar actions are a separate guided acceptance, not simulated here.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time
import winreg

ROOT = Path(__file__).resolve().parents[1]
KEY = r'Software\Microsoft\Windows\CurrentVersion\Uninstall\JV Desktop'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def registration():
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, KEY) as key:
            return winreg.QueryValueEx(key, 'DisplayVersion')[0]
    except FileNotFoundError:
        return None


def webview_version():
    """Read only Microsoft's documented public runtime version registry values."""
    suffix = r'Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    for hive, prefix in [(winreg.HKEY_LOCAL_MACHINE, 'SOFTWARE\\WOW6432Node\\'),
                         (winreg.HKEY_CURRENT_USER, 'SOFTWARE\\')]:
        try:
            with winreg.OpenKey(hive, prefix + suffix) as key:
                value = winreg.QueryValueEx(key, 'pv')[0]
                if isinstance(value, str) and re.fullmatch(r'[0-9]+(?:\.[0-9]+){3}', value) and value != '0.0.0.0':
                    return value
        except FileNotFoundError:
            pass
    return 'not detected'


def main():
    if os.name != 'nt':
        raise SystemExit('Native Windows required')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('installer', type=Path)
    args = parser.parse_args()
    if registration() is not None:
        raise SystemExit('Use a clean Windows user without an existing Desktop installation')
    installer = args.installer.resolve(strict=True)
    version = json.loads((ROOT / 'package.json').read_text(encoding='utf-8'))['version']
    cache = ROOT / '.cache/installer-smoke'
    cache.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix='run-', dir=cache))
    install = run / 'application'
    app = install / 'jv-desktop.exe'
    uninstaller = install / 'uninstall.exe'
    state = Path(os.environ['LOCALAPPDATA']) / 'com.openjvspace.desktop'
    state.mkdir(parents=True, exist_ok=True)
    descriptor, canary_name = tempfile.mkstemp(prefix='installer-acceptance-', suffix='.txt', dir=state)
    os.close(descriptor)
    canary = Path(canary_name)
    canary.write_text('synthetic state outside application binaries\n', encoding='utf-8')
    expected_canary = sha(canary)
    cli = Path(os.environ['LOCALAPPDATA']) / 'Programs/JV CLI/jvcli.exe'
    expected_cli = sha(cli) if cli.is_file() else None
    results = []

    def check(name, condition):
        if not condition:
            raise AssertionError(name)
        results.append(name)
        print('PASS', name, flush=True)
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(f'::notice title=Installed package assertion::{name}: PASS', flush=True)

    env = {k: os.environ[k] for k in ('SYSTEMROOT', 'WINDIR', 'USERPROFILE', 'LOCALAPPDATA', 'APPDATA', 'TEMP', 'TMP') if k in os.environ}
    env['PATH'] = str(Path(os.environ['SYSTEMROOT']) / 'System32')
    for attempt in range(2):
        subprocess.run([str(installer), '/S', f'/D={install}'], env=env, check=True, timeout=180)
        check('install' if attempt == 0 else 'same-version reinstall', app.is_file() and uninstaller.is_file() and registration() == version)
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            print(f'::notice title=WebView2 prerequisite::Runtime version after install: {webview_version()}', flush=True)
        subprocess.run(['node', 'scripts/windows-webview-acceptance.mjs', str(app), '--package-smoke'], cwd=ROOT, check=True, timeout=90)
        check('installed WebView launch' if attempt == 0 else 'reinstalled WebView launch', True)
        check('separate state preserved' if attempt == 0 else 'reinstall state preserved', sha(canary) == expected_canary)
    subprocess.run([str(uninstaller), '/S'], env=env, check=True, timeout=180)
    deadline = time.monotonic() + 30
    while (app.exists() or registration() is not None) and time.monotonic() < deadline:
        time.sleep(0.1)
    check('uninstall removes application and registration', not app.exists() and registration() is None)
    check('uninstall preserves separate state', sha(canary) == expected_canary)
    check('external JV CLI untouched', (sha(cli) if cli.is_file() else None) == expected_cli)
    (run / 'results.json').write_text(json.dumps({'version': version, 'installer_sha256': sha(installer), 'results': results, 'skips': 0, 'production_calls': 0}, indent=2) + '\n', encoding='utf-8')
    # Remove only the synthetic file created by this test, never state directories.
    if sha(canary) != expected_canary:
        raise AssertionError('Synthetic state changed before cleanup')
    canary.unlink()
    print(f'{len(results)} installer assertions passed, zero skips', flush=True)


if __name__ == '__main__':
    main()
