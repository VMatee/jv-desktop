# Security boundaries

Native Linux execution now uses the reviewed CLI 0.4.9 bridge. Controlled Linux live acceptance passed before v0.1.0 release preparation. The default mode remains Offline demo. Opening native mode
only detects the executable; selecting a folder starts an unauthenticated bridge.
An explicit Sign in action is required before any authentication/session request.

## Process and workspace authority

Only Rust selects the fixed JV CLI executable and invokes it with native arguments
and an explicit validated cwd. Linux uses `bridge --stdio`. Windows prefers the
per-user `Programs\JV CLI\jvcli.exe`, then absolute PATH entries, and uses
`bridge --stdio --execution-mode trusted-host`. Relative/current-directory and
workspace executable lookup, reparse candidates and UNC/device paths are rejected.
Frontend requests cannot select executable, environment, shell, PID or engine
arguments. No shell interpolation, terminal scraping, PTY or argv prompt exists.
The old exec gate remains closed and is not a fallback.

The selected directory must exist and must not be filesystem root, home root or
the executable directory. The CLI adds its own authoritative workspace checks.
Selection creates no project files. Bridge state uses the application's own
private data directory; terminal CLI accounts/sessions are not opened by Desktop.
Installed executables are trusted native code, not sandboxed by version detection.

### Windows trusted-host boundary

Windows Native JV CLI intentionally gives model tools the normal permissions of
the dedicated Windows account: full filesystem access, network enabled, approvals
never. The UI shows a fixed trusted-host policy. Rust rejects manipulated read-only
or network-denied requests before execution and validates full-access session
responses only on Windows. Linux sandboxed policies are unchanged.

Use an ordinary dedicated account. Do not run as Administrator or disable
Windows Firewall/other host protections. This does not authorize credential
leakage, arbitrary executable IPC or termination of unrelated processes. Pinned
Codex 0.149.1 restricted-token Windows socket denial remains unsupported.

Desktop owns an anonymous kill-on-close Job Object. The bridge joins it during
native creation before code runs; only three stdio handles are inherited. Nested
CLI/tool Jobs remain inside Desktop ownership. Closing/terminating Desktop's
Job cleans its local descendants and never selects processes by name. Programs
started by external services/brokers and remote work are outside this local
process-tree guarantee.

Windows `JVCLI_HOME` is the absolute `app_local_data_dir()/jvcli-state` child,
distinct from terminal CLI state. Desktop rejects reparse ancestors and lets
JV CLI create/protect its private state; it does not port chmod or copy ACL logic.

## Memory-only sign-in

Explicit user input travels React memory → Tauri IPC → Rust memory → bridge stdin
`auth.login`. Password is never stored in frontend storage, files, configuration,
logs, argv or child environment. Frontend state is cleared after success/failure;
request buffers/references are discarded. This is best-effort lifetime reduction,
not a claim of secure erasure of JavaScript/Rust/Python allocations or crash dumps.
The bridge owns the bearer token and logout. Desktop neither receives nor stores
it. There is no account discovery, telemetry, persistent chat or diagnostic export.
Do not put credentials in ordinary chat text or Vite build variables.

The child uses a minimal environment allowlist, excluding `JV_API_PASSWORD`,
loader overrides and credential variables. Raw stderr is bounded and discarded;
errors crossing IPC are static categories. Known event fields are allowlisted and
bounded, unknown fields discarded. Tool/model content remains untrusted user data;
this is not a general secret detector for arbitrary text users ask to display.

## Protocol, UI and capabilities

See [Architecture](ARCHITECTURE.md) for byte/count/deadline bounds. Malformed known
frames, protocol mismatch, broken pipes and nonzero exits fail safely. Only bridge
protocol 1 with CLI>=0.4.9 is accepted. No old CLI fallback exists.

Plain React text rendering escapes HTML. Changed paths are labels only and never
opened, resolved or executed on receipt. No eval or raw-HTML rendering is used.
CSP permits bundled assets and IPC; no remote frontend scripts or fetches.
Development CSP additionally permits loopback Vite HMR.

The main window has only custom commands: choose_workspace, detect_backend,
open_bridge, close_bridge, authenticate, run_turn and cancel_turn. There is no
core:default, generic filesystem, JavaScript dialog or shell capability. The
native dialog is invoked in Rust. Its transitive filesystem crate grants no
frontend filesystem authority. Rust remains the sole process entry point.

Stop sends turn.cancel and reports **local=true, remote_state=unknown**. Shutdown
is cooperative first, followed by bounded termination of the owned bridge only.
A remote job may continue; forced local termination cannot prove remote or
escaped-descendant cancellation.

## Development and isolation

Repositories and histories remain independent; never copy CLI source into Desktop.
No production calls, real credentials or live scenarios in ordinary tests. Tests
use deterministic fake bridge/engine binaries and synthetic sentinel passwords.
Frontend tests reject fetch/XHR; browser acceptance blocks non-loopback traffic.
Historical Linux fixture tests ran in a Bubblewrap network namespace. Windows
native tests use disposable state, native fixtures and local listeners only.
The opt-in installed CLI smoke exercises readiness/shutdown and a loopback-only
synthetic rejected login, with no real account/token/job. Actual Tauri WebView
acceptance uses a separate opt-in fixture build for sign-in and turns. Live
acceptance requires separate authorization. Build/runtime caches remain ignored.
