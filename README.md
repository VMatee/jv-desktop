# JV Desktop 0.2.3 (release candidate in preparation)

The installation links below remain on the published stable release until the
Windows candidate completes acceptance. Structured image-assisted coding pairs
this candidate with JV CLI 0.5.3; install CLI first.

A desktop workspace for JV conversations: choose a project folder and follow
commands, changed files and assistant replies in one window. Supports Linux
x86_64 and native Windows 10/11 AMD64.

## Requirements

- **[JV CLI](https://github.com/VMatee/jv-cli/releases)** installed
  separately using its official instructions. Desktop neither bundles nor updates it.
  Windows requires **0.5.0 or newer**; Linux requires **0.4.9 or newer**.
  **CLI 0.5.2 is recommended** for enhanced remote support-code/status reporting.
  Older compatible versions retain legacy generic failure behavior.
- A JV account for native conversations. Offline Demo needs no account.
- **Linux:** a graphical desktop; built on Ubuntu 22.04. Debian/RPM packages need
  GTK3 and WebKitGTK4.1 as listed by your package manager. AppImage still requires
  compatible system libraries; it is not a universal Linux guarantee.
- **Windows:** Windows 10/11 AMD64 and Microsoft WebView2. The installer can obtain
  WebView2 if missing. A dedicated ordinary Windows user is recommended.

## Install on Windows

First install the official [JV CLI Windows x86_64 release](https://github.com/VMatee/jv-cli/releases/tag/v0.5.2).
Its installer includes the CLI runtime; no separate Python, Node or Codex installation is required.
Then download `JV-Desktop_0.2.2_x64-setup.exe` and `SHA256SUMS` from
[Releases](https://github.com/VMatee/jv-desktop/releases/tag/v0.2.2).
Check the installer hash against its entry in SHA256SUMS:

```powershell
Get-FileHash .\JV-Desktop_0.2.2_x64-setup.exe -Algorithm SHA256
```

Stop if verification fails. Run the installer and open **JV Desktop** from the
Start menu. Installation is per user, normally `%LOCALAPPDATA%\JV Desktop`,
with user shortcuts and uninstall registration. No Administrator rights or PATH
change is needed. JV CLI must already be installed separately, normally at
`%LOCALAPPDATA%\Programs\JV CLI\jvcli.exe`; safe absolute PATH locations are
also supported.

The Windows installer is **unsigned**. Windows may display SmartScreen or
publisher warnings. SHA-256 verifies bytes but does not provide publisher identity.

Windows Native JV CLI uses **Trusted host**: full tool filesystem access,
network enabled and no tool approval prompts. Tools receive the normal privileges
of the Windows account, including access outside the selected project. Use this
mode only on a dedicated trusted worker. Administrator is not required; keep
Windows Firewall enabled. Read-only and network-denied Windows execution are not
supported. This is unrestricted execution, not Windows sandboxing.

## Install on Linux

Download one x86_64 package and SHA256SUMS from the same release. Verify the
downloaded files in that directory:

```sh
sha256sum --check --ignore-missing SHA256SUMS
```

For a complete release download, `sha256sum -c SHA256SUMS` verifies all four
packages and BUILD-INFO.json. Choose one Linux format:

**AppImage** — no system installation:

```sh
chmod +x 'JV-Desktop_0.2.2_amd64.AppImage'
./'JV-Desktop_0.2.2_amd64.AppImage'
```

If FUSE is unavailable, use `--appimage-extract-and-run`. Do not run Desktop as
root or disable OS protections to work around a launch problem.

**Debian/Ubuntu** — open the DEB in your software installer, or use:

```sh
sudo apt install './JV-Desktop_0.2.2_amd64.deb'
```

**RPM-based Linux** — open the RPM in your software installer, or use:

```sh
sudo dnf install './JV-Desktop-0.2.2-1.x86_64.rpm'
```

Launch **JV Desktop** from the applications menu, or run `jv-desktop`.
JV CLI must be available as `jvcli` on your desktop session's PATH.
Release packages are unsigned.

## First run

1. Install JV CLI and confirm `jvcli --version` reports 0.5.0 or newer on Windows,
   or 0.4.9 or newer on Linux.
2. Open JV Desktop. **Offline Demo** is the default and never contacts JV.
3. Select **Native JV CLI**, then **Open Folder** and choose a dedicated project.
   Choose **Structured — agent screenshot inspection** for local agent image tools
   with a compatible CLI and service account. **Legacy — text tools only** is an
   explicit fallback for older installations or unsupported accounts. Changing
   modes requires reopening the folder; failed requests are never automatically
   resent. This does not add image attachments to the chat composer.
   Select the folder again if you switched from Demo.
4. Check that JV CLI is detected. Older versions require an upgrade; there is no
   fallback that puts prompts in command-line arguments.
5. Enter your username, usual service URL and password in Desktop.
   On **Linux**, choose permissions before signing in:
   - **Read only**: tool network disabled.
   - **Workspace write**, Tool network checked: edits and tool network allowed.
   - **Workspace write**, Tool network unchecked: edits allowed, tool network denied.
   On **Windows**, the fixed Trusted host status describes unrestricted tool access.
6. Press **Sign in** once. When **Session ready** appears, type a task and send it.

Tool Network controls model tools, not login/service connectivity. Session policy
is fixed; reopen the folder to start another session. Enter sends; Shift+Enter
adds a newline. Active IME composition does not submit. If sign-in fails or the
bridge exits, reopen the folder and sign in again. Raw server/child errors are
not displayed. CLI detection does not prove service health.

## Stop, privacy and state

**Stop stops local work. A submitted remote job may continue and its state may
be unknown.** Do not immediately duplicate cancelled work without checking its state.

Passwords are memory-only: no remember-password, frontend storage, password
file, argv, environment or password logging. Messages use private stdin pipes.
Linux follows the selected CLI sandbox policy. Windows explicitly launches
`jvcli.exe bridge --stdio --execution-mode trusted-host`.

Desktop CLI state is separate from terminal CLI state:

- Windows: `%LOCALAPPDATA%\com.openjvspace.desktop\jvcli-state`.
- Linux: `$XDG_DATA_HOME/com.openjvspace.desktop/jvcli-state`, normally
  `~/.local/share/com.openjvspace.desktop/jvcli-state/`.

Session/runtime data may persist there; chat UI state is in memory.
See [Security](docs/SECURITY.md).

## Upgrade and uninstall

Close Desktop first. On Windows, run the newer installer; same-version reinstall
is supported. Use **Installed apps → JV Desktop → Uninstall** to remove it.
Leave the optional remove-app-data checkbox unchecked to retain sessions and data.

On Linux, replace the AppImage or use the same package manager. Remove the
AppImage, `sudo apt remove jv-desktop`, or `sudo dnf remove jv-desktop` to uninstall.

Normal app removal preserves projects, the separately installed JV CLI and
Desktop user data. Back up/review application data before explicitly removing it.
Never remove terminal CLI state as part of Desktop removal. No automatic updater
is included.

## Current limits

AMD64/x86_64 only. Windows trusted-host only; restricted network-denied execution
is unsupported with the pinned Windows backend. No WSL, ARM64, macOS, chat image
attachments, integrated terminal, PTY/ConPTY, automatic updates or guaranteed
remote cancellation. Desktop does not restore the previous chat UI session.

## Development

Tauri 2 + React/TypeScript + Rust. No CLI source is embedded. See
[Development](docs/DEVELOPMENT.md), [Architecture](docs/ARCHITECTURE.md),
[Validation](docs/VALIDATION.md) and [Windows integration](docs/WINDOWS_PLAN.md).
Ordinary tests use fake executables and never contact production.
