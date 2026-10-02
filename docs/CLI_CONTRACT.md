# JV CLI compatibility baseline

Desktop's native adapter targets **JV CLI 0.4.9 or newer with bridge protocol 1**.
The installed/source mismatch observed in Milestone2 (installed0.4.7/source0.4.8)
is superseded by the verified0.4.9 release and local upgrade. Older versions report
Upgrade required; detection never falls back to `exec` positional prompts.

Linux invocation is exactly `jvcli bridge --stdio`. Windows invocation is
`jvcli.exe bridge --stdio --execution-mode trusted-host`, with selected workspace as cwd and
Desktop-owned `JVCLI_HOME`. Startup must return bridge.ready, protocol 1, compatible
CLI version and matching workspace. Detection is local-only and proves neither
authentication nor service health. Windows integration targets the independent
accepted CLI checkpoint `4a00ee480db70d5bcac619d95afa64056cd1905c`
(CLI 0.4.9, Codex 0.149.1). Desktop neither embeds nor updates that runtime.

| Action | Bridge request |
| --- | --- |
| Explicit sign-in | auth.login, username/base_url/password over stdin |
| Session creation after auth | session.start |
| User message | turn.start, text over stdin |
| Stop | turn.cancel, current Rust-owned turn_id |
| Workspace/app close | bridge.shutdown |

| Linux policy | read_only | allow_network |
| --- | --- | --- |
| Read only | true | false |
| Workspace write + network | false | true |
| Workspace write without tool network | false | false |

On Linux, read-only plus network is rejected. Tool network denial
does not disable the JV service connection. The CLI owns sandbox/session rules.

Windows Native JV CLI has one fixed trusted-host policy: `access=workspace-write`,
`network=true` in Desktop IPC, with the explicit trusted-host process argument.
This selects full filesystem access, network enabled and approvals never. Rust
rejects read-only or network-false requests before authentication/execution; it
does not silently reinterpret them. A session response must report
`sandbox=danger-full-access` and `network=true` on Windows. Linux still requires
the exact requested read-only/workspace-write sandbox and rejects full access.

Pinned Codex 0.149.1 restricted-token mode does not enforce direct socket denial
on Windows. This limitation is not fixed by Desktop. Dedicated worker machines
deliberately use trusted host; users requiring untrusted network-denied execution
must not use this profile.

Passwords never enter argv, environment, config or session metadata. The bridge
holds token ownership and performs logout; Desktop receives no token. Prompts
use stdin at both Desktop→bridge and bridge→engine boundaries. Existing terminal
exec/resume and their JSON modes remain separate, unchanged interfaces.

The protocol is documented in the independent CLI repository's
`docs/BRIDGE_PROTOCOL.md`. Desktop uses allowlisted normalized bridge events,
not raw engine events. Cancellation is local-only; remote state remains unknown.
Current Desktop does not expose bridge resume, new-thread or images. Controlled Linux acceptance passed using CLI 0.4.9. It is not repeated by
release tests; any further live work needs separate authorization.
