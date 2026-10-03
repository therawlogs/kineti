# What Undo Cannot Reverse

Kineti's undo system (SAGA-based rollback) only reverses **explicitly registered inverse commands**. It is not a filesystem snapshot or time machine. This document lists what undo cannot reverse.

---

## 1. Unregistered Changes

**Undo only reverses steps that were explicitly registered** via `kineti undo push` or by the agent calling `register` before making changes.

| Scenario | Reversible? |
|----------|-------------|
| Agent edits file after calling `kineti undo push "label" "git checkout -- file"` | ✅ Yes |
| Agent edits file without registering inverse first | ❌ No |
| Manual edit by human in terminal | ❌ No |
| External tool modifies files (CI, formatters, linters) | ❌ No |

**Rule**: If no inverse was registered, undo cannot reverse it.

---

## 2. Failed Inverse Commands

When `kineti undo rollback` runs, it executes registered inverses newest-first. If an inverse command fails (non-zero exit code):

- The step remains **pending** (not marked as undone)
- Rollback exits with code 1 and prints `rollback incomplete`
- Failed steps stay in the ledger for manual review or retry

**Undo does not skip failed steps** — it stops and requires human intervention.

---

## 3. Committed Runs

A run marked with `kineti undo commit` (or `commit` via MCP) is **permanently sealed**. Undo will refuse to roll back a committed run:

```
kineti: run abc123 already committed
```

---

## 4. Tampered Ledger

The SAGA ledger (`.kineti/saga.jsonl`) has cryptographic integrity checks:

| Check | Failure Result |
|-------|----------------|
| Inverse command hash mismatch | `saga ledger TAMPER: inverse hash mismatch; refusing rollback` |
| Ledger file modified externally (mtime/size/owner) | `saga ledger changed outside push/register; refusing rollback` |
| Ledger owner/group changed | `saga ledger owner changed; refusing rollback` |

**Undo refuses to operate** if integrity checks fail. Inspect `.kineti/saga.jsonl` manually.

---

## 5. Non-Command Side Effects

Undo runs shell commands. It cannot reverse:

- **Network calls** already sent (API requests, webhooks, deployments)
- **Database mutations** unless the inverse explicitly runs a compensating transaction
- **External state** (cloud resources, secrets rotated, emails sent)
- **Process state** (running servers, background jobs)

**Only what the inverse command explicitly does** gets reversed.

---

## 6. Concurrent or Parallel Edits

If multiple agents or processes edit files simultaneously:

- Only inverses registered **in the same run** are rolled back together
- Edits from other runs, other terminals, or unregistered agents are not touched
- Race conditions between registration and execution are not prevented

---

## 7. Git History

Undo does **not** rewrite git history. It runs inverse commands (e.g., `git checkout -- file`). The git log still shows the original commits. Use `git reset` or `git revert` separately if needed.

---

## Summary Table

| Category | Reversible by Undo? |
|----------|---------------------|
| Registered inverse commands (success) | ✅ |
| Registered inverse commands (failed) | ❌ (stays pending) |
| Unregistered file edits | ❌ |
| Committed runs | ❌ |
| Tampered ledger | ❌ (refuses) |
| Network/API side effects | ❌ |
| External tool modifications | ❌ |
| Git commits | ❌ (use git directly) |

---

## Best Practices

1. **Register before mutate**: Always call `kineti undo push "label" "inverse-command"` before the agent edits files.
2. **Test inverses**: Ensure inverse commands actually work (e.g., `git checkout -- file` restores the exact previous state).
3. **Commit when done**: Call `kineti undo commit` when a task is complete to seal the run.
4. **Verify after rollback**: Run `kineti test check` to confirm code state is correct.
5. **Inspect ledger**: Check `.kineti/saga.jsonl` if rollback behaves unexpectedly.

---

## See Also

- `kineti undo --help` — Command reference
- [PLAN.md](./PLAN.md) — SAGA transactional reversibility architecture
- [HOWTO-daily-loop.md](./HOWTO-daily-loop.md) — Daily workflow with undo