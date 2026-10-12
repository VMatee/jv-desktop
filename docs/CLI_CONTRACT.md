# JV CLI compatibility baseline

Prepared Desktop 0.2.3 pairs Structured sessions with JV CLI 0.5.3. Protocol 1
negotiates `capabilities.tool_images` and `capabilities.transports` before sending
`session.start` with explicit `transport: structured` or `transport: legacy`.
Older bridges keep the platform compatibility floors below and can use Legacy.
Structured tool images stay inside CLI; Desktop events carry no image payloads.
Chat attachments remain unavailable. A failed Structured turn is never resent
automatically in Legacy. See [release preparation](RELEASE_PREPARATION.md).

Desktop's native adapter requires **bridge protocol 1** and platform-specific
released CLI versions: **0.5.0 or newer on Windows**, **0.4.9 or newer on Linux**.
The same floor is enforced by both version detection and bridge.ready validation.
Older versions report Upgrade required; detection never falls back to `exec`
positional prompts. BackendStatus exposes minimumCliVersion as structured data.

Linux invocation is exactly `jvcli bridge --stdio`. Windows invocation is
`jvcli.exe bridge --stdio --execution-mode trusted-host`, with selected workspace as cwd and
Desktop-owned `JVCLI_HOME`. Startup must return bridge.ready, protocol 1, compatible
CLI version and matching workspace. Detection is local-only and proves neither
authentication nor service health. Windows users install the independent
[official JV CLI 0.5.2 Windows release](https://github.com/VMatee/jv-cli/releases/tag/v0.5.2)
before Desktop. It retains Codex 0.149.1 and protocol 1. Desktop neither embeds
nor updates that runtime; Linux users may keep their released 0.4.9 CLI.

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


## Optional remote support codes

Protocol 1 `turn.failed` may additionally carry `source: remote`, `support_code`,
and optional `public_message`. Legacy `code: engine_error` remains valid. JV
CLI 0.5.2 preserves public job/response fields through a typed failure and a
per-turn memory field and is the recommended companion release for enhanced
remote support-code/status reporting. Older compatible CLIs remain supported
with legacy generic failures and may omit intermediate status events.

Rust and TypeScript independently require a full match of
`JV-(?:[A-Z]{1,16}-){1,3}[0-9]{3}`, at most 32 ASCII characters. No provider mapping
is maintained here. Unknown well-formed codes are displayed unchanged. Invalid
codes or missing remote source fall back to the existing local/generic error.

Public messages are at most 256 bytes and must exactly match one of:

- `The AI service requires administrator authentication.`
- `The service restarted after submission. The request was not sent again to prevent a duplicate.`
- `Provider job failed.`
- `Provider is temporarily unavailable.`

All other prose and all additional JSON fields are dropped. A valid code survives
an absent/rejected message. Extending this public text vocabulary requires review
at the CLI, Rust, and TypeScript boundaries; it does not require mapping codes.

Desktop represents these as `remote-failed`, source `remote`, with an optional
code in the shared DesktopError/event model. The banner and activity history show
selectable plaintext codes. Provider login failure does not clear the user's JV
sign-in. Errors remain in memory only.

## Nonterminal administrator attention

Protocol 1 optionally emits `turn.status` with `source: remote`, a current
`turn_id`, and `status: waiting_for_auth | running`. Rust requires the current
turn correlation and these exact enum values, then applies the same support-field
allowlist. TypeScript validates again. Waiting with missing/rejected public fields
shows a generic waiting notice; running discards all support fields.

`waiting_for_auth` displays “Waiting for administrator attention”, a selectable
support code when available, and reviewed public text. The turn remains active:
Send and workspace changes stay disabled, Stop remains available, and the user's
JV sign-in is retained. Running clears the active notice; a later success displays
the same turn's answer. Terminal failure, cancellation, local timeout, and process
failure also clear the active notice. Activity history may retain the earlier
waiting entry without treating it as the current status.

The CLI continues polling the same remote job and does not resubmit/cancel it
because it needs authentication. Existing local deadlines remain unchanged
(including Desktop's one-hour turn deadline). Older CLIs simply omit status events.
The opt-in structured Responses API currently hides this intermediate provider
state as `in_progress`; no auth-specific notice can be inferred from that envelope.

Native Windows acceptance of the remote support-code and authentication-wait
feature passed in the real Tauri WebView with synthetic bridge fixtures. See
[Validation](VALIDATION.md) for coverage; no live service validation is implied.
