# Task Dispatch: Challenger 1 (Core Defects Adversarial Verifier)

## Assignment
Empirically verify the core defects and proposed remediations reported in `docs/AUDIT_REPORT.md`:
1. Reproduce and verify Finding 1 (Critical): `bin/kineti-state.ts` gate lookup bug (`get gate.spec`). Test if proposed fix resolves the bug.
2. Reproduce and verify Finding 2 (Critical): `tests/memory-job.test.ts` baseline test failure. Test if proposed fix resolves `bun test tests/`.
3. Reproduce and verify Finding 3 (High): `bin/kineti-spend.ts` double `.kineti` path creation. Test if proposed fix resolves path construction.
4. Reproduce and verify Finding 4 (High): `bin/kineti-evidence.ts` fingerprint invalidation by `.agents/`. Test if proposed fix excludes `.agents/`.
5. Reproduce and verify Finding 5 (High): `skills/spec/SKILL.md` illegal state `gate.spec pending`. Test if proposed fix supports `pending`.

NOTE: Run tests and verifications in isolated temp directories or test commands. Maintain non-destructive boundary on repository files.

Deliver your empirical verification verdict (APPROVE or REQUEST_CHANGES) in:
`/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_challenger_1/handoff.md`
and send a completion message to parent.
