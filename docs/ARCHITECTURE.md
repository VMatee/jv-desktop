# Architecture

JV Desktop owns presentation and local process coordination. The separately
installed JV CLI continues to own engine/service behavior and workspace safety.
Nothing in this repository changes the CLI or existing server.

## Boundaries

```text
React UI (App, Conversation, Composer, ActivityFeed)
   |
JvBackend + normalized DesktopEvent
   |                         |
offline MockBackend          TauriBackend (invoke + per-run Channel)
                             |
                       Tauri commands/events
                             |
                       Rust JV process adapter
                             |
                       installed jvcli --json
                             |
                       existing JV service
```

The native turn path in this diagram is **gated**, not enabled. Tauri provides
native folder selection and `--version` detection. The demo emits deterministic
events entirely in memory. Rust executable integration is tested independently
using `fake-jvcli`, without a webview or any service credentials.

## Modules and ownership

| Module | Responsibility |
| --- | --- |
| `src/components` | Accessible plaintext UI; no command construction or IPC |
| `src/types/events.ts` | Stable discriminated events, requests and backend interface |
| `src/lib/jsonl.ts` | Framing, validation and normalization of engine JSONL |
| `src/lib/*backend.ts` | Transport selection, sanitized failures and subscriptions |
| `crates/jv-process` | Permission flags, workspace validation, process runner, detection |
| `crates/jv-process/src/platform.rs` | Isolated Linux process-group semantics |
| `src-tauri` | Main-window permissions, native picker, command registration, closed native gate |

`runTurn(request, onEvent)` subscribes to one run until its promise settles;
`cancelTurn(runId)` is idempotent. React associates every callback with the current
run ID and drops stale callbacks. It disables mode/workspace/policy changes while
a run is active. New workspace selections clear previous conversation/activity.
There is no disk history or resume implementation. A new demo turn is independent.

`chooseWorkspace()` returns a canonical native selection or cancellation. Rust
also stores the selected path and compares it against each native request. It
rejects implicit paths, missing/non-directory paths, filesystem root, home root
and the app executable's directory. No file is created as a validation probe.
Browser preview explicitly selects a virtual demo path; no file API is invoked.

## Structured events

Rust reads byte chunks into bounded lines. Tauri's reserved per-run channel carries
transport envelopes; the frontend adapter alone handles them. Presentation only
accepts `DesktopEvent`, never arbitrary subprocess objects.

| Engine event | Desktop event |
| --- | --- |
| `thread.started` | `session-started` |
| `turn.started` | `turn-started` |
| `item.started` / `command_execution` | `command-started` |
| `item.completed` / `command_execution` | `command-completed` |
| `item.completed` / `file_change` or `file_changes` | `files-changed` |
| `item.completed` / `agent_message` | `assistant-message` |
| `turn.completed` | `turn-completed` |
| `turn.failed` or `error` | sanitized `error` |
| malformed, oversized, invalid or unknown input | static `diagnostic` category |

The backend also emits `run-finished` with a completion/cancellation/failure
outcome. Process completion and engine turn completion are distinct concepts.
Raw stderr and arbitrary nested error messages are never presented. Blank lines
are ignored; CRLF and a final line without newline are supported. Unknown types
produce a diagnostic without copying their payload.

Limits: 64 KiB per line, 4 MiB per stdout/stderr stream, 4,000 stdout lines per
process, 32 KiB prompt, 16,384 Unicode code points per displayed text field,
100 file changes per event, 200 activity entries and 100 messages in memory.
Oversized lines are discarded through the next newline. Overall stream/event
overflow terminates the owned fixture process. All limits are explicit constants.

## Process core

Native `Command` uses an absolute program, argument vector, canonical cwd and a
pipe for fixture stdin. No shell is involved. The environment is cleared and
receives only a filtered absolute PATH, fixed locale and Python isolation flags.
The public API cannot inject an executable, environment, PID or shell command
through Tauri. CLI discovery skips relative/empty PATH entries.

Readers and writer use nonblocking pipes, bounded queues and size caps. A three-
second detection deadline is independent from GUI rendering. Linux spawns a new
process group and uses SIGTERM followed by SIGKILL after a 200 ms grace period.
`waitid(WNOWAIT)` reserves the leader PID through group cleanup before reaping,
avoiding reuse of a reaped PID as a signal target. An ownership guard cleans up on
errors/unwind. A descendant that deliberately creates another session escapes
group containment; bounded pipe shutdown prevents it from hanging the caller.
This is **not a sandbox** or a proof of cancellation for the real CLI's engine.
See [CLI contract](CLI_CONTRACT.md).

Windows process execution returns an unsupported category. POSIX signaling is
compiled only on Linux. A future Windows implementation must use native process
ownership/job objects. The UI and request/event contracts remain shared.

## Migration path

```text
current integration target (gated): Desktop → jvcli exec/resume --json
future:                            Desktop → jvcli bridge --stdio
```

Before enabling the current target, resolve the public prompt-argv limitation,
review authentication, revalidate known JSONL shapes, and establish cancellation
and app-close lifecycle semantics. The bridge can replace the adapter behind
`JvBackend`, retaining React components and normalized events. It must add framed
request IDs, version negotiation, bounded backpressure and reviewed secret
transport. No bridge, resume backend, or terminal is implemented in this phase.
