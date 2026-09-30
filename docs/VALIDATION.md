# Validation record

## Milestone 2: native Linux build validation — 2026-09-30

Starting commit: `42f46531efe912c8c78304a1b5087b7d2b98721e`, clean `main`.
This is **offline development verification**, not production acceptance.
**Zero real JV jobs were submitted.** No JV password, token, account/session
configuration or private-key content was read or persisted. The unchanged
`native_turn_gate` remains unconditionally closed.

### Host prerequisites

The owner installed the native development dependencies before this milestone.
No OS package installation or global configuration changes were performed here.
`pkg-config --modversion` now succeeds for every requested prerequisite:

| Metadata | Detected version |
| --- | --- |
| gtk+-3.0 | 3.24.33 |
| webkit2gtk-4.1 | 2.50.4 |
| javascriptcoregtk-4.1 | 2.50.4 |
| libsoup-3.0 | 3.0.7 |
| dbus-1 | 1.12.20 |

The foundation's missing D-Bus/GTK/WebKit development-library blocker is resolved.

### Checks and evidence boundaries

| Check | Milestone 2 result |
| --- | --- |
| Frontend unit/component tests | **21 passed**, including Thai/Unicode, IME, mock activity/Stop, permissions and native unavailable UI |
| TypeScript + production frontend build | **Passed** |
| Rust core unit/integration tests | **10 passed** (4 unit + 6 Linux fake-executable integration) |
| Rust formatting/core check/Clippy | **Passed**, Clippy warnings denied |
| Tauri native check | **Passed**: `npm run tauri:check` |
| Tauri IPC boundary tests | **3 passed**: `npm run tauri:test`, Linux Tauri MockRuntime with the actual generated capability context |
| Tauri crate Clippy | **Passed**: `node scripts/cargo.mjs clippy -p jv-desktop --all-targets --locked -- -D warnings` |
| Offline Chromium browser acceptance | **Blocked in this session** before the test: loopback connection to `127.0.0.1:1420` returns `EPERM`; Vite cannot start under the sandbox restrictions |
| Native production executable | **Built**, optimized x86-64 ELF |
| Debian and RPM bundles | **Built**, final `npm run tauri -- build --bundles deb,rpm` |
| AppImage | **Blocked**, downloading Tauri's `AppRun-x86_64` helper failed with DNS resolution unavailable |
| Actual native window/WebView launch | **Attempted, unsuccessful**: GTK initialization failed, exit 101 |
| Production/live JV validation | **NOT PERFORMED**, intentionally forbidden |

The original browser acceptance test passed during foundation work. It was
rerun unchanged here and could not reach its test body. That historical success
is not a new browser pass and does not establish native WebView behavior.

New IPC tests exercise the registered `run_turn` command for every valid
permission policy and assert `authentication-required`, despite a selected valid
repository-local workspace. They also verify invalid root selection is rejected,
a frontend-supplied executable field is rejected, cancellation remains idempotent,
and an ungranted core command is denied. They do not discover an installed CLI,
launch a WebView, open a folder dialog or invoke a live task.

The test WebView's data directory is explicitly repository-local. An initial test
attempt exposed Tauri's default external data-directory creation and was rejected
by the read-only filesystem; the harness now supplies an ignored test directory.
No production gate or assertion was weakened.

### Generated artifacts

All paths below are relative to this repository and ignored by Git:

- `target/release/jv-desktop`
- `target/release/bundle/deb/JV Desktop_0.1.0_amd64.deb`
- `target/release/bundle/rpm/JV Desktop-0.1.0-1.x86_64.rpm`

The initial all-bundle build completed executable/Debian/RPM generation before
failing at the AppImage download. A subsequent explicit Debian/RPM build covers
the final capability changes. No package was installed, uploaded or released.
No AppImage success is claimed. Build success does not prove package installation
on another Linux distribution.

### Native runtime limitation and remaining acceptance

The session advertises X11 `DISPLAY=:0`, but connecting to its X11 socket returns
`EPERM`. An isolated executable launch failed to initialize GTK. Its XDG data,
configuration, cache and runtime paths were redirected under `.cache`; user X11
authentication material was not read. No system settings, display authorization,
WebKit sandbox settings or CSP were relaxed. No window was confirmed open.

The following still require a separately available graphical session:

1. Start the built application in Offline demo; confirm the main window and React
   content render in the real WebView.
2. Open the native folder dialog; choose a disposable directory under this
   checkout's ignored `.cache` area. Confirm selecting it creates no project files.
3. Select an invalid workspace such as filesystem root and confirm a safe error.
4. Enter Thai text with an actual IME; composition must not submit, Shift+Enter
   must add a newline, and Enter must send only after composition finishes.
5. Exercise read-only/write/network controls, offline conversation, activity,
   changed-file display and Stop.
6. Select native mode and confirm detected-but-disabled if the trusted installed
   CLI is present. Do not provide credentials or attempt a real task.

Native folder picking, WebView Thai/IME, native mock conversation/activity/Stop
and the detected-but-disabled native screen are **not yet exercised**. Unit tests
cover their shared logic; the registered Rust command gate is tested separately.

### Capability and production review

The main window grants only the four application commands. Unused `core:default`
was removed. There is no generic shell/process/filesystem permission, remote
capability origin or JavaScript dialog grant. Folder selection is Rust-owned.
Frontend JavaScript cannot select an executable. The dialog dependency's
transitive filesystem crate does not authorize frontend file access. Production
CSP is unchanged; no remote script/network permission was added.

Installed `jvcli --version` reports **0.4.7** using a cleared, minimal child
environment. The independent source `jvcli/VERSION` reports **0.4.8**. Neither was
changed. These versions are not assumed machine-interface compatible. See
[CLI contract](CLI_CONTRACT.md). Prompt-over-argv and password handling remain
unimplemented; real integration requires separately reviewed version, stdin/stdio,
authentication and cancellation contracts. No PTY or terminal workaround exists.

### CI and repository isolation

The existing workflow was inspected: Ubuntu/Windows frontend and process-core
checks, Linux browser acceptance, Windows Tauri compilation, read-only repository
permissions and no JV credentials, production calls or release job. No obvious
workflow configuration issue was found. The new Linux IPC test needs native
libraries and is currently a local check, not a Linux CI step.

An unauthenticated query for baseline GitHub Actions runs failed to resolve
`api.github.com`; remote CI status could not be independently verified.

Sibling HEAD/status snapshots are stored only inside this repository's `.git`
for final comparison. All four sibling repositories retain their initial HEAD
and clean tracked/staged/untracked state after validation. Workspace-root exception: `RPALLMs/.git` was already an
empty directory when this milestone began. RPALLMs is **not a valid Git
repository**, but literal `.git` absence cannot be claimed. That directory was
left untouched because it is outside the authorized modification boundary.

## Toolchain and foundation coverage

Repository-local Node 24.21.0 and Rust/Cargo 1.98.1 were used; global tools were
not changed. Package versions remain React/React DOM 19.3.0, TypeScript 7.0.2,
Vite 8.3.1, React Vite plugin 6.1.1, Vitest 5.0.2, Tauri API/CLI/Rust 2.12.0,
tauri-build 2.7.0, dialog 2.8.0 and Playwright 1.63.0. Committed lockfiles retain
the exact dependency resolutions.

The unchanged process-core suite uses its fake executable for argv/cwd/stdin
isolation, Unicode, structured activity, malformed/unknown/oversized events,
invalid UTF-8, stream limits, failures, detection, timeout and owned Linux
process-tree cancellation. This does not prove cancellation of the real CLI's
detached engine or a remote job. No native Windows backend, WSL automation,
account persistence or production release is implemented.
