# Kineti 0.6.0 — main reconciliation and release contract

Date: 2026-10-25. Version: 1. Status: owner-approved implementation contract;
acceptance pending. This document establishes requirements, not completion.

## 1. Authority and baseline

This contract supplements `GATE-0.6.0-CONTRACT.md`. Its gate-only product boundary,
accounting safeguards and acceptance requirements remain binding. It grants no
merge, publication, deployment or accounting-recovery approval.

| Location | Recorded commit |
| --- | --- |
| Local and GitHub `codex/gate-060` | `6ebe8b57233329fd4344a32362b96159562c7ba0` |
| GitHub main | `307331ad37451b174f31b40bc0da04b544685ed0` |
| Local main | `88b518d` |

The branches have 11 candidate-only and eight main-only commits. Recheck remote
heads before integration; additional commits require review under this contract.
Preserve untracked `design/`, existing modifications, the locked project goal,
signatures, historical releases, user data and unresolved accounting. Preview
changes and record undo before editing. Backups and generated evidence stay
inside this project.

## 2. Parallel review and integration

Assign one review agent to each main-only commit, with at most three running
concurrently:

| Commit | Review responsibility |
| --- | --- |
| `39da025` | Evaluator safety and reusable gate safeguards |
| `73ebb09` | Routing changes and reusable acceptance controls |
| `071fed9` | Merge-specific resolutions and changes absent from its parents |
| `937fdfc` | Separation of operational receipts and test defaults |
| `dcacfe2` | Uncertainty after completion-recording failure |
| `1e20360` | Revocations, partial costs and unresolved accounting |
| `a71638c` | CLI validation before accounting locks |
| `307331a` | Execution controller and isolated spending checker |

Each agent inspects the actual diff and dependencies and delivers:

1. Every meaningful change classified as already covered, port, exclude or
   blocked, with reasons and source references.
2. Gate-relevant invariants, compatibility risks and required regression tests.
3. Proposed native changes and CLI, MCP, receipt or accounting-format effects.

Reviews may run in parallel. Shared-tree edits and integration run sequentially
under the primary agent. Inspect merge-parent differences and deduplicate across
commits; do not blindly cherry-pick.

Port applicable integrity, signing, accounting, recovery and validation safeguards
into the native gate. Keep Companion, WhatsApp setup, routing, execution
controllers, evaluator services and other removed features excluded. Do not
expand the public interface or change preserved exit meanings without an explicit
contract amendment. Acceptance requires a disposition for all eight commits and
every substantive change, including evidence that excluded functionality
contributes no missing gate safeguard.

## 3. Accounting and resource limits

Continue only under the approved subscription-only implementation exception:

1. Authorize $0 new paid spending. Keep the hard $0 Actions limit unchanged;
   quota exhaustion blocks work requiring those resources.
2. Preserve recorded spending, unknown costs, qualified zeros, reservations and
   recovery records. Never infer monetary amounts from token counts.
3. Verify accounting integrity before implementation and at checkpoints.
   Corruption, uncertain writes or unresolved monetary reservations block work.
4. Recovery remains separate and requires the exact proposal, evidence and
   explicit approval.
5. Publication remains blocked until monetary uncertainty is resolved or a
   separate exact publication-policy exception is approved.

Record new usage and uncertainty without replacing or erasing historical records.

## 4. Verification and review packet

Existing evidence belongs to candidate `6ebe8b5`: successful run `40129520948`,
37 native tests, eight candidate installations and two pinned Codex checks. It
does not establish acceptance of reconciled source.

After integration:

1. Add meaningful regression tests for each ported safeguard. Preserve legacy
   signature, freshness, accounting and interrupted-undo compatibility.
2. Run formatting, Clippy and retained suites under UTC and Asia/Kolkata. Publish
   the actual inventory rather than imposing the previous count.
3. Build final artifacts and test isolated Cargo/npm installations on actual
   Linux x64/arm64 and macOS x64/arm64 runners, outside the checkout. Verify runtime
   execution with Node and Bun absent from PATH.
4. Repeat isolated Codex configuration loading, discovery and a real native MCP
   call without a model turn on exactly `0.163.0-alpha.11.2` and stable `0.164.0`.
5. Measure both installation routes including initial integration files. Each
   installed footprint must be below 10,000,000 bytes. Report download and
   unpacked sizes separately.
6. Audit README, GitHub Action, MCP registry metadata, website and package
   descriptions against final behavior. Preserve the required headline and limits.

Replace the publication packet with the exact integrated commit, eight-commit
disposition matrix, artifacts/digests, runner identities, results, Codex executable
digests, footprints, accounting and remaining limitations. A source or artifact
change invalidates affected prior evidence.

## 5. Approval and completion gates

Reconciliation is complete only when all eight reviews are resolved, native
changes are committed, required acceptance passes and the packet matches the
exact candidate.

Merge requires separate explicit approval. Recheck main immediately beforehand.
New changes and conflict resolutions require review and affected verification;
do not silently incorporate them.

Publication requires separate exact-artifact approval and the accounting gate.
Target exactly `0.6.0`; an occupied Cargo/npm version or release tag blocks it
without an automatic version change. Changed or rebuilt approved artifacts need
renewed approval.

After authorized publication, repeat installation and self-test on all four native
targets using the exact published packages. Record each registry outcome and any
partial publication explicitly. Until these gates are met, describe the work as
a candidate, never merged, published or accepted against published packages.
