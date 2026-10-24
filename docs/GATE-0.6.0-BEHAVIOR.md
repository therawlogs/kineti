# Native gate behavior and compatibility map

Contract: [0.6.0 acceptance contract](GATE-0.6.0-CONTRACT.md). This inventory is
recorded before deletion. A mapped test is not a passing test until its saved
result exists. Historical 631-test evidence stays historical, not a new target.

| Retained behavior | Former source / fixtures | Native test responsibility |
| --- | --- | --- |
| Legacy signature bytes and issuer trust | bin/kineti-evidence.ts unsignedRecord/verifyReceiptSignature; new public TS-signed golden fixtures | compat::tests: original byte parity, signatures, altered fields, issuer substitution, numeric/order/Unicode/optional cases |
| Cross-language native canonical bytes | src/security/canonical-json.ts | compat::tests: canonical golden parity and unsafe-value rejection |
| Command result recording and exact argv | evidence run; evidence-guards, exec-safe tests | receipt integration: argv/empty arguments, exit/output, declared inventory, mutation during run |
| Freshness and trust, operational receipts | validateReceipt/validateOperationalReceipt; evidence-fingerprint, evidence-guards | receipt integration: changed inputs/dependencies/environment/runtime, expired/future timestamps, untrusted/altered/missing receipt, legacy authority unchanged |
| Participating CI/verify gating | kineti-ci, kineti-verify-gate; ci, ci-seed | gate integration: required checks, missing/damaged state, stage/security/receipt policy, exit semantics |
| Spending and confirmation | kineti-spend; completion-regressions, checkpoint-a2 | spend integration: integer totals/stages, legacy ledger and journal hash/checkpoint formats, lock/concurrency, corrupted history, reservations, thresholds, reset without headroom, audited confirmation, interrupted recovery and recovery-view integrity |
| Registered undo and recovery | kineti-saga; lib, completion-regressions | undo integration: argv, newest-first, intent/completion, definite failure continuation, uncertain interruption and reviewed replay |
| Usable initialization and host ownership | setup.sh, kineti-mcp init; clean-machine-setup, test-setup | init integration: explicit goal, missing fields, no damaged-record repair, diff/backup/preservation/repetition/uninstall/restart |
| MCP | kineti-mcp gate tools; mcp tests | MCP integration: initialized protocol, typed validation, actual tools, exact argv; pinned alpha and stable app-server acceptance |
| Self-test (new) | no former implementation | disposable-fixture self-test: stale/fresh release, invalid signature, fabricated passing-output limit |
| Installation/release/footprint | release workflows, installer/package tests | four native environments, Cargo/npm artifact installs, runtime without Node/Bun, checksums, ignored scripts/unsupported targets, per-route <10,000,000 bytes |

## Command disposition

Retain: init, mcp, test, evidence, undo, saga, spend, status, verify, ci.
Add: self-test. Version/help remain metadata commands.

Remove explicitly: companion, state (use status/init), swarm, seed (use init),
egress, memory, evidence-verify (use test check), protected-write, workflow,
incident, context, linux-service, service-client, native-memory, conformance;
native assistant chat, test-all, epistemic, epistemic-eval, anti-drift.
No-command native assistant chat is removed; show gate help instead.

Before deletion, retain the complete discovered subcommand and MCP schemas in
the generated command inventory. Rejection tests must consume that inventory.

MCP gate names retained where applicable: kineti_status, kineti_evidence_record,
kineti_evidence_check, kineti_spend_record, kineti_spend_status, kineti_saga_push,
kineti_saga_rollback, kineti_verify_gate_status. Goal initialization moves to
explicit init; stage/gate mutation is not an MCP authority claim.
Remove kineti_lock_goal, kineti_set_stage, kineti_set_gate, kineti_linux_service,
kineti_protected_write, kineti_context_memory, kineti_workflow,
kineti_egress_record, kineti_talk, kineti_epistemic_eval.

## Exit-code contract

| Context | Code | Meaning |
| --- | --- | --- |
| All | 0 | Successful check/operation |
| CI/verify, recorded command, incomplete undo | 1 | Policy/check/command failure; incomplete undo |
| All | 2 | Invalid arguments/options or required confirmation absent |
| Spend and integrity/recovery guards | 3 | Spending stop or damaged/uncertain accounting/undo state |
| Evidence check | 4 | Stale/expired/changed inputs |
| Evidence check | 5 | Missing evidence |
| Evidence check | 6 | Legacy/unsupported evidence cannot authorize exact-input gate |
| Evidence check | 7 | Invalid/untrusted evidence |
| Verify | 9 | Missing verify declaration or absent/stale command trust |

Removed commands get an explicit removal message, with invalid-input code 2.
CI collapses evidence failures to policy-failure code 1; structured output
retains the underlying evidence reason. No protected-write exit semantics remain.
