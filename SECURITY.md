# Security Policy

## Reporting
Report vulnerabilities privately via GitHub
[Security Advisories](https://github.com/therawlogs/kineti/security/advisories/new).
Please do not open public issues for exploitable findings. Expect a response
within 7 days.

## Scope
Kineti v0.2 is a ship proof + spend fuse: gateway meter (reserve/settle) and
offline receipt (`evidence → ship-check → verify`). In scope:

- Path-fence escapes (tools escaping the project root, symlink tricks)
- Prompt-injection paths that lead to tool execution from untrusted content
- Hash-chain verification bypasses (tampered `evidence.json` / journals verifying as OK)
- Spend breaker bypasses (reservation races, ledger-lock contention, `.kineti/spend.reset` bypass)
- Daemon IPC attacks: `.kineti/kineti.sock` access control (must stay 0600)
- OAuth token theft: `~/.kineti/auth/*` must remain mode 0600 and never leak
  into logs or child-process environments
- Egress hash-chain tampering (receipt chain broken but `verify --all` passes)
- Worktree teardown path traversal: `destroy()` refusing anything outside
  `.kineti/worktrees/<id>` (legacy swarm, still in tree)

## Honest limitations (v0.2)

- `bash` (via `evidence --cmd` or wrapped agents) runs with your user's
  permissions inside the project directory; there is no syscall sandbox yet
  (planned: platform sandbox profiles). Blast radius is bounded by scope
  partitioning + `ship-check` refusing stale proofs, not by the kernel.
- Gateway is stateless workers + one ledger per org; receipts store
  `fingerprint`, `chain_head`, `cost_usd`, `cmd`, `passed` — never raw prompts
  (hashes + counts only). A compromised gateway worker sees prompts in memory
  while forwarding.
- `verify` and `ship-check` are offline and trust local files; a fully
  compromised host can rewrite `.kineti/evidence.json` and `.kineti/journal.jsonl`
  before verification.
- OAuth tokens on disk are only as safe as the user account that owns them;
  a fully compromised host reads them like any other secret.
- The daemon socket is same-user-only (0600); it is not protection against
  malware running as your user.
- Network access of spawned processes is not restricted.

Treat Kineti as blast-radius reduction and audit infrastructure — not as a
hard security boundary against a fully compromised host.

## Honest limitations (v0.1 legacy — swarm pipeline, frozen)

These apply only to the frozen 13-stage runner (`kineti run --legacy`,
`docs/v0.1.md`, tag `v0.1.0`). The pipeline is hidden (`kineti run --help`
shows `--legacy`) and not the default in v0.2, but the code remains in-tree
(`src/stages.rs`, `src/swarm.rs`, `src/worktree.rs`):

- Swarm workers' bash is likewise unfenced INSIDE their worktrees — isolation
  is via worktree directories + git-based integration, not the kernel.
- Quarantine is heuristic pattern-matching, not true dual-model isolation.
- Same worktree teardown, network, and token caveats as above.
