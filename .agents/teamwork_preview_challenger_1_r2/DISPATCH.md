# Task Dispatch: Challenger 1 (Core Defects Adversarial Verifier - Replacement)

## Assignment
Empirically verify the core defects reported in docs/AUDIT_REPORT.md:
1. Reproduce Finding 1 (Critical): `bin/kineti-state.ts` gate lookup bug (`get gate.spec`). Verify if proposed diff fixes it in an isolated temp test.
2. Reproduce Finding 2 (Critical): `tests/memory-job.test.ts` baseline test failure. Verify root cause (hash delimiter mismatch & missing hash on record).
3. Reproduce Finding 3 (High): `bin/kineti-spend.ts` double `.kineti` path creation.
4. Reproduce Finding 4 (High): `bin/kineti-evidence.ts` fingerprint invalidation by `.agents/`.
5. Reproduce Finding 5 (High): `skills/spec/SKILL.md` illegal state `gate.spec pending` rejected by `kineti-state.ts`.

Maintain non-destructive boundary on repository files. Execute tests in isolated temp directories.
Write your adversarial verification report and verdict (APPROVE or REQUEST_CHANGES) to:
`/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_challenger_1_r2/handoff.md`
When complete, send a message to parent reporting your verdict and referencing the handoff path.
