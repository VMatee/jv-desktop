# JV Desktop contributor contract

This is the independent `VMatee/jv-desktop` repository. Read the parent workspace
AGENTS.md when present. Local source is authoritative.

- Modify only this repository. Never modify sibling repositories, production
  services, workspace infrastructure, or global configuration for Desktop work.
- JV CLI is an external executable. Never copy its source here. Its existing
  commands and JSONL interfaces are compatibility contracts.
- Ordinary unit/integration tests MUST use deterministic fake/mock JV CLI.
  No live service calls, accounts, scenarios, `jvcli exec`, or `jvcli resume`.
  Live acceptance requires separate explicit authorization.
- Never inspect secrets. Never log, persist in frontend storage, commit, or pass
  secrets in command-line arguments. No telemetry or password persistence.
- Use bounded structured JSONL, never terminal-screen scraping or raw HTML.
- Invoke processes with native argument vectors; never shell interpolation.
  Set cwd explicitly to a validated, explicitly chosen workspace.
- Native Linux integration uses reviewed `jvcli bridge --stdio` protocol 1 with
  CLI>=0.4.9. Never fall back to exec argv prompts or password environment variables.
  Keep tests synthetic/offline; production acceptance needs separate authorization.
- Native Windows uses `jvcli.exe bridge --stdio --execution-mode trusted-host`
  with official CLI>=0.5.0 on dedicated ordinary-user worker PCs. Its fixed full-access/network-enabled
  policy rejects read-only or network-denied requests. Never apply this policy
  to Linux, or copy JV CLI's implementation into Desktop.
- Keep platform process behavior isolated. Windows requires native semantics;
  never pretend POSIX cancellation works on Windows.
- Keep dependency caches, toolchains, temporary files and build output inside
  this repository. Do not install system packages or change global tools.
- Preserve local work. No force pushes, destructive resets/cleans/restores, or
  unrelated history changes. Review staged files and run checks before publishing.

Checks: `npm test`, `npm run build`, `npm run rust:fmt`, `npm run rust:check`,
`npm run rust:test`, `npm run rust:clippy`. See docs/DEVELOPMENT.md for GUI
prerequisites and the separate Tauri check. Do not claim a desktop build from
frontend/core tests alone.
