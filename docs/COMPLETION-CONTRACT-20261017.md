# Kineti completion and cleanup contract

**Prepared: October 17, 2026**

## 1. Goal and agreed scope

Finish the remaining developer-tool functionality and repair the defects documented in the repository audit. Preserve Kineti’s locked goal: **build a universal agent harness with cryptographic verification**.

Kineti remains an MIT open-source project. Users run and configure their own infrastructure. **Kineti will not host runners, an evaluator service or a shared account platform.**

Confirmed choices:

- Reuse existing **Ed25519** signing. Build the missing evaluator that independently executes owner-approved checks on users’ GitHub runners.
- Target **user-run Linux** for the first strong protected-output service.
- Test actual **Cursor and Antigravity** installations. The owner will open both apps so they can be located.
- Complete memory and incidents for **local OS users and separate projects**.
- Demonstrate the complete workflow using a small, synthetic **local task tracker**.
- Remove **R15 and R18: SSO/admin sample**, including their requirements from active instructions.
- Remove **R06 and R24: containment comparison and ingestion research**. Necessary isolation tests for the protected service remain engineering acceptance checks.
- Keep **R25–R27 public-demo work parked**, along with Proof Strength, Soul and Echo.

Keep original roadmap identifiers, titles and scope tags. Mark removed rows **“removed by owner decision”**, never “completed.” Preserve historical contracts and evidence with explicit amendment notices.

Contract approval authorizes implementation, reversible cleanup and isolated local acceptance. Publication, deployment, production provisioning, purchases and live PR comments remain outside this authorization.

## 2. Delivery checkpoints and acceptance

### Checkpoint A — Repair existing behavior and perform cleanup

Reconcile the current checkout and remote/open work before changes. Preserve the existing uncommitted implementation.

Repair every reproduced audit defect:

| Area | Required result |
|---|---|
| Companion | Generated dashboard JavaScript parses and initializes; status and controls work after login; mobile layout is usable. |
| CI summaries | Use the strict receipt validator. Unsigned, forged, stale, unsupported, skipped and zero-case evidence cannot produce success. |
| Spending | Serialize accounting updates; preserve exact totals during simultaneous writes; damaged initialized state blocks work rather than becoming `$0`. |
| Integrity records | Reject malformed lines, invalid chains and unexpected truncation. Writers verify history under the same lock used for appending. |
| Journal sync | Import through a validated, provenance-preserving import operation; merging two valid histories must leave a valid destination history. |
| Scheduler | Preserve local-time cron matching while always returning an instant strictly after the anchor, including both repeated-hour occurrences. |
| Undo | Preserve structured arguments, including spaces, empty strings and Unicode, through registration and execution. |
| Canonical JSON | Use validated cross-language canonical serialization; reject invalid numbers and unsupported values. |
| PR reporting | Require the expected author and stable marker; check the current PR head and preserve explicit stale/error states. |
| Evidence instructions | Correct executable examples and require a declared test inventory. Recording incomplete evidence must not report an unqualified success. |
| Secrets | Document safe credential inputs; avoid logging secret values while preserving exact command/input binding. |
| Maintenance | Make weekly operations enter each approved project and use strict, locked journal maintenance. |

Compatibility and recovery requirements:

- Keep old receipts and native identifiers readable under their original formats. Introduce versioned formats where corrected serialization or identity rules would change existing hashes.
- Do not rewrite old signed records or silently give them stronger authority.
- Preserve the damaged egress ledger byte for byte. Add explicit owner recovery that starts a visibly linked replacement ledger and records the unresolved historical damage.
- Inspect malformed stored undo commands before allowing their execution. Do not run a blanket rollback.
- Reject corrupt existing goal/state files rather than treating them as uninitialized.

Perform cleanup after preserving undo information:

- Remove research-specific commands, schemas, package exports, fixtures and active instructions associated with removed scope. Preserve Git history and saved audit evidence.
- Retire the obsolete standalone-download installer in favor of the supported package path.
- Align active documentation, skill examples, command indexes, capability declarations and privacy/product claims with implemented behavior.
- Replace hardcoded acceptance baselines with explicit candidate/baseline inputs.
- Typecheck acceptance scripts and verify generated router consistency.
- Define and test the intentional npm package contents.
- Consolidate duplicated Cargo workspace maintenance.
- Remove only reproducible build outputs and disposable caches. Preserve configuration, keys, receipts, journals, recovery data and evidence.

**Acceptance:** each audit reproduction first demonstrates the defect, then passes a focused regression check after repair. Cleanup must preserve unrelated work and existing user configuration.

### Checkpoint B — Complete trusted evaluation and protected output

**R01–R03 and R05:** extend the existing signed protocol into executable software.

The evaluator must:

- Load owner-approved checks, policy and contract from trusted configuration, independently of the candidate.
- Evaluate an immutable artifact with exact-byte identity.
- Run the candidate in an isolated Linux environment. Trusted black-box checks communicate through a declared CLI or HTTP interface.
- Keep signing keys, evaluator code and policy outside candidate access. Never import candidate code into the unrestricted checker.
- Bind the artifact, contract, check inventory, arguments, environment, dependency/runtime image and outputs to the signed result.
- Distinguish passed, failed, skipped, unsupported, unrun and unknown checks. Only complete required success can authorize an effect.
- Treat candidate-authored test suites as diagnostics, separate from trusted black-box checks.

Implement one protected action: **export approved artifact bytes to an allowlisted relative path beneath a service-owned output root**.

The Linux service must own its policy, immutable inputs, signing authority, destination and durable request store. Candidate processes and host clients cannot modify them directly.

Requests bind the evaluation, caller identity, unique request ID, destination, arguments, byte digest, expiry and policy. Results distinguish **denied, pending, applied and unknown**.

Use durable transactions, serialized destination handling and startup reconciliation. Missing or damaged initialized replay state requires explicit owner recovery; it cannot reset used authorizations.

**Acceptance:** authentic control succeeds; artifact/check/policy/key substitutions, traversal, wrong bytes/arguments/identity, expiry, revocation and replay are rejected. Deterministic concurrency and crash tests establish reconciliation behavior. Denied operations leave sentinels unchanged. No universal workspace interception or distributed exactly-once claim.

### Checkpoint C — Finish packaging and host integrations

**R04, R07–R09, R19–R21:**

- Reuse the working Cursor merge helper, installer and MCP server.
- Deliver a reusable GitHub Action integration alongside the existing repository CI.
- Load trusted evaluator/action code from an owner-approved pinned source; do not execute privileged evaluator logic from an untrusted PR checkout.
- Keep candidate execution separate from signing credentials and privileged reporting.
- Package Codex and Claude Code guidance using mechanisms their hosts actually support.
- Connect Cursor to the protected Linux action.
- Complete Cursor and Antigravity workflow context, status and natural-language routing through supported host facilities.
- Keep capability declarations versioned and evidence-based. Guidance remains guidance; unsupported capabilities remain explicitly unsupported.

**Acceptance:** isolated install, repeat install, upgrade, conflicts, missing runtime/policy, offline failure, spaces in paths and ownership-aware uninstall. Run actual Cursor protected-action allow/deny calls and actual Cursor/Antigravity routing and background-behavior cases.

Use existing host subscriptions for model acceptance. Record available usage; unavailable billing remains unknown. Do not purchase credits or add paid API use.

Remote GitHub acceptance is a final gate after the workflow changes are concrete and reviewable. Obtain any missing repository permission or authorization then; do not request full-account access.

### Checkpoint D — Complete the workflow

**R10–R14, R16–R17:** use the synthetic task tracker to demonstrate:

**Mode selection → discovery → storyboard/mock-data approval → no-data decision → feature/tool/architecture choices → outcome-contract approval → build and verification.**

The sample has a small task list with add/edit/complete behavior, local synthetic persistence, and relevant empty/error states. It has no accounts, SSO, admin platform or paid services.

Complete and verify:

- Explicit Harness/Gate/Both selection without a default.
- Ordinary chat creates no project state or scaffolding.
- Durable artifact versions, decisions, approvals and task-to-outcome links.
- Restart and host handoff without lost decisions.
- Changed artifacts invalidate only dependent approvals.
- Conflicting requests produce numbered choices and pause affected work.
- Separate projects cannot share state accidentally.

The owner approves the rendered storyboard and completed outcome contract against their actual versions. Fixture approvals are used only for automated tests. Other independent work continues while a real approval is pending.

### Checkpoint E — Complete memory and incident integration

**R22–R23:**

- Preserve the existing local APIs and encrypted storage while completing restart, correction, deletion and source-resolution behavior.
- Derive identity from the actual OS account and project; do not accept caller-selected identity as authorization.
- Apply the same scope, correction and deletion rules to every active retrieval path, including native graph/vector paths.
- Complete native persistence used by the product. Remove unsupported durable-storage claims from any remaining prototype.
- Connect prepared memory to supported host requests and inspect the delivered context.
- Keep deleted content out of subsequent current and historical-context retrieval. Retain only permitted audit metadata; disclose backup/snapshot limits.
- Complete incident commands/API with inspectable evidence, tasks and dependencies.
- Distinguish observations, reported causes and hypotheses; display missing or conflicting evidence.

**Acceptance:** two real OS identities and two projects through ingest, persist, restart, retrieve, correct and delete. Capture actual outgoing context, then exercise live host use. Test incident correction, restart, missing/forged references, out-of-order events and scope isolation.

No shared account-management or identity-provider platform is included.

## 3. Public interfaces and compatibility

Add documented, versioned interfaces for:

- **Evaluation:** submit an immutable artifact, execute an owner-approved inventory, inspect a signed decision and verify it against pinned authorities.
- **Protected output:** submit a bound request, inspect its durable outcome and perform owner-controlled recovery.
- **Host capabilities:** declare installation, guidance, background behavior, routing and protected-action support separately.
- **Recovery:** inspect damage and explicitly recover local ledgers without concealing historical failures.

Reuse existing CLI/MCP entry points where practical. Keep application source changes unnecessary for installing the named adapter.

Legacy project state remains readable. Old local receipts remain local receipts. New trusted acceptance requires the new evaluator path.

## 4. Verification and final handoff

Before changes, save undo steps and check spending. Keep fixtures, homes, caches and evidence inside the project. Record outbound requests before sending them; repair/recover egress logging before remote checks.

Run focused regressions and then:

- Router generation/freshness and all applicable TypeScript checks.
- Full TypeScript tests under UTC, Asia/Kolkata and America/New_York.
- Installer and candidate-package install/upgrade/uninstall acceptance.
- Rust formatting, Clippy and tests.
- Trusted evaluator, protected-service and conformance cases on Linux.
- Actual Cursor/Antigravity acceptance and the approved workflow sample.
- Memory/incident identity, restart, deletion and delivered-context checks.

Save and recheck evidence against the final relevant source. Do not add overlapping test counts or label unsupported cases passed.

The final handoff includes:

- Updated R01–R27 matrix, with R06/R15/R18/R24 removed and R25–R27 parked.
- Audit defect resolutions and remaining failures.
- Exact tested source/package identities and environment versions.
- Migration, installation, upgrade, uninstall and recovery instructions.
- Package-size/startup observations and cleanup results.
- Actual host/GitHub acceptance evidence and any outstanding owner gate.

A document, scaffold, mocked response or passing unit suite does not by itself complete the behavior. No feature is marked complete until its stated acceptance runs.
