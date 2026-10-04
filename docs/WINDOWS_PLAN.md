# Windows native integration

Target: AMD64 dedicated JV worker PCs, using an ordinary Windows account and
WebView2. Desktop 0.2.0 adds official Windows 10/11 AMD64 support. ARM64 and
WSL are not supported. JV CLI remains a separately installed runtime dependency.

## Execution contract

```text
React → JvBackend → Tauri IPC → Rust BridgeSupervisor
      → installed jvcli.exe bridge --stdio --execution-mode trusted-host
      → shared bridge protocol 1
```

The external Windows CLI must be the official 0.5.0 release or newer, with
Codex 0.149.1. CLI 0.5.2 is recommended for enhanced remote support-code/status
reporting; older compatible versions retain legacy generic failure behavior.
Download the [Windows x86_64 installer](https://github.com/VMatee/jv-cli/releases/tag/v0.5.2),
install CLI first, then install Desktop, select Native JV CLI and a workspace,
and sign in. No source checkout, Python, Node or separate Codex installation is needed.
Linux's released 0.4.9 minimum remains unchanged.
Desktop does not copy, rebuild, bundle, update or silently install JV CLI.
Resolution prefers `%LOCALAPPDATA%\Programs\JV CLI\jvcli.exe`, then absolute
PATH entries. It excludes current/project directories and reparse/UNC/device
candidates. Detection checks only local version/readiness, not service health.

Windows Native JV CLI always displays **Trusted host**: full filesystem access,
network enabled, no tool approval prompts. Tools run with the dedicated account's
normal privileges. Incompatible permission controls are hidden and conflicting
IPC policies are rejected. Session responses must confirm danger-full-access and
network true. Linux retains its existing sandboxed permission controls and args.

Windows restricted-token network-denied execution remains unsupported with the
pinned Codex 0.149.1 backend. Trusted host is intentional unrestricted authority,
never a fallback after isolation failure. Administrator, services, sandbox-account
provisioning and Firewall/WFP changes are not required. Keep Windows Firewall on.

## Desktop ownership

Desktop sets a separate absolute
`%LOCALAPPDATA%\com.openjvspace.desktop\jvcli-state` child. JV CLI creates and
protects its own private state; Desktop does not duplicate Windows ACL logic.
Terminal CLI state at `%LOCALAPPDATA%\JV\jv-cli` is not reused.

Native creation establishes an anonymous kill-on-close Job before bridge code
runs. Descendants and nested CLI Jobs remain owned, with no breakaway fallback.
Only private stdio handles are inherited. Bounded native pipe reads/writes,
deadlines, correlation, cancellation and normalized protocol behavior are shared
with Linux. Shutdown is cooperative, followed by owned Job cleanup when necessary.
Unrelated processes are never selected by name or terminated.

## Validation and next steps

Ordinary tests use native fake executables and synthetic accounts. The opt-in
installed CLI smoke covers version, ready, exact workspace/state, nested Job
ownership, shutdown, owner death and a loopback-only rejected login. Actual Tauri
WebView acceptance covers rendered UI, real CLI detection and fixture sign-in,
Thai events, Stop, folder reopen and cooperative bridge close. See
[Validation](VALIDATION.md) for exact coverage and limits and
[Development](DEVELOPMENT.md) for commands.

UNC/device namespaces and reparse workspaces/state are rejected. Ordinary local
drive paths, spaces, Thai and case aliases are tested; broad long-path/remote
filesystem support is not certified. These offline checks alone do not prove
production account acceptance; the separately authorized controlled live record
is described below.
The real native folder dialog (including Cancel and workspace replacement) and
title-bar X passed native manual-action acceptance with automated state/process
postconditions in the normal release app. Native automation remains unavailable;
the operator performed only the GUI actions. The separate driver also verified
cleanup with a fake bridge that ignores shutdown and unrelated-process survival.
A fixture picker or JavaScript close does not prove those native controls. See
[Validation](VALIDATION.md) and the [manual-action procedure](MANUAL_GUI_ACCEPTANCE.md#native-manual-actions-with-automated-postconditions).

Controlled live testing on 2026-10-02 proved real authentication, a trusted-host
coding turn, exact UTF-8/Thai file creation/readback and the matching assistant
result. Both normal native window closes cleaned up their owned bridge/process
trees. The operator explicitly requested one additional attempt after locally
stopping the first stalled turn; the first remote job's state remains unknown.
**Controlled Windows live acceptance: PASS.** No separate `files.changed`
activity appeared because the successful write used command execution without
an engine `file_change`/`file_changes` item. Synthetic tests cover that event
when present; its absence does not invalidate the verified live result. See
[the controlled live record](VALIDATION.md#controlled-windows-live-acceptance--2026-10-02)
for evidence and limits. No further real task is authorized by this record.

## Release packaging

The unsigned `JV-Desktop_0.2.2_x64-setup.exe` uses Tauri NSIS current-user
installation, normally `%LOCALAPPDATA%\JV Desktop`. No Administrator privileges
are required for installation or runtime. The installer creates user shortcuts
and uninstall registration; it does not add Desktop to PATH or install JV CLI.
WebView2 is required and may be obtained from Microsoft by the standard installer.

Normal reinstall/upgrade and uninstall preserve Desktop application data.
The uninstaller offers a separate, unchecked remove-data choice; do not select it
when retaining sessions. Project files, terminal CLI state and the CLI installation
are outside the installer inventory. Windows may show warnings for this unsigned
installer. SHA-256 confirms bytes, not publisher identity.

Release 0.2.0 is a minor version because it adds a supported platform, rather than
a patch to 0.1.1. Release 0.2.1 is a patch correcting the Windows dependency floor
and installation guidance after official CLI publication; it adds no execution mode.
Release 0.2.2 adds backward-compatible remote support-code and recovery-status
display; the CLI compatibility minimum and execution policy remain unchanged.
Both native package jobs build the same source/version; one job
combines verified inventories and publishes only after green main checks. See
[Development](DEVELOPMENT.md#release-preparation) for the offline release gates.
WSL, updates, terminal embedding, PTY/ConPTY and signing remain outside this release.
