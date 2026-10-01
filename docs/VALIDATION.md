# Validation record

## v0.1.0 release preparation — 2026-10-01

Release baseline: `adec8b76e436a656080972925acb94a191957ea7`.
Ubuntu and Windows checks, including Windows Tauri compilation, were independently
confirmed green in Actions run36831439975. No new live task/authentication is part
of release preparation. The previously completed controlled Linux acceptance
proved authentication/session creation, three normal turns (inspection, same-session
follow-up, exact33-byte disposable write), local cancellation, owned-child shutdown
and lock release. After cancellation remote job state remained unknown; no health
turn was submitted. No source fix was needed. Acceptance workspace evidence remains
local and no account/job identifiers are published here.

The release keeps version0.1.0 across npm, Cargo, Tauri and the visible version label.
Changes are limited to first-run/error guidance, Linux package metadata, public
install/uninstall documentation and tag-only publication. Protocol/supervisor, CSP,
capabilities and dependency versions are unchanged.

Release evidence is recorded in `BUILD-INFO.json` and `SHA256SUMS` attached to the
release, including exact commit, tools, OS, filenames, sizes and hashes. Local
ignored evidence lives in `.cache/release-010`. CI builds from the exact tagged
commit on Ubuntu22.04; local and CI hashes may differ. No deterministic-build
certification is claimed. Package inspection/extraction does not certify root
package-manager transactions across every distribution. No host package install,
uninstall, system modification, credential retrieval or service call is required.

## Bridge integration — 2026-10-01

Starting Desktop main: `08d55bb9ec3df356946985b28822a4c8fd7325a4`.
This section supersedes earlier disabled-native/version-mismatch descriptions;
historical results below describe their original milestones.

### Current verified checks

- **27 frontend tests**, **17 Rust core/fixture tests** (10 original + 7 bridge),
  **4 registered Tauri IPC tests**, and **1 browser acceptance test** passed.
- TypeScript/frontend build, rustfmt, Rust check/Clippy, Tauri native check passed.
- The full local test/check/browser sequence ran with Bubblewrap `--unshare-net`.
  Browser non-loopback blocking remains intact. All authenticated turns used the
  deterministic fake bridge with synthetic credentials; no production endpoint.
- Rust BridgeSupervisor independently started the installed **JV CLI 0.4.9**, checked
  protocol 1 readiness and shut down in a network namespace. No auth request sent.
- Linux production executable and **deb/rpm/AppImage all built successfully**.
  An initial AppImage attempt encountered ETXTBSY while the test executable was
  running. Closing only our test window and rebuilding resolved it. No security
  permission, bundle configuration or system setting was loosened.
- No dependency versions or lockfiles changed. Generated artifacts stay ignored:
  `target/release/jv-desktop`, `target/release/bundle/deb/JV Desktop_0.1.0_amd64.deb`,
  `target/release/bundle/rpm/JV Desktop-0.1.0-1.x86_64.rpm`, and
  `target/release/bundle/appimage/JV Desktop_0.1.0_amd64.AppImage`.

### Native graphical validation

Existing X11 access worked without display configuration changes. Launched the
production executable with a network namespace and repository-local XDG/runtime
paths, leaving the owner's existing Desktop process untouched. GTK used software
rendering after a GL-context warning. In our PID-specific test window, verified:

- Main window/WebView and native folder dialog opened.
- Disposable repository-local workspace selected successfully.
- Thai `ไทย` entered using GTK Unicode composition; confirming composition did
  not submit. Shift+Enter added a newline, then Enter sent only an offline turn.
- Thai demo reply, command/activity and completion rendered; demo Stop worked.
- Workspace-write enabled tool networking; the checkbox toggled.
- Native mode reported detected; choosing the disposable directory opened the
  unauthenticated bridge and displayed the sign-in form. No credential entered.
- Closed our window cleanly. The installed CLI version was separately verified
  as 0.4.9; the exact version label was not independently read from accessibility.

This is not a live-auth/turn test. Owner-preferred Thai IME and native resize are
not newly validated here; automated IME and browser responsive checks pass.

### CI and security

Desktop remote Ubuntu/Windows CI could not be independently inspected: the private
repository API returned 404 and the existing GitHub connector was disconnected.
No credentials/configuration were changed to obtain access. Workflow now also runs
on feature/bridge-client; shared tests and Windows compile gate remain enabled.
Local Linux build/test gates passed. No new Windows runner success is claimed.

Only seven narrowly scoped application commands are granted. CSP is unchanged;
no shell, generic process, filesystem or unrestricted core permission was added.
IPC tests reject executable/environment injection and unauthenticated turns.
Passwords are memory-only via IPC/stdin; prompts never enter argv. Terminal state
is isolated from Desktop JVCLI_HOME. Stop reports local cancellation with remote
state unknown. Legacy exec gate remains closed and no argv fallback is provided.

JV CLI 0.4.9 was released from `b891df33365080af9ff587d69d74a434b79087ba`;
its full 377-test regression (3 skips), 46 bridge tests, package verification,
scripted engine smoke and release CI passed. Published normal/offline ZIP checksums
were verified. Official upgrade preserved installed settings/sessions/runtime.

**Zero production authentication attempts, zero real jobs/inference requests,
zero real passwords used.** No server/service change. The next functional Linux
integration step is separately authorized controlled live Desktop acceptance.

## Offline/native follow-up — 2026-09-30

Starting published commit: `b160011f152c2b0c68ba5c6c5d725563a83e8231`.
Local `main` and freshly fetched `origin/main` matched; the worktree was clean.
`RPALLMs/.git` is now absent. No workspace-root cleanup was performed here. All four siblings retained their
starting HEADs and clean status at the final local regression check.
This section supersedes the earlier environment blockers recorded below.

### Current results

- **24 frontend/tooling tests passed** (the original 21 plus 3 child PATH tests).
- **10 Rust process-core tests and 3 Tauri IPC tests passed**.
- TypeScript/frontend build, Rust fmt/core check/Clippy and Tauri native check pass.
- **Offline browser acceptance passed** with its existing non-loopback request
  blocking unchanged. It covers Thai conversation, mock Stop and compact layout.
- Native executable, Debian and RPM artifacts were inspected read-only using
  `file`; Debian control metadata was also inspected with `dpkg-deb --info`.
  No package was installed.
- **AppImage build passed** using `npm run tauri -- build --bundles appimage`.
  Artifact: `target/release/bundle/appimage/JV Desktop_0.1.0_amd64.AppImage`,
  **83,016,184 bytes**. It was identified as an x86-64 ELF AppImage runtime.
  The AppImage itself was not launched or installed; the native executable was.
  Nothing was uploaded or released.

### Actual native graphical evidence

`DISPLAY=:0`, X11, no Wayland variable. `xdpyinfo` successfully connected using
existing session access. No display security configuration was changed. The
production executable opened a viewable **JV Desktop** window and its React
content was inspected through the accessibility tree. Validation used ignored
repository-local XDG directories and a disposable `.cache/gui-validation/workspace`.
A local-only GIO VFS override was used after isolated runtime paths produced GVFS
warnings; no global setting was changed.

Verified in the real native application:

- Open Folder opened the native GTK chooser (also visually inspected).
- Selecting the disposable workspace succeeded; the selected directory remained empty.
- Selecting the user's home root returned the expected safe workspace error.
- Thai characters entered through GTK's **Ctrl+Shift+U Unicode composition**
  appeared intact. Enter confirming composition did not submit the conversation.
  This exercises a real composition path, not every Thai keyboard/IME implementation.
- Shift+Enter inserted a newline; a subsequent ordinary Enter sent the offline demo.
- Simulated Thai assistant output, command activity and turn completion rendered.
- Stop cancelled a second simulated turn and displayed the stopped message.

Remaining real-GUI cases: intentional resize, root/application-folder rejection,
permission-control interaction, owner-preferred Thai IME, native detected-but-disabled
screen and direct IPC from a real WebView. Root validation and the registered IPC
gate are covered by automated Rust tests; the IPC tests use Tauri MockRuntime,
not the real WebView. The visible workspace later changed to an owner-selected
path outside the disposable area, so automation stopped interacting with that
window. No test turn was sent against that selection. Follow the
[manual acceptance checklist](MANUAL_GUI_ACCEPTANCE.md) to finish those cases.

### CI diagnosis and correction

The baseline [Actions run](https://github.com/VMatee/jv-desktop/actions/runs/36666359733)
was inspected through the existing GitHub connector after an unauthenticated
API request returned 404 for this private repository:

- Ubuntu: **passed**, including browser acceptance.
- Windows: **failed at `npm run rust:fmt`**, reporting Cargo unavailable.
  Frontend tests/build passed; later Rust checks and Windows Tauri compilation
  were skipped.

The wrapper spread Windows' `Path` then assigned a separate `PATH` from an
undefined `env.PATH`, discarding the inherited Cargo lookup path for child
processes. The fix preserves the existing key spelling, removes duplicate
Windows spellings using Node's precedence, and prepends the local tool directory.
Three regression tests cover Windows Path, duplicate spellings and Unix behavior.
See [Node child-process environment behavior](https://nodejs.org/api/child_process.html).
No CI gate was removed or weakened. A new Windows runner result is still required
before claiming Windows native compilation passes.

### Unchanged security and integration boundary

Only the four application commands are granted to the main window; no general
shell/process/filesystem or unrestricted core capability was added. Production
CSP and `native_turn_gate` remain unchanged. Direct registered IPC tests still
return `authentication-required` for every valid policy, reject executable
injection and deny unused core commands. Rust remains the process boundary.

Installed local-only `jvcli --version`: **0.4.7**. Source `jvcli/VERSION`: **0.4.8**.
Neither changed. A later milestone must choose and verify one compatibility
baseline, then separately review prompt transport, authentication and cancellation
before allowing real turns. No bridge, password UX, resume, terminal or Windows
backend was implemented.

**Zero real JV jobs and zero production inference requests. No JV credential was
used. No credential or private-key contents were inspected or persisted.** Existing
GitHub connector/SSH authentication was used only for authorized repository work;
no authentication configuration was changed.

## Previous run: native Linux build validation — 2026-09-30

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
