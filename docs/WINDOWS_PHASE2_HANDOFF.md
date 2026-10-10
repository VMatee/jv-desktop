# Windows Phase 2 completion and Linux Phase 3 handoff

Completed 2026-10-10 on the native Windows development PC. Windows-only release
candidates are published and public downloads have been verified. Both accepted
products remain installed. No production services were modified and no real
provider account was used. This is synthetic transport acceptance, not real
multimodal model acceptance.

## Exact source and immutable release references

| Product | Windows integration branch | Accepted build commit | RC tag |
| --- | --- | --- | --- |
| JV CLI 0.5.3 | feature/structured-vision-windows | fdf3328ddc83fa07b6f461362bbe17f3785435f8 | windows-rc-0.5.3-1 |
| JV Desktop 0.2.3 | feature/structured-vision-windows | c8b1083d93f07850750bd6f4bcd7f4d0b82b51e3 | windows-rc-0.2.3-1 |

Both integration heads were pushed normally and match their remote SHAs. Both
RC tags point to these exact commits. This documentation-only branch is
`docs/windows-phase2-handoff`, based on the accepted Desktop commit. It does not
change either integration branch, installer, RC tag or build provenance.

Linux Phase 1 history is preserved as ancestors:

- CLI: `3a1bfab0868ecb8b91cd8aa80bd9fbdf1b458e61`.
- Desktop: `41ea03fcbed04a75f6d493b5f7bd8e7e20b246c5`.

Clean local histories were preserved before fetching with rollback refs and
verified Git bundles. No repository was recreated, no force push/reset/clean was
used, and the independent repositories and existing SSH authentication remain
separate. Installation backups were explicitly waived by the operator; old
applications were removed using official uninstallers, without deliberately
removing account data or unrelated files.

## Public releases and assets

- [JV CLI Windows RC](https://github.com/VMatee/jv-cli/releases/tag/windows-rc-0.5.3-1)
- [JV Desktop Windows RC](https://github.com/VMatee/jv-desktop/releases/tag/windows-rc-0.2.3-1)

Both are published Pre-releases, not drafts or Latest. The repositories and
release pages are publicly accessible (anonymous HTTP 200). Stable Latest stays
CLI `v0.5.2` and Desktop `v0.2.2`. Final `v0.5.3` and `v0.2.3` tags were not created.

| Download | Bytes | SHA-256 |
| --- | ---: | --- |
| jv-cli-0.5.3-windows-x86_64-setup.exe | 99073428 | e68cbdc32c1816f773aaad2e5d0ac78310c4d2e429d92fb1cbbc13b45ee61f5b |
| jv-cli-0.5.3-windows-x86_64.zip | 144147744 | 7cf72fb3dea38620d9c29799bffa71ca0d12c9bad6cbc340eba71a6c43d184b2 |
| JV-Desktop_0.2.3_x64-setup.exe | 2181576 | 9ad19a7e086370065b2f8593398f826bb288ae4db3226375aba7a6916fada6d8 |

Each release also contains `SHA256SUMS` and `BUILD-INFO.json`. Metadata hashes:

| Product | BUILD-INFO.json | SHA256SUMS |
| --- | --- | --- |
| CLI | ca76be032117d142d5b129fac221e8d9093b2a3d7749fd50ac5e6506a1ec445a | c038e14e9f95f396fb7b6e6229b9a476ed61e83d44924ccbd33e44c07470db2e |
| Desktop | d77fca7784fc6c255faccee2f6e5a610c0f00989affd069a245dd737114fb094 | 761ffde69b21496d737bb518b768744ff8bd3088476b527cb7f9d255456b841c |

All seven public assets were downloaded into fresh ignored directories after
publication. Official verifiers accepted exact inventory, hashes, versions and
commits, including the CLI ZIP's package integrity. Downloaded bytes match the
locally accepted installation assets. No existing tag/asset was replaced.

CLI verification command (Windows Python launcher; use python3 on Linux):

```text
py -3.10 -B scripts/release_assets.py verify --output DOWNLOAD_DIRECTORY --platform windows --version 0.5.3 --commit fdf3328ddc83fa07b6f461362bbe17f3785435f8
```

Desktop verification command at its accepted source:

```text
py -3.10 -B scripts/release-assets.py --verify DOWNLOAD_DIRECTORY --platform windows
```

Publication used the supported GitHub CLI after local acceptance and exact-commit
feature CI passed. Existing official `v*.*.*` workflows, green-main requirements,
multi-platform gates and permissions were unchanged. RC tags do not match the
official release trigger. No official workflow failure was bypassed or relabeled.
The existing CLI release's targetCommitish metadata defaults to main; its actual
immutable tag and BUILD-INFO both resolve to the accepted Windows commit above.

## Windows environment and installation

- Windows 11 Pro, AMD64, version 10.0.26300 build 26300.
- Python 3.10.11; Pillow 12.0.0 for synthetic screenshot conversion.
- Repository-local Node 24.21.0, npm 11.19.0, Rust/Cargo 1.98.1.
- Tauri CLI 2.12.0; Windows WebView2 154.0.4258.62.
- Pinned engine 0.149.1 and bridge protocol 1 preserved.
- CLI built with `scripts/build_windows.py`; embedded Python 3.10.11, verified
  pinned engine and Inno Setup 6.4.3. No separately installed Python is required
  by the distributed runtime.
- Desktop built normally with `npm run tauri -- build --bundles nsis`.
  No offline-fixture variant was distributed.

Installed CLI: `%LOCALAPPDATA%\Programs\JV CLI\jvcli.exe`, reporting 0.5.3.
Its BUILD.json source_commit is the accepted CLI SHA and source_manifest_sha256
is `20b95faa35ed98d6bb40cb3c8d418771d508d0e41ab4329352c929c1d82de583`.

Installed Desktop: `%LOCALAPPDATA%\JV Desktop\jv-desktop.exe`, registered and
rendering 0.2.3. Its SHA-256 is
`9cfca60c7013197a498934a267dd21e2e64eb1e65ff405dd2f6d8176126103f1`.
Tauri changes its three-byte bundle marker from UNK to NSS inside the NSIS
payload; this explains the difference from the unbundled target/release EXE.
The installer and published copy match exactly. Desktop BUILD-INFO's Python
platform label says Windows 10; the native host identification above is Windows 11.

Install CLI first, then Desktop. Open a new terminal to see installer PATH
changes. Verify `jvcli --version`, launch Desktop, select Native JV CLI and open
a disposable workspace. Windows tools use fixed trusted-host full filesystem
and network access. Restricted-token network denial remains unsupported and
fails closed. No security boundary was relaxed to obtain these results.

## Implementation and shared changes

CLI Windows changes adapt the existing synthetic fixture to PowerShell argv and
fail-fast chaining, add a native queue-based bridge reader and real bundled
launcher acceptance, and use native private-ACL disposable image/history state.
The shared image fixture checks response call IDs, image dimensions and ordered
results. Existing Linux functionality was reused. Version metadata, release notes
and the 147-file source manifest were synchronized.

Desktop's shared Rust vision probe now supports the Windows supervisor. Fake
bridge fixtures cover explicit mode negotiation, unsupported-service failure and
genuine process exit. WebView tests use fresh browser data and assert that failed
turns retain a live session. Source/package/Cargo/Tauri versions are synchronized.

Browser E2E's Windows Vite failure was investigated: cold dev-server transforms
made page startup about 21.5 seconds, with a timeout and a 27.9-second retry.
The E2E server now builds and serves the production frontend with Vite preview.
The existing timeout, assertions and non-loopback network block remain intact.
Three consecutive Windows browser acceptance runs passed at about 2.4 seconds
per test. The failure was fixed, not skipped or hidden by a longer timeout.

## Acceptance evidence

CLI native checks passed:

- Windows preflight: 92 tests; native primitives/contracts: 212 tests, zero skips.
- Pinned engine check: 2; trusted acceptance: 11.
- Full bridge gate: all 212 categorized contracts plus real engine/trusted tests
  and six PowerShell/Python/Node IPv4/IPv6 socket cases.
- Structured image/history contracts: 13 included in the native gates; malformed
  image rejection, history/limits and unsupported-service ordering preserved.
- Release contracts: 19; source/package integrity: 147 source files verified.
- Actual bundled installer acceptance: 17 inner tests, zero skips, 19 bundled
  module provenance checks and six tool socket cases. Installation, version,
  help/diagnostics, Thai console/Ctrl+C, active-use refusal, rollback, reinstall,
  portable extraction/inventory and uninstall/PATH/state retention passed.
- Source, bundled/installed CLI and real Desktop Rust supervisor each completed
  synthetic image coding. Direct bridge restart/resume, cancellation contracts,
  single unsupported submission and explicit fresh Legacy sessions passed.

Desktop checks passed: 65 frontend tests; 43 Rust unit/bridge/process/native
checks; 6 registered Tauri IPC tests; 14 release-asset tests; frontend build,
Rust format/check/Clippy and Tauri check/test. Browser E2E passed three repeats.
Normal production WebView checks passed three assertions. Separate test-only
fixture WebView acceptance passed 25 assertions, including support codes,
Legacy selection, cancellation and genuine bridge termination; it was not shipped.

Official Desktop installer acceptance passed nine assertions: installation,
ordinary-user WebView launch, separate-state preservation, same-version reinstall,
second WebView launch, uninstall/registration removal, preserved state and unchanged
external CLI. Final application was then installed in its normal per-user path.
Installed native smoke passed three checks; installed WebView passed three.

Native automation was unavailable. The operator used the actual Windows folder
picker and title-bar X; the official driver verified six postconditions with
zero skips: version/Thai offline UI, CLI detection/trusted policy, distinct state
path, canonical space/Thai folder with real installed bridge, workspace IPC
rejection, and clean Desktop/bridge/descendant shutdown while an unrelated
sentinel survived. Evidence: `.cache/manual-gui/run-92Jrzj/results.json`.

A supplementary installed production GUI run used the actual installed CLI,
pinned engine and loopback synthetic provider. It generated HTML/CSS/JavaScript,
built/tested the website, captured browser screenshots, returned validated
PNG/JPEG/WebP view_image results, edited the website after inspection, reran final
tests, completed and accepted another turn in the same session. Unsupported
service returned JV-AGENT-UNSUPPORTED-001 without session loss, resubmission or
Legacy fallback. Password/token/image transport markers were absent from UI/logs.
The operator closed the native window normally. Provider counters proved two
completed tasks, three images, zero Legacy submissions and no fixture errors.
Evidence: `.cache/installed-vision-hgGUW6/report.json` and
`.cache/phase2-installed-full-vision.log` in Desktop. Temporary harness corrections
fixed an obsolete exact status-label assertion and accounted for Send being
disabled with an empty draft; no application check or product code was bypassed.

Runtime logs, test accounts, session files, installers and caches are ignored and
not committed. Terminal account state was not inspected. Synthetic Desktop sign-in
used only a loopback test account through the normal UI; real credentials were
never provided to the agent or source/CI.

Exact-build remote checks were independently verified after secure GitHub login:

- [CLI regression success](https://github.com/VMatee/jv-cli/actions/runs/38042569657)
- [Desktop checks success](https://github.com/VMatee/jv-desktop/actions/runs/38042615617)

## Safe Linux Phase 3 continuation

1. On Linux, inspect both existing repositories, worktrees, status, local commits
   and remotes. Preserve newer work before fetching. Keep repositories separate;
   never copy .git, caches, Windows runtimes or generated installers between them.
2. Fetch each `feature/structured-vision-windows` branch and RC tag normally.
   Verify full SHAs against the table above and confirm the respective Linux
   baseline is an ancestor. Fetch Desktop `docs/windows-phase2-handoff` for this
   report. Preserve existing SSH/configuration and all immutable RC tags/assets.
3. Create Linux Phase 3 integration branches from the current appropriate local
   baseline, then merge the Windows feature history (fast-forward where possible).
   Resolve conflicts by reviewing both implementations. Do not reset, force-push,
   cherry-pick away independent history or merge main prematurely.
4. Rerun complete Linux source/engine/bridge/image/history/session and Desktop
   frontend/Rust/Tauri/browser/native GUI acceptance. The shared fixture changes
   require Linux regression; Windows passes do not substitute for this. Revalidate
   PNG/JPEG/WebP, follow-up edits, cancellation/resume, errors and explicit Legacy.
5. Rebuild and verify all Linux packages, including supported package installation
   acceptance. Review the combined source diff, versions, notices, manifests and
   package metadata. Keep CLI 0.5.3/Desktop 0.2.3 synchronized unless tag availability
   or new changes require a deliberate version decision.
6. Complete authorized real-provider smoke separately through secure local sign-in
   if available. Current synthetic results do not prove semantic vision analysis.
7. Before official releases, obtain green exact-main coordinated multi-platform
   workflows and satisfy their existing gates. Source changes require affected
   Windows regression and new accepted Windows builds at the official source SHA;
   do not relabel these RC binaries as builds of a later commit.
8. Only after coordinated prerequisites pass, create final v0.5.3/v0.2.3 tags and
   publish the complete official cross-platform releases with new verified
   inventories and fresh public-download checksum verification. Do not move RC tags.

Remaining limitations: installers are unsigned; real-model vision smoke remains
pending; Linux integration/revalidation and final cross-platform publication belong
to Phase 3. No remaining mandatory Windows Phase 2 acceptance/publication blocker.
