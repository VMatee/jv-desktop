# Validation record

## Desktop 0.2.1 dependency patch — 2026-10-03

Starting Desktop main: `153de47a93287c704e3b4849eb9a416976d4770a` (0.2.0).
This patch requires official CLI **0.5.0+ on Windows**, retaining **0.4.9+ on
Linux**, in both local detection and bridge.ready validation. BackendStatus
provides a structured minimumCliVersion; unavailable/upgrade UI names the
appropriate requirement and official download location. Execution policy,
protocol 1 and the external-runtime architecture are unchanged.

The [public JV CLI 0.5.0 release](https://github.com/VMatee/jv-cli/releases/tag/v0.5.0)
was verified before Desktop work. All eight assets matched their public GitHub
digests and strict release inventory. The public Windows setup was
99,114,165 bytes, SHA256
`5400e8abde06867e829bbc251fcae7f21772f6d43da4a3d24ccccf1a9ab09984`.
Installed public CLI acceptance passed 17 tests, zero skips, six real IPv4/IPv6
PowerShell/Python/Node connections, and installer/portable/rollback/state/PATH
boundaries. Public CLI 0.5.0 with Codex 0.149.1 is installed at the normal per-user
location; Desktop does not bundle, rebuild or install it.

Desktop checks passed: `npm ci`, `npm test` (**34**), `npm run build`,
`npm run rust:fmt`, `npm run rust:check`, `npm run rust:test` (**37**: 8 core,
8 bridge, 6 runner, 15 Windows native), `npm run rust:clippy`,
`npm run tauri:check`, `npm run tauri:test` (**6 IPC**), and release-asset contracts
(**14**). Native real-public-CLI smoke passed **3** tests. No ignored/security
skips or production requests were used. Tests explicitly reject Windows 0.4.9
and preserve Linux's 0.4.9 acceptance and editable permission modes.

The normal, non-fixture `npm run tauri -- build --bundles nsis` produced the
0.2.1 executable and unsigned candidate setup (2,173,442 bytes, SHA256
`30ca83ce8d22ad60dd9a6e958074f3ffb17cf7d2f0b8beb1d93ed6a55b9e60e8`).
`scripts/windows-installer-smoke.py` passed **9 installer assertions** with zero
skips, plus two ordinary-user installed WebView launches. Reinstall and normal
uninstall preserved the synthetic separate-state canary and public CLI binary.
The normal per-user installation was restored for native acceptance.

Native manual-action acceptance with automated postconditions passed Offline
Demo's Thai conversation and **5** real-public-CLI/native-control assertions:
detected/trusted-host status, separate Desktop HOME, canonical Thai/space drive
folder, Rust workspace/IPC validation, and real title-bar-X cleanup. Desktop,
bridge and all captured descendants exited; no hidden owned process remained;
the unrelated sentinel survived. Native automation remains unavailable (OS error
2), so the operator performed Open Folder/selection and title-bar X. No login or
coding task was sent. Two preliminary harness runs exposed instruction/click
races, corrected only in the driver; their failure cleanup was not counted as
native-close acceptance. The final native-only run independently passed all five
postconditions. The separate Offline Demo success is retained as evidence.

Version 0.2.1 is a patch for the official Windows dependency floor and guidance,
not a new execution mode. The existing controlled live result below remains
valid; no additional production acceptance was performed. Final feature/main/tag
CI and public Desktop download/install checks are required before publication is
reported complete. Runner-built installer bytes may differ from this candidate.

## Windows 0.2.0 release candidate — 2026-10-03

Starting feature SHA: `5fcaa5d1b939bc464d0b396083b655bad8e34dce`, clean and
equal to the fetched branch; main remained `86dab4b71c9bf222cfb7222aa3434bfa8db33aaf`.
Version 0.2.0 is a minor release for the new supported Windows platform. CLI 0.4.9,
Codex 0.149.1 and bridge protocol 1 are unchanged. No additional production login,
coding task or credential inspection was performed.

`npm ci`, `npm test` (32), `npm run build`, `npm run rust:fmt`, `npm run rust:check`,
`npm run rust:test` (35: 6 core, 8 bridge, 6 runner, 15 native Windows),
`npm run rust:clippy`, `npm run tauri:check` and `npm run tauri:test` (6 IPC) passed.
All tests ran with zero ignored/security skips. Offline release contracts passed
14 tests, including mixed-commit rejection and four-package download verification.
The opt-in real installed CLI smoke passed 3 tests without production requests.
Both workflows passed checksum-verified actionlint 1.7.7 validation.

`npm run tauri -- build --bundles nsis` built the normal, non-fixture release app
and current-user NSIS installer. Local candidate installer: 2,173,116 bytes,
SHA256 `cd28c8d13164466cbd019d820f1a77c97dc0b7a00609c9dc21526eaffbbda51f`,
unsigned. Runner release builds may have different bytes; public checksums must
identify the exact published runner artifacts.

| Installed candidate check | Result |
| --- | --- |
| Per-user install / same-version reinstall / uninstall | **PASS**, 9 automated installer assertions, zero skips. |
| State preservation | **PASS**, a synthetic canary in Desktop's actual application-data directory survived reinstall and normal uninstall. Only that test-created canary was removed afterward. Private CLI state was not inspected. |
| Installed frontend / Offline Demo | **PASS**, actual installed `jv-desktop.exe` rendered 0.2.0 and completed a Thai simulated conversation, with no service calls. |
| Real installed CLI / trusted host | **PASS**, detected separately installed 0.4.9, fixed full-access/network-enabled policy shown, unauthenticated protocol-1 bridge opened. |
| Real native folder selection | **PASS**, canonical local drive path with spaces/Thai matched the selected Rust workspace; different/root IPC workspaces were rejected. |
| Native title-bar X / owned cleanup | **PASS**, Desktop exit code 0, real CLI bridge and all captured descendants exited, no hidden owned descendant remained; unrelated sentinel survived. |
| External boundaries | **PASS**, JV CLI binary hash/version unchanged; disposable workspace stayed empty. Installer inventory excludes JV CLI, terminal state and project files. |
| Final local reinstall / launch | **PASS**, normal `%LOCALAPPDATA%\JV Desktop` installation restored and 3 installed WebView/readiness/shutdown checks passed. |

Native automation recovery failed with OS error 2. Folder selection and native X
were **manual actions with automated state/process postconditions**; they were not
JavaScript close or forced-termination acceptance. The successful final installed
native run passed 5 assertions, separately from the completed Offline Demo check.
Two preliminary harness runs failed assumptions about state creation/process
observation; scoped failure cleanup was not counted as native close. The harness
now allows bounded process-identity observation and does not assume startup creates
persistent state. No product runtime change was required.

Tauri uses Windows' local-data known folder for `jvcli-state`; it is not relocated
by a child LOCALAPPDATA override. The path remains separate from terminal CLI
state. Native Rust/real-CLI tests separately verify the absolute launch HOME with
disposable state. Release packaging CI repeats native builds and automated installed
acceptance on clean runners; one publisher combines same-commit Windows/Linux
inventories. Main/tag/public download gates must pass before release completion.

## Controlled Windows live acceptance — 2026-10-02

Desktop baseline: `635f16ccc48e47d79900e4f2f506cf4b133255cb`, version
0.1.1, clean and equal to the freshly fetched feature branch. Its exact-SHA
[Windows and Linux checks](https://github.com/VMatee/jv-desktop/actions/runs/37028443014)
passed. The existing normal release executable matched SHA256
`7f8a53614b1fbab7bc1cf74ac076651f7b32f11c35efbe7907687175ada13a00`;
no `offline-fixture`, fake CLI or fake service was used. The installed per-user
CLI reported 0.4.9, with the accepted Codex 0.149.1 and bridge protocol 1.
Neither product source nor the independent CLI checkout was changed.

The operator explicitly authorized live acceptance and personally entered all
credentials in Desktop. Native computer-use recovery still failed with OS error
2. Folder selection, sign-in and native window-close actions were manual; UI
status, workspace contents and scoped process postconditions were checked locally.
The temporary observer used only process-local loopback WebView debugging and
selected UI status fields. It enabled no Network/Fetch event capture, auth tracing,
IPC interception or screenshots, and disconnected entirely during credential entry.

### Task count and results

**Two real tasks were submitted.** The original allowance was one. Its session
became ready, but approximately ten minutes passed with only a turn-start entry
and no command/assistant result. The operator then explicitly requested cancellation
and one resubmission. Local Stop completed; the first workspace stayed empty.
The operator closed the app with its native title-bar X: exit code 0, installed
bridge and all 26 captured owned processes exited. The first remote job's state
remains **unknown**; no remote cancellation or successful first-job result is claimed.

The additional attempt used a fresh empty folder outside both repositories:
`C:\Users\Matee\Desktop\NotScan\Work\jv-live-acceptance-retry-20261002-232817`.
The first folder, `jv-live-acceptance`, was preserved rather than deleting unknown
state. A new normal Desktop instance and operator-entered login started the fresh
session. Each attempt had a separate local at-most-once submission marker. No
third task, benchmark, package installation or production configuration change
was performed.

| Check | Live evidence |
| --- | --- |
| Installed CLI / native backend | **PASS**. Accepted `jvcli.exe` detected; native bridge owned by a Job Object. |
| Authentication / session | **PASS**. Both operator-entered logins reached Session ready; no credential value was observed. |
| Trusted host / workspace | **PASS**. Fixed trusted-host policy and the exact canonical disposable workspace displayed. |
| Second turn / tools | **PASS**. Turn started, three command-start/completion pairs, assistant response and Process finished rendered; Turn complete displayed and Desktop returned to ready. |
| File / Thai contents | **PASS**. Only `JV_WINDOWS_LIVE_ACCEPTANCE.txt` exists. Strict UTF-8, no BOM, exact two requested lines, CRLF, no trailing newline, 90 bytes. SHA256: `8d56e046da73b9e1830a74bbac31595bd8b9f9e8cc7eda197a6f919da7980311`. |
| Assistant verification | **PASS**. The displayed reply reports the correct relative path and an exact content match; independent filesystem verification agrees. |
| Separate `files.changed` activity | **NOT OBSERVED**. The nine rendered activity entries contained commands, assistant and terminal completion, with no file-change entry. |
| Final native close / cleanup | **PASS**. Operator used the real title-bar X. Desktop exited with code 0; the installed bridge and all 12 processes captured before final close exited. No forced termination or process-name-wide kill was used. Codex was already absent from the owned descendant tree after turn completion. |
| Credential exposure | **None observed**. Frontend storage lengths were zero. No credential fields, auth traffic, tokens, private state contents, argv/environment secret values or raw stderr were inspected. |

Desktop-specific CLI state remained
`%LOCALAPPDATA%\com.openjvspace.desktop\jvcli-state`, separate from terminal
`%LOCALAPPDATA%\JV\jv-cli`. These locations were verified from the established
launch contract, not by reading credentials or session contents. Account identifiers,
service responses, authentication bodies and private transcripts are not recorded.

### Controlled Windows live acceptance: PASS

**JV DESKTOP WINDOWS CONTROLLED LIVE ACCEPTANCE PASSED.** The explicitly
authorized second task completed, its file contents and assistant result matched,
and native close cleaned up the owned processes.

Event coverage note: the successful production turn modified the workspace through
PowerShell command execution. The engine did not emit a separate
`file_change`/`file_changes` item for that turn, so no `files.changed` event was
available to Desktop. Synthetic bridge coverage separately verifies Desktop
handling when `files.changed` is present. This is event-coverage information,
not a failed file write or Desktop integration. No event was fabricated and no
additional production task is needed.

The temporary monitor initially expected a legacy Turn completed activity label.
Protocol 1 intentionally maps `turn.completed` to Process finished and the
Turn complete outcome. Completion was therefore independently checked against
that real UI outcome and the existing source mapping; this is a monitor limitation,
not evidence of a Desktop completion defect. No extra production task was sent to
correct the observer. Documentation-only validation uses `git diff --check`;
new publication still requires exact-SHA Windows/Linux CI. This live record does
not authorize further production requests.

## Windows native GUI acceptance gap closure — 2026-10-02

Baseline: `feature/windows-native` at
`f660cbceda66b78f1e38b7cc2130d81cc0d28536`, clean and freshly fetched.
The normal release executable was rebuilt with `npm run tauri -- build
--no-bundle`, without `offline-fixture`. Its SHA256 was
`7f8a53614b1fbab7bc1cf74ac076651f7b32f11c35efbe7907687175ada13a00`.
Desktop remains 0.1.1. No product/backend/CLI source or dependency change was
needed; this checkpoint adds the acceptance harness, its syntax checks and evidence.

Supported native computer-use initialization, retry and reset still failed with
OS error 2 (native pipe unavailable). The operator therefore performed four
native actions in the actual release app; all postconditions were automated.
This is **native manual-action acceptance + automated postconditions**.

| Native control | Result and evidence |
| --- | --- |
| Real Windows folder picker | **PASS**. Operator confirmed the native dialog appeared and selected a disposable local drive path ending `workspace ไทย\tree`. The displayed canonical path matched the workspace used by the Rust-owned bridge; protocol readiness validated that same workspace. |
| Native picker Cancel | **PASS**. Operator clicked Cancel. Canonical selection and the existing bridge/child/grandchild remained unchanged. |
| Workspace replacement | **PASS**. Operator selected `workspace ไทย\hung-tree`. Held process handles proved the previous bridge exited before the replacement bridge was created; its child/grandchild also exited. |
| Real native title-bar X | **PASS**. Operator explicitly confirmed using the title-bar X. Desktop exited with code 0; bridge, child and grandchild exited, every captured app descendant exited, no hidden owned descendant remained, and the unrelated sentinel survived. |

The replacement fake bridge deliberately ignores `bridge.shutdown`; the existing
bounded Job Object fallback remained unchanged and cleaned the tree. A separate
registered IPC check rejected a different valid folder and a drive root without
executing a turn. Root selection in the native dialog itself was not newly tested.
Initial harness-only corrections were completed before the final successful run;
failure cleanup was never counted as native window-close acceptance.

`scripts/windows-native-gui-acceptance.mjs --guided` completed **6 checks, zero
skips**, with results/screenshots isolated under ignored `.cache/manual-gui`.
`scripts/windows-gui-process-watch.py` retains process handles to avoid PID reuse
and never terminates processes by name. The normal production resolver selects
the fake CLI from a test-only per-user installation through isolated
`LOCALAPPDATA`; neither the real CLI installation nor ordinary user state changes.
No picker replacement, JavaScript close or forced Desktop termination was used
in the successful acceptance run.

Local regression commands all passed:

| Command | Result |
| --- | --- |
| `npm test` | 32 passed |
| `npm run build` | passed |
| `npm run rust:fmt`, `rust:check`, `rust:clippy` | passed |
| `npm run rust:test` | 35 passed: 6 core, 8 bridge, 6 runner, 15 Windows native |
| `npm run tauri:check` | passed |
| `npm run tauri:test` | 6 passed |
| Node driver / Python watcher syntax | both passed |

Zero ignored/security-relevant skipped tests. The accepted runtime architecture,
trusted-host policy, external CLI, credentials handling, private state, protocol
and Linux behavior remain unchanged. No production authentication, password,
job or inference was used. Installer lifecycle, live acceptance, signing,
versioning and release remain separate milestones. Each published successor SHA
still requires its own Windows/Linux CI verification after publication.

## Windows native integration — 2026-10-02

Starting Desktop main: `86dab4b71c9bf222cfb7222aa3434bfa8db33aaf`, version 0.1.1.
The accepted external CLI checkpoint is
`4a00ee480db70d5bcac619d95afa64056cd1905c` (CLI 0.4.9, Codex 0.149.1,
protocol 1). Its existing setup artifact was verified against SHA256
`1e4a76fabffd76a10e33d06162e4d39688a4872193978f8b67f392bf905ab82d`
and installed per-user without rebuilding/modifying the CLI repository.

### Native implementation and evidence

Windows 11 Pro build 26200, AMD64, ordinary non-Administrator account;
PowerShell 5.1, repository-local Node 24.21.0/Rust 1.98.1, existing MSVC/SDK.
The fixed per-user executable is resolved before safe absolute PATH entries.
Drive-root cwd does not suppress that explicit install; cwd/project candidates
and relative/reparse/UNC/device paths are rejected. Detection uses only local
`--version`, not authentication or service health.

Windows argv is exactly `bridge --stdio --execution-mode trusted-host`; Linux
retains `bridge --stdio`. Structured capabilities lock Windows UI/IPC to network
true and full access. Conflicting requests are rejected before execution, and
session responses must match the platform's policy. Linux cannot accept a
surprise danger-full-access session.

Atomic CreateProcessW Job-list assignment precedes child code. Kill-on-close,
restricted inherited handles, child/grandchild cleanup, nested Jobs, forced
shutdown, owner death and unrelated sentinel survival are proven natively.
Native PeekNamedPipe reads and a bounded stdin worker preserve framing/limits,
write deadlines and cancellation; timed-out writers are cancelled and joined.
Desktop state stays separate and CLI-owned: a loopback-only rejected synthetic
login created protected private state without persisting its password. Ready/
shutdown alone correctly creates no state. No terminal state was read.

### Local checks

| Command | Result |
| --- | --- |
| `npm ci` | 104 packages, zero reported vulnerabilities |
| `npm test` | 32 passed |
| `npm run build` | passed |
| `npm run rust:fmt`, `rust:check`, `rust:clippy` | passed |
| `npm run rust:test` | 35 passed: 6 core, 8 bridge, 6 runner, 15 Windows native |
| `npm run tauri:check` | passed |
| `npm run tauri:test` | 6 registered IPC tests passed |
| Tauri Clippy, normal and `offline-fixture` builds | passed, warnings denied |
| `py -3.10 -B scripts/windows-native-smoke.py` | 3 real installed CLI tests passed |
| `py -3.10 -B -m unittest discover -s tests -p 'test_release*.py' -v` | 7 offline release-asset tests passed |
| Linux target process-core `cargo check --all-targets --features test-fixture --target x86_64-unknown-linux-gnu --locked` | passed; compile evidence, not Linux runtime acceptance |
| Linux target process-core Clippy, all targets, warnings denied | passed; not Linux runtime acceptance |

There are no ignored/security-relevant skipped tests in these gates. Test fixture
binary targets have zero unit tests; their behavior is exercised by integration
tests. All secrets/prompts were synthetic and stdin-only. No production JV
authentication, credentials, jobs or inference were used.

### Actual Windows application

`npm run tauri -- build --bundles nsis` produced an actual optimized AMD64
`target/release/jv-desktop.exe` and an unsigned development
`target/release/bundle/nsis/JV Desktop_0.1.1_x64-setup.exe`. The real executable
was launched, rendered bundled assets and detected the installed CLI. This is
local build evidence, not installer installation/public release acceptance.

`scripts/windows-webview-acceptance.mjs` attached to the actual Tauri WebView2
using process-local loopback debugging. Normal build: **3 checks passed** (render,
real native detection/fixed policy, cooperative IPC bridge close). Separate
`offline-fixture` build: **10 checks passed** (render, demo Thai/Stop, native
capabilities, Rust-selected folder, synthetic stdin sign-in/full-access session,
Thai assistant/command/file events, second-turn Stop, folder reopen, bridge close).
Frontend external requests were blocked and none were observed. Password storage
was empty; raw stderr was not rendered. The fixture build cannot fall back to a
real CLI. Screenshots/state remain ignored in `.cache/gui`.

Native computer-use initialization and documented recovery failed because its
native pipe is unavailable (OS error 2). JavaScript window.close did not exit the app.
The native folder dialog and window-manager close button were **not accepted in
that integration run**. They are now covered by the separate native manual-action
acceptance above. The fixed fixture folder and forced test-process exit do not
count as those native UI checks. Owned parent-death cleanup is independently
proven by Rust and real-CLI tests.

### Publication / CI / remaining work

Baseline exact-SHA Desktop checks and Linux release checks were green. The new
workflow runs Windows native fixture/IPC tests plus the actual executable build,
and retains Linux process/frontend/browser/release regressions with Tauri tests.
Published runtime checkpoint `cefcfdd405ac9159c5f31585f1ca1fd0d90b88a9` was
freshly fetched and matched the clean local branch. Exact-SHA Actions run
[37017410910](https://github.com/VMatee/jv-desktop/actions/runs/37017410910)
completed successfully for both `Desktop native checks (windows-latest)` and
`Desktop native checks (ubuntu-24.04)`, including the actual Windows executable
build and Linux Tauri/browser regressions. Any successor checkpoint must have its
own exact-SHA checks verified; local results alone do not prove CI success.

The initial cached HTTPS identity lacked write access and the GitHub write
connector became administrator-disabled. At the operator's explicit request a
dedicated deploy key was generated outside the checkout, protected for the current
Windows user only. After the operator added its public key with write access,
SSH authenticated specifically to `VMatee/jv-desktop`. Only this repository's
remote/local SSH command changed; no global SSH configuration, private-key read,
force push or sibling repository modification was used.

At that checkpoint native folder/window-close controls remained unverified;
the separate acceptance above closes those gaps. Controlled live account
acceptance, Windows
release/signing/versioning and any CLI bundling decision are separate milestones.
Pinned Windows restricted-token socket denial remains unsupported; dedicated
workers intentionally use trusted host with normal account privileges and Firewall
enabled. JV CLI stays separately installed and Desktop remains version 0.1.1.

## v0.1.1 release filename fix — 2026-10-01

The reported v0.1.0 Linux release run36840098720 failed only at published-byte
verification: GitHub changed spaces in uploaded package names to dots, while
SHA256SUMS retained the original names. The v0.1.0 tag/release/assets are preserved.
v0.1.1 changes release collection, version metadata and documentation only.

The collector now copies bundles to `JV-Desktop` filenames without spaces under
`target/release-assets/0.1.1/`. BUILD-INFO records those names, sizes, hashes,
source commit and tool/runner information. SHA256SUMS covers the three packages
and BUILD-INFO. The upload gate rejects unexpected/unsafe names, empty files,
missing/duplicate checksums and size/hash mismatches. The version-derived workflow
retains matching-main/green-CI/no-existing-release gates and published-byte checks,
including two fresh downloads and API digests when provided by GitHub.

Seven offline release tests passed, including collection into a download-like
directory followed by `sha256sum -c SHA256SUMS`, without access to source bundle
paths. All **27 frontend**, **17 Rust core/fixture**, **4 Tauri IPC** and **1 browser**
tests passed, as did frontend build, rustfmt, Rust check/Clippy and native Tauri
check. Application checks ran with Bubblewrap `--unshare-net` and isolated HOME.
Workflow YAML and shell blocks parsed successfully. No new live acceptance,
authentication, credential use or JV task was performed.

Local build evidence is kept in ignored `.cache/release-011` and version-specific
release output. Local artifacts are not evidence of successful GitHub publication;
main CI, tag workflow and downloaded published bytes must be independently verified
before calling v0.1.1 released. No reproducible-build certification is claimed.

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
