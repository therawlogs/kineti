# Kineti remaining acceptance contract

**Completion requires one real agent to deliver a verified diff under one approved task grant, with zero further permission prompts, and the same candidate to pass A01–A14.** All fourteen rows are open as of November 9, 2026. The integration owner maintains one candidate and one evidence matrix. An independent reviewer checks the final source, artifacts and observations. [Original acceptance contract, sections 3–9](https://github.com/therawlogs/kineti/blob/8d85962eb903f0265bb358a61949c8aba08c226e/docs/AUTONOMOUS-BUILD-CONTRACT.md)

## Starting evidence

| Item | Observed result |
| --- | --- |
| Reviewed candidate | Parent `657c2d070b137d712fb71df23ce0de171e2c63f5` plus complete patch SHA-256 `ab29da32362b8917c269f5b8d19888ed7f0d77aef9fb5b4b630bb083c87baea3`. Release binary SHA-256 `262fb8c42a7f8898326c88ad96d0070f37f57d28e42ecc111295c24520cfbf4b`. This identifies the separately retained implementation candidate. |
| Last live run | The owner authenticated, created clone `k9` and observed the grant. Full Lima inventory failed with `bufio.Scanner: token too long` before the first boot intent. A narrow query returned `k9 Stopped 0 0`. Both VM leases were idle. No guest check, repair or delivery occurred. The one-run approval is spent. |
| Local proof | macOS arm64 passed formatting, Clippy, 67 library tests, 73 integration tests and two opt-in fixtures in each of UTC and Asia/Kolkata. The isolated installed copy measured 1,824,718 bytes. A separate local SDK fixture observed check, repair and retest exits of 1, 0 and 0. |
| Preservation | The original checkout, dirty tracked files, untracked files, deterministic fixture and 38 protected files or historical prefixes matched their baselines after the live attempt. All thirteen historical valuations remain unresolved. |

## Order of work and exit tests

1. **Repair pre-boot inspection.** Provide one reproducible command that shows project-only status and the precise pre-boot failure using read-only inspection of the retained clone. Request bounded inventory fields sufficient to verify the exact VM identity, configuration, isolation, lifecycle state and driver ownership. A stopped name and two zero PIDs establish only the stop observation. Test missing, ambiguous, malformed and mismatched fields, an active driver, and the retained large provisioning configuration. The normal CLI must verify the stopped clone and reject each unsafe case before any boot. Preserve the retained clone and journal for inspection.
2. **Complete one deterministic guest task.** Rebuild and hash the corrected candidate. Present the exact next invocation, grant, expiry, boot count, resource limits and recovery reference for the applicable live approval. Start with dirty tracked and untracked files. Through the CLI, owner, authenticated worker and pinned SDK, record a genuinely failing check, exactly one authorized repair and a fresh passing retest. Record intent before each effect and completion after observation. Confirm stop and driver reap, export read-only on a fresh boot, and bind the delivered patch to the grant, session, worker, SDK, boot identities, retest receipt and exported bytes. Verify the patch against the initial snapshot and compare the user's current files at delivery.
3. **Prove interruption and recovery.** Repeat with cancellation, active revocation, owner loss, interrupted resume and a concurrent user edit. For each case record the last durable event, processes stopped, effects already observed, file hashes and modes, and the next safe action. Restore known task-owned changes automatically only within the original recovery grant. Preserve conflicting paths and identify them to the user. An uncertain outcome stays inspectable until reconciled.
4. **Run the real-agent case.** Obtain exact scope for the supported login and model turn. Use the same task, repository and pinned host for the baseline and candidate. Capture edits, build, deliberate failure, repair, retest and delivered diff. Count permission requests, host prompts, authentication prompts and preference questions separately. A03 requires zero further permission prompts after the initial task approval, including host permission prompts. If cold restart requires another login, record the observed incompatibility for an owner decision.
5. **Close the matrix and hand over.** On actual macOS arm64/x64 and Linux musl arm64/x64 hosts, run formatting, Clippy and complete native suites in both time zones. Confirm runner availability within the spending limit before dispatch. Check the exact website distribution artifacts through isolated installation, runtime, CLI/MCP and footprint tests on each target. The installed footprint must be below 10,000,000 bytes per target. Record the commands, artifact hashes and inventories. [Native gate contract](GATE-0.6.0-CONTRACT.md) · [Website distribution decision](https://github.com/therawlogs/kineti/blob/8d85962eb903f0265bb358a61949c8aba08c226e/docs/WEBSITE-ONLY-DISTRIBUTION-20261031.md)

## Acceptance matrix

| Row | Evidence required to close |
| --- | --- |
| A01 | Compare the locked goal, user files and modes, original records, signatures and unresolved costs across setup, failure, resume and cleanup. |
| A02 | One owner grant supports the complete task. Reject forged, expanded, expired, revoked and cross-project or cross-session grants. Observe active revocation stopping affected work. |
| A03 | Supply the real-agent transcript, prompt counts, fresh results and delivered diff from step 4. |
| A04 | Block an out-of-scope effect before it happens. Issue one precise expansion request, deduplicate repeats, and allow independent authorized work to continue. |
| A05 | Demonstrate equivalent CLI/MCP controls and decisions, valid-grant resume, shared quotas and safe file ownership under concurrency. |
| A06 | Demonstrate snapshots, known automatic rollback, preserved concurrent edits and modes, named conflicts, and inspection after an uncertain interruption. |
| A07 | Show automatic reruns when inputs, scripts, dependencies, runtime, environment or policy change. Reuse evidence only within its declared validity. |
| A08 | Verify the host boundary on every claimed tool path for outside writes, secret reads, network access, policy changes and subprocesses. Record unsupported paths. |
| A09 | An approved included-resource task proceeds when the owner-controlled record binds every covered entry to its exact ID, content digest and approval evidence. Verify the original ledger byte-for-byte and report the same blocking reason through CLI and MCP. Missing approval, a new or changed unknown, duplicate ID, unresolved reservation, integrity failure, exhausted allowance or possible extra charge stops dependent work. |
| A10 | Show bounded retries, time, storage and spending. Cancel and revoke during work, preserve the workspace, and account for in-flight effects. |
| A11 | In a controlled fixture, demonstrate separate exact approvals for external actions, invalidation after artifact, destination or scope changes, and reconciliation before retry. |
| A12 | Verify scoped credential use and outbound-data enforcement during the real task. Check argv, logs, grants, receipts and snapshots for fixture secrets, including across restart. |
| A13 | Supply the four native target reports, both time zones and the exact installed-artifact results from step 5. |
| A14 | Verify installed instructions against observed behavior for autonomy defaults, supported hosts, approval and prompt ownership, and recovery limits. |

## Operating limits and handover

Keep **$0 new paid spending**, integer total and stage limits, the 95% stop and the thirteen original unknown valuations. Recheck accounting and resource eligibility before approval, execution and resume. After two unsuccessful fixes for the same blocker, preserve the evidence and return the next proposed action for review. Keep general macOS admission closed during acceptance. Login, model turns, live VM operations, global installation, release, publication and deployment retain their applicable approval boundaries.

For each row record the source and artifact digests, actual environment, exact command, grant, expected result, observed result and limitations. Mark any missing host, approval or observation **open** and name the dependency. Use the 37-workflow inventory to identify affected regression cases. After a fix, rerun affected rows on the final candidate and recheck preservation. Deliver one packet containing the verified task diff, completed matrix, artifact hashes, independent review, recovery reference and user instructions. Store raw runtime evidence locally under `.kineti/` and keep credentials and personal paths out of Git.
