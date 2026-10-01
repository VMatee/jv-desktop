# Architecture

```text
React chat / activity / memory-only sign-in
  → JvBackend interface + normalized DesktopEvent
    → Offline MockBackend OR TauriBackend commands/channels
      → Rust BridgeSupervisor
        → installed jvcli bridge --stdio (Linux, CLI >= 0.4.9, protocol 1)
          → existing JV service (only after explicit user sign-in)
```

The CLI owns authentication, engine execution, session safety and sandbox policy.
Desktop embeds no CLI source and changes no server contract. The old `exec`
argument builder and closed `native_turn_gate` remain for compatibility tests;
neither is the native UI execution path. There is no argv-prompt fallback.

## Ownership and lifecycle

- `src/components`: accessible plaintext controls; no subprocess knowledge.
- `src/types/events.ts`: transport-independent backend and event contracts.
- `src/lib/tauri-backend.ts`: IPC, password request lifetime, per-run channels.
- `src/lib/bridge-events.ts`: validated presentation mapping of bridge fields.
- `crates/jv-process/src/bridge.rs`: fixed-argument child, protocol, state and limits.
- `platform.rs`: Linux process-group ownership; Windows execution unsupported.
- `src-tauri`: native folder picker, command allowlist, application-owned supervisor.

One bridge belongs to one explicitly selected canonical workspace. Rust remembers
the selection and compares it with every turn and the supervisor's workspace.
Workspace open/close operations are serialized. Changing folders shuts down the
previous child before starting a new one with explicit `current_dir`; the app
never globally changes cwd or sends a workspace field in bridge requests.
Selecting a folder does not create or edit project files.

`JVCLI_HOME` is the Tauri local application data directory plus `jvcli-state`.
On Linux this normally resolves to
`~/.local/share/com.openjvspace.desktop/jvcli-state/`. It is created with mode 0700,
with symlink ancestors rejected. Terminal CLI state is not reused. A future Windows
backend will use the same application-data API with Windows-native path/security
semantics, not a hardcoded Linux path.

The supervisor owns the child and all pipes. A mutex serializes protocol I/O;
blocking work runs outside the async UI thread. Cancellation and stopping are
atomic signals, so Stop does not wait for the turn I/O mutex. Auth and session
setup are synchronous with bounded deadlines. The UI locks permissions after
session creation; reopening the folder creates a new session policy. Request IDs
are Rust-generated; session and turn IDs remain in Rust. UI run IDs discard stale
channel callbacks. There is one active turn, no task queue, no Desktop resume yet.

## Transport and limits

Child argv is exactly `bridge --stdio`; passwords/prompts use NDJSON stdin.
Environment is cleared, then receives only filtered absolute PATH, HOME, fixed
locale/Python isolation settings and Desktop-owned JVCLI_HOME. Detection uses
`--version` with a3-second timeout. Readiness must identify protocol 1, a compatible
CLI version and the exact workspace before credentials can be sent.

Limits: 128KiB per inbound frame, 256KiB outbound request, 32KiB prompt, 4,000 frames
and 4MiB per turn, 64KiB discarded stderr per child lifetime. Bridge display limits
are 32KiB assistant text, 8KiB command, 16KiB command output,50 changed-file entries
with 1KiB paths. UI retains 200 activity entries and 100 messages. No unbounded
application event queue exists. Unknown future event types are ignored within
these limits; malformed known frames, unexpected correlation, EOF and overflow
fail safely and close the supervisor. Raw errors/stderr never cross IPC.

Deadlines: ready 5s, pipe write 3s, auth 40s, session 15s, turn 1hour, cancellation 20s.
Shutdown requests `bridge.shutdown` and waits 8s, then signals only the owned bridge
process group with TERM (20s grace), finally KILL/reap if necessary. The reviewed
bridge owns cleanup of its engine, which creates a separate session. Forced
termination after a pathological bridge hang cannot prove detached descendants
or remote work have stopped. No unrelated CLI PID is selected or killed.

## Event mapping

| Bridge | Desktop |
| --- | --- |
| turn.started | running |
| command.started / command.completed | command activity / result |
| files.changed | display-only changed paths |
| tool.started | tool activity |
| assistant.message | plaintext assistant message |
| turn.completed | ready/completed |
| turn.failed | sanitized error |
| turn.cancelled | stopped locally; remote state unknown |

React does not consume raw engine JSON. The legacy JSONL parser remains for the
original process fixtures; native bridge frames follow the separate protocol 1
mapping. Windows shares UI/contracts but returns unsupported for real execution.
Images, resume, PTY/ConPTY, terminal embedding and WSL are deferred.
