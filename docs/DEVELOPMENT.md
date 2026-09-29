# Development

## Prerequisites

- Node 24 LTS, **24.15.0 or newer**; validated with 24.21.0 and its bundled npm.
- Rust **1.98 or newer**, Cargo, rustfmt and Clippy; validated with 1.98.1.
- A C/C++ linker for the host when compiling Rust applications.
- Native desktop builds additionally need the
  [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).
  Linux needs GTK 3, WebKitGTK 4.1, JavaScriptCoreGTK 4.1, libsoup 3 and D-Bus
  development metadata/libraries. Windows needs the MSVC C++ build tools,
  Windows SDK and WebView2 runtime for launching the app.

The development host initially had Node 20.20.2 and Rust 1.75.0. Rather than
changing global tools, this foundation used Node 24.21.0 under `.tools/node` and
Rust 1.98.1 under `.tools/rustup`, with Cargo shims/cache under `.cache/cargo`.
Those ignored local artifacts are not required to be committed or copied.

If this checkout has that local Node installation, prefix commands as follows:

```sh
PATH="$PWD/.tools/node/bin:$PATH" npm ci
PATH="$PWD/.tools/node/bin:$PATH" npm run dev
```

That prefix affects only the child command. Do not modify shell startup files or
global PATH. The scripts detect the local Rust installation when present. On a
normal contributor machine, an already-installed compatible Rust toolchain is
also supported. Do not install system packages or global toolchains as part of
automated Desktop work; document missing host prerequisites instead.

## Install and run offline

From the repository root:

```sh
npm ci
npm run dev
```

This is Vite's loopback development server, not an application server or new JV
API. The product itself bundles static frontend assets in Tauri. No code in this
repository implements a web backend.

The browser opens in **Offline demo**. **Open Folder** explicitly selects a
virtual `/demo/workspace`; send any non-secret sample message to see a simulated
command and Thai response. Stop cancels its timer stream. The demo neither reads
files nor invokes any executable. Browser native mode reports unavailable.

On a host with Tauri GUI prerequisites:

```sh
npm run tauri -- dev
```

The native shell uses the real folder picker for both modes. Selecting a folder
does not edit it. Native mode detects `jvcli --version` once when selected or on
**Check again**, with a three-second timeout. Even if detected, native turn
submission is disabled in Rust and UI. No authentication form is implemented.

## Checks and what they prove

| Command | Scope |
| --- | --- |
| `npm test` | JSONL mapping, limits, malformed data, Unicode, permissions, mock lifecycle, IME and UI states |
| `npm run typecheck` | Strict TypeScript checking |
| `npm run build` | TypeScript plus production frontend assets, not a desktop bundle |
| `npm run rust:fmt` | Rust formatting across both crates |
| `npm run rust:check` | Standalone process core, no GUI libraries |
| `npm run rust:test` | Core unit tests and Linux fake-executable integration tests |
| `npm run rust:clippy` | Core and fixture linting, warnings denied |
| `npm run browser:install` | Download a repository-local Chromium headless shell; no OS packages |
| `npm run test:e2e` | Offline browser workflow, Stop, native blocker and responsive layout |
| `npm run tauri:check` | Compile/type-check native Tauri code; requires GUI development libraries |
| `npm run tauri -- build --no-bundle` | Compile native executable; requires full host prerequisites |

The current host cannot perform a Linux Tauri check/build because native system
libraries are missing; see [Validation](VALIDATION.md). Do not run a package
manager or `sudo` to repair the host. Finish the independent checks instead.

All Cargo scripts put downloads in `.cache/cargo`, temporary files in `.cache/tmp`
and compiled artifacts in `target`. npm uses `.cache/npm`. Browser downloads live
in `.cache/ms-playwright`, screenshots in `test-results`. These are ignored.
Never stage them, account files, `.env` files or private test evidence.

## Fake executable

`crates/jv-process/tests/fixtures/fake_jvcli.rs` builds as `fake-jvcli` only with
the `test-fixture` feature. Cargo integration tests obtain its absolute path via
`CARGO_BIN_EXE_fake-jvcli`; they never discover or run a real installed CLI.

The fixture accepts `--version`, or a synthetic argument vector containing
`--fixture SCENARIO` and a prompt on stdin. Supported scenarios include success,
command-fails, malformed, unknown, oversized, invalid-utf8, nonzero, slow, tree,
no-stdin, flood, flood-no-newline, stderr-flood and inspect. Success includes a changed file and Thai
assistant response. Inspect echoes only synthetic test argv/cwd/prompt and the
names of the minimal child environment keys. The fixture has no network client.
The stdin protocol is **not** supported by public JV CLI 0.4.7; see
[CLI contract](CLI_CONTRACT.md).

Frontend tests reject fetch/XHR. Browser checks abort non-loopback requests before
they can be sent and assert none were requested. Rust tests create only ignored
repository-local workspace fixtures. No JV account, real password, token or live
scenario is needed. Do not use a production task as a build or publication check.

## CI

GitHub Actions runs frontend checks and Rust core checks on Ubuntu and Windows.
Linux additionally exercises real fixture process-group cancellation and the
browser workflow. Windows builds the fake fixture but does not run Linux-only
process integration tests; shared validation tests run on both platforms.

A separate Windows step checks the Tauri crate with the runner's MSVC tooling.
It validates the desktop shell's build, not native Windows JV functionality.
Linux full GUI compilation requires a separately provisioned runner. Actions
use pinned revisions, a read-only repository token with credential persistence
disabled, and no JV credentials/deploy keys. There is no release workflow.

## Before publishing

Inspect the worktree, staged files, untracked files and both lockfiles. Run all
available checks and record GUI environment blockers honestly. Fetch the intended
origin, inspect divergence and push normally only if histories are compatible.
Never reset siblings or overwrite unexpected remote commits. Re-check workspace
isolation against the initial sibling status/HEAD record.
