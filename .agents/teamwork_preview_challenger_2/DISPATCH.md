# Task Dispatch: Challenger 2 (Script & Security Adversarial Verifier)

## Assignment
Empirically verify the script robustness, security boundaries, and shell vulnerabilities reported in `docs/AUDIT_REPORT.md`:
1. Reproduce and verify Finding 4 (Critical): `bin/kineti-verify-gate.ts` programmatic self-trust bypass without TTY or authentication.
2. Reproduce and verify Finding 3 (Critical): `bin/lib.ts:48-58` `readJsonl` silent data erasure on corrupted line returning `[]`.
3. Reproduce and verify Finding 2 (Critical): `scripts/weekly.sh` unhandled missing repo pointer crash and word-splitting on paths containing spaces.
4. Reproduce and verify Finding 8 (High): `bin/kineti-spend.ts` circuit breaker reset bypass via static `--i-am-human` flag.
5. Reproduce and verify Finding 7 (High): `bin/kineti-memory-job.ts` uncaught TypeError crash when `--dir` is passed without an argument.

NOTE: Run verifications in isolated subshells or temporary test paths. Maintain non-destructive boundary on repository files.

Deliver your empirical verification verdict (APPROVE or REQUEST_CHANGES) in:
`/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_challenger_2/handoff.md`

## 2026-09-05T19:43:22Z
You are challenger_2, a code-executing adversarial verifier.
Your working directory is:
/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_challenger_2

MANDATORY FIRST STEP:
Read the Original Request at:
/Users/praveen/Documents/Products/kineti local harness/.agents/ORIGINAL_REQUEST.md
Also read your specific task dispatch at:
/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_challenger_2/DISPATCH.md
Also read the audit report at:
/Users/praveen/Documents/Products/kineti local harness/docs/AUDIT_REPORT.md

YOUR MISSION:
Empirically verify the script robustness, security boundaries, and shell vulnerabilities reported in docs/AUDIT_REPORT.md:
1. Reproduce Finding 4 (Critical): `bin/kineti-verify-gate.ts` programmatic self-trust bypass without TTY or confirmation.
2. Reproduce Finding 3 (Critical): `bin/lib.ts:48-58` `readJsonl` silent data erasure on corrupted line returning `[]`.
3. Reproduce Finding 2 (Critical): `scripts/weekly.sh` unhandled missing repo pointer crash and word-splitting on paths containing spaces.
4. Reproduce Finding 8 (High): `bin/kineti-spend.ts` circuit breaker reset bypass via static `--i-am-human` flag.
5. Reproduce Finding 7 (High): `bin/kineti-memory-job.ts` uncaught TypeError crash when `--dir` is passed without an argument.

Maintain non-destructive boundary on repository files. Execute tests in isolated subshells or temporary directories.
Write your adversarial verification report and verdict (APPROVE or REQUEST_CHANGES) to:
/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_challenger_2/handoff.md
When complete, send a message to parent reporting your verdict and referencing the handoff path.
