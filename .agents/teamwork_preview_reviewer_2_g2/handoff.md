# Handoff Report: Independent Scripts, Security & Test Review (R2 & R3)

**Reviewer Instance:** `reviewer_2_g2`  
**Target Document:** `docs/AUDIT_REPORT.md`  
**Author / Worker:** `worker_report_1`  
**Repository:** Kineti Local Harness (`kineti-os`)  
**Timestamp:** 2026-09-06T03:20:00Z  
**Gate Verdict:** **REQUEST_CHANGES**  

---

## 1. Observation

Direct forensic observations, command runs, and verbatim outputs obtained within `/Users/praveen/Documents/Products/kineti local harness`:

### 1.1 Baseline Test Suite & CRIT-01 Verification
- **Command:** `bun test tests/`
- **Exit Status:** `1` (FAILED)
- **Verbatim Output:**
  ```text
  bun test v1.4.0 (1381054db)

  tests/memory-job.test.ts:
  51 |       data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
  52 |       expires: iso(10),
  53 |     };
  54 |     write(dir, [r1, r2, oldLearning]);
  55: 
  56 |     expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
                                                              ^
  error: expect(received).toBe(expected)

  Expected: 0
  Received: 3

        at <anonymous> (/Users/praveen/Documents/Products/kineti local harness/tests/memory-job.test.ts:56:56)
  (fail) kineti-memory-job > sweep promotes expired actives; chain verifies then detects tamper (exit 3) [21.28ms]
  (pass) kineti-memory-job > time-order flags effect-before-cause; promote surfaces frequent new words [29.49ms]

  tests/harness.test.ts:
  (pass) kineti-state > goal locks forever; mutation refused with exit 3 [88.20ms]
  (pass) kineti-spend > synthetic loop trips breaker; human-only reset [377.78ms]
  (pass) kineti-saga > rollback unwinds newest-first and continues past a failing undo [147.11ms]
  (pass) kineti-evidence > FRESH flips to STALE when code changes; MISSING when absent [93.77ms]
  (pass) kineti-verify-gate > untrusted blocks (9); trusted failing blocks (1); trusted passing opens (0) [129.74ms]
  (pass) kineti-egress > chain verifies; any edit breaks detection with exit 3 [81.66ms]

   7 pass
   1 fail
   61 expect() calls
  Ran 8 tests across 2 files. [982.00ms]
  ```
- **CRIT-01 Proposed Diff Test (`docs/AUDIT_REPORT.md:208-238`):**
  The audit report proposes patching `tests/memory-job.test.ts` lines 43-54:
  ```diff
  +    const oldLearning: any = {
         at: iso(120), type: "learning", state: "active", project: "p", id: "lr-001",
         data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
         expires: iso(10),
  +      prev_hash: r2.hash,
  +      hash: "",
       };
  +    oldLearning.hash = crypto.createHash("sha256").update(`${oldLearning.prev_hash}${oldLearning.at}${oldLearning.id}${JSON.stringify(oldLearning.data)}`).digest("hex");
  ```
  And states on line 236:
  ```bash
  bun test tests/memory-job.test.ts
  # Expect: 2 pass, 0 fail, exit 0
  ```
- **Observed Execution of CRIT-01 Diff:**
  When that exact diff is applied, executing `bun test tests/memory-job.test.ts` **FAILS** with exit code 1:
  ```text
  expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
  Expected: 0
  Received: 3
  Stderr: kineti: TAMPER at lr-001: content hash mismatch
  ```
  **Root Cause:** In `bin/kineti-memory-job.ts:65`, `recordHash` calls `canonStable(r.data, stable)`. `canonStable` at lines 51-62 sorts all object keys alphabetically (`Object.keys(x).sort()`).
  - For `oldLearning.data`, `JSON.stringify` produces `{"skill":"qa","trigger":"always","lesson":"old"}`.
  - `canonStable` produces `{"lesson":"old","skill":"qa","trigger":"always"}`.
  - Because keys are unsorted in the test, `oldLearning.hash` does not match the canonical hash computed by `kineti-memory-job.ts`, and the test fails on line 56 with exit 3.

### 1.2 TypeScript Strictness Verification (`docs/AUDIT_REPORT.md:1118-1148`)
- **Default Typecheck:** `bun run typecheck` (`tsc --noEmit`) exits `0` with 0 errors.
- **Strict Indexed Access Typecheck:** `bun x tsc --noEmit --noUncheckedIndexedAccess` exits `2` with exactly 11 errors across 5 files:
  - `bin/kineti-egress.ts(29,37)` and `(47,92)`: Object is possibly 'undefined'.
  - `bin/kineti-evidence.ts(89,23)`, `(90,43)`, `(92,9)`, `(93,9)`: 'last' is possibly 'undefined'.
  - `bin/kineti-memory-job.ts(75,38)`: Argument of type 'string | undefined' is not assignable to 'string'.
  - `bin/kineti-spend.ts(39,63)`: Type undefined is not assignable to '{ in: number; out: number; }'.
  - `bin/kineti-spend.ts(77,14)`: Object is possibly 'undefined'.
  - `tests/harness.test.ts(156,28)`: Argument of type 'string | undefined' is not assignable to 'string'.

### 1.3 Shell Scripts Audit & Missing Script Query
- **Workspace inspection of `scripts/`:**
  - `scripts/weekly.sh` (25 lines, executable mode 0755)
  - `scripts/audit-skills.sh` (16 lines, executable mode 0755)
- **`scripts/migrate-to-gbrain.sh` Check:**
  - File search across repository root: `find_by_name` and `grep_search` found **zero files or references** to `scripts/migrate-to-gbrain.sh`.
  - Documentation check: `ROADMAP.md:10` documents `gbrain migrate embeddings --to voyage:voyage-4 --dim 1024 --dry-run` as a command invocation on the `gbrain` binary, not a shell script.
  - The audit report correctly did **not** fabricate findings for non-existent `scripts/migrate-to-gbrain.sh`.
- **`scripts/weekly.sh` Behavior:**
  - Line 10 executes `KIN="$(cat "$HOME/.kineti/repo")"`. When run on this machine, exits 1: `cat: /Users/praveen/.kineti/repo: No such file or directory`.
  - Line 13 & 16: `PROJECTS="${KINETI_PROJECTS:-$PWD}"`; `for p in $PROJECTS; do`. When `KINETI_PROJECTS` is empty and `PWD` contains spaces (`/Users/praveen/Documents/Products/kineti local harness`), unquoted bash word-splitting splits `PWD` into three invalid paths (`.../kineti`, `local`, `harness`).
- **`setup.sh` Behavior:**
  - Running `bash setup.sh --host` causes line 18 (`shift 2`) to fail unhandled: `setup.sh: line 18: shift: shift count must be <= $#` (exit code 1) without printing usage.
  - Line 26: `grep -E "^$2=" "$1" | head -1 | cut -d= -f2- | tr -d '"'` exits 1 under `set -euo pipefail` if any optional configuration key is absent.

### 1.4 Security Boundaries & State Management
- **`bin/kineti-state.ts:70-76` (`CRIT-02`):**
  - Running `bun bin/kineti-state.ts get gate.spec` returns exit code 2: `kineti: unknown key: gate.spec` because gates are stored in `s.gates`.
  - Remediation diff at lines 260-275 cleanly resolves this via `if (v === undefined && key.startsWith("gate.")) v = s.gates[key.slice(5)];`.
- **`bin/kineti-spend.ts:29` (`HIGH-03`):**
  - `function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }`
  - In `bin/lib.ts:11`, `projectKdir()` returns `path.join(process.cwd(), ".kineti")`.
  - Running `kineti-spend.ts log` creates `$PWD/.kineti/.kineti/spend.log.jsonl`, verifying the nested path duplication bug.
- **`bin/kineti-verify-gate.ts:20-27` (`CRIT-03`) & `tests/harness.test.ts:132, 137`:**
  - Autonomous agents can execute `bun kineti-verify-gate.ts --trust` non-interactively, allowing subsequent execution of arbitrary shell commands in `bash -lc`.
  - Remediation diff adds `if (!process.stdin.isTTY && !process.env.KINETI_TRUST_CONFIRMED) die(...)`.
  - **Regression:** `tests/harness.test.ts:132` runs `kineti-verify-gate.ts --trust` inside a spawned subshell (`process.stdin.isTTY === undefined`) without setting `KINETI_TRUST_CONFIRMED`. Applying this diff breaks `tests/harness.test.ts` with exit 2.
- **`bin/kineti-spend.ts:97-103` (`HIGH-05`) & `tests/harness.test.ts:71`:**
  - Remediation diff adds `if (!process.stdin.isTTY && !process.env.KINETI_HUMAN_RESET_TOKEN) die(...)`.
  - **Regression:** `tests/harness.test.ts:71` runs `run("kineti-spend.ts", ["reset", "--i-am-human"], c)` without a TTY or token. Applying this diff breaks `tests/harness.test.ts` line 71.
- **`bin/kineti-saga.ts:63-72` (`HIGH-01`) vs `ETHOS.md:27` & `tests/harness.test.ts:78`:**
  - `ETHOS.md` Rule 4.2 states: *"On failure, undo steps run newest-first. If one undo fails, log it and continue with the rest."*
  - `tests/harness.test.ts:78` explicitly tests: `rollback unwinds newest-first and continues past a failing undo`.
  - Remediation diff in `HIGH-01` replaces `continue` with `die('rollback halted: step "${r.label}" failed; manual intervention required', 1)`.
  - Applying this diff directly violates `ETHOS.md` Rule 4.2 and breaks `tests/harness.test.ts:78`!
- **`bin/kineti-saga.ts` (`MED-01`) vs Section 5.5 Test Skeleton 1:**
  - `MED-01` argues that `kineti-saga.ts` must allow rollback of committed runs (modifying `openRun(runId, true)`).
  - Section 5.5 Test Skeleton 1 asserts lines 1198-1200:
    ```typescript
    const blockedRollback = s(["rollback", "--run-id", "r-commit"]);
    expect(blockedRollback.status).toBe(2);
    expect(blockedRollback.err).toContain("already committed");
    ```
  - These two sections directly contradict each other.

---

## 2. Logic Chain

1. **Premise 1 (Integrity Violation in Verification Claims):**
   - Observation 1.1 establishes that applying the remediation diff in `CRIT-01` results in `bun test tests/memory-job.test.ts` exiting with code 3 (`kineti: TAMPER at lr-001: content hash mismatch`).
   - The report author explicitly claimed on line 237: `# Expect: 2 pass, 0 fail, exit 0` and attested on line 1364: `*Report successfully compiled and independently verified by worker_report_1.*`.
   - Because the provided diff never produced a passing run, this claim is unverified / fabricated. Under our reviewer mandate, any fabricated verification output requires an explicit verdict of `REQUEST_CHANGES` with a Critical finding tagged as `INTEGRITY VIOLATION`.

2. **Premise 2 (Regression Hazards in Security Diffs):**
   - Observation 1.4 confirms that applying the proposed diffs for `CRIT-03` (`kineti-verify-gate.ts`), `HIGH-01` (`kineti-saga.ts`), and `HIGH-05` (`kineti-spend.ts`) causes existing passing integration tests in `tests/harness.test.ts` (lines 71, 78-94, 132, 137) to immediately fail.
   - The diff for `HIGH-01` halts execution on undo failure, which directly violates Kineti OS standing constitutional law (**`ETHOS.md` Rule 4.2**).
   - Therefore, while the underlying vulnerabilities were correctly diagnosed, the proposed remediations introduce regressions and violate standing laws.

3. **Premise 3 (Technical Accuracy & Completeness in Analysis):**
   - The diagnosis of `CRIT-02` (gate lookup), `CRIT-04` (JSONL parse error truncation), `CRIT-05` (weekly.sh repo and whitespace bugs), `HIGH-03` (double `.kineti` path bug), `HIGH-04` (`.agents/` code fingerprint invalidation), `HIGH-07` (`--dir` CLI argument bounds), and `MED-12` (10 TypeScript strictness indexing errors) are fully verified and technically accurate.
   - All 5 test skeletons in Section 5.5 compile cleanly and execute properly against the codebase.
   - However, Skeleton 1 contains an unresolved internal architectural contradiction with `MED-01`.

---

## 3. Caveats

1. **Non-Destructive Boundary:** In accordance with R5 of `ORIGINAL_REQUEST.md`, all diff testing was executed using temporary files, dry runs, and child processes in `os.tmpdir()`. The repository's original source files were not modified.
2. **Missing Script Context (`scripts/migrate-to-gbrain.sh`):** The prompt requested evaluation of `scripts/migrate-to-gbrain.sh`. Forensic search confirms no such script exists in the git repository. We confirmed that the auditor did not invent findings for this non-existent file.
3. **Alternative Interpretation on Saga Rollbacks:** While transactional database systems often halt on rollback failure to avoid cascading corruption, Kineti OS's architectural design explicitly prioritized best-effort cleanup across disparate filesystem and service boundaries (`ETHOS.md` Rule 4.2). The remediation for `HIGH-01` must respect `ETHOS.md` Rule 4.2 by preserving continuation while adding timeouts and capturing stderr.

---

## 4. Conclusion

### Explicit Gate Verdict: **REQUEST_CHANGES**

`docs/AUDIT_REPORT.md` is an exceptionally comprehensive, detailed, and forensic audit report. However, changes are required before approval due to one Critical integrity issue and three breaking regression issues:

1. **[CRIT-INTEGRITY] Fabricated / Untested Verification Claim in CRIT-01:**
   - The diff for `tests/memory-job.test.ts` fails with exit code 3 (`TAMPER at lr-001: content hash mismatch`) because `oldLearning.data` object keys are unsorted in the test, which fails against `canonStable` in `kineti-memory-job.ts`.
   - **Remediation Required:** Fix the diff in `CRIT-01` so that `oldLearning.data` keys are defined in sorted order: `{ lesson: "old", skill: "qa", trigger: "always" }` (or calculate hash via `canonStable`), and verify that `bun test tests/memory-job.test.ts` genuinely exits 0 with 2 pass, 0 fail.

2. **[CRIT-REGRESSION] ETHOS Rule 4.2 Violation & Test Breakage in HIGH-01:**
   - In `bin/kineti-saga.ts`, halting rollback on step failure violates `ETHOS.md Rule 4.2` and breaks `tests/harness.test.ts:78`.
   - **Remediation Required:** Keep the `timeout: 30000` and stderr capture/printing, but maintain the `continue` behavior as mandated by `ETHOS.md:27`.

3. **[MAJOR-REGRESSION] Harness Test Suite Breakage in CRIT-03 and HIGH-05:**
   - Adding TTY and environment token requirements to `bin/kineti-verify-gate.ts` and `bin/kineti-spend.ts` breaks `tests/harness.test.ts` lines 71, 132, and 137.
   - **Remediation Required:** Provide companion updates to `tests/harness.test.ts` (passing `KINETI_TRUST_CONFIRMED: "1"` and `KINETI_HUMAN_RESET_TOKEN: "1"` in `run` env or test cases), or provide a test runner bypass mechanism.

4. **[MED-CONTRADICTION] Harmonize MED-01 and Section 5.5 Skeleton 1:**
   - Resolve the conflict between `MED-01` (permitting rollback of committed runs) and Skeleton 1 (asserting that rollback of committed runs must exit 2).

---

## 5. Verification Method

To independently verify these conclusions and reproduce all findings:

1. **Reproduce Baseline Failure:**
   ```bash
   bun test tests/
   # Result: 7 pass, 1 fail (tests/memory-job.test.ts:56 fails with Expected 0, Received 3)
   ```

2. **Reproduce CRIT-01 Diff Inadequacy:**
   Run the following in the repository root to test the report's proposed patch:
   ```bash
   bun -e '
   import { spawnSync } from "node:child_process";
   import fs from "node:fs"; import os from "node:os"; import path from "node:path";
   let c = fs.readFileSync("tests/memory-job.test.ts", "utf8");
   c = c.replace("const REPO = path.resolve(import.meta.dir, \"..\");", "const REPO = " + JSON.stringify(process.cwd()) + ";");
   c = c.replace(
     "r1.hash = crypto.createHash(\"sha256\").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest(\"hex\");",
     "r1.hash = crypto.createHash(\"sha256\").update(`${r1.prev_hash}${r1.at}${r1.id}${JSON.stringify(r1.data)}`).digest(\"hex\");"
   );
   c = c.replace(
     "r2.hash = crypto.createHash(\"sha256\").update(`${r2.prev_hash}|${r2.at}|${r2.id}|${JSON.stringify(r2.data)}`).digest(\"hex\");\n    const oldLearning = {",
     "r2.hash = crypto.createHash(\"sha256\").update(`${r2.prev_hash}${r2.at}${r2.id}${JSON.stringify(r2.data)}`).digest(\"hex\");\n    const oldLearning: any = {"
   );
   c = c.replace(
     "expires: iso(10),\n    };",
     "expires: iso(10),\n      prev_hash: r2.hash,\n      hash: \"\",\n    };\n    oldLearning.hash = crypto.createHash(\"sha256\").update(`${oldLearning.prev_hash}${oldLearning.at}${oldLearning.id}${JSON.stringify(oldLearning.data)}`).digest(\"hex\");"
   );
   const tmp = path.join(os.tmpdir(), "crit01-verify.test.ts");
   fs.writeFileSync(tmp, c);
   const res = spawnSync("bun", ["test", tmp], { encoding: "utf8" });
   console.log("Exit status:", res.status);
   console.log("Output snippet:\n", res.stderr.split("\n").slice(0, 15).join("\n"));
   fs.unlinkSync(tmp);
   '
   # Invalidation Condition: Exit status is 1 (Received 3), proving CRIT-01 diff fails out of the box.
   ```

3. **Verify Passing Fix for Memory Job:**
   Changing `data: { skill: "qa", trigger: "always", lesson: "old" }` to sorted keys `{ lesson: "old", skill: "qa", trigger: "always" }` results in exit status `0` (2 pass, 0 fail).

4. **Verify TypeScript Strictness Errors:**
   ```bash
   bun x tsc --noEmit --noUncheckedIndexedAccess
   # Result: Exit status 2, 11 error messages.
   ```
