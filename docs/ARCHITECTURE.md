# Architecture

```text
React chat / activity / memory-only sign-in
  → JvBackend interface + normalized DesktopEvent
    → Offline MockBackend OR TauriBackend commands/channels
      → Rust BridgeSupervisor
        → installed JV CLI (Linux >= 0.4.9 / Windows AMD64 >= 0.5.0, protocol 1)
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
- `platform.rs`: Linux process-group ownership; `platform/windows.rs`: native
  Windows Job ownership, process creation, UTF-16 arguments and bounded pipes.
- `bridge_transport.rs`: shared NDJSON framing/correlation with isolated native I/O.
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
with symlink ancestors rejected. Windows normally resolves to
`%LOCALAPPDATA%\com.openjvspace.desktop\jvcli-state`. Desktop rejects reparse
ancestors and supplies the absolute child path without pre-creating it; JV CLI
creates/protects its private state when needed. Terminal CLI state is not reused.
No Windows chmod approximation or duplicate ACL implementation is used.

The supervisor owns the child and all pipes. A mutex serializes protocol I/O;
blocking work runs outside the async UI thread. Cancellation and stopping are
atomic signals, so Stop does not wait for the turn I/O mutex. Auth and session
setup have bounded deadlines and observe shutdown; a stopped operation cannot
install a late result. The UI locks Linux permissions after
session creation; reopening the folder creates a new session policy. Request IDs
are Rust-generated; session and turn IDs remain in Rust. UI run IDs discard stale
channel callbacks. There is one active turn, no task queue, no Desktop resume yet.

## Transport and limits

Linux argv is exactly `bridge --stdio`. Windows argv is exactly
`bridge --stdio --execution-mode trusted-host`; passwords/prompts use NDJSON stdin.
Environment is cleared. Linux receives filtered absolute PATH, HOME, fixed
locale/Python isolation settings and Desktop-owned JVCLI_HOME. Windows receives
only reviewed absolute SYSTEMROOT, WINDIR, USERPROFILE, LOCALAPPDATA, TEMP, TMP,
filtered absolute PATH and explicit JVCLI_HOME. APPDATA, COMSPEC, PATHEXT, proxy,
credential and Python injection variables are not inherited. Detection uses
`--version` with a3-second timeout. Readiness must identify protocol 1, a compatible
CLI version and the exact workspace before credentials can be sent.

Limits: 128KiB per inbound frame, 256KiB outbound request, 32KiB prompt, 4,000 frames
and 4MiB per turn, 64KiB discarded stderr per child lifetime. Bridge display limits
are 32KiB assistant text, 8KiB command, 16KiB command output,50 changed-file entries
with 1KiB paths. UI retains 200 activity entries and 100 messages. No unbounded
application event queue exists. Unknown future event types are ignored within
these limits; malformed known frames, unexpected correlation, EOF and overflow
fail safely and close the supervisor. Raw errors/stderr never cross IPC.

Linux pipes remain nonblocking. Windows has one owner per synchronous pipe,
uses PeekNamedPipe before bounded reads, and a bounded one-element completion
channel for the sole stdin writer. A write deadline terminates only the owned
Job, cancels synchronous writer I/O and joins its thread; no writer is detached.
CreateProcessW uses an explicit stdio handle list and atomic Job-list attribute,
with no retry outside ownership. Kill-on-Job-close and absent breakaway flags
contain child/grandchild and nested CLI Jobs, including Desktop owner death.

Deadlines: ready 5s, pipe write 3s, auth 40s, session 15s, turn 1hour, cancellation 20s.
Shutdown requests `bridge.shutdown` and waits 8s. Windows then terminates the
owned Job and waits at most 5s for its root. Linux signals only the owned bridge
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
| turn.status | nonterminal administrator wait / resumed; retains active turn |
| turn.failed | legacy local/generic error or validated remote support code |
| turn.cancelled | stopped locally; remote state unknown |

React does not consume raw engine JSON. The legacy JSONL parser remains for the
original process fixtures; native bridge frames follow the separate protocol 1
mapping. Structured backend status declares `executionMode`: Linux `sandboxed`,
Windows `trusted-host`. React never parses security behavior from status prose.
Windows hides incompatible controls and fixes workspace-write/network-true IPC;
Rust independently rejects conflicts and requires danger-full-access/network-true
session responses. Linux rejects unexpected full-access responses and retains its
existing permission model. Desktop does not impose Windows filesystem sandboxing.
Images, resume, PTY/ConPTY, terminal embedding and WSL are deferred.


Remote error flow: public server job/response fields → CLI typed remote failure
and current-turn adapter state → optional protocol 1 failure fields → Rust field
allowlist → TypeScript validation → shared DesktopError → banner/activity. Public
messages use an exact reviewed vocabulary, not heuristic secret redaction. The
code grammar and public text contract are in CLI_CONTRACT.md. Local BackendError
and remote events use the same UI model; neither raw diagnostics nor provider
exception mappings enter React.


Authentication waiting is a status, not a failure. CLI job polling retains the
remote job, and an optional coalesced `turn.status` crosses the same correlation
and public-field validators. React tracks the current notice separately from
terminal errors, using shared PublicSupport fields. Resumption or any terminal
outcome clears the notice; only the backend turn promise ending releases busy
state. Existing cancellation and deadline policies continue to apply.
