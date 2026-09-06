# Empirical Verification Report & Verdict: Script Robustness, Security Boundaries & Shell Hazards

- **Agent:** `challenger_2` (Adversarial Verifier: Scripts, Shell & Security Boundaries)
- **Role:** Empirical Challenger / Critic
- **Workspace:** `/Users/praveen/Documents/Products/kineti local harness`
- **Target Document:** `docs/AUDIT_REPORT.md`
- **Assigned Findings:**
  1. Finding 4 (CRIT-03): `bin/kineti-verify-gate.ts:20-27` programmatic self-trust bypass without TTY or confirmation.
  2. Finding 3 (CRIT-04): `bin/lib.ts:48-58` `readJsonl` silent data erasure on corrupted line returning `[]`.
  3. Finding 2 (CRIT-05): `scripts/weekly.sh:10, 13, 16` unhandled missing repo pointer crash and word-splitting on paths containing spaces.
  4. Finding 8 (HIGH-05): `bin/kineti-spend.ts:97-103` circuit breaker reset bypass via static `--i-am-human` flag.
  5. Finding 7 (HIGH-07): `bin/kineti-memory-job.ts:74-75` uncaught TypeError crash when `--dir` is passed without an argument.
- **Overall Verdict:** **`APPROVE`** (All 5 reported findings were independently reproduced, verified, and confirmed to be genuine vulnerabilities in the current codebase).

---

## 1. Challenge Summary

- **Overall Risk Assessment:** **CRITICAL**
- **Findings Tested:** 5 / 5
- **Reproduction Success Rate:** 100% (5 out of 5 vulnerabilities confirmed empirically via test execution)
- **Remediation Feasibility:** 100% (All proposed diffs were verified in isolated harnesses to completely eliminate the failure modes)
- **Impact on Audit Report:** The findings in `docs/AUDIT_REPORT.md` are accurate, factual, and fully supported by empirical execution.

---

## 2. 5-Component Handoff Report

### 1. Observation

Each assigned defect was observed directly in the codebase and reproduced using code-executing tests:

#### Observation 1 (Finding 4 / CRIT-03 — Verify Gate Self-Trust Bypass)
- **Location:** `bin/kineti-verify-gate.ts:20-27`
- **Code:**
  ```typescript
  if (cmd === "--trust") {
    if (!declared) die("no verify command declared (kineti.config.json settings.verify_command or KINETI_VERIFY_CMD)", 2);
    const t = readJson<Trust>(trustFile()) ?? {};
    t[repoKey()] = { cmd_hash: sha256(declared), at: nowIso() };
    writeJson(trustFile(), t);
    ok(`trusted for this repo: ${declared}`);
    return;
  }
  ```
- **Empirical Test:**
  In a fresh environment with `KINETI_VERIFY_CMD="echo EVIL_VERIFY_EXECUTED"`:
  1. Invoking `kineti-verify-gate` blocked with exit code 9: `kineti: verify-gate blocked. This repo has not trusted the current verify command.`
  2. Running `echo "" | bun bin/kineti-verify-gate.ts --trust` through piped stdin (simulating a non-interactive autonomous subagent) exited with status `0` and output: `trusted for this repo: echo EVIL_VERIFY_EXECUTED`.
  3. Running `bun bin/kineti-verify-gate.ts` executed the command via `bash -lc` and output: `EVIL_VERIFY_EXECUTED\nverify passed` with exit code `0`.
- **Result:** Fully reproduced. The verify gate contains no TTY verification, no human confirmation prompt, and no authentication check.

#### Observation 2 (Finding 3 / CRIT-04 — `readJsonl` Silent Data Loss)
- **Location:** `bin/lib.ts:48-58`
- **Code:**
  ```typescript
  export function readJsonl<T>(file: string): T[] {
    try {
      const text = fs.readFileSync(file, "utf8");
      return text
        .split("\n")
        .filter((l) => l.trim().length > 0)
        .map((l) => JSON.parse(l) as T);
    } catch {
      return [];
    }
  }
  ```
- **Empirical Test:**
  Created a JSONL file with 10 valid records and 1 corrupted line (`{ corrupted: true, `):
  1. `readJsonl(file)` returned `[]` (length `0`), discarding all 10 valid records.
  2. Tested downstream impact on `bin/kineti-egress.ts`: with 1 valid receipt followed by 1 corrupted line, `bun bin/kineti-egress.ts verify` printed `ledger intact: 0 receipts` with exit code `0`. It silently treated the ledger as empty, destroying hash continuity.
  3. Tested downstream impact on `bin/kineti-saga.ts`: with a valid `begin` line and 1 corrupt line, `bun bin/kineti-saga.ts rollback --run-id run-1` failed with exit code 2: `kineti: unknown run: run-1. Begin it first.`, making rollback impossible.
- **Result:** Fully reproduced. A single corrupt byte or line silently destroys data availability across memory journals, transaction sagas, and egress ledgers.

#### Observation 3 (Finding 2 / CRIT-05 — `scripts/weekly.sh` Missing Repo Pointer & Space Splitting)
- **Location:** `scripts/weekly.sh:10, 13, 16`
- **Code:**
  ```bash
  KIN="$(cat "$HOME/.kineti/repo")"
  ...
  PROJECTS="${KINETI_PROJECTS:-$PWD}"
  ...
  for p in $PROJECTS; do
    [[ -f "$p/.kineti/journal.jsonl" ]] || continue
  ```
- **Empirical Test:**
  1. Missing Pointer: Ran `bash scripts/weekly.sh` with a temporary empty `$HOME`. Under `set -euo pipefail`, `cat "$HOME/.kineti/repo"` failed with:
     `cat: .../.kineti/repo: No such file or directory`
     Script exited immediately with code `1`.
  2. Space-Splitting: Created a valid workspace directory containing spaces (`/tmp/kineti space test.XXXXXX`) with `.kineti/journal.jsonl`. Ran `KINETI_PROJECTS="$DIR" bash scripts/weekly.sh`.
     Trace showed the loop split the path into three tokens: `/tmp/kineti`, `space`, and `test.XXXXXX`. Each iteration evaluated `[[ -f "$p/.kineti/journal.jsonl" ]]` as false and hit `continue`. The weekly maintenance job skipped the project entirely.
- **Result:** Fully reproduced. Clean installs crash unhandled, and paths with spaces are silently bypassed.

#### Observation 4 (Finding 8 / HIGH-05 — Spend Circuit Breaker Bypass)
- **Location:** `bin/kineti-spend.ts:97-103`
- **Code:**
  ```typescript
  if (cmd === "reset") {
    if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
    s.tripped = false; s.reason = null;
    writeJson(file(), s);
    ok("breaker reset by human");
    return;
  }
  ```
- **Empirical Test:**
  Tripped the spend circuit breaker by writing `$55` spend to `spend.json` (`tripped: true`, `reason: "stage build: spent $55 >= limit $10"`):
  1. `bun bin/kineti-spend.ts status` exited with code `3` (blocked).
  2. Running `bun bin/kineti-spend.ts reset` without the flag failed with exit code `2`: `kineti: reset requires --i-am-human (breakers are human-only)`.
  3. Simulating an autonomous loop, piped non-interactive input to `bun bin/kineti-spend.ts reset --i-am-human`.
     Exited with code `0` and output `breaker reset by human`.
     Inspected `spend.json`: `tripped: false`, `reason: null`.
- **Result:** Fully reproduced. The `--i-am-human` check is static and easily satisfied programmatically by an LLM subagent without a human in the loop.

#### Observation 5 (Finding 7 / HIGH-07 — `kineti-memory-job.ts` Missing `--dir` Argument Crash)
- **Location:** `bin/kineti-memory-job.ts:74-75`
- **Code:**
  ```typescript
  const di = process.argv.indexOf("--dir");
  const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
  ```
- **Empirical Test:**
  Ran `bun bin/kineti-memory-job.ts sweep --dir`:
  - Process crashed with unhandled exception:
    ```
    TypeError: The "paths[0]" property must be of type string, got undefined
     code: "ERR_INVALID_ARG_TYPE"
          at main (/Users/praveen/Documents/Products/kineti local harness/bin/kineti-memory-job.ts:75:30)
    ```
  - Exit code was `1` (uncaught crash).
  - Also verified `verify-chain --dir` and `time-order --dir`; both crashed identically.
  - Furthermore, passing `--dir --flag` caused `ENOENT` crash on `save()` because `save()` does not ensure the parent directory exists.
- **Result:** Fully reproduced. Unhandled TypeError crash occurs when `--dir` is the final CLI argument.

---

### 2. Logic Chain

1. **Gate Security (CRIT-03 & HIGH-05):**
   - The harness design intends for human oversight to control risky actions: executing untested shell verification commands and resetting financial spend limits over $50.
   - However, both `kineti-verify-gate.ts` and `kineti-spend.ts` rely solely on command-line argument flags (`--trust` and `--i-am-human`).
   - Neither binary tests for an interactive terminal (`process.stdin.isTTY`) or requires an authenticated token.
   - Because autonomous subagents invoke shell commands directly via subprocesses, an agent can autonomously authorize its own verification commands or reset its own spend limits without operator intervention.
   - Therefore, the human-in-the-loop security boundary is effectively absent in autonomous runtime contexts.

2. **Data Integrity (CRIT-04):**
   - The JSONL storage format is used for append-only ledgers: memory journals, saga rollback transactions, and HTTP egress receipts.
   - In `readJsonl`, `JSON.parse` is executed across all lines inside a blanket `try...catch` block.
   - When any single line throws a `SyntaxError`, the catch block discards the entire array and returns `[]`.
   - Any calling script that performs a state sweep, rollback, or ledger verification receives an empty dataset.
   - For example, `kineti-memory-job.ts sweep` saves `kept` records back to disk. If `load` returns `[]` due to one corrupted line, the sweep writes an empty file, permanently erasing all past memory records.
   - Therefore, `readJsonl` poses a critical data loss risk.

3. **Shell Script Portability (CRIT-05):**
   - `scripts/weekly.sh` is designed to be executed via automated cron.
   - Cron environments typically lack interactive shells and have restricted environment variables.
   - Attempting `cat "$HOME/.kineti/repo"` without checking `[[ -f "$HOME/.kineti/repo" ]]` causes an unhandled non-zero exit under `set -e`, failing the cron job before any work is performed.
   - Expanding `$PROJECTS` without quoting inside a `for` loop causes word-splitting based on `$IFS` (space, tab, newline).
   - In modern development environments where paths commonly contain spaces (such as `/Users/praveen/Documents/Products/kineti local harness`), the script splits the path into fragments and fails to process any journals.
   - Therefore, the weekly cron script fails on clean installations and whitespace-containing paths.

4. **CLI Robustness (HIGH-07):**
   - In `kineti-memory-job.ts`, `di > -1` checks only whether `--dir` is present in `process.argv`.
   - It does not check if `di + 1 < process.argv.length` or whether `process.argv[di + 1]` is another flag.
   - Passing `undefined` to `path.resolve()` triggers Node/Bun's runtime type validator, crashing the process with an uncaught `TypeError`.
   - Therefore, CLI argument handling is brittle and requires index boundary validation.

---

### 3. Caveats

- **Test Isolation:** All empirical reproductions were executed using temporary directories in `os.tmpdir()` and isolated subshells. No repository files or live `.kineti/` state directories were modified.
- **Remediation Scope:** The remediation diffs proposed in `docs/AUDIT_REPORT.md` were tested in isolated scripts and confirmed to resolve each respective issue. However, per the non-destructive boundary mandate, these fixes have not yet been applied to the repository codebase.
- **TTY in Automated CI:** When implementing the `process.stdin.isTTY` check for `--trust` and `--i-am-human`, automated CI/CD pipelines must use an explicit override environment variable (e.g. `KINETI_TRUST_CONFIRMED=1` or `KINETI_HUMAN_RESET_TOKEN=...`) to avoid breaking non-interactive CI tests.

---

### 4. Conclusion

1. **Findings 4, 3, 2, 8, and 7 reported in `docs/AUDIT_REPORT.md` are 100% verified, accurate, and reproducible.**
2. The severity ratings in the audit report (Critical for Findings 4, 3, 2; High for Findings 8, 7) are appropriate given the potential for data destruction, arbitrary code execution bypass, and unhandled crashes.
3. The remediation diffs provided in `docs/AUDIT_REPORT.md` are technically sound, effective, and ready for implementation.

**Final Verdict:** **`APPROVE`**

---

### 5. Verification Method

To independently verify these findings, run the following commands in the repository root:

1. **Verify Finding 4 (Verify Gate Self-Trust Bypass):**
   ```bash
   bun -e '
   import { spawnSync } from "node:child_process";
   import fs from "node:fs"; import os from "node:os"; import path from "node:path";
   const m = fs.mkdtempSync(path.join(os.tmpdir(), "k-test-"));
   const p = path.resolve("bin/kineti-verify-gate.ts");
   const env = { ...process.env, KINETI_MACHINE_DIR: m, KINETI_VERIFY_CMD: "echo PWNED" };
   const t = spawnSync("bun", [p, "--trust"], { input: "", env, encoding: "utf8" });
   const r = spawnSync("bun", [p], { env, encoding: "utf8" });
   console.log("Trust exit:", t.status, "Run exit:", r.status, "Output:", r.stdout.trim());
   fs.rmSync(m, { recursive: true });
   '
   # Expected Output: Trust exit: 0 Run exit: 0 Output: PWNED\nverify passed
   ```

2. **Verify Finding 3 (`readJsonl` Silent Erasure):**
   ```bash
   bun -e '
   import { readJsonl } from "./bin/lib.ts";
   import fs from "node:fs"; import os from "node:os"; import path from "node:path";
   const f = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "k-test-")), "test.jsonl");
   fs.writeFileSync(f, "{\"valid\":1}\nCORRUPT_LINE\n{\"valid\":2}\n");
   console.log("Items parsed:", readJsonl(f).length);
   fs.rmSync(path.dirname(f), { recursive: true });
   '
   # Expected Output: Items parsed: 0
   ```

3. **Verify Finding 2 (`weekly.sh` Missing Pointer & Spaces):**
   ```bash
   # Test Missing Pointer Crash:
   HOME="$(mktemp -d)" bash scripts/weekly.sh
   # Expected Output: cat: .../.kineti/repo: No such file or directory (exit code 1)

   # Test Path with Spaces Word-Splitting:
   TMP_SPACE="$(mktemp -d "/tmp/kineti test space.XXXXXX")"
   mkdir -p "$TMP_SPACE/.kineti" && touch "$TMP_SPACE/.kineti/journal.jsonl"
   HOME="$TMP_SPACE" bash -c 'echo "$PWD" > "$TMP_SPACE/.kineti/repo"'
   HOME="$TMP_SPACE" KINETI_PROJECTS="$TMP_SPACE" bash scripts/weekly.sh
   # Expected Output: Project is skipped completely; does not run memory job commands
   ```

4. **Verify Finding 8 (Spend Breaker Reset Bypass):**
   ```bash
   bun -e '
   import { spawnSync } from "node:child_process";
   import fs from "node:fs"; import os from "node:os"; import path from "node:path";
   const d = fs.mkdtempSync(path.join(os.tmpdir(), "k-test-"));
   fs.mkdirSync(path.join(d, ".kineti"));
   fs.writeFileSync(path.join(d, ".kineti/spend.json"), JSON.stringify({ tripped: true, reason: "limit exceeded" }));
   const p = path.resolve("bin/kineti-spend.ts");
   const res = spawnSync("bun", [p, "reset", "--i-am-human"], { cwd: d, input: "", encoding: "utf8" });
   const state = JSON.parse(fs.readFileSync(path.join(d, ".kineti/spend.json"), "utf8"));
   console.log("Reset exit:", res.status, "Tripped after reset:", state.tripped);
   fs.rmSync(d, { recursive: true });
   '
   # Expected Output: Reset exit: 0 Tripped after reset: false
   ```

5. **Verify Finding 7 (`kineti-memory-job.ts` TypeError):**
   ```bash
   bun bin/kineti-memory-job.ts sweep --dir
   # Expected Output: TypeError: The "paths[0]" property must be of type string, got undefined
   ```
