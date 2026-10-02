# Offline native GUI acceptance

Run from a normal Ubuntu graphical terminal. **Offline only: no JV password,
no real task, no exec/resume command, no production workspace.** Do not click Sign in. This checklist validates the shell and unauthenticated
bridge only; production acceptance is separately authorized.

```sh
cd /home/small-server/Desktop/RPALLMs/jv-desktop
mkdir -p .cache/manual-gui/workspace .cache/manual-gui/config .cache/manual-gui/data .cache/manual-gui/cache
export XDG_CONFIG_HOME="$PWD/.cache/manual-gui/config"
export XDG_DATA_HOME="$PWD/.cache/manual-gui/data"
export XDG_CACHE_HOME="$PWD/.cache/manual-gui/cache"
export GSETTINGS_BACKEND=memory
./target/release/jv-desktop
```

The exports apply only to this terminal and its children; do not put them in
shell startup files. Do not install a package or change display permissions.

- [ ] Window opens and React content renders. Resize near 800 × 700; composer,
  conversation and activity remain usable without horizontal clipping.
- [ ] Keep **Offline demo** selected. Open Folder opens the native dialog.
  Select this checkout's `.cache/manual-gui/workspace`; its path appears.
- [ ] Select `/`, your home directory itself, and this checkout's `target/release`
  in turn. Each must show a safe error. Re-select the disposable workspace.
- [ ] Enter `ช่วยตรวจ project นี้` using your usual Thai keyboard/IME. Confirm
  Enter during active composition does not submit; Shift+Enter adds a newline;
  Enter after composition sends only an offline simulated turn.
- [ ] See the simulated Thai assistant message and command/turn activity.
- [ ] Send a second harmless demo message and immediately click Stop. Events stop
  and a cancellation message appears; no project files change.
- [ ] Read only disables tool networking. Workspace write enables the network
  checkbox. Switching back to Read only clears/disables it. These Linux policies
  remain unchanged; Windows Native JV CLI uses the fixed policy below.
- [ ] Select **Native JV CLI**. The installed trusted CLI should be detected by
  its local `--version` check as 0.4.9 or newer. Select the disposable folder
  to start an unauthenticated bridge. The sign-in form appears; leave it empty.
  Send remains disabled until explicit authentication/session creation.
  Do not enter credentials or attempt a real task.

The registered Rust IPC boundary is checked safely without a WebView or CLI:

```sh
PATH="$PWD/.tools/node/bin:$PATH" npm run tauri:test
```

All registered IPC tests must pass, including `authentication-required` for valid native
requests. This does not claim a direct invocation from the production WebView;
that remains a separate harness-level check. Do not enable production devtools,
add permissions or authenticate to production to perform it.

Record date, OS/display session, executable version and pass/fail observations.
Known automated native coverage is in [Validation](VALIDATION.md); do not mark
untested cases passed. Package installation and real JV integration are outside
this checklist.

## Native Windows checklist

Use an ordinary Windows account, the separately installed accepted native CLI,
and an unsigned development Desktop build. Keep Windows Firewall enabled. Use
only a disposable project under this checkout's `.cache/manual-gui` directory.
No real Sign in or production turn is authorized by this checklist.

- [ ] Launch `target\release\jv-desktop.exe`; the native window renders.
- [ ] Offline demo still works, including Thai composition and Stop.
- [ ] Open Folder invokes the real native dialog. Select the disposable local
  drive folder with spaces/Thai in its name. Relative paths, drive root, home root,
  app directory and reparse/UNC paths are rejected.
- [ ] Native JV CLI detects the per-user executable and shows **Trusted host**,
  full filesystem access, network enabled and no tool approvals. Read-only and
  network-denied controls are unavailable. Leave the sign-in form empty.
- [ ] Reopen the folder; the prior owned bridge exits and a fresh one starts.
- [ ] Close the native window while the unauthenticated bridge is open. The
  owned tree exits, an unrelated CLI/sentinel survives, and reopening works.

Fixture WebView acceptance may exercise synthetic sign-in/turns separately, but
its fixed folder does not substitute for this real native dialog/close checklist.
