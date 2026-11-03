# Kineti autonomous building contract

Date: 2026-10-30. Version: 1.

Status: proposed implementation and acceptance contract. Creating this document
does not authorize implementation, change existing policy, grant command trust,
resolve accounting, or approve publication, merging, deployment or paid usage.
All acceptance rows below are pending. No release version is selected.

## 1. Outcome

After one task approval, an agent can inspect, edit, build, test, encounter
failures, repair them, and deliver a verified diff without further permission
prompts while operating inside the approved boundary. Kineti prepares recovery
and maintains evidence automatically. It asks only when authority must expand,
an external commitment requires approval, or safe continuation is uncertain.

The locked project goal remains unchanged. This proposal extends the native
gate rather than restoring the removed model router, swarm, memory product,
dashboard or general workflow engine. It does not promise safe AI behaviour.

## 2. Existing foundations and gaps

Source reviewed when drafting: `core-native/crates/kineti-cli/src/`.

1. `gate.rs::verify` persists exact-argv trust. Unchanged trusted commands do
   not require repeated trust confirmation. This is not a task permission grant.
2. `receipt.rs::run` records commands, input inventories, environment/runtime
   fingerprints and observed results. An agent currently coordinates the loop.
3. `undo.rs` provides registered inverse commands, durable intent/completion,
   newest-first rollback and uncertain-completion guards. It does not snapshot
   arbitrary edits automatically.
4. `spend.rs::load` rejects historical unknown costs; `verify` and `ci` depend
   on spending inspection. There is no action-specific continuation policy.
5. `mcp.rs` exposes eight gate tools, without task grants or host interception.
   Its evidence interface has fewer controls than the CLI.
6. `receipt.rs::execute` starts ordinary child processes. A project working
   directory and process group are not filesystem/network/credential isolation.
7. Project instructions and the host's permission system can independently
   require approval. Old pipeline/memory/host fields in configuration are not
   evidence of active native workflow enforcement.

These are source observations, not freshly executed acceptance results.

## 3. Task authority

One reviewed task grant must identify:

- Grant ID/version, project identity and canonical workspace roots.
- Task purpose, approved operations, read/write boundaries and exclusions.
- Network destinations and purposes, including what data may leave the project.
- Credentials available through the host or broker, with permitted operations;
  never credential values in grants, receipts or process arguments.
- Resource class, incremental paid-spend ceiling, concurrency, wall-clock and
  repair-attempt limits. Current project authority stays at $0 new paid spending.
- Verification commands, input/environment declarations and completion checks.
- Recovery scope and preauthorized local rollback conditions.
- Expiry, revocation, task/session binding, host identity and approval evidence.

Grant approval occurs once through a supported owner-controlled mechanism.
An agent cannot expand, renew or approve its own grant by editing project files
or supplying a confirmation flag. Protect the authoritative grant outside the
agent's writable scope; a project-local copy is informational. Resume may reuse
a valid grant only after checking identity, scope, expiry and revocation.
Approval must remain a readable summary rather than a command/hash typing ritual.

Default decision table:

| Action | Inside approved task boundary | Outside boundary |
| --- | --- | --- |
| Read project files; edit task-owned files; build/test locally | Continue automatically | Request narrow expansion |
| Download approved public dependencies from approved registries | Continue within network/resource policy | Request destination/data expansion |
| Read credentials or transmit private source | Only through explicitly permitted host operation | Deny or request exact scope; never infer permission |
| Restore task-owned changes with known outcome | Automatic if preauthorized and conflict-free | Inspect and present affected changes |
| Merge, publish, deploy, modify external data, or delete valuable user data | Separate exact action approval under current rules | Deny pending approval |
| Add paid usage, change billing/security settings, or broaden credentials | Separate exact approval | Deny pending approval |
| Corrupt evidence/accounting or uncertain monetary writes/reservations | Stop dependent actions; preserve originals | No permission flag bypass |

Ordinary test/build commands can execute arbitrary code. Their names alone do
not make them safe; enforce the approved effects through host controls.

## 4. Continuous work and automatic evidence

1. At task start, resolve and display the effective grant and host capability
   status once. Separate configured, enforced and unsupported restrictions.
2. Discover verification commands from project metadata and existing config.
   Reuse exact existing trust where applicable; obtain approval for new command
   authority in the initial grant. Never silently trust discovered scripts.
3. Snapshot the starting work before the first edit. Prepare previews and undo
   records automatically; preparing them does not itself require a human prompt.
4. Record executions and relevant inputs automatically through supported hooks
   or the task runner. Preserve argv fidelity without persisting secrets: use
   credential handles and reject secret-bearing invocation paths when needed.
   Do not claim universal secret detection or silently redact signed history.
5. Missing/stale evidence schedules a rerun within the grant. Failed checks
   trigger bounded repair and retesting, not an approval request.
6. Start with conservative input invalidation. Only narrow dependencies when
   supported by evidence; never reuse a result after relevant scripts, inputs,
   dependencies, runtime, environment or policy change.
7. Separate execution permission from result freshness. Editing code must not
   repeatedly revoke task authority, but must invalidate affected test evidence.
8. On limits or lack of progress, stop the retry loop and report the concrete
   blocker, attempts and useful partial result. Do not automatically ask for
   more retries or buy resources. Reuse valid permission rather than re-asking.
9. Finish with diff, verification results, remaining gaps and recovery reference.
   Signatures remain distinct from trustworthy tests and independent execution.

## 5. Recovery and parallel work

Snapshots must include existing tracked modifications, relevant untracked files,
modes and task file ownership, with explicit exclusions and storage limits.
Do not stage, commit, clean, stash or overwrite the user's work as a side effect.
Keep generated project artifacts under `.kineti/` and out of commits.

Use an isolated task workspace or equivalent conflict-aware checkpoints. Before
restoring any path, compare its current bytes and ownership with the task's
recorded output. Preserve concurrent user/agent changes and report conflicts.
Keep intent/completion durable across crashes. Known local recovery can run
without another approval if granted; uncertain completion still requires
inspection. Do not blindly replay an external operation or inverse command.
Parallel agents share the task's aggregate budget and cannot multiply authority.
Local snapshots cannot reverse remote actions, leaked secrets or consumed money.

## 6. Proposed accounting amendment

This section requires explicit owner approval before implementation or use.
It proposes an action-aware eligibility decision separate from accounting health.

1. Preserve every original historical unknown, qualified zero, signature, cost,
   reservation and recovery record. Keep current limits and the 95% stop. Never
   represent an unvalued subscription allocation as $0 historical cost.
2. Represent historical valuation, integrity, outstanding monetary reservations,
   resource eligibility and current incremental spending as separate dimensions.
3. Historical valuation unknowns alone may permit explicitly granted work on
   verified included resources that cannot create an additional charge. Paid
   fallback, purchases, upgrades and uncertain charge exposure remain denied.
4. Loss of resource-eligibility evidence stops dependent resource use. Ordinary
   subscription allowance must remain available; wait for renewal on exhaustion.
5. Integrity failures and uncertain monetary writes/reservations remain hard
   blockers for accounting-dependent actions. Read-only inspection remains
   available without granting execution or financial authority.
6. `status` must expose preserved uncertainties and action eligibility rather
   than hiding all information behind one failure. Preserve legacy command/exit
   compatibility unless an explicit versioned amendment documents changes.
7. Publication, merge and deployment retain separate approval and any applicable
   accounting review. This amendment supplies no standing release exception.

## 7. Host enforcement and interaction

Prefer supported host permissions/sandbox APIs. Build a small adapter interface
and deliver one pinned host end to end before adding hosts. Codex MCP discovery
alone does not prove native-tool interception or autonomous agent acceptance.

The adapter must cover the execution paths claimed: terminal commands, direct
file edits, subprocesses, network access and credential operations. Restrict
out-of-scope effects at the actual host/OS boundary. Test symlink/path traversal,
shell wrappers, script changes, child processes and attempts to change policy.
If a host cannot enforce a restriction, identify the unsupported capability;
do not call that mode enforced or silently broaden permission. Host-owned safety
prompts cannot be suppressed by Kineti instructions or blanket approval flags.

Use stable structured decisions: allow, deny, needs-scope, needs-action-approval,
retry-within-grant, and inspect-uncertain. Include the reason, affected action and
smallest necessary change. Expose equivalent grant/status/check/recovery controls
through CLI and MCP. Exact command/API names are selected during design review.

Notify at meaningful milestones or blockers. Bundle independent scope questions
and deduplicate unchanged requests. Never bundle unrelated irreversible actions
into vague approval. Planning preferences are optional questions, not mandatory
permission gates. Keep explicit user-requested pauses and revocation immediate.

## 8. Acceptance requirements

Every row is pending. Evidence must bind source/artifact digests, exact commands,
environment, host/version, grant, results and limitations. Fixture tests cannot
substitute for live host acceptance. Do not expose credentials in evidence.

| ID | Required outcome and evidence |
| --- | --- |
| A01 | Existing locked goal, dirty tracked/untracked user files, historical records/signatures and monetary unknowns survive setup, work, failure, resume and cleanup byte-for-byte where unchanged. |
| A02 | One owner-approved grant permits the complete supported task; unapproved expansion, agent-written grants, expired/revoked grants and cross-project/session reuse are rejected. Test revocation during execution and stop affected running operations. |
| A03 | Real supported-host agent task performs edits, build, deliberately failing test, repair, retest and verified diff with zero further permission prompts after initial approval. No hidden auto-approval or preapproved broad access. Capture every attempted/completed action and prompt. |
| A04 | An attempted out-of-scope effect is blocked before occurrence and produces one precise expansion request. Repeating it does not create repeated identical prompts. Denial leaves independent authorized work usable. |
| A05 | CLI and MCP provide equivalent policy decisions and required controls; restart/resume reuses only valid grants. Concurrent agents share quotas and file ownership safely. |
| A06 | Snapshots preserve initial dirty/untracked work and modes. Automatic known local rollback restores task changes while preserving concurrent user edits; conflicts stop restoration. Crash-injected uncertain recovery is not replayed blindly. |
| A07 | Stale/missing evidence reruns automatically; failed tests repair within limits. Input/script/dependency/runtime/environment/policy changes invalidate relevant evidence. Fresh results are reused only within declared validity. |
| A08 | A harmless-looking command attempting outside writes, secret access, forbidden network, policy edits or child-process escape is blocked through every claimed tool path. Record exact host controls; unsupported paths prevent an enforcement claim. |
| A09 | Historical valuation unknowns remain visible and unchanged while an approved included-resource task proceeds. Possible incremental charges, resource exhaustion, corrupted accounting and unresolved monetary reservations stop the appropriate dependent action. No guessed costs. |
| A10 | Retry/time/storage/spend limits terminate loops predictably without purchases or repeated limit-extension prompts. Cancellation/revocation preserves work and stops new actions; record any unavoidable in-flight effects. |
| A11 | Merge/publication/deployment and sensitive external changes still require separate exact approval. Changed artifact/destination/scope invalidates approval. Failure/partial completion is reconciled before retry. |
| A12 | Logs/receipts/grants contain no fixture secrets; credential values are absent from argv and snapshots. Existing historical records are preserved. Verify scoped credential and outbound-data enforcement separately from log hygiene. |
| A13 | Full native regression, formatting and Clippy checks freshly pass under UTC and Asia/Kolkata on every supported release target. Recheck Cargo/npm install/runtime and footprint requirements if packaging/runtime changes. |
| A14 | Documentation and installed instructions state autonomy defaults, approval boundaries, unsupported hosts, recovery limits and prompt ownership accurately. Old configuration fields are not presented as enforced capabilities. |

For A03, record baseline and proposed behaviour on the same repository/task and
pinned host. Count owner permission requests separately from optional preference
questions, required authentication and host prompts. A claim of zero additional
permission prompts includes host prompts: if the host interrupts the task, the
end-to-end requirement has not passed. Report sample size; one task is a minimum
acceptance case, not evidence of general reliability. Include separate denial,
dirty-workspace, concurrent-edit and interrupted-resume scenarios.

## 9. Delivery sequence and approval boundaries

1. Audit existing source, host permissions and actual prompt sources. Produce
   capability map, architecture, grant schema, compatibility plan and denial
   tests. Choose/pin the first host based on verified support; stop if the
   proposed enforcement boundary is infeasible. Do not start a custom cross-OS
   sandbox as an implicit fallback.
2. Implement grants, structured decisions and automatic snapshots. Keep the
   existing evidence/accounting core and original recovery safeguards.
3. Integrate evidence reruns, bounded repair and the explicitly approved
   action-aware accounting amendment. Update instructions and CLI/MCP parity.
4. Connect one host adapter; run live autonomy and escape/denial acceptance.
5. Present the complete A01–A14 matrix and remaining gaps for owner acceptance.
   Prepare any exact release artifacts only under separately approved scope.

Implementation approval must explicitly approve this contract and section 6's
accounting amendment. This document alone grants neither. Existing user-granted
authority persists; no routine implementation decision needs a new approval.
No provider settings, credentials/scopes, billing, merging, publication or
deployment change is authorized here. Broader P01–P27 release-controller proposals
remain separate. Never mark a requirement complete from a plan or local prototype.
