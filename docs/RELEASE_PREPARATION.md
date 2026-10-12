# JV Desktop 0.2.3 source preparation — 2026-10-13

Official publication is pending. Stable Latest remains 0.2.2. Keep `v0.2.3`
unpublished until final Windows acceptance and exact-main checks/package Actions
workflows are green. Prepared Structured sessions pair with external CLI 0.5.3;
install CLI separately first. No CLI implementation is embedded here.

## Integrated history

- Prior main: `b9d9ff3e2576552d7bc57b7303351646ab3656c9`.
- Linux vision: `41ea03fcbed04a75f6d493b5f7bd8e7e20b246c5`.
- Immutable `windows-rc-0.2.3-1`:
  `c8b1083d93f07850750bd6f4bcd7f4d0b82b51e3`, already containing Linux vision.
- Historical Windows handoff: `ad14b475a85efca0febfd6c1fbe6664ece1b9b66`.

Original branches/tags remain. package.json, package-lock.json, Cargo.toml,
Cargo.lock and Tauri configuration agree on 0.2.3. Protocol 1 preserves Linux
sandbox policy and Windows trusted-host policy. Structured mode requires
negotiated capabilities; older supported bridges can use explicit Legacy.
Failed turns are never automatically resubmitted.

## Offline Linux validation

Frontend: 65 tests passed. Rust process/core/bridge: 26 passed; Windows-only
targets contain no Linux tests. Tauri IPC: five passed. Release asset/version
contracts: 14 passed. Offline browser acceptance passed. Frontend build,
rust:fmt/check/test/clippy, tauri:check/test and a normal native executable build
passed. The external CLI and real Rust supervisor passed two synthetic vision
turns, PNG/JPEG/WebP, further edits and credential/image-event exclusion.

The normal Linux Tauri app additionally passed native folder selection, synthetic
login, two turns, unsupported-service behavior without fallback, and native close
in private Xvfb/D-Bus. No live account, job or production service was used. These
results do not certify Windows installers. Exact published SHAs and CI status
are recorded in the platform's 2026-10-13 source checkpoint.

## Windows handoff

Use a clean checkout at that exact main SHA. Prerequisites: AMD64 Windows 10/11,
Node 24 LTS (24.15.0+, validated 24.21.0), Rust/Cargo/rustfmt/Clippy 1.98.1,
MSVC C++ tools, Windows SDK, WebView2, and Python for offline package drivers.
Keep tools/caches repository-local. Tauri CLI is 2.12.0; retain the lockfiles.

Run npm ci/test/build, rust:fmt/check/test/clippy, tauri:check/test and the 14 Python
release-asset tests. Exercise native Jobs, owned pipes, cancellation, deadlines,
tree cleanup, workspace validation, private state and mode negotiation.
Build `vision_fixture.exe` with `node scripts/cargo.mjs build -p
jv-desktop-process --example vision_fixture --locked`; use CLI's
`scripts/windows_vision_acceptance.py --desktop-probe ABSOLUTE_FIXTURE_EXE`
with the accepted external CLI launcher and its required engine/browser arguments.

Build with `npm run tauri -- build --bundles nsis`. Never distribute an
`offline-fixture` build. Run `scripts/windows-standard-user-smoke.py`,
`scripts/windows-installer-smoke.py`, `scripts/windows-native-smoke.py`, actual
WebView fixture acceptance, and `scripts/windows-native-gui-acceptance.mjs --app
ABSOLUTE_INSTALLED_EXE --installed --guided` as documented in DEVELOPMENT.md and
MANUAL_GUI_ACCEPTANCE.md. Native folder selection/Cancel, title-bar close,
installed launch, Unicode paths, reinstall/uninstall and state retention require
actual Windows verification. Ordinary UI tests use mocked CLI.

Tauri produces `target/release/bundle/nsis/JV Desktop_0.2.3_x64-setup.exe`.
The canonical public name is `JV-Desktop_0.2.3_x64-setup.exe`; the collector performs
that rename. CLI assets remain separate.

Generate metadata from actual builds with `python -B scripts/release-assets.py
--platform windows`; verify with `scripts/release-assets.py --verify DIRECTORY
--platform windows`. Check SHA256SUMS, BUILD-INFO.json, exact main/version, sizes,
hashes and unsigned status. Linux/Windows inventories must have identical commits
before `--combine`. Never reuse RC metadata for final-main packages.

After Windows acceptance and both green exact-main workflows, the separately
authorized official phase may create new `v0.2.3` at that same main SHA and push
normally. Existing CI must build, verify, combine and publish once, then verify
two fresh public downloads. Preserve RC1 and existing tags/assets. Historical
[Windows results](WINDOWS_PHASE2_HANDOFF.md) remain evidence for their original
commits. Any live service acceptance requires separate authorization and current
account eligibility. [Release notes](releases/0.2.3.md) and [changelog](CHANGELOG.md)
describe prepared user-facing behavior.
