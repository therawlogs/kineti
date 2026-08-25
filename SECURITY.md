# Security Policy

## Reporting

Report vulnerabilities privately via GitHub
[Security Advisories](https://github.com/therawlogs/kineti/security/advisories/new).
Please do not open public issues for exploitable findings. Expect a response
within 7 days.

## Scope

Kineti is an agent harness that executes model-chosen commands on your machine.
The following are in scope:

- Path-fence escapes (tools escaping the project root, symlink tricks)
- Prompt-injection paths that lead to tool execution from untrusted content
- Hash-chain verification bypasses (tampered journals verifying as OK)
- Spend breaker bypasses (reservation races, ledger-lock contention bugs)
- Daemon IPC attacks: `.kineti/kineti.sock` access control (must stay 0600)
- OAuth token theft: `~/.kineti/auth/*` must remain mode 0600 and never leak
  into logs or child-process environments
- Worktree teardown path traversal: destroy() refusing anything outside
  `.kineti/worktrees/<id>`

## Honest limitations (v0.1)

- `bash` runs with your user's permissions inside the project directory; there
  is no syscall sandbox yet (planned: platform sandbox profiles). Swarm
  workers' bash is likewise unfenced INSIDE their worktrees — blast radius is
  bounded by scope partitioning plus git-based integration, not by the kernel.
- Quarantine is heuristic pattern-matching, not true dual-model isolation.
- Network access of spawned processes is not restricted.
- OAuth tokens on disk are only as safe as the user account that owns them;
  a fully compromised host reads them like any other secret.
- The daemon socket is same-user-only (0600); it is not protection against
  malware running as your user.

Treat Kineti as blast-radius reduction and audit infrastructure — not as a
hard security boundary against a fully compromised host.
