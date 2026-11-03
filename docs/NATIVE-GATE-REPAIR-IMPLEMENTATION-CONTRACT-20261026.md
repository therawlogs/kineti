# Kineti native gate repair implementation contract

Date: 2026-10-26. Version: 1.

Status: contract prepared from the approved CEO and engineering review decisions.
All five repairs remain unimplemented and unaccepted. The owner requested this
document after choosing to keep the review plan for later. Creating this contract
does not authorize implementation, declare a repair complete or grant release
authority. A later explicit implementation instruction is required.

## 1. Authority and scope

This document turns the five approved corrective tasks into an execution and
verification contract. It does not change v1 product scope, the locked goal,
the [0.6.0 acceptance requirements](GATE-0.6.0-CONTRACT.md), its approved
accounting amendment, or any separate publication/recovery requirement.
Existing dated release evidence remains historical evidence.

The approved decisions are:

| Task | Approved scope | Actual decision |
| --- | --- | --- |
| T1 / RUN-01 | Repair the shared execution/output deadline and incomplete capture | CEO D2=1; engineering D1=1 polling loop and D2=1 one-second cleanup |
| T2 / DOC-01 | Refresh contributor guide and PR template for the native gate | CEO D3=1 |
| T3 / SEC-DOC-01 | Refresh the security guide without weakening commitments | CEO D4=1 |
| T4 / CFG-01 | Add an explicit Cargo argv override, retaining the old string and unrelated config | CEO D5, explicit recommended-text answer |
| T5 / PERM-01 | Owner-only creation of new append/lock files; inventory historical modes and prepare a separate recovery proposal | Engineering D3=1 |

No architecture or remedy decision is unanswered in the source review.
Implementation and acceptance evidence are still outstanding. A new policy,
changed outcome, dependency, public interface or scope expansion requires an
explicit decision; do not infer it from this contract.

## 2. Source record

The full review and QA plan are local, uncommitted runtime artifacts. Preserve
them; do not commit `.kineti/` to make these references available.

| Record | Project-relative location | SHA-256 at contract creation |
| --- | --- | --- |
| Approved review | `.kineti/reviews/ceo-repo-20261026/plan.md` | `3c503df0dbe5960cabec8747e5a98ceba15e9d97be99d49f4ff6c7cf088dcb26` |
| Detailed QA plan, CP1–CP6 and EC1–EC6 | `.kineti/reviews/eng-gstack-state/projects/therawlogs-kineti/praveen-main-eng-review-test-plan-20261026.md` | `a0c55c07ef6752385e2c09f074b8c98bc5d6cc1736d95c0c94fc890bcf0a66c7` |
| Existing acceptance contract | `docs/GATE-0.6.0-CONTRACT.md` | `bcbfc69b9b677bc5ea82545ac30a343366b9d07949c41f479e6a42f6a02b5c17` |

The requirements below are sufficient to execute the bounded repair without
depending on private review artifacts. The source hashes identify the reviewed
baseline; they are not signatures, independent approval or proof of honest tests.

## 3. Preconditions and preservation

1. Before implementation, read current `AGENTS.md`, `.kineti/state.json`, this
   contract and the existing acceptance contract. Inspect current source and
   user changes rather than blindly applying an old patch. Preserve the locked
   goal, historical signatures/bytes, interrupted work and unresolved costs.
2. Preview each change and register its exact undo before editing. Keep backups,
   fixtures, generated evidence and isolated homes/configuration inside the
   project. Do not roll back unrelated user work.
3. Use the native `kineti` executable, built from `core-native/Cargo.toml` when
   working from source. Record its digest. Use `kineti spend check` and
   `kineti status` to report current state; never clear a denial to proceed.
4. Retain the approved subscription-only exception and **$0 new paid spending**.
   Unknown historical costs remain unknown. The recorded $55 is not a value to
   force into the ledger. Corruption, uncertain monetary writes, unresolved
   monetary reservations or possible extra charges require stopping affected
   work. An implementation-only exception never authorizes publication.
5. Inspect outbound destinations before transmitting project information and
   preserve request audit records. No credentials, paid fallback, remote runner
   dispatch or provider-setting changes are authorized by this document.

## 4. Implementation requirements

### T1. Shared runner supervision

Primary files: `core-native/crates/kineti-cli/src/receipt.rs` and
`core-native/crates/kineti-cli/tests/native_gate.rs`.

R01. Keep `receipt::execute` as the shared helper for `receipt::run`,
`gate::verify` and `undo::rollback`. Preserve its existing tuple/Result interface,
exact shell-free argv, exit meanings, output assembly and receipt format. Use one
nonblocking polling loop with existing std/libc; no async runtime, reader threads,
new dependency, service or public testing flag.

R02. Require definite child status and EOF from **both** output streams before
the execution/capture deadline for a complete result. Leader exit alone or EOF
alone is insufficient. Use monotonic time; activity, EINTR and EAGAIN must not
reset the deadline. Distinguish EOF, pending reads and nonretryable errors.

R03. Drain both streams fairly with bounded read work per iteration and finite
waits bounded by remaining time. Retain the existing 8,000,000-byte cap per
stream, continue draining beyond retention, and preserve stdout/newline/stderr
assembly and lossy UTF-8 decoding. Avoid busy loops and unbounded extra buffers.
The raw retention cap is not a total memory limit.

R04. Guard ownership on every post-spawn path, including setup, read, poll and
status errors. Close owned pipes and terminate only the owned execution group
when required. Confirm direct-child reaping rather than assuming dropping the
handle does it. Never signal a reused numeric identity after ownership is lost.
Verify feasible POSIX lifecycle control on Linux and macOS; unsupported control
must deny rather than silently weaken the behavior.

R05. Allow at most **one additional monotonic second** after the execution/capture
deadline for nonblocking cleanup confirmation. No blocking join, read or wait
may undermine that bound. Deadline expiry is terminal for success; late EOF or
status cannot turn failure into a pass. Unconfirmed cleanup returns explicit
`Error(3)` with identity/diagnostic information. The bound applies to supervision,
not pre/post inventory hashing, key setup, persistence or OS suspension; it is
not a hard real-time or escaped-descendant containment guarantee.

R06. Preserve this caller outcome contract:

| Outcome | Helper result | Receipt / verify | Undo |
| --- | --- | --- | --- |
| Executable definitely did not start | `Some(127)`, diagnostic, not truncated | Fail | Record definite failure and continue |
| Definite exit and both EOF before deadline | Actual exit and existing truncation/output semantics | Existing conditions decide pass/fail | Record completion; nonzero failures continue |
| Retention exceeded, but fully drained with definite exit | Actual exit, `truncated=true` | Deny passing evidence/verify | Preserve definite completion; retention alone is not uncertainty |
| Deadline or nonretryable capture failure, cleanup confirmed | `None`, bounded retained output/diagnostic | Cannot pass | Preserve intent; inspection required before replay |
| Cleanup unconfirmed within bound | `Error(3)` and diagnostic/identity | Nonzero failure | Preserve unresolved intent; block replay |

Incomplete capture must never become `Some(0)` with only a truncation flag:
undo intentionally treats definite exit separately from retained output.

### T2. Contributor workflow

Files: `CONTRIBUTING.md` and `.github/pull_request_template.md`.

R07. Replace active Bun/dashboard/eight-crate instructions with the actual
single-crate native workflow. Verify paths, commands, flags and expected denial
messages through a disposable native walkthrough. Preserve DCO, review, spending,
undo and publication-approval rules. Do not rewrite dated historical documents.

### T3. Security guide

File: `SECURITY.md`.

R08. Ground current versions, modules, runtime surfaces and limits in source and
fixtures. Preserve legacy support, disclosure/reporting and response commitments.
Document T5's new-file behavior separately from existing-file limitations. Do not
silently weaken a promise, claim historical permissions were repaired or infer
private-key exposure from broad log permissions. Any additional runtime mismatch
or proposed guarantee change returns for its own decision.

### T4. Repository verification declaration

File: `kineti.config.json`.

R09. Add only this explicit override under `settings`, preserving the original
`verify_command` string and every unrelated value:

```json
"verify_command_argv": ["cargo", "test", "--locked", "--manifest-path", "core-native/Cargo.toml"]
```

R10. In a disposable copy of the original configuration, prove exact argv
resolution, field preservation and untrusted-command denial before command
effects. Old argv trust must not authorize the new declaration. Preserve legacy
precedence and malformed/missing verify exit 9. The change invalidates old
evidence; do not modify old receipts or auto-trust the new argv. Exact trust review
is a later separate action, not a side effect of configuration repair.

### T5. File creation permissions

Files: `core-native/crates/kineti-cli/src/store.rs` and
`core-native/crates/kineti-cli/tests/native_gate.rs`.

R11. Explicitly request `0600` for **new** files in `store::append` and
`Lock::take`, reusing the existing Unix creation-mode pattern. Preserve atomic
state/private-key behavior, symlink denial, durable append, lock contention and
interrupted-lock inspection. Do not add a generalized migration abstraction.

R12. Inventory existing affected file modes and prepare an exact metadata-only
recovery proposal for later approval. Do not apply it, silently chmod existing
files, rewrite historical contents/signatures or call existing files repaired.
Preserve pre-existing modes and bytes except for the normal explicitly requested
operation's append. No cross-account or parent-directory security claim follows
from observing mode bits alone.

## 5. Verification and acceptance checklist

These checks specify evidence for the five repairs; they do not amend or replace
the existing release acceptance requirements. Every row starts **not run for the
repair**. Existing green results cannot stand in for tests of changed source.

| ID | Requirements | Required evidence and observable assertions |
| --- | --- | --- |
| V01 / CP1 | R01, R06 | All three callers preserve zero/nonzero/signal/spawn-failure meanings; definite undo failures continue. Extend existing tests rather than duplicate them. |
| V02 / CP2, EC1 | R02, R04–R06 | Controlled readiness reproduces leader-exits-first inherited pipes; sleeping leader, one stream open, EOF-before-exit and exit-before-EOF all fail or complete correctly within supervision bounds. A one-second deadline must not pass after a helper later closes pipes. |
| V03 / EC2 | R02–R03, R06 | Simultaneous noisy streams, empty/invalid UTF-8 output and retention at/beyond each cap do not starve deadlines; completed truncation denies evidence/verify while retaining definite undo completion. |
| V04 / EC3 | R02, R04 | EINTR/EAGAIN remain pending within remaining time; nonretryable read/setup/poll/status faults cannot masquerade as EOF or abandon child ownership. Use private local tests/real boundaries, no public fault flags. Report an uninducible fault as missing proof. |
| V05 / CP3, EC4 | R04–R06 | Confirmed/unconfirmed cleanup respects the extra one-second bound; expired work never recovers to success; identity-safe signaling/reaping is verified. Incomplete capture retains undo intent; repeated rollback cannot repeat effects. |
| V06 / CP4 | R09–R10 | Copied repository configuration resolves the exact approved argv, retains all unrelated values/old string, stays untrusted and denies with exit 9 before effects. Old trust does not carry over. |
| V07 / CP5, EC5 | R11–R12 | Isolated subprocess with umask 000 creates append/held-lock files at 0600; atomic/private-key defaults remain 0600. Inspect lock while held. Historical fixture modes/bytes are preserved, and inventory/proposal does not apply recovery. Do not change global test-process umask. |
| V08 / CP6 | R07–R08 | Disposable native contributor walkthrough and source/fixture security claim map; paths/flags/outcomes correct, commitments intact, old files and remaining limitations accurately described. |
| V09 / EC6 | R01, R06, R10–R11 | Retain exact/spaced argv, malformed-option validation, freshness/input/runtime/environment, signature/issuer, skipped/missing summary, corrupt/unresolved spending and undo completion-persistence regressions, including no repeated side effect. |
| V10 | All | Formatting, Clippy and native suite freshly run under UTC and Asia/Kolkata; saved native run/check evidence with exact argv and hashes; final diff restricted to approved work and preserved user changes. |

Use Cargo's existing test framework and `native_gate.rs`, with private helper
tests where appropriate. No tests or genuine legacy fixtures are retired; retain
the fabricated-output demonstration. Use behavioral assertions, controlled
process readiness and independent watchdog/cleanup for process fixtures, not
sleep-only timing or existence tests. Separate helper timing from CLI overhead.

Required native suite command:

```sh
cargo test --locked --manifest-path core-native/Cargo.toml
```

Run formatting and Clippy using the existing repository workflow. Offline runs
may add `--offline` and use the project-local Cargo cache; record that exact
command without calling it the configured argv. Record and check fresh results
through `kineti test run` and `kineti test check`, binding the exact test command,
declared inputs and environment. Use disposable accounting for fixtures; real
historical accounting denials must remain visible and unchanged.

For each V-row, save command/argv, exit result, assertions, source and executable
digests, artifacts/output digests, environment, timezone, OS/architecture and
timing where relevant. Record failures, skips and missing proof explicitly.
Provide a final requirement-to-evidence matrix and remaining-blockers list.

Actual Linux x64/arm64 and macOS x64/arm64 lifecycle proof remains required before
new release/platform claims. Missing runner access or uncertain billing is an
external blocker, never substituted with cross-compilation or local macOS tests.
Native feasibility does not establish installed Cargo/npm acceptance. Independent
outside review was unavailable in the review session; do not claim a second
opinion or silently make that absence a new release requirement.

## 6. Execution order and undo

| Lane / step | Work | Dependency |
| --- | --- | --- |
| A1 | T1 runner and native regressions | Explicit later implementation instruction; preconditions checked |
| A2 | T5 new-file permissions and mode/history tests | A1, because both edit native integration tests |
| B | T2 contributor guide and PR template | Existing native command contract; can proceed independently of A |
| C | T4 configuration and disposable checks | Can proceed independently; final combined evidence follows A |
| D | T3 security guide and combined verification | T5 observations and A/B/C results |

Three isolated worktree lanes are possible, not required to start now. Sequence
shared native test edits, integrate reviewed changes, then perform D and fresh
combined evidence. Independent documentation/configuration work does not wait
for unrelated runtime tests unless a real dependency arises.

Before every edit, preview and register a project-local undo preserving the
current user version. Undo follows existing intent/completion rules: definite
failures continue, uncertain effects stop for inspection. Do not blindly replay
or use rollback to clear unknown costs, revert user changes or weaken a gate.

## 7. Completion and separate approvals

Local repair completion requires all five tasks, V01–V10 evidence, documented
external gaps and a reviewed final diff. Unrun, skipped, failed or unsupported
checks cannot count as accepted. If required proof is absent, report the affected
task incomplete; do not reinterpret the requirement to obtain a pass. Historical
receipt counts are a baseline, not completion proof or a required future count.

Keep the following actions separate from local repair completion:

1. Approval to begin implementation, currently withheld by the owner's keep-for-later choice.
2. Exact existing-file permission recovery or any accounting recovery.
3. Exact argv trust review/grant after the configuration and source changes.
4. Credentials/provider settings, paid exceptions, remote CI and actual package acceptance.
5. Artifact approval, version/publication choice, push, merge and deployment.

No version bump, release tag, registry upload, PR, merge or deployment is part
of this repair contract. Publication remains subject to accounting review and
the existing separate release contracts.

## 8. Retained product and trust boundary

Kineti records checks, detects changed inputs and gates participating commands
on evidence and reported spending. A receipt signature is distinct from test
trust and independent control of the issuer, policy and evidence. Executed tests
can fabricate passing summaries. Same-account adversary protection, universal
interception, protected writes, independent evaluator/publisher authority and
provider-bill verification remain outside v1 claims.

The [trust-boundary research note](research/TRUST-BOUNDARY.md) remains research.
No checker service, dashboard, harness, model routing, caching/indexing project,
new benchmark system or automatic historical-data migration is introduced.
Full-file inventory/history costs remain unchanged. The existing acceptance
requirements, approval boundaries and historical records remain authoritative.
