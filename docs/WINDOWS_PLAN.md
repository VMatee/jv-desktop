# Windows plan

The Tauri/React UI, permission model, normalized events and `JvBackend` contract
are shared between Linux and Windows. This does not imply that a Windows CLI
backend exists. The currently official JV CLI is Linux-only.

## Foundation behavior

Windows can use the offline demo and native folder selection once the desktop
shell is built. Native backend detection returns **unsupported** with a clear
message; it does not try to run a Linux executable or guess WSL configuration.
The process core has an isolated Windows unsupported branch. POSIX signals,
process groups and wait semantics are compiled only on Linux.

The Windows CI step checks shell compilation on an actual Windows runner. That
is not proof of a working native JV backend, an installed WebView2 runtime on
every target machine, a signed installer or live acceptance. No Windows installer
or release is part of this phase.

## Later milestones

1. A Windows preview may offer an explicitly selected WSL2 compatibility backend.
   Review path translation, distribution selection, permissions, authentication
   and process ownership. No automatic WSL installation or distribution mutation.
2. The final goal is an official native Windows JV CLI. Treat its CLI/bridge
   protocol as an external compatibility contract rather than embedding its code.
3. Implement Windows process lifecycle using native semantics, including Job
   Objects/owned child handles, pipe draining, cancellation, app-close cleanup
   and verification of descendants. Do not port `killpg` or scatter OS checks
   through React components.
4. Test UTF-16/UTF-8 paths, drive roots, UNC/extended paths, spaces, Thai input,
   executable discovery, blocked sign-in and missing-backend recovery.

A future interactive terminal would use **ConPTY** on Windows and **PTY** on
Linux. An integrated terminal is not part of this phase. There is no xterm.js,
PTY, ConPTY, embedded shell, terminal scraping or arbitrary shell execution UI.

The existing `jvcli-windows` sibling repository must remain unchanged during
Desktop work. Its contents are not copied, built, modified or presented as an
implemented Desktop backend. WSL automation, code signing, installers/releases,
auto-updates and terminal work require separate milestones.

Future bridge state will use Tauri app_local_data_dir()/jvcli-state on Windows,
with Windows-specific private-directory and owned-process semantics. Linux
protocol 1 integration does not enable native Windows or WSL execution.
