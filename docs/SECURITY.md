# Security boundaries

This foundation intentionally has no live task submission. Rust's native gate
returns a sanitized blocker before spawning exec/resume. UI disabling is a second
layer, not the security boundary. There is no environment switch or hidden flag
that enables native submission.

## Repository isolation

Desktop is an independent repository. Do not modify sibling CLI/platform trees,
copy CLI implementation, change production services, or install global tools as
part of Desktop development. Build output, dependency caches, local toolchains
and temporary test paths are ignored and remain inside this repository. Do not
run destructive Git cleanup or force pushes. Preserve existing work.

## Child execution and workspace trust

- Native process APIs use an argument vector and explicit cwd. No `sh -c`,
  `bash -c`, PowerShell interpolation, embedded shell or arbitrary UI executable.
- Prompt text is never executed. The fixture reads it from stdin. Current public
  JV CLI requires positional prompt arguments; Desktop blocks that path rather
  than silently exposing prompts in a process listing.
- Folder selection performs metadata/canonicalization checks only. It neither
  edits nor creates project files. The CLI remains responsible for authoritative
  write/sandbox checks when a future native path is enabled.
- PATH entries must be absolute for discovery. An installed executable is still
  trusted native code: `--version` is safe for the inspected official CLI, not a
  sandbox for an arbitrary replacement. Users must install a trusted CLI.
- Linux cancellation owns a child process group, never an arbitrary supplied PID.
  Detached sessions can escape it. Real CLI cancellation and remote-job state
  require further review; Stop currently stops only the in-memory demo in the UI.

## Secrets and authentication

No password form, password persistence, token cache, account discovery, telemetry
or raw child-environment logging exists. No localStorage/sessionStorage is used.
Chat/activity state stays in memory and is discarded when the app closes. Do not
enter credentials into prompts; this is not a secret manager.

The runner uses `env_clear` and a minimal allowlist. It does not inherit
`JV_API_PASSWORD`, authorization variables, SSH state, loader overrides or proxy
credentials. It never reads account files or secret sources. Raw subprocess
stderr is bounded and discarded, and only static error categories cross IPC.
Child stdout is untrusted content; it can contain workspace data, so it is never
logged or persisted by Desktop. Future diagnostic export needs a separate review.

The inspected CLI expects `JV_API_PASSWORD` for noninteractive authentication.
There is no safe shortcut in this foundation. A future sign-in design must be
explicitly user-entered, memory-only, redacted, excluded from arguments/logs, and
delivered only through a reviewed narrow child environment or bridge protocol.
It also needs lifecycle, zeroization limitations, crash handling and remote-job
review. Never add a `.env` password workaround. Never put secrets in `VITE_*`:
Vite embeds those values into frontend assets.

## Untrusted JSONL and rendering

64 KiB byte framing occurs before JSON parsing. Total streams, event counts,
display fields and UI histories are capped. Invalid UTF-8, malformed JSON,
unexpected shapes and unknown event types produce static diagnostics. No eval,
dynamic JavaScript, object spreading of subprocess data into UI props, terminal
control interpretation, unsafe Markdown or `dangerouslySetInnerHTML` is used.
Plaintext React rendering escapes HTML; changed-file paths are labels, not links
or instructions to open/write files.

The production CSP permits local application assets and Tauri IPC, not remote
scripts or network fetches. Only the local main window has application command
permissions. Folder picking occurs through a custom Rust command; the frontend
has no generic filesystem or shell plugin permission. The main capability grants
only `allow-choose-workspace`, `allow-detect-backend`, `allow-run-turn` and
`allow-cancel-turn`. Unused `core:default` permissions were removed during
Milestone 2; direct IPC tests verify an ungranted core command is rejected.
The dialog plugin is invoked from Rust, with no JavaScript dialog permission.
Its transitive filesystem dependency does not grant frontend filesystem access.
Development CSP additionally permits the loopback Vite HMR connection and its injected development scripts.

## Tests and production

Ordinary tests must never invoke real `jvcli exec` or `resume`, call
`ai.openjvspace.com`, consume scenarios, run migrations, or use an account/password.
The Rust test binary uses only stdio, timers and synthetic events; it contains no
HTTP client. Frontend unit tests reject fetch/XHR, and browser tests abort all
non-loopback requests. Native tests use an absolute Cargo-provided fixture path.
CI needs no JV secrets or deploy key and cannot publish a release.

Separately authorized live acceptance is a later activity, after the execution
gate is reviewed. Offline checks and backend detection are not production
acceptance, authentication, remote cancellation or service-health evidence.
