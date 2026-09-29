# Foundation validation

This record distinguishes offline development verification from production
acceptance. **No real JV job was submitted.** No real password, token, account
configuration or private-key content was read or persisted by this work.

## Toolchain and selected packages

Validated with repository-local Node 24.21.0 and Rust/Cargo 1.98.1. Core selected
packages: React/React DOM 19.3.0, TypeScript 7.0.2, Vite 8.3.1, React Vite plugin
6.1.1, Vitest 5.0.2, Tauri API/CLI/Rust 2.12.0, tauri-build 2.7.0 and dialog
plugin 2.8.0. Browser testing uses Playwright 1.63.0. Exact npm/Cargo transitive
resolutions are recorded in their committed lockfiles.

Registry stable releases and engine requirements were checked before selection.
The host's original Node 20 and Rust 1.75 installations were not modified.

## Verified offline

Final suite size: **21 frontend unit/component tests, 10 Rust tests (4 unit + 6
Linux integration), and 1 multi-step browser acceptance test**. All pass.

- Frontend unit/component tests cover JSONL validation/mapping, bounded framing,
  Unicode/Thai, safe rendering, permissions, explicit workspace, native
  unavailable state, sanitized errors, mock success/cancellation and IME behavior.
- Rust core unit/integration tests use the dedicated fake executable for
  success, failed commands, changed files, Thai output, malformed/unknown lines,
  oversized output, invalid UTF-8, output floods, nonzero exit, missing executable,
  version detection, stdin/cwd/argv isolation, timeouts, stalled stdin and owned
  Linux process-tree cancellation.
- Strict TypeScript and production Vite build pass.
- Rust core check, formatting and Clippy (warnings denied) pass.
- Chromium offline browser workflow passes: explicit folder, Thai conversation,
  Stop, unavailable native mode, and compact layout. All non-loopback requests
  are blocked; screenshots are generated under ignored `test-results` and were
  visually reviewed. Browser testing does not launch a Tauri webview.

## Native build blocker

`npm run tauri:check` was attempted on the Linux development host. It failed in
`libdbus-sys 0.2.7`: pkg-config cannot find `dbus-1 >= 1.6` / `dbus-1.pc`.
Independent prerequisite probes also found these pkg-config entries absent:

- `gtk+-3.0`
- `webkit2gtk-4.1`
- `javascriptcoregtk-4.1`
- `libsoup-3.0`

The corresponding development packages include `libdbus-1-dev`, `libgtk-3-dev`,
`libwebkit2gtk-4.1-dev` and their dependencies on Debian/Ubuntu. None were
installed; no system configuration was changed. See the official
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

**No Linux native executable or desktop bundle was built, and the native folder
dialog was not exercised on this host.** Rust process-core success does not
validate the GTK/WebKit/Tauri shell. The CI workflow includes a Windows Tauri
compile check; its remote result must be inspected independently after publication.

## Deliberate product gates

Real exec/resume is blocked before spawn. Public CLI 0.4.7 requires positional
prompt arguments and environment-based noninteractive authentication. Neither
is enabled by this repository. Linux process-group tests do not establish real
CLI engine cancellation because that engine creates a separate session. No
native Windows/WSL backend, terminal, account persistence or release exists.

The initial and final sibling HEAD/status records are compared locally inside
`.git`, outside published source. Siblings and the surrounding non-Git workspace
are not part of this repository's history.
