# JV Desktop

A desktop workspace for conversations with JV: choose a project folder, set tool
permissions, and follow structured activity alongside the conversation.

**Offline-validated integration preview.** The demo works without a service.
Linux native mode uses installed JV CLI>=0.4.9 through bridge protocol 1, with
memory-only sign-in and isolated Desktop state. Production acceptance has not
been performed. Windows shares the UI but has no native JV CLI backend yet.

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
The Native JV CLI mode detects the version locally and starts an unauthenticated
bridge when a folder is selected. Sign in is an explicit service-contact action. Detection
does not establish authentication or production health.

Native Linux compilation and Debian/RPM/AppImage packaging have been validated.
The real X11/WebView shell, folder picker, Thai Unicode composition and offline
conversation/Stop were exercised. Remaining GUI checks are listed in
[Manual acceptance](docs/MANUAL_GUI_ACCEPTANCE.md).
See [Validation](docs/VALIDATION.md) for the distinction between compiled artifacts,
mock IPC checks, browser checks and actual native runtime validation.

## Architecture

```text
React chat + activity
  → JvBackend interface + normalized DesktopEvent
    → offline mock, or Tauri commands/channels
      → Rust process adapter
        → external installed jvcli bridge --stdio
          → existing JV service (no server changes)
```

JV Desktop is independent from `VMatee/jv-cli`. No CLI source is embedded or
copied. Existing terminal CLI commands remain unchanged. See the
[CLI contract](docs/CLI_CONTRACT.md) and [Architecture](docs/ARCHITECTURE.md) for
workspace binding, state isolation, memory-only authentication and local Stop.

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

Native submission requires a compatible bridge, chosen workspace and explicit sign-in.
Passwords are memory-only and never persisted or put in argv/environment. All chat
state stays in memory and output is plaintext. Do not put credentials into chat.
Tool-network denial does not disable the authenticated service connection.

Linux is the first backend. Windows-native JV CLI, WSL, Desktop session resume,
images and integrated terminals remain deferred. Live production acceptance is
separately authorized work; ordinary tests must always use fake dependencies.
