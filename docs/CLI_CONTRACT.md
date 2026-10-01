# JV CLI compatibility baseline

Desktop's native adapter targets **JV CLI 0.4.9 or newer with bridge protocol 1**.
The installed/source mismatch observed in Milestone2 (installed0.4.7/source0.4.8)
is superseded by the verified0.4.9 release and local upgrade. Older versions report
Upgrade required; detection never falls back to `exec` positional prompts.

Invocation is exactly `jvcli bridge --stdio`, with selected workspace as cwd and
Desktop-owned `JVCLI_HOME`. Startup must return bridge.ready, protocol 1, compatible
CLI version and matching workspace. Detection is local-only and proves neither
authentication nor service health. Linux is the only current executable backend.

| Action | Bridge request |
| --- | --- |
| Explicit sign-in | auth.login, username/base_url/password over stdin |
| Session creation after auth | session.start |
| User message | turn.start, text over stdin |
| Stop | turn.cancel, current Rust-owned turn_id |
| Workspace/app close | bridge.shutdown |

| Policy | read_only | allow_network |
| --- | --- | --- |
| Read only | true | false |
| Workspace write + network | false | true |
| Workspace write without tool network | false | false |

Read-only plus network is rejected. No bypass mode exists. Tool network denial
does not disable the JV service connection. The CLI owns sandbox/session rules.

Passwords never enter argv, environment, config or session metadata. The bridge
holds token ownership and performs logout; Desktop receives no token. Prompts
use stdin at both Desktop→bridge and bridge→engine boundaries. Existing terminal
exec/resume and their JSON modes remain separate, unchanged interfaces.

The protocol is documented in the independent CLI repository's
`docs/BRIDGE_PROTOCOL.md`. Desktop uses allowlisted normalized bridge events,
not raw engine events. Cancellation is local-only; remote state remains unknown.
Current Desktop does not expose bridge resume, new-thread or images. No live
production compatibility acceptance has yet been performed; the next acceptance
must be separately authorized and controlled.
