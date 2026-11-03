# Offline host boundaries and independent agent review

Date: 2026-11-01. Status: this bounded offline pass completed; full contract acceptance remains incomplete.

## Result and source changes

One owner-approved read-only reviewer agent examined source and saved acceptance evidence. It did not modify files, execute project code, access credentials, or initiate authentication or acceptance inference. The review service rejected its initial broad pass before findings; the completed pass was limited to defensive correctness and evidence consistency. This is independent agent review within this chat, not human or third-party assurance or independent re-execution.

The reviewer found two source defects in the preceding candidate:

1. Owner-side restoration could continue after revocation acknowledged the task-group stop, then overwrite the stopped phase. The service now holds a recovery mutex through restoration and its completion record. Stop signals cancellation, drains the task group, waits for that mutex, and reasserts the final stop phase. Restoration checks authority before mutations and between bounded copy chunks; partial work and quarantine are preserved.
2. A failure path discarded unsuccessful group cleanup before considering authentication resume or automatic restoration. It now halts authority, records terminal `cleanup-uncertain`, and returns before either branch. Unconfirmed cleanup does not become resumable authority.

Guarded native undo and exact backups were registered before each source/test/document edit. The source changes are in `task_owner.rs` and `task_snapshot.rs`; two integration regressions cover cancellation after a quarantine move and during a 64 KiB copy. No old signed records, interrupted work or unresolved costs were reset. Previous candidates remain unchanged.

The new optimized Linux arm64 candidate is root-protected at a new fixture path. SHA-256: `0b53ebfdaae68330df59ccbbb99175ceb85487c8719c5c89b1d7a9fc0fc9ea41`. Official Codex remains pinned to 0.166.0, SHA-256 `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`; helper SHA-256 remains `587436a943a59c18cabdbe83f07800d237b60ddd2e0f54d05b23d8b37a0b36f4`. No custom sandbox or process-group fallback was introduced.

## Fresh offline observations

| Case | Observed result |
| --- | --- |
| Stock host boundary suite | Fourteen attempted effects denied, followed by one successful authorized write: absolute outside write, original-workspace write, original excluded-secret read, outside readable canary, symlink write/read, real parent traversal, protected policy write, excluded-copy read/write, outside hard link, shell-child outside write, TCP creation/connect and abstract Unix socket creation/connect. |
| Local network controls | Owner-side TCP and abstract Unix listeners each accepted one positive-control connection before task execution. Task socket creation returned EPERM; no extra listener connection occurred. No external destination or private source was transmitted. |
| Recovery-stop synchronization | The owner observed active restoration of a 600-file copied task, revoked it, captured inventory immediately on native revoke return, then compared it after worker completion and an observation interval. Hashes matched. All task outputs survived in work/quarantine; original files remained unchanged and final state was revoked. |
| Cleanup failure | After a definite check result with pending execution false, the fixture paused its own controller and reversibly added root-group write mode to only its synthetic cgroup. Native protection validation then refused cleanup. The service recorded terminal uncertainty and started neither automatic restore nor authentication resume. The fixture restored the original mode and explicitly reconciled through the existing native owner primitive before closing. |
| Secret and preservation checks | Synthetic excluded-secret bytes were absent from protected authority records. Prior owner-store files were hash-checked, all fixture services/groups were closed, no task UID processes or persistent task-account auth file remained, and the disposable VM was stopped with disk retained. |
| Regression | macOS arm64: 26 library and 61 integration tests passed separately under UTC and Asia/Kolkata. Linux arm64: 27 library and 61 integration tests passed separately under both zones. Strict Clippy passed on both; formatting and whitespace checks passed. |

The stock app-server, native owner service, kernel filesystem/network controls and cgroup controls performed these checks without authentication or inference. Synthetic root-readable/world-writable canaries made the filesystem tests meaningful beyond ordinary DAC denial where appropriate; protected authority also retains root ownership/DAC. Exact commands, fixed check program, request, limits, output hashes, environment and final records are saved. The probe suite uses Python commands through the stock terminal sandbox; it is not a model-native direct-edit negative test. No universal tool interception is claimed.

The reviewer also found two evidence weaknesses: the first traversal probe targeted a nonexistent sibling, and the first recovery comparison began after waiting for the worker. Both fixtures were corrected and rerun on the same candidate. Only `boundaries-reviewed.jsonl` and `recovery-stop-reviewed.jsonl` support the corrected claims. Earlier samples, listener-thread diagnostics, missing-trait compilation failure, helper-file assertion failure and the kernel-rejected cgroup rename injection remain preserved and are not counted as passes. The accepted cleanup-failure case is `cleanup-fault-protected-mode.jsonl`.

Final read-only review found no outstanding blocking finding within the reviewed restoration/drain scope and confirmed the two corrected evidence claims. Initial snapshot preparation remains outside the recovery mutex; setup-time cancellation of all root-side mutations is not accepted. No live model acceptance was rerun against this changed native candidate. The earlier ten aggregate real turn attempts and closed login sessions remain historical and unchanged.

## Accounting, evidence and remaining gates

Evidence is retained in `.kineti/autonomy-escape-review-20261101/`: review authorization and dispositions, exact guest programs, successful and failed diagnostics, both-platform source-check logs, `guest-reviewed-audit.json`, source/artifact hashes, saved-evidence verifier and native operational receipt. Device codes, passwords and credential values were not needed. Operational signatures and local test summaries do not establish independent test trust.

The repository spending assessments before and after this work retain verified integrity, clear monetary reservations, thirteen historical valuation unknowns and identical reported amounts. Original ledger/signature bytes and earlier evidence/undo prefixes remain preserved. New paid spending, fresh logins and acceptance model turns were all zero. Only the separately approved read-only review agent was used; no external model provider or paid service was invoked. No merge, push, registry change, package/website publication or deployment occurred.

`AUTONOMOUS-BUILD-ACCEPTANCE-MATRIX-20261101.md` maps the remaining A01–A14 gaps. All rows remain partial/pending owner acceptance. Next offline priorities include setup-time revocation and additional tool-path coverage, followed by separately approved live replay of a reviewed candidate. Four-target installed acceptance, broader independent assurance and release/accounting gates remain separate. This pass does not authorize fresh logins, extra acceptance model turns or release actions.
