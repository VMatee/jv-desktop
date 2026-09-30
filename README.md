# JV Desktop

A desktop workspace for conversations with JV: choose a project folder, set tool
permissions, and follow structured activity alongside the conversation.

**Foundation preview, not a production client.** The offline demo works today.
Native Linux CLI detection and a tested process adapter are implemented, but
native turn submission is deliberately disabled pending secure authentication
and prompt transport. Windows has the shared UI and an honest unsupported-backend
state; there is no native Windows JV CLI integration yet.

## What works

- Tauri 2 shell with a native folder picker and conservative path validation.
- React chat with Unicode/Thai text, IME-safe Enter, Shift+Enter, and Stop in demo.
- Read-only/workspace-write controls and explicit tool-network permissions.
- Bounded JSONL normalization, activity, command results and changed-file events.
- A frontend `JvBackend` interface independent of Tauri transport details.
- A standalone Rust process core tested with an offline fake executable:
  argument vectors, explicit cwd, stdin, limits, timeouts and Linux process groups.
- No terminal, telemetry, password storage or project-file editing by the UI.

## Quick start: no JV account or service required

Use Node **24.15+ within the 24 LTS line** (validated with 24.21.0).

```sh
npm ci
npm run dev
```

Open the loopback URL printed by Vite. **Offline demo** is the default. In browser
preview, **Open Folder** explicitly selects a virtual `/demo/workspace`; it does
not access your disk. In the Tauri shell the same button opens a native folder
picker. The demo never reads or changes the selected folder.

For the native desktop shell, install Rust 1.98+ and the platform prerequisites
described in [Development](docs/DEVELOPMENT.md), then use `npm run tauri -- dev`.
The Native JV CLI mode only performs a bounded local `--version` check. Detection
does not establish authentication or production health.

Native Linux compilation and Debian/RPM packaging have been validated. Native
GUI acceptance remains pending: this agent session cannot access the X11 display.
See [Validation](docs/VALIDATION.md) for the distinction between compiled artifacts,
mock IPC checks, browser checks and actual native runtime validation.

## Architecture

```text
React chat + activity
  → JvBackend interface + normalized DesktopEvent
    → offline mock, or Tauri commands/channels
      → Rust process adapter
        → external installed jvcli (native turns currently blocked)
          → existing JV service (no server changes)
```

JV Desktop is independent from `VMatee/jv-cli`. No CLI source is embedded or
copied. Existing CLI commands remain unchanged. See the inspected
[CLI contract](docs/CLI_CONTRACT.md) for why public `exec` cannot safely be treated
as a stdin protocol yet, and [Architecture](docs/ARCHITECTURE.md) for the future
`jvcli bridge --stdio` migration boundary.

## Checks

```sh
npm test
npm run build
npm run rust:fmt
npm run rust:check
npm run rust:test
npm run rust:clippy
npm run browser:install
npm run test:e2e
npm run tauri:check
```

Frontend and process-core checks do not need a JV account or GUI libraries. The
last command requires native Tauri development libraries. All subprocess tests
invoke the **fake** executable, never real `jvcli exec` or `resume`. The browser
test blocks every request outside loopback.

See [Development](docs/DEVELOPMENT.md), [Security](docs/SECURITY.md),
[Windows plan](docs/WINDOWS_PLAN.md) and [Validation](docs/VALIDATION.md) for commands,
limitations and the recorded native build/runtime results. `package-lock.json` and
`Cargo.lock` pin dependency resolution. No release or installer is published.

## Security status

Native submission fails closed in Rust, even if frontend controls are bypassed.
There is no password UI or password persistence. Do not put credentials into chat
messages. All chat state stays in memory, and output is rendered as plaintext.
Tool networking is separate from the service connection: `--no-network` is not
a promise that a future authenticated native turn will be offline.

Linux is the first intended executable backend. Windows-native JV CLI, optional
WSL compatibility, session resume, a reviewed live sign-in flow, and integrated
terminals are later milestones, not capabilities of this preview.
