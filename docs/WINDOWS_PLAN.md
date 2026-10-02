# Windows native integration

Target: AMD64 dedicated JV worker PCs, using an ordinary Windows account and
WebView2. The native backend is implemented on `feature/windows-native`; Desktop
remains version 0.1.1 and this is not a published Windows production release.

## Execution contract

```text
React → JvBackend → Tauri IPC → Rust BridgeSupervisor
      → installed jvcli.exe bridge --stdio --execution-mode trusted-host
      → shared bridge protocol 1
```

The external CLI must be 0.4.9 or newer; the accepted Windows dependency is CLI
checkpoint `4a00ee480db70d5bcac619d95afa64056cd1905c`, with Codex 0.149.1.
Desktop does not copy, rebuild, bundle, update or silently install JV CLI.
Resolution prefers `%LOCALAPPDATA%\Programs\JV CLI\jvcli.exe`, then absolute
PATH entries. It excludes current/project directories and reparse/UNC/device
candidates. Detection checks only local version/readiness, not service health.

Windows Native JV CLI always displays **Trusted host**: full filesystem access,
network enabled, no tool approval prompts. Tools run with the dedicated account's
normal privileges. Incompatible permission controls are hidden and conflicting
IPC policies are rejected. Session responses must confirm danger-full-access and
network true. Linux retains its existing sandboxed permission controls and args.

Windows restricted-token network-denied execution remains unsupported with the
pinned Codex 0.149.1 backend. Trusted host is intentional unrestricted authority,
never a fallback after isolation failure. Administrator, services, sandbox-account
provisioning and Firewall/WFP changes are not required. Keep Windows Firewall on.

## Desktop ownership

Desktop sets a separate absolute
`%LOCALAPPDATA%\com.openjvspace.desktop\jvcli-state` child. JV CLI creates and
protects its own private state; Desktop does not duplicate Windows ACL logic.
Terminal CLI state at `%LOCALAPPDATA%\JV\jv-cli` is not reused.

Native creation establishes an anonymous kill-on-close Job before bridge code
runs. Descendants and nested CLI Jobs remain owned, with no breakaway fallback.
Only private stdio handles are inherited. Bounded native pipe reads/writes,
deadlines, correlation, cancellation and normalized protocol behavior are shared
with Linux. Shutdown is cooperative, followed by owned Job cleanup when necessary.
Unrelated processes are never selected by name or terminated.

## Validation and next steps

Ordinary tests use native fake executables and synthetic accounts. The opt-in
installed CLI smoke covers version, ready, exact workspace/state, nested Job
ownership, shutdown, owner death and a loopback-only rejected login. Actual Tauri
WebView acceptance covers rendered UI, real CLI detection and fixture sign-in,
Thai events, Stop, folder reopen and cooperative bridge close. See
[Validation](VALIDATION.md) for exact coverage and limits and
[Development](DEVELOPMENT.md) for commands.

UNC/device namespaces and reparse workspaces/state are rejected. Ordinary local
drive paths, spaces, Thai and case aliases are tested; broad long-path/remote
filesystem support is not certified. No production account acceptance is implied.
Native folder-dialog interaction and window-manager close require a working
native GUI automation surface or separate manual acceptance; a fixture picker
does not prove those native controls.

Controlled live acceptance, Windows release packaging/signing and versioning are
separate milestones. JV CLI remains separately installed. WSL, updates, terminal
embedding, PTY/ConPTY and public release are outside this integration pass.
