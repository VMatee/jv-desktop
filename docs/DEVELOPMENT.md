# Development

## Prerequisites

- Node 24 LTS, **24.15.0 or newer**; validated with 24.21.0 and its bundled npm.
- Rust **1.98 or newer**, Cargo, rustfmt and Clippy; validated with 1.98.1.
- A C/C++ linker for the host when compiling Rust applications.
- Native desktop builds additionally need the
  [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).
  Linux needs GTK 3, WebKitGTK 4.1, JavaScriptCoreGTK 4.1, libsoup 3 and D-Bus
  development metadata/libraries. Windows needs the MSVC C++ build tools,
  Windows SDK and WebView2 runtime for launching the app.

The development host initially had Node 20.20.2 and Rust 1.75.0. Rather than
changing global tools, this foundation used Node 24.21.0 under `.tools/node` and
Rust 1.98.1 under `.tools/rustup`, with Cargo shims/cache under `.cache/cargo`.
Those ignored local artifacts are not required to be committed or copied.

If this checkout has that local Node installation, prefix commands as follows:

```sh
PATH="$PWD/.tools/node/bin:$PATH" npm ci
PATH="$PWD/.tools/node/bin:$PATH" npm run dev
```

That prefix affects only the child command. Do not modify shell startup files or
global PATH. The scripts detect the local Rust installation when present. On a
normal contributor machine, an already-installed compatible Rust toolchain is
also supported. Do not install system packages or global toolchains as part of
automated Desktop work; document missing host prerequisites instead.

On Windows with repository-local Node at `.tools/node`, use this process-local
PowerShell setup (MSVC/Windows SDK and WebView2 must already be available):

```powershell
$env:PATH = "$PWD\.tools\node;$env:PATH"
npm ci
npm run tauri -- dev
```

The Windows Rust shims are under `.cache/cargo/bin` and the toolchain under
`.tools/rustup`; the existing wrapper selects them without changing global tools.

## Install and run offline

From the repository root:

```sh
npm ci
npm run dev
```

This is Vite's loopback development server, not an application server or new JV
API. The product itself bundles static frontend assets in Tauri. No code in this
repository implements a web backend.

The browser opens in **Offline demo**. **Open Folder** explicitly selects a
virtual `/demo/workspace`; send any non-secret sample message to see a simulated
command and Thai response. Stop cancels its timer stream. The demo neither reads
files nor invokes any executable. Browser native mode reports unavailable.

On a host with Tauri GUI prerequisites:

```sh
npm run tauri -- dev
```

The native shell uses the real folder picker for both modes. Selecting a folder
does not edit it. Native mode detects `jvcli --version` once when selected or on
**Check again**, with a three-second timeout. CLI 0.4.9 or newer is required. Native folder selection starts a bridge without
authentication. Do not use Sign in during ordinary development: use Offline demo
and fake bridge tests. Live acceptance requires separate authorization.

## Checks and what they prove

| Command | Scope |
| --- | --- |
| `npm test` | JSONL mapping, limits, malformed data, Unicode, permissions, mock lifecycle, IME and UI states |
| `npm run typecheck` | Strict TypeScript checking |
| `npm run build` | TypeScript plus production frontend assets, not a desktop bundle |
| `npm run rust:fmt` | Rust formatting across both crates |
| `npm run rust:check` | Standalone process core, no GUI libraries |
| `npm run rust:test` | Core and shared fake-executable tests; native Linux process groups or Windows Jobs/pipes |
| `npm run rust:clippy` | Core and fixture linting, warnings denied |
| `npm run browser:install` | Download a repository-local Chromium headless shell; no OS packages |
| `npm run test:e2e` | Offline browser workflow, Stop, native blocker and responsive layout |
| `npm run tauri:check` | Compile/type-check native Tauri code; requires GUI development libraries |
| `npm run tauri:test` | Linux/Windows registered IPC/ACL safety using MockRuntime; no display or real CLI |
| `npm run tauri -- build --bundles deb,rpm` | Production Linux executable plus Debian/RPM packages, no installation or publication |
| `npm run tauri -- build --no-bundle` | Compile native executable; requires full host prerequisites |
| `npm run tauri -- build --bundles nsis` | Unsigned development Windows executable/installer; no installation or publication |

The host now passes native compilation, Debian/RPM/AppImage packaging and
offline browser acceptance. X11 access is available and the native shell has
been exercised. See [Validation](VALIDATION.md) for exact coverage and
[Manual acceptance](MANUAL_GUI_ACCEPTANCE.md) for the remaining GUI cases.
Do not change system configuration or disable security controls for validation.

All Cargo scripts put downloads in `.cache/cargo`, temporary files in `.cache/tmp`
and compiled artifacts in `target`. npm uses `.cache/npm`. Browser downloads live
in `.cache/ms-playwright`, screenshots in `test-results`. These are ignored.
Never stage them, account files, `.env` files or private test evidence.
Tauri bundler tools use `target/.tauri` (`useLocalToolsDir=true`), keeping Windows
NSIS/WiX build downloads inside the checkout as well.

## Fake executable

`crates/jv-process/tests/fixtures/fake_jvcli.rs` builds as `fake-jvcli` only with
the `test-fixture` feature. Cargo integration tests obtain its absolute path via
`CARGO_BIN_EXE_fake-jvcli`; they never discover or run a real installed CLI.

The fixture accepts `--version`, or a synthetic argument vector containing
`--fixture SCENARIO` and a prompt on stdin. Supported scenarios include success,
command-fails, malformed, unknown, oversized, invalid-utf8, nonzero, slow, tree,
no-stdin, flood, flood-no-newline, stderr-flood and inspect. Success includes a changed file and Thai
assistant response. Inspect echoes only synthetic test argv/cwd/prompt and the
names of the minimal child environment keys. The fixture has no network client.
This older fixture remains for runner regression. New `fake-bridge` uses the
public protocol 1 shape and synthetic auth/session/turn/cancel/shutdown responses.
It verifies workspace restart, older CLI rejection, broken pipe, malformed and
oversized output, failures and Unicode. It never uses a network client.

Frontend tests reject fetch/XHR. Browser checks abort non-loopback requests before
they can be sent and assert none were requested. Rust tests create only ignored
repository-local workspace fixtures. No JV account, real password, token or live
scenario is needed. Do not use a production task as a build or publication check.

## CI

GitHub Actions runs frontend, process core, bridge and registered Tauri IPC checks
on Ubuntu and Windows. Windows runs real native fixture Job/pipe lifecycle tests
and builds the actual Desktop executable. Linux retains process-group tests,
offline release-asset tests and browser acceptance, with Tauri libraries installed
only on the ephemeral runner. Actions
use pinned revisions, a read-only repository token with credential persistence
disabled, and no JV credentials/deploy keys. The tag-only Linux release workflow requires successful main checks, builds on
Ubuntu 22.04, and publishes packages/checksums using the scoped GitHub token.
It does not install JV CLI or make JV service calls.

## Before publishing

Inspect the worktree, staged files, untracked files and both lockfiles. Run all
available checks and record GUI environment blockers honestly. Fetch the intended
origin, inspect divergence and push normally only if histories are compatible.
Never reset siblings or overwrite unexpected remote commits. Re-check workspace
isolation against the initial sibling status/HEAD record.

## Optional installed bridge compatibility probe (no authentication)

This is an explicit manual probe, separate from automated fixture tests. With a
trusted installed CLI>=0.4.9, create a disposable directory and isolated state:

```sh
mkdir -p .cache/bridge-probe/workspace
node scripts/cargo.mjs run -p jv-desktop-process --example bridge_probe --locked -- \
  "$PWD/.cache/bridge-probe/workspace" "$PWD/.cache/bridge-probe/state"
```

The example resolves the fixed `jvcli` name, checks version/readiness and sends
shutdown only. It has no login/turn code. Use a network namespace for independent
offline assurance. Never add credentials to this probe.

## Release preparation

Keep package.json, Cargo workspace, lockfiles and Tauri version consistent. Run
the documented checks, commit validated source, then build all Linux bundles from
that clean commit. `python3 scripts/release-assets.py` copies the three bundles to
canonical filenames without spaces, plus SHA256SUMS and BUILD-INFO.json, under
ignored `target/release-assets/<version>/`. Collection requires an empty output
directory and preserves earlier versions. SHA256SUMS covers all bundles and
BUILD-INFO.json; it cannot checksum itself. Its tool
commands need the same Node PATH as the build. No reproducible-build certification
is implied. CI builds independently from the exact tag; published hashes identify
those runner artifacts and can differ from local acceptance bundles.

Run `python3 -B -m unittest discover -s tests -p 'test_release*.py' -v` for the
offline collection/download round-trip and filename/integrity rejection cases.
Before upload, `python3 -B scripts/release-assets.py --verify DIRECTORY` checks
the exact inventory, safe filenames, nonempty files, unique checksums and
BUILD-INFO sizes/hashes. `sha256sum -c SHA256SUMS` must also pass in that directory.
The tag workflow derives its version from package.json, requires matching main
and green main CI, refuses an existing release, and verifies two fresh downloads
without renaming files. Published SHA256SUMS must byte-match the generated file.

Inspect/extract packages without modifying the development machine's package
database. Test extracted executable/AppImage with isolated XDG paths and no network.
Package-manager install/remove instructions are user operations, not automated
permission to use sudo on a developer machine. No custom install/remove hooks or
user-state deletion are included. Close the app before upgrading/removing it.

## Windows native acceptance (offline)

The accepted external CLI remains separate. Ordinary tests never discover it.
To explicitly smoke-test the installed CLI through Desktop's Rust process layer:

```powershell
py -3.10 -B scripts/windows-native-smoke.py
```

This uses native Jobs, strict child environment, disposable Desktop-specific
state and a synthetic loopback HTTP 401 fixture. It verifies local readiness,
shutdown, owner death and state protection, without real account/token/job or
inference. It does not test production authentication. Do not change the CLI
repository or use an unverified installer to satisfy this test.

Actual Tauri WebView acceptance uses the existing Playwright dependency and
process-local loopback WebView2 debugging; no system browser install is required:

```powershell
npm run tauri -- build --no-bundle
node scripts/windows-webview-acceptance.mjs "$PWD\target\release\jv-desktop.exe"
```

For synthetic sign-in and turns, build the explicit acceptance-only feature:

```powershell
npm run rust:test
npm run tauri -- build --no-bundle --features offline-fixture
# Copy this executable and target\debug\fake-bridge.exe to an ignored
# .cache\gui\fixture-app directory before running the acceptance driver.
node scripts/windows-webview-acceptance.mjs "$PWD\.cache\gui\fixture-app\jv-desktop.exe" --fixture
```

`offline-fixture` is off by default and never selects an executable from frontend
input or an environment override. Rust fixes the sibling fake-bridge and the
disposable folder; a missing fixture never falls back to the installed CLI.
Rebuild without the feature before testing/distributing the real application.

The driver checks the actual bundled WebView and Rust IPC. It does not automate
the native folder dialog or window-manager close button. Script window close did
not exit the application; terminating it after cooperative bridge shutdown is
not counted as native close acceptance. Use the native GUI checklist separately
when the automation surface is available. Keep all test state/output ignored.
