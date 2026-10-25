# Local integrity and recovery

The locked project goal is never recreated from damaged state. Stop and inspect damage before making a replacement. These records detect accidental edits, broken chains and truncation relative to saved local checkpoints; they do not prevent the machine owner from rewriting every local file.

## Egress replacement

Inspect the original ledger without editing it. Owner recovery makes a new project-relative directory, copies the original ledger and available state byte for byte, and records their SHA-256 identities. Its first receipt states that historical damage remains unresolved. The replacement does not certify the old history.

```sh
bun bin/kineti-egress.ts recover --i-am-human \
  --source "$HOME/.kineti/egress.jsonl" \
  --destination .kineti/recovered-egress \
  --reason "Owner inspected the damaged history; cause remains unresolved" --activate
bun bin/kineti-egress.ts verify
```

`--activate` saves a project-local egress pointer. It changes only egress logging, not machine keys, audit history or other configuration. `KINETI_MACHINE_DIR` remains an explicit override for isolated tests. Missing replacement metadata, malformed initialized state, invalid chains and incomplete lines block work. Recovery requires a new destination and never overwrites an earlier replacement.

## Journals and audits

New writes verify history while holding the same lock used for appending. A byte checkpoint and initialized marker detect later removal, edits and truncation. A process interrupted between a data write and checkpoint commit leaves an error requiring owner inspection. Do not delete lock or checkpoint files merely to make a check pass.

Legacy histories retain their original hash rules. Their first valid write starts checkpoint coverage from the observed bytes. No tool can prove whether a legacy tail disappeared before a checkpoint existed; this limitation is not repaired by signing the observed history. Old signed records are not rewritten or granted trusted-evaluator authority.

Encrypted snapshot import validates each source record and the tail chain, then appends local records containing exact source records and snapshot identity as provenance. Imported records are historical material, not local approvals or live contextual memory. Importing a snapshot again does not append duplicate source records.

## Undo and spending

Inspect pending inverses before rollback. New structured inverses use `--inverse-argv` with a JSON array and preserve spaces, empty strings and Unicode. Legacy command strings remain readable; unmatched quotes and trailing escapes are rejected before any pending inverse executes. Do not run a blanket rollback to repair stored commands.

Spending reconciliation must preserve the original accounting bytes, the owner's exact new cap and the unresolved cause of any mismatch. A breaker reset does not create headroom below the 95% stop. Recorded totals are not provider invoices. Static model aliases are estimates; use actual billing with `--usd` or explicit provider rates when available.

## Safe credentials

Put secrets in protected files or environment inputs supported by the target program, not command arguments, test names, journal text or request descriptions. Do not declare secret variables with evidence `--env`; that flag is for nonsecret reproducibility inputs. Execution egress records bind exact argument arrays and working directories using hashes without storing argument values. Local evidence manifests still contain their declared command arguments, so credential-bearing argv is unsuitable. Do not echo credentials in test output.

Use a declared inventory for test evidence:

```sh
bun bin/kineti-evidence.ts run --label unit --required-test tests -- bun test tests/
bun bin/kineti-evidence.ts check --label unit
```

A completed command with no declared inventory records diagnostics and exits unsupported. Typechecking uses `--kind check`; packaging uses `--kind package`. Neither authorizes the test gate.
