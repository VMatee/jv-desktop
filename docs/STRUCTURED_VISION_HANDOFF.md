# Structured vision: Windows and coordinated release handoff

Linux Phase 1 uses `feature/structured-vision-linux` in the independent
`VMatee/jv-cli` and `VMatee/jv-desktop` repositories. Obtain exact published SHAs
from the Phase 1 report and compare them with fetched refs. No release version,
tag or installer publication is authorized by this phase. Development packages
still carry CLI 0.5.2 and Desktop 0.2.2; they are not new official releases.

## Start on the Windows PC

Verify the execution host and these historical paths before using them:

- `C:\Users\Matee\Desktop\NotScan\Work\jv-desktop`
- `C:\Users\Matee\Desktop\NotScan\Work\jvcli-window\jv-cli`

Read applicable AGENTS.md files. Inspect each repository's status, staged and
unstaged diffs, untracked files, branches, remotes, HEAD, upstream and history.
Save bundles, binary patches and untracked source outside the repositories.
Create rollback refs. Fetch only after inspection; compare the Linux branch with
local Windows history. Integrate on a new feature branch, preserving both trees.
Do not blindly pull, reset, clean, replace branches, or force-push.

## Native CLI work and tests

Use the existing repository instructions and Python/Node prerequisites. Inspect
script help/source for paths before executing these PowerShell commands from CLI:

```powershell
py -3.10 -B scripts/windows_preflight.py
py -3.10 -B scripts/windows_native_tests.py
py -3.10 -B scripts/windows_engine_check.py --engine ABSOLUTE_ENGINE_EXE
py -3.10 -B scripts/windows_trusted_acceptance.py --engine ABSOLUTE_ENGINE_EXE
py -3.10 -B scripts/windows_bridge_acceptance.py --engine ABSOLUTE_ENGINE_EXE
py -3.10 -B scripts/build_windows.py
py -3.10 -B scripts/windows_package_acceptance.py --installer ABSOLUTE_INSTALLER_EXE --portable ABSOLUTE_PORTABLE_ZIP --node ABSOLUTE_NODE_EXE
```

Keep pinned engine 0.149.1, Windows trusted-host semantics, credential isolation,
pipe cancellation and process ownership intact. Do not weaken OS security.
Port the Linux synthetic browser/image acceptance using native Windows commands
and pipes; the current `vision_bridge_smoke.py` intentionally rejects Windows.
Test PNG/JPEG/WebP `view_image`, structured byte validation, subsequent file edits
and commands, completion, restart/resume, cancellation, unsupported provider
(no automatic resend), explicit fresh Legacy fallback and genuine bridge exit.
Ensure logs/UI frames contain neither credentials nor image bytes. Use synthetic
accounts only; do not replay the original Snake Game or consume production jobs.

## Desktop checks and real GUI acceptance

Install/locate the just-validated CLI first, verify `jvcli --version` and its
repository-documented diagnostics. Native Desktop launches the CLI's documented
bridge command with protocol 1 and Windows trusted-host options; do not replace
it with Linux command assumptions. Build from the integrated Desktop source:

```powershell
npm ci
npm test
npm run build
npm run rust:fmt
npm run rust:check
npm run rust:test
npm run rust:clippy
npm run tauri:check
npm run tauri:test
npm run test:e2e
py -3.10 -B -m unittest discover -s tests -p 'test_release*.py' -v
py -3.10 -B scripts/windows-native-smoke.py
npm run tauri -- build --no-bundle
node scripts/windows-webview-acceptance.mjs "$PWD\target\release\jv-desktop.exe"
npm run tauri -- build --bundles nsis
```

Run the repository's installer smoke on a clean test registration and its guided
native GUI driver, as documented in DEVELOPMENT.md. An installer built is not an
installer tested. Record integrity hashes, installed paths/version, reinstall,
uninstall and CLI compatibility. Never package `offline-fixture` builds.

Manual GUI verification with synthetic loopback service:

1. Launch the installed production Desktop; select Native JV CLI and Check again.
2. Open a disposable workspace via the Windows native picker.
3. Choose Structured session and permitted trusted-host policy. Sign in with only
   synthetic credentials and the fixture URL.
4. Generate/build/test a small website, capture screenshots, inspect all three
   formats, then verify a follow-up edit and tests finish in the same session.
5. Send another turn; verify continuity and usable controls. Exercise Stop.
6. Return an unsupported-service error: verify the support code, failed-turn
   message, live bridge and no automatic Legacy submission. Reopen and explicitly
   choose Legacy to verify fallback.
7. Separately cause a fixture-owned bridge process exit and verify the connection
   failure UI differs from a failed turn. Verify native window close stops owned
   processes and no secret/image payloads appear in application logs.

A scripted provider proves transport and execution, not real vision-model
acceptance. Record that separately; production inference requires its own scope.

## Synchronize and release gates

After native checks pass, review and commit Windows/shared fixes and push normal
feature branches. Record exact SHAs in each independent repo. On Linux, preserve
local work, fetch and review divergence, integrate the Windows commits and rerun
CLI tests, real-engine/synthetic vision acceptance, Desktop frontend/Rust/Tauri/
browser/native GUI checks and Linux packaging against the intended final source.
If source changes after either platform's acceptance, rerun affected checks.

Verify CLI 0.5.3 and Desktop 0.2.3 remain available against actual GitHub tags and
releases before proposing version changes. Follow each repository's coordinated
release gates and required CI. Match Windows/Linux source revisions within each
repository and preserve the tested CLI/Desktop pairing. Release/install CLI first,
then test its corresponding Desktop release. Do not publish while Linux regression,
Windows installer/GUI acceptance, CI or release gates remain incomplete.
