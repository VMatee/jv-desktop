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
  checkbox. Switching back to Read only clears/disables it. No bypass mode exists.
- [ ] Select **Native JV CLI**. The installed trusted CLI should be detected by
  its local `--version` check as 0.4.9 or newer. Select the disposable folder
  to start an unauthenticated bridge. The sign-in form appears; leave it empty.
  Send remains disabled until explicit authentication/session creation.
  Do not enter credentials or attempt a real task.

The registered Rust IPC boundary is checked safely without a WebView or CLI:

```sh
PATH="$PWD/.tools/node/bin:$PATH" npm run tauri:test
```

All four tests must pass, including `authentication-required` for valid native
requests. This does not claim a direct invocation from the production WebView;
that remains a separate harness-level check. Do not enable production devtools,
add permissions or authenticate to production to perform it.

Record date, OS/display session, executable version and pass/fail observations.
Known automated native coverage is in [Validation](VALIDATION.md); do not mark
untested cases passed. Package installation and real JV integration are outside
this checklist.
