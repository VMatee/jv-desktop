# JV Desktop 0.1.0

A Linux desktop workspace for JV conversations: choose a project folder, set
permissions, and follow commands, changed files and assistant replies in one window.

## Requirements

- Linux x86_64 with a graphical desktop; built and tested on Ubuntu 22.04.
- **[JV CLI 0.4.9 or newer](https://github.com/VMatee/jv-cli/releases)** installed
  separately using its official installation/upgrade instructions and available
  as `jvcli` on your desktop session's PATH. Desktop neither bundles nor updates it.
- A JV account for native conversations. Offline demo needs no account.
- Debian/RPM packages require GTK3, WebKitGTK4.1 and the dependencies listed by
  your package manager. AppImage includes its desktop libraries but still depends
  on compatible Linux system libraries; it is not a universal Linux guarantee.

## Install

Download an x86_64 package and `SHA256SUMS` from
[Releases](https://github.com/VMatee/jv-desktop/releases/tag/v0.1.0).
Verify downloaded files in that directory:

```sh
sha256sum --check --ignore-missing SHA256SUMS
```

Stop if verification fails. Choose one format:

**AppImage** — no system installation:

```sh
chmod +x 'JV Desktop_0.1.0_amd64.AppImage'
./'JV Desktop_0.1.0_amd64.AppImage'
```

If FUSE is unavailable, run with `--appimage-extract-and-run` instead. Do not
run Desktop as root or disable its sandbox to work around a launch problem.

**Debian/Ubuntu** — open `JV Desktop_0.1.0_amd64.deb` in your software installer,
or have your system administrator install it with:

```sh
sudo apt install './JV Desktop_0.1.0_amd64.deb'
```

**RPM-based Linux** — open `JV Desktop-0.1.0-1.x86_64.rpm` in your software
installer, or use your distribution's package manager:

```sh
sudo dnf install './JV Desktop-0.1.0-1.x86_64.rpm'
```

Launch **JV Desktop** from the applications menu, or run `jv-desktop`.
Release packages are unsigned; SHA256 verifies file integrity, not a signing identity.

## First run

1. Install JV CLI, then confirm `jvcli --version` reports 0.4.9 or newer.
2. Open JV Desktop. **Offline demo** is the default and never contacts JV.
3. Select **Native JV CLI**, then **Open Folder** and choose a dedicated project.
   If you selected a demo folder first, select it again after switching modes.
4. Check that JV CLI is detected. Older versions require an upgrade; there is no
   fallback that puts prompts into command-line arguments.
5. Enter your username and password, leaving the service URL at your usual JV
   endpoint. **Choose permissions before pressing Sign in**:
   - **Read only**: tool network disabled.
   - **Workspace write**, Tool network checked: edits and tool networking allowed.
   - **Workspace write**, Tool network unchecked: edits allowed, tool network denied.
6. Press **Sign in** once. This authenticates and starts the session with that
   policy. When **Session ready** appears, type a task and send it.

Tool Network controls model tools, **not login/service connectivity**. Session
permissions are fixed; reopen the folder to choose another policy. Enter sends,
Shift+Enter adds a newline, and active IME composition does not submit.

If the bridge exits or sign-in fails, reopen the folder and sign in again. Raw
server/child errors are not displayed. Detection does not prove service health.

## Stop, privacy and state

Execution follows the chosen workspace and CLI sandbox policy. Selecting a folder
does not edit project files. Passwords are memory-only: no remember-password,
frontend storage, password file, argv, environment or password logging.

**Stop stops local work. A submitted remote job may continue and its state may be
unknown.** Do not immediately duplicate cancelled work without checking its state.

Desktop uses `$XDG_DATA_HOME/com.openjvspace.desktop/jvcli-state`, normally
`~/.local/share/com.openjvspace.desktop/jvcli-state/`. This is separate from terminal
CLI state. Session/runtime data may persist there; chat UI state is in memory.
See [Security](docs/SECURITY.md) for boundaries and limitations.

## Upgrade and uninstall

Close Desktop first. Replace the AppImage, or install a newer package through the
same package manager. No automatic updater is included.

To uninstall, delete the AppImage you downloaded, or remove package `jv-desktop`
through your software manager (`sudo apt remove jv-desktop` or
`sudo dnf remove jv-desktop`). Removing the app does **not** uninstall JV CLI,
delete your projects or automatically erase Desktop's user data. Review/back up
its application-data directory before manually removing it. Never delete terminal
CLI state as part of Desktop removal.

## Current limits

Linux native bridge execution is supported and controlled end-to-end acceptance
has passed. Windows Desktop compilation is checked in CI, but its native JV CLI
backend is **not supported yet**. Bridge images, Desktop session resume, WSL,
integrated terminals and guaranteed remote cancellation are not included.

## Development

Tauri 2 + React/TypeScript + Rust. No CLI source is embedded. See
[Development](docs/DEVELOPMENT.md), [Architecture](docs/ARCHITECTURE.md),
[Validation](docs/VALIDATION.md) and [Windows plan](docs/WINDOWS_PLAN.md).
Ordinary tests use fake executables and must not contact production.
