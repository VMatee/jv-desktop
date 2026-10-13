# Windows acceptance and official release handoff — 2026-10-13

Both official cross-platform releases are published and verified. The operator's
latest authorization explicitly requested official v0.5.3/v0.2.3 publication after
real Structured acceptance and all release gates passed, superseding the earlier
RC-only phase restriction. Existing RC1 tags and assets were preserved.

## Exact accepted source and publication

| Product | Accepted main and tag commit | Official release | Tag workflow |
| --- | --- | --- | --- |
| JV CLI 0.5.3 | `279ee7f2546ed29de589feb1b1ebc4d38eb302a4` | [v0.5.3](https://github.com/VMatee/jv-cli/releases/tag/v0.5.3) | [38370879613](https://github.com/VMatee/jv-cli/actions/runs/38370879613) |
| JV Desktop 0.2.3 | `afb937ed12181b15f2dc7bac53f31e6442050467` | [v0.2.3](https://github.com/VMatee/jv-desktop/releases/tag/v0.2.3) | [38370979085](https://github.com/VMatee/jv-desktop/actions/runs/38370979085) |

Both releases are public, non-draft, non-prerelease and Latest. Anonymous release
page requests returned HTTP 200. Native Windows work was performed on this PC;
the existing GitHub Actions workflows performed their coordinated Linux builds.
No Linux production host was used or changed from Windows.

Before tagging, exact-main CLI regression/package runs 38288133900/38288134250
and Desktop checks/package runs 38288162856/38288162822 were green. Both tag
workflows then passed every verification, native build, installer acceptance,
assembly and publication job. Their two fresh public-download checks passed.
A further fresh Windows download verified every asset, embedded provenance,
SHA256SUMS, BUILD-INFO, byte size and GitHub API SHA-256 digest.

Tags were created once and pushed normally using existing repository-specific
SSH configuration. No force push or existing tag/asset replacement occurred.
Public release bodies were updated after successful publication to replace
the historical source-preparation wording with current installation instructions.
This documentation branch does not change either accepted main or tagged binary.

## Published inventory and verified SHA-256

| Product | Asset | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| cli | BUILD-INFO.json | 3356 | `2785e8073f7cb223f31e7b35f0cfec36222b67fe358003532849fe0e10081f3e` |
| cli | jv-cli-0.5.3-linux-x86_64-offline.zip | 168362646 | `413f2cc512dd34294a5bd2529b8983f1bb01a64c5ef6a06dec3d966f81ccb887` |
| cli | jv-cli-0.5.3-linux-x86_64-offline.zip.sha256 | 104 | `eaf57b56521d7fbc94ab3d60dbe76c99edcaff6096989db5dd8ae3722fe2f8cf` |
| cli | jv-cli-0.5.3-linux-x86_64.zip | 394611 | `4785b92432d226cfc001eb7f82ad5aa6dcb73a355ff3555e4ca701e221e86e88` |
| cli | jv-cli-0.5.3-linux-x86_64.zip.sha256 | 96 | `2a06083006bf573024e57d812a897d70734a37471b3d897bc86a875321d071c7` |
| cli | jv-cli-0.5.3-windows-x86_64-setup.exe | 99083533 | `f971527adaa347983246e61c1067156361a745a6fcc8992dbc421397a3c8db68` |
| cli | jv-cli-0.5.3-windows-x86_64.zip | 144147916 | `7fa21c1e7b22fbc0e050bc482be7d909aaee987d7393b2738e54c5f53892b887` |
| cli | SHA256SUMS | 698 | `889db3bcc5dd2ae199a32eb0af2ca08885cc634de6a8e399fccbffe22e4e92e6` |
| desktop | BUILD-INFO.json | 2967 | `9784195fa1ea5ebc454f3ac6f83eaa1c9ca265e914467edbe7f8d46df7615341` |
| desktop | JV-Desktop-0.2.3-1.x86_64.rpm | 2840650 | `11be121b94aa8b8f2a4787367f0ab38ba5164e577a620b0c665e01b886d76bfb` |
| desktop | JV-Desktop_0.2.3_amd64.AppImage | 81263096 | `1473532c0cdb4a7276ef0d9fbe8078c5a5c4d061b62ad860aa71055c59ccb1ce` |
| desktop | JV-Desktop_0.2.3_amd64.deb | 2839534 | `60606c1f6c24a4852b324a05f7a53376916de73ae25d18b4857b6bdd5dfc2f51` |
| desktop | JV-Desktop_0.2.3_x64-setup.exe | 2181098 | `718fe6231f7ee4742d7cd6a6c104aae69cf50ad927347f349c5342f3006674a4` |
| desktop | SHA256SUMS | 466 | `c10181f07a321ee85292c060fca568465961e05f4d8418c7403c4cdc56ee3d9e` |

The CLI release has eight assets: Windows installer/portable ZIP, Linux normal
and offline ZIPs with individual checksum files, and combined metadata/checksums.
Desktop has six: Windows NSIS EXE, Linux DEB/RPM/AppImage, and metadata/checksums.
SHA256SUMS covers every other asset; its own hash was checked against GitHub's
asset digest. Windows builds are unsigned and are not claimed reproducible.

## Windows environment and local acceptance

Windows 11 Pro AMD64, kernel 10.0.26300/build 26300; Python 3.10.11; repository-local
Node 24.21.0/npm 11.19.0; Rust/Cargo 1.98.1; Tauri CLI 2.12.0; WebView2
154.0.4258.62. Engine 0.149.1 and bridge protocol 1 remain pinned.

CLI checks: preflight 92; native contracts 214 with zero skips; engine 2;
trusted-host 11; full bridge/ACL/process/auth/grammar/journal/image/history gates
and six real PowerShell/Python/Node IPv4/IPv6 cases passed. Source manifest
verification covered 149 shipped files. Source, installed CLI and real Desktop
Rust supervisor each passed synthetic PNG/JPEG/WebP coding, further edits,
unsupported-service single submission, and explicit fresh Legacy selection.
Direct bridge restart/resume passed. Security and image/history limits remain intact.

Desktop checks: 65 frontend, 43 Rust core/bridge/native, six Tauri IPC, and 14
release-contract tests passed. Frontend build, Rust format/check/Clippy, Tauri
check/test and Browser E2E passed. Packaging used the normal production NSIS
workflow, never an offline-fixture release variant.

Native manual-action acceptance plus automated postconditions passed six checks
on the accepted installed main build: Offline Demo/Thai UI, external CLI detection
and trusted-host policy, separate state path, real folder picker with a path
containing spaces and matching real bridge, invalid workspace IPC rejection,
and title-bar X cleanup of Desktop/bridge/owned descendants while an unrelated
sentinel survived. Native controls were operated by the user; these are not
claimed as fully automated GUI interactions.

After publication, the actual downloaded CLI installer passed the official
install/reinstall/failed-replacement rollback/active-use refusal/console/portable/
uninstall/PATH/state acceptance: 17 inner tests, zero skips, 19 bundled-module
checks and six tool network cases. All 78 installed payload files verified.
The downloaded Desktop NSIS passed all nine official installer assertions,
including ordinary-user WebView launch, reinstall, uninstall, state retention
and unchanged external CLI. Both public products were restored to normal paths.
The public installed Desktop rendered its UI, detected CLI trusted-host
capabilities and shut down its bridge cooperatively. The final public CLI also
passed the real Desktop Rust supervisor's synthetic three-image/two-turn test,
unsupported-service handling and explicit Legacy selection.

## Real JV service acceptance

The user signed in locally through the normal Desktop form with the dedicated
test account. Credentials were never read, logged or placed in source/CI.
Testing used an explicitly disposable workspace, not the Snake Game project.
The accepted main builds completed exactly three new user turns:

1. Harmless text-only Structured admission and the expected response.
2. PNG/JPEG/WebP view_image calls, correct blue/green shape observations, creation
   and modification of HTML/CSS/JavaScript, and a bounded command check.
3. A follow-up file edit in the same authenticated session.

All three completed with no public support code. Content-free journal diagnostics
confirmed three forwarded image MIME types, three view_image calls, subsequent
apply_patch/shell actions, registered responses and zero uncertain-submission
retries. The generated website passed a separate Chromium check: expected final
heading, working button, zero page errors and zero external requests. A screenshot
was visually inspected. The user closed the native window with title-bar X;
the native process exited zero.

This is bounded real model/vision acceptance, beyond the separate synthetic
transport tests. It establishes eligibility of this account at the time tested,
not universal eligibility of all accounts/providers. The real test used local
builds from the exact tagged commits; public CI builds from those same commits
subsequently passed downloaded-installer and installed synthetic acceptance.

The earlier Windows grammar correction is retained as ancestor
`ffe028d76bf20ba893e54fb0295cfd45bcb54936`. Historical Linux evidence separately
identified owner/fallback admission mismatches. After the reported Linux-side
correction, current real admission and visual coding succeeded. Windows did not
alter service routing/configuration or retrospectively establish the previously
uncaptured deployed rejection message. Grammar canonicalization alone is not
claimed to have resolved the original end-to-end failure.

## Harness observations and evidence

An initial native authentication test timed out while concurrent dependency
installation loaded the PC. Its focused check and the full 214-test suite passed
on a quiet rerun; no test, timeout or product behavior was weakened.

The temporary live-test driver initially expected an obsolete Activity label.
It was corrected to observe the actual completed outcome without resubmitting.
Its visual-turn observer deadline later expired just before the existing turn
completed successfully; the same result was reconciled without replay. The final
report records both observations and all three successful turns.

During final public Desktop installation, Python argv quoting caused NSIS /D
to be ignored and the installer reused the owned acceptance destination. The
official uninstaller removed that disposable installation; an unquoted final /D
suffix, with no shell, restored the normal path. This was a temporary driver
correction, not a product defect or source change. Final registry/path/version
checks passed. No application state or unrelated files were deliberately deleted.

Private ignored evidence locations:

- CLI `.cache/official-053-20261013/`: native/source/package/vision logs and
  content-free real transport observation.
- Desktop `.cache/official-023-20261013/`: build/test/package/WebView logs,
  real website browser result and screenshot.
- Desktop `.cache/manual-gui/run-KzxVbT/results.json`: installed native controls.
- Desktop `.cache/official-real-Cd8KcF/final-report.json`: real three-turn result.
- Each repository `.cache/official-public-VERSION-20261013.json` and
  `.cache/public-installed-VERSION-20261013.json`: public bytes and final install.

Generated installers, runtime state, credentials, prompts and image contents
are not committed. The operator waived installation backups. Official
uninstallers preserved separate account/session data.
The disposable Desktop `JV Test` folder remains: automatic approval review
rejected its optional removal with "blocked by policy". Its marker and seven-file
inventory identify the test workspace. This cleanup is not a release gate.

## Final installed pair

- JV CLI 0.5.3: `%LOCALAPPDATA%\Programs\JV CLI\jvcli.exe`; installed EXE SHA-256 `f2e9deed4da2938b8f30739ad43fafdb569468779b60f71e8a32066c3d70ccc1`.
- JV Desktop 0.2.3: `%LOCALAPPDATA%\JV Desktop\jv-desktop.exe`; installed EXE SHA-256 `01b3d85f9c94058652f08566de655297763d3447feec65c41f1ac2dc0e006519`.

Install CLI first, then Desktop. Open a new terminal after PATH changes.
Windows uses explicit trusted-host full account filesystem/network access;
unsupported restricted-token network denial continues to fail closed.

## Linux continuation

The original Linux baselines remain ancestors of the accepted main commits:
CLI `3a1bfab0868ecb8b91cd8aa80bd9fbdf1b458e61` and Desktop
`41ea03fcbed04a75f6d493b5f7bd8e7e20b246c5`. Windows integration branches remain
at CLI `fdf3328ddc83fa07b6f461362bbe17f3785435f8` and Desktop
`c8b1083d93f07850750bd6f4bcd7f4d0b82b51e3`; their history was integrated into
the authoritative main commits above. Both local main trees are clean.

On Linux, inspect local status/worktrees/independent commits before fetching.
Fetch main, the official tags, and Desktop `docs/windows-official-release-20261013`
normally. Verify the full main/tag SHAs above. Fast-forward only when appropriate;
preserve divergent local work on a branch and review any merge. Keep repositories,
Git configuration, credentials and platform runtimes separate. Do not reset or
force-push, copy Windows build directories, or recreate/move existing tags.

The formerly pending coordinated official publication has now completed through
the existing cross-platform workflows. No further Phase 3 release is needed for
these version numbers. Future changes need new versions, fresh platform acceptance
and new immutable release tags. Any additional Linux runtime or deployment work
remains subject to its own checks and authorization; Windows made no production
service changes. There is no remaining mandatory Windows release blocker.
