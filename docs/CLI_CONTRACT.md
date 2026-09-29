# Inspected JV CLI contract

The installed official CLI source was inspected read-only during foundation work:
**JV CLI 0.4.7**, public `lib/jvcli/cli.py` parser and dispatch, plus its launcher.
No account configuration, session, environment secret, credential or key content
was opened. No real exec/resume command was run. Re-inspect this contract before
enabling native submission; version detection alone cannot prove compatibility.

## Public input versus internal engine input

The public parser defines:

```text
jvcli exec [permission flags] --json PROMPT [PROMPT ...]
jvcli resume SESSION_ID [permission flags] --json [PROMPT ...]
```

`exec` has a required positional prompt (`nargs="+"`). The dispatcher joins the
prompt arguments with spaces. It does **not** implement an exec stdin sentinel.
Passing `-` would be a literal prompt, not an instruction to read stdin. Without
a positional prompt, noninteractive use is rejected. The engine subprocess
inside JV CLI separately uses stdin, but that private implementation does not
make stdin available to Desktop at the public CLI boundary.

Desktop therefore does not invent `exec --json -`, pipe a prompt into an
unsupported interface, or put user text into argv. The Rust `exec_args` function
constructs only the verified prefix/permission flags. `native_turn_gate` blocks
the request before a process is created. A future explicit CLI stdin option or
reviewed `bridge --stdio` contract is needed to preserve prompt privacy.

The fake executable supports stdin **only as its documented fixture contract**.
Its `--fixture SCENARIO` flag cannot be passed through the Desktop IPC API. This
is a deterministic test transport, not a claim of compatibility with public exec.

## Permission matrix

| Desktop choice | Verified CLI flag |
| --- | --- |
| Read only; tool network off | `--read-only` |
| Workspace write; tool network on | `--allow-network` |
| Workspace write; tool network off | `--no-network` |
| Read only; tool network on | Rejected as a conflict |

The default CLI write policy allows tool networking; Desktop nevertheless
constructs an explicit allow/deny flag in write mode. There is no elevated,
bypass or unrestricted mode. These flags govern tools, not the authenticated
JV service connection. The CLI remains authoritative for its sandbox and writes.

## Authentication and detection

In this version, `_password()` consumes `JV_API_PASSWORD` for noninteractive
authentication; otherwise it requires a terminal password prompt. Saved account
configuration can contain username/origin, but not a reusable password. Desktop
does not open that configuration, inherit the password variable, launch login,
or implement a temporary password form. The native execution gate remains closed.

`--version` is handled by argparse before `_ensure_state()` and account/session
dispatch. The installed launcher also disables Python bytecode writes. Detection
uses that local-only path with a three-second timeout, cleared environment and
bounded output; it is tested against a fake version command. `doctor` is not used
or needed for Desktop detection. Desktop development did not run any real task.

## Cancellation limitation

The CLI's internal engine is started in a new session. Killing only Desktop's
child process group cannot prove every real engine/tool descendant is terminated.
The foundation runner's group cancellation is verified with its owned fake
process tree. A real CLI cancellation contract and remote-job behavior must be
reviewed before native execution is enabled. Local Stop must never be presented
as proof that a submitted remote job was cancelled.
