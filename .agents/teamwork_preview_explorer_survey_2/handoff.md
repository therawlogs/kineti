# Handoff Report: Architecture, Configuration, and Documentation Consistency Audit

**Agent**: `explorer_survey_2`  
**Working Directory**: `/Users/praveen/Documents/Products/kineti local harness/.agents/teamwork_preview_explorer_survey_2`  
**Status**: COMPLETE (Hard Handoff)  
**Milestone**: M0 (Survey & Investigation)  

---

## 1. Executive Summary

This investigation conducted an exhaustive, non-destructive audit of the Kineti local harness repository covering configuration files (`kineti.config.json`, `package.json`, `tsconfig.json`), documentation (`ETHOS.md`, `WORKFLOWS.md`, `MEMORY.md`, `MIGRATION.md`, `ROADMAP.md`, `README.md`, `AGENTS.md`, `docs/`), harness binaries in `bin/`, installation scripts and hooks (`setup.sh`, `scripts/`, `hooks/`, `hosts/`), all 17 skills in `skills/`, and test suites in `tests/`.

A total of **19 issues** were identified across 5 severity categories:
- **Critical (2)**:
  1. `bin/kineti-state.ts` fails to look up gate states via `get gate.<name>`, crashing build pre-flight (`skills/build/SKILL.md:23`) and ship pre-flight (`skills/ship/SKILL.md:29-30`) with exit code 2.
  2. `tests/memory-job.test.ts` fails baseline execution (`bun test tests/`) due to hash calculation discrepancies (pipe delimiters vs no delimiters) and scope conflicts where non-run-record types are rejected by `verify-chain`.
- **High (3)**:
  3. `bin/kineti-spend.ts` constructs a double `.kineti` path creating nested `.kineti/.kineti/spend.log.jsonl`.
  4. `bin/kineti-evidence.ts` does not exclude `.agents/` or `design/screenshots/` from code fingerprinting, causing agent liveness updates and visual QA artifacts to falsely invalidate test proofs (`STALE`).
  5. `skills/spec/SKILL.md` attempts to set `gate.spec pending`, which is rejected by `bin/kineti-state.ts` validation (restricts values to `pass` or `fail`), crashing with exit code 2.
- **Medium (3)**:
  6. `bin/kineti-saga.ts` rollback rejects already-committed runs, breaking the `watch` stage recovery procedure.
  7. `ETHOS.md` Rule 8 ("Clean files") lacks any implementation script, git hook, or automated check.
  8. Missing `UX Blueprint` integration in `skills/officehours/SKILL.md` and `skills/design/SKILL.md`, violating `ETHOS.md` Rule 1.3 and `WORKFLOWS.md` Stage 1 deliverables.
- **Low (4)**:
  9. Mismatch in `kineti.config.json` record type naming (`project-dossier` vs `dossier`).
  10. Stale reference in `skills/officehours/SKILL.md` to non-existent `journal.md` instead of `journal.jsonl`.
  11. `hooks/claude.txt` lists only 14 skills, omitting `anchors`, `second-opinion`, and `skillify`.
  12. Unused `hook_file` directives in `hosts/*.conf` conflicting with `hooks/*.txt`.
- **Informational (7)**:
  13. `package.json` and `bin/README.md` document 6 programs, omitting `kineti-memory-job.ts` (7th binary).
  14. `ETHOS.md` Rule 1.3 references absolute filesystem path `/design` rather than `design/screens/`.
  15. `tests/test-setup.sh` is orphaned from the `package.json` test script.
  16. Unquoted `$K` in `docs/HOWTO-daily-loop.md` causes argument splitting on paths with spaces.
  17. Version desynchronization (`package.json` and `kineti.config.json` at 3.0.0, Git tag at v3.0.1, Roadmap post-3.0.1).
  18. Discrepancy on gate count (3 gates in docs vs 4 `gate: true` entries in `kineti.config.json`).
  19. Direct contradiction between `skills/design/SKILL.md` (12 UI components optional) and Kineti Master Directives (12 UI components mandatory).

---

## 2. Observations & Detailed Findings

### Finding 1 (Critical): Broken Gate Lookup in `bin/kineti-state.ts` Crashes Build and Ship Pre-Flights
- **File**: `bin/kineti-state.ts`, lines 70–77
- **Dependent Files**:
  - `skills/build/SKILL.md`, line 23:
    ```sh
    bun "$K/kineti-state.ts" get gate.spec | grep -q pass || { echo "spec not approved"; exit 2; }
    ```
  - `skills/ship/SKILL.md`, lines 29–30:
    ```sh
    bun "$K/kineti-state.ts" get gate.spec      # must be pass
    bun "$K/kineti-state.ts" get gate.security  # must be pass
    ```
- **Observed Code Snippet** (`bin/kineti-state.ts`):
  ```ts
  70:   if (cmd === "get") {
  71:     const key = rest[0];
  72:     if (!key) { console.log(JSON.stringify(s, null, 2)); return; }
  73:     const v = (s as any)[key];
  74:     if (v === undefined) die(`unknown key: ${key}`, 2);
  75:     console.log(typeof v === "string" ? v : JSON.stringify(v, null, 2));
  76:     return;
  77:   }
  ```
- **Verbatim Error Reproduction**:
  Running `bun bin/kineti-state.ts get gate.spec` after setting `gate.spec pass` produces:
  ```
  kineti: unknown key: gate.spec
  (exit code 2)
  ```
- **Root Cause**:
  `cmd === "set"` parses `gate.<name>` keys via `if (key.startsWith("gate.")) { const g = key.slice(5); s.gates[g] = value; }`.
  However, `cmd === "get"` does not have any `gate.` prefix handling. It performs `(s as any)[key]`, which evaluates to `undefined` because gates are stored in `s.gates[g]`.
- **Architectural Impact**:
  Stage 7 (`build`) and Stage 11 (`ship`) cannot execute because their pre-flight checks unconditionally fail with `unknown key: gate.spec`.
- **Concrete Remediation**:
  ```diff
  --- a/bin/kineti-state.ts
  +++ b/bin/kineti-state.ts
  @@ -71,7 +71,10 @@ function main() {
       if (cmd === "get") {
         const key = rest[0];
         if (!key) { console.log(JSON.stringify(s, null, 2)); return; }
  -      const v = (s as any)[key];
  +      let v = (s as any)[key];
  +      if (v === undefined && key.startsWith("gate.")) {
  +        v = s.gates[key.slice(5)];
  +      }
         if (v === undefined) die(`unknown key: ${key}`, 2);
         console.log(typeof v === "string" ? v : JSON.stringify(v, null, 2));
         return;
  ```
- **Verification Method**:
  Run: `bun bin/kineti-state.ts init --project test && bun bin/kineti-state.ts set gate.spec pass && bun bin/kineti-state.ts get gate.spec`
  Must output `pass` and exit with code 0.

---

### Finding 2 (Critical): `tests/memory-job.test.ts` Fails Baseline Test Suite
- **File**: `tests/memory-job.test.ts`, lines 43, 48, 54–56
- **Dependent Files**:
  - `bin/kineti-memory-job.ts`, lines 64–66, 97–120
  - `MEMORY.md`, lines 26–36, 70–75
- **Observed Command & Verbatim Error**:
  Running `bun test tests/` output:
  ```
  tests/memory-job.test.ts:
  56 |     expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
                                                              ^
  error: expect(received).toBe(expected)
  Expected: 0
  Received: 3
        at tests/memory-job.test.ts:56:56
  (fail) kineti-memory-job > sweep promotes expired actives; chain verifies then detects tamper (exit 3)
  ```
- **Root Cause**:
  1. In `tests/memory-job.test.ts` lines 43 and 48, the test computes hashes with pipe delimiters:
     `r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest("hex");`
     Whereas `bin/kineti-memory-job.ts` computes:
     `sha256(`${r.prev_hash}${r.at}${r.id}${canonStable(r.data, stable)}`)` without pipe delimiters, matching `MEMORY.md` line 74: `hash = sha256(prev_hash + at + id + canonical_json(data))`.
  2. `MEMORY.md` specifies that `run-record` items form the project hash chain. `learning` and `dossier` items do not have `prev_hash` or `hash`. Yet `bin/kineti-memory-job.ts` enforces `prev_hash`/`hash` on all records in `journal.jsonl`. Line 54 of the test includes `oldLearning` (which has no hash), causing `verify-chain` to fail with exit code 3 (`CHAIN BROKEN at lr-001: missing prev_hash/hash`).
- **Architectural Impact**:
  Default repository test command `bun test` fails immediately. Automated CI and developer verification are broken.
- **Concrete Remediation**:
  Update `bin/kineti-memory-job.ts` to scope `verify-chain` to records that participate in the hash chain (or records of type `run-record`), and align `tests/memory-job.test.ts` hash calculation.
  ```diff
  --- a/bin/kineti-memory-job.ts
  +++ b/bin/kineti-memory-job.ts
  @@ -99,7 +99,8 @@ function main() {
       // (pre-float-stable) hash acceptance for day<3 journals.
  -    const chain = load(dir);
  +    const chain = load(dir).filter((r) => r.type === "run-record");
       let prev = "GENESIS";
       for (const r of chain) {
  ```
  ```diff
  --- a/tests/memory-job.test.ts
  +++ b/tests/memory-job.test.ts
  @@ -42,3 +42,3 @@
       const crypto = require("node:crypto");
  -    r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest("hex");
  +    r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}${r1.at}${r1.id}${JSON.stringify(r1.data)}`).digest("hex");
  @@ -48,3 +48,3 @@
  -    r2.hash = crypto.createHash("sha256").update(`${r2.prev_hash}|${r2.at}|${r2.id}|${JSON.stringify(r2.data)}`).digest("hex");
  +    r2.hash = crypto.createHash("sha256").update(`${r2.prev_hash}${r2.at}${r2.id}${JSON.stringify(r2.data)}`).digest("hex");
  ```
- **Verification Method**:
  Run `bun test tests/memory-job.test.ts`; expect 2 passing tests.

---

### Finding 3 (High): Path Duplication Bug in `bin/kineti-spend.ts` Creates `.kineti/.kineti/`
- **File**: `bin/kineti-spend.ts`, line 29
- **Dependent Files**: `bin/lib.ts`, lines 10–12
- **Observed Code Snippet**:
  ```ts
  // bin/lib.ts:
  10: export function projectKdir(): string {
  11:   return path.join(process.cwd(), ".kineti");
  12: }

  // bin/kineti-spend.ts:
  28: function file(): string { return path.join(projectKdir(), "spend.json"); }
  29: function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }
  ```
- **Verbatim Error Reproduction**:
  Calling `bun bin/kineti-spend.ts log ...` creates the file at `.kineti/.kineti/spend.log.jsonl`.
- **Root Cause**:
  `projectKdir()` already includes `.kineti`. Adding `".kineti"` again creates a nested directory.
- **Architectural Impact**:
  Violates directory cleanliness and repository layout standards (`ETHOS.md` Rule 1.5, `kineti.config.json` `run_state: ".kineti/"`).
- **Concrete Remediation**:
  ```diff
  --- a/bin/kineti-spend.ts
  +++ b/bin/kineti-spend.ts
  @@ -29,1 +29,1 @@
  -function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }
  +function logFile(): string { return path.join(projectKdir(), "spend.log.jsonl"); }
  ```
- **Verification Method**:
  Run `bun bin/kineti-spend.ts log --stage build --tokens-in 1 --tokens-out 1`. Verify file is created at `.kineti/spend.log.jsonl` and `.kineti/.kineti/` is not created.

---

### Finding 4 (High): Missing `.agents` and `design/screenshots` in `bin/kineti-evidence.ts` Code Fingerprint Exclusion
- **File**: `bin/kineti-evidence.ts`, lines 11–15
- **Dependent Files**:
  - `skills/qa/SKILL.md`, line 30 (`design/screenshots/`)
  - `skills/ship/SKILL.md`, lines 31–32 (`kineti-evidence check --label qa`)
  - `.gitignore`, lines 1–7
- **Observed Code Snippet**:
  ```ts
  11: const EXCLUDE_DIRS = new Set([
  12:   ".git", ".kineti", "node_modules", "dist", "build", ".next",
  13:   "coverage", "tmp", ".cache", "legacy",
  14: ]);
  ```
- **Verbatim Reproduction**:
  ```
  Initial fingerprint: 9fe573c3...
  After creating .agents/progress.md: 1ab22a56...
  Did .agents change fingerprint? true
  ```
- **Root Cause**:
  `.agents` is not in `EXCLUDE_DIRS`. Every time an agent updates `progress.md` or outputs a report, `fingerprint()` changes.
  Likewise, `design/screenshots/` (generated during QA in `skills/qa/SKILL.md`) is not in `EXCLUDE_DIRS`.
- **Architectural Impact**:
  Stage 11 (`ship`) checks `bun bin/kineti-evidence.ts check --label qa`. Because the agent modified `.agents/` or QA generated screenshots, evidence returns `STALE (code changed after the run)` with exit code 4, completely blocking `ship`.
- **Concrete Remediation**:
  ```diff
  --- a/bin/kineti-evidence.ts
  +++ b/bin/kineti-evidence.ts
  @@ -11,4 +11,5 @@
   const EXCLUDE_DIRS = new Set([
  -  ".git", ".kineti", "node_modules", "dist", "build", ".next",
  -  "coverage", "tmp", ".cache", "legacy",
  +  ".git", ".kineti", ".agents", "node_modules", "dist", "build", ".next",
  +  "coverage", "tmp", ".cache", "legacy", "screenshots",
   ]);
  ```
  And add `.agents/` and `design/screenshots/` to `.gitignore`:
  ```diff
  --- a/.gitignore
  +++ b/.gitignore
  @@ -6,2 +6,4 @@
   .env
  +.agents/
  +design/screenshots/
  ```
- **Verification Method**:
  Run `bun bin/kineti-evidence.ts fingerprint`, touch `.agents/progress.md`, re-run `bun bin/kineti-evidence.ts fingerprint`. Hashes must match.

---

### Finding 5 (High): `skills/spec/SKILL.md` Directs Illegal State Transition `gate.spec pending`
- **File**: `skills/spec/SKILL.md`, line 49
- **Dependent Files**: `bin/kineti-state.ts`, lines 95–98
- **Observed Code Snippet**:
  In `skills/spec/SKILL.md`:
  ```sh
  49: bun "$K/kineti-state.ts" set gate.spec pending
  ```
  In `bin/kineti-state.ts`:
  ```ts
  95:     } else if (key.startsWith("gate.")) {
  96:       const g = key.slice(5);
  97:       if (value !== "pass" && value !== "fail") die("gate value must be pass|fail", 2);
  98:       s.gates[g] = value;
  ```
- **Verbatim Reproduction**:
  ```
  kineti: gate value must be pass|fail
  (exit code 2)
  ```
- **Root Cause**:
  `kineti-state.ts` strictly enforces that gate values must be `pass` or `fail`. `spec/SKILL.md` instructs running `set gate.spec pending`.
- **Architectural Impact**:
  Executing the spec skill causes an unexpected exit code 2 crash at step 7.
- **Concrete Remediation**:
  Support `pending` in `bin/kineti-state.ts`:
  ```diff
  --- a/bin/kineti-state.ts
  +++ b/bin/kineti-state.ts
  @@ -97,3 +97,3 @@
  -      if (value !== "pass" && value !== "fail") die("gate value must be pass|fail", 2);
  +      if (value !== "pass" && value !== "fail" && value !== "pending") die("gate value must be pass|fail|pending", 2);
  ```
- **Verification Method**:
  Run `bun bin/kineti-state.ts set gate.spec pending`; must exit with code 0.

---

### Finding 6 (Medium): `kineti-saga.ts` Rollback Forbids Committed Runs, Breaking `watch` Recovery
- **File**: `bin/kineti-saga.ts`, lines 15–20, 55–57
- **Dependent Files**: `skills/watch/SKILL.md`, lines 39–41; `docs/HOWTO-daily-loop.md`, line 29
- **Observed Code Snippet**:
  ```ts
  15: function openRun(runId: string): void {
  16:   const ls = lines();
  17:   const begin = ls.find((l) => l.kind === "begin" && l.run_id === runId);
  18:   if (!begin) die(`unknown run: ${runId}. Begin it first.`, 2);
  19:   if (ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
  20: }
  ```
- **Root Cause**:
  `openRun(runId)` is invoked on `rollback`. If the build completed and committed (`kineti-saga commit --run-id $RUN`), `openRun` dies because `run already committed`.
- **Architectural Impact**:
  `skills/watch/SKILL.md` line 39 states: "if red, propose rollback using the saga undo steps from the build run". However, the build run was already committed at stage 7 end, so `rollback` aborts with an error.
- **Concrete Remediation**:
  Allow `rollback` to execute on committed runs while preserving the prohibition on registering new steps to committed runs.
  ```diff
  --- a/bin/kineti-saga.ts
  +++ b/bin/kineti-saga.ts
  @@ -15,5 +15,5 @@
  -function openRun(runId: string): void {
  +function openRun(runId: string, allowCommitted = false): void {
     const ls = lines();
     const begin = ls.find((l) => l.kind === "begin" && l.run_id === runId);
     if (!begin) die(`unknown run: ${runId}. Begin it first.`, 2);
  -  if (ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
  +  if (!allowCommitted && ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
   }
  @@ -56,3 +56,3 @@
     if (cmd === "rollback") {
  -    openRun(runId);
  +    openRun(runId, true);
  ```
- **Verification Method**:
  Run `begin`, `register`, `commit`, then `rollback`. Verify rollback completes.

---

### Finding 7 (Medium): Zero Enforcement Tooling for ETHOS Rule 8 ("Clean Files")
- **File**: `ETHOS.md`, lines 44–48
- **Dependent Files**: `kineti.config.json`, `bin/`, `scripts/`
- **Observed Text**:
  ```markdown
  ## 8. Clean files
  8.1 Committed files contain no personal names, no home paths (use `$HOME`), no real client names, no secrets.
  8.2 Verified by search before every commit. Zero matches required.
  ```
- **Root Cause**:
  No script or hook in `scripts/`, `bin/`, or `hooks/` performs pattern searches or secret scans.
- **Architectural Impact**:
  `skills/anchors/SKILL.md` line 38 mandates: "Never write a rule no program or procedure enforces — mark it as convention explicitly instead." Rule 8 is currently standing law without an enforcement tool.
- **Concrete Remediation**:
  Add `scripts/clean-check.sh` that checks for `$HOME` violations, secrets, and hardcoded local usernames before git commit, and integrate it into `skills/ship/SKILL.md`.
- **Verification Method**:
  Verify the script flags hardcoded absolute home paths (e.g. `/Users/` or `/home/`).

---

### Finding 8 (Medium): Missing UX Blueprint Integration in Intake and Design Skills
- **File**: `skills/officehours/SKILL.md`, `skills/design/SKILL.md`
- **Dependent Files**: `ETHOS.md` (Rule 1.3), `WORKFLOWS.md` (lines 24, 26)
- **Observed Text**:
  `ETHOS.md` Rule 1.3 mandates:
  "UX-First Blueprint Before Tokens: Before selecting visual archetypes, tokens, or generating code, complete a detailed User Experience Blueprint... All HTML screen variants in design must strictly derive from this blueprint."
  `WORKFLOWS.md` line 24 specifies stage 1 produces:
  `brief.md + locked goal + UX Blueprint (Persona, Journey, Interaction Matrix, 3-Layer Split)`.
  However, `grep_search` across `skills/` for `blueprint` yielded 0 results.
- **Root Cause**:
  The mandate was added to `ETHOS.md` and `WORKFLOWS.md`, but `skills/officehours/SKILL.md` and `skills/design/SKILL.md` were never updated.
- **Architectural Impact**:
  Agents executing `officehours` or `design` skip creating or referencing the UX Blueprint because it is missing from the skill procedures.
- **Concrete Remediation**:
  Update `skills/officehours/SKILL.md` step 6 to include UX Blueprint sections in `brief.md`, and update `skills/design/SKILL.md` step 3 to reference the blueprint.
- **Verification Method**:
  Grep for `UX Blueprint` in `skills/officehours/SKILL.md` and verify procedure steps exist.

---

### Finding 9 (Low): Record Type Naming Mismatch in `kineti.config.json`
- **File**: `kineti.config.json`, line 40
- **Dependent Files**: `MEMORY.md`, lines 16, 33; `bin/kineti-memory-job.ts`, line 16
- **Observed Discrepancy**:
  `kineti.config.json`: `"record_types": ["run-record", "learning", "project-dossier"]`  
  `MEMORY.md`: `"type": "run-record | learning | dossier"`  
  `bin/kineti-memory-job.ts`: `type: "run-record" | "learning" | "dossier";`
- **Root Cause & Impact**:
  `project-dossier` vs `dossier`. Schema validators validating against `kineti.config.json` reject valid records written by `kineti-memory-job.ts`.
- **Concrete Remediation**:
  Change `"project-dossier"` to `"dossier"` in `kineti.config.json` line 40.
- **Verification Method**:
  Check equality of record type arrays across `kineti.config.json` and `MEMORY.md`.

---

### Finding 10 (Low): Stale Reference in `skills/officehours/SKILL.md` to Non-Existent `journal.md`
- **File**: `skills/officehours/SKILL.md`, line 32
- **Dependent Files**: `MEMORY.md` (line 6), `docs/HOWTO-daily-loop.md` (line 63)
- **Observed Discrepancy**:
  `skills/officehours/SKILL.md:32`: `otherwise read <project>/.kineti/journal.md if present.`
  The durable memory store is `.kineti/journal.jsonl`.
- **Root Cause & Impact**:
  Typo/stale markdown extension. Agents without gbrain fail to read fallback memory.
- **Concrete Remediation**:
  Replace `journal.md` with `journal.jsonl` in `skills/officehours/SKILL.md:32`.
- **Verification Method**:
  Grep for `journal.md` across repository; verify 0 matches.

---

### Finding 11 (Low): Incomplete Skill Listing in `hooks/claude.txt`
- **File**: `hooks/claude.txt`, lines 4–5
- **Dependent Files**: `skills/` (17 skills), `README.md` (lines 19–24)
- **Observed Discrepancy**:
  Lists 14 skills: `officehours, diagnose, design, architecture, feasibility, spec, build, review, qa, security, ship, watch, retro, learn.`  
  Missing: `anchors`, `second-opinion`, `skillify`.
- **Root Cause & Impact**:
  Claude Code host instruction hook omits 3 utility skills installed by `setup.sh`.
- **Concrete Remediation**:
  Append `anchors, second-opinion, skillify` to `hooks/claude.txt` line 5.
- **Verification Method**:
  Count skills listed in `hooks/claude.txt`; must equal 17.

---

### Finding 12 (Low): Dead and Conflicting `hook_file` Directives in `hosts/*.conf`
- **File**: `hosts/claude.conf`, `hosts/codex.conf`, `hosts/gemini.conf`, `hosts/opencode.conf`, line 5
- **Dependent Files**: `setup.sh`, `hooks/*.txt`
- **Observed Discrepancy**:
  `hosts/claude.conf`: `hook_file="$HOME/.claude/settings.json"` (conflicts with `hooks/claude.txt`: `CLAUDE.md`).  
  `hosts/codex.conf`: `hook_file=".../config.toml"` (conflicts with `hooks/codex.txt`: `AGENTS.md`).  
  `hosts/gemini.conf`: `hook_file=".../settings.json"` (conflicts with `hooks/gemini.txt`: `GEMINI.md`).  
  `setup.sh` never references `hook_file`.
- **Root Cause & Impact**:
  Dead configuration directive causing confusion.
- **Concrete Remediation**:
  Update `hosts/*.conf` to reference actual target instruction files and document manual paste instructions in `setup.sh`.
- **Verification Method**:
  Verify consistency between `hosts/*.conf` and `hooks/*.txt`.

---

### Finding 13 (Informational): Program Count Mismatch in `package.json` and `bin/README.md`
- **File**: `package.json` (line 5), `bin/README.md` (lines 3, 6–12)
- **Dependent Files**: `ROADMAP.md` (line 32), `bin/`
- **Observed Discrepancy**:
  `package.json` and `bin/README.md` say "six enforcement programs" and omit `kineti-memory-job.ts`.
  `ROADMAP.md` correctly says "7 harness programs".
- **Concrete Remediation**:
  Add `kineti-memory-job` row to `bin/README.md` and update `package.json` description to "seven harness programs".
- **Verification Method**:
  Check program count in `bin/README.md` matches `ls bin/kineti-*.ts | wc -l`.

---

### Finding 14 (Informational): Absolute Path `/design` in `ETHOS.md`
- **File**: `ETHOS.md`, line 9
- **Dependent Files**: `kineti.config.json` (line 43), `skills/design/SKILL.md` (line 40)
- **Observed Text**:
  `All HTML screen variants in /design must strictly derive from this blueprint.`
- **Concrete Remediation**:
  Change `/design` to relative path `design/screens/`.
- **Verification Method**:
  Inspect `ETHOS.md` line 9.

---

### Finding 15 (Informational): `tests/test-setup.sh` Omitted from `package.json` Test Script
- **File**: `package.json`, line 8
- **Dependent Files**: `tests/test-setup.sh`
- **Observed Code**:
  `"test": "bun test tests/"`
  `bun test` ignores `.sh` scripts.
- **Concrete Remediation**:
  Update `package.json` to `"test": "bun test tests/ && bash tests/test-setup.sh"`.
- **Verification Method**:
  Run `bun test` and observe `PASS: installer smoke test (5 checks)`.

---

### Finding 16 (Informational): Unquoted `$K` in `docs/HOWTO-daily-loop.md`
- **File**: `docs/HOWTO-daily-loop.md`, lines 37–38
- **Observed Code**:
  `bun $K/kineti-evidence.ts run --label qa -- -- bun test`
- **Concrete Remediation**:
  Quote `"$K/kineti-evidence.ts"`.
- **Verification Method**:
  Verify quotes in `docs/HOWTO-daily-loop.md`.

---

### Finding 17 (Informational): Repository Version Desynchronization (v3.0.0 vs v3.0.1)
- **File**: `package.json` (line 3), `kineti.config.json` (line 2)
- **Dependent Files**: `git tag`, `ROADMAP.md` (line 1)
- **Observed Text**:
  `package.json` and `kineti.config.json` specify `"version": "3.0.0"`.
  Git tag `v3.0.1` exists; `ROADMAP.md` says `# ROADMAP.md — What Comes After v3.0.1`.
- **Concrete Remediation**:
  Bump version in `package.json` and `kineti.config.json` to `"3.0.1"`.
- **Verification Method**:
  Compare version strings against latest git tag.

---

### Finding 18 (Informational): Gate Count Ambiguity (3 Gates vs 4 Gates)
- **File**: `kineti.config.json`, lines 15–16
- **Dependent Files**: `ETHOS.md` (section 10), `WORKFLOWS.md` (line 3), `README.md` (line 38), `skills/security/SKILL.md` (line 3)
- **Observed Discrepancy**:
  Docs state "Three gates" (Feasibility, Spec, Ship).
  `kineti.config.json` marks both Stage 10 (`security`) and Stage 11 (`ship`) as `"gate": true`.
- **Concrete Remediation**:
  Clarify in `WORKFLOWS.md` and `kineti.config.json` whether Security is an independent gate or a hard constraint of the Ship gate.
- **Verification Method**:
  Review gate definitions across config and documentation.

---

### Finding 19 (Informational): Philosophical Conflict on 12 UI Components
- **File**: `skills/design/SKILL.md`, lines 65–66
- **Dependent Files**: Kineti OS Global Directives / ETHOS.md
- **Observed Discrepancy**:
  `skills/design/SKILL.md`: `The 12-component UI library is fallback vocabulary for when nothing better fits — never a mandatory checklist.`  
  Global Directives: `Every screen must be assembled exclusively from these 12 components... Never a deviation.`
- **Concrete Remediation**:
  Harmonize guidance: taste-driven references determine visual styling and design tokens; production layout assembly uses the 12 primitives.
- **Verification Method**:
  Review `skills/design/SKILL.md` against Master Directives.

---

## 3. Logic Chain

1. **Gate Evaluation Logic**:
   - Observation: `skills/build/SKILL.md:23` runs `bun "$K/kineti-state.ts" get gate.spec`.
   - Observation: `bin/kineti-state.ts:73` evaluates `(s as any)["gate.spec"]`.
   - Observation: `s.gates` is a nested map `{ spec: "pass" }`. Top-level key `"gate.spec"` is `undefined`.
   - Deduction: `kineti-state` exits with code 2 on every pre-flight gate query, deadlocking the pipeline at stage 7 and stage 11.

2. **Evidence Stale Invalidation Logic**:
   - Observation: `bin/kineti-evidence.ts:20` computes `fingerprint()` by hashing all non-excluded files.
   - Observation: `bin/kineti-evidence.ts:11` does not include `.agents` or `design/screenshots` in `EXCLUDE_DIRS`.
   - Observation: Agent orchestration frameworks write heartbeats and handoffs to `.agents/`. QA saves visual snapshots to `design/screenshots/`.
   - Deduction: Code fingerprint changes after running tests, causing `kineti-evidence check` to unconditionally report `STALE (code changed after the run)`, blocking `ship`.

3. **Memory Job Test Failure Logic**:
   - Observation: `tests/memory-job.test.ts` fails with exit code 3 on `verify-chain`.
   - Observation: The test creates an item of type `learning` with no hash and invokes `verify-chain`.
   - Observation: `bin/kineti-memory-job.ts:98-106` requires `prev_hash` and `hash` on every record in `journal.jsonl`.
   - Observation: `MEMORY.md:72` specifies that only `run-record` items participate in the hash chain.
   - Deduction: The test failure reveals a direct contract conflict between the documentation (`MEMORY.md`), the implementation (`kineti-memory-job.ts`), and the test suite (`memory-job.test.ts`).

---

## 4. Caveats

1. **Read-Only Non-Destructive Mode**: No repository source files were edited. All remediation proposals are provided as diffs.
2. **External Host Environments**: The inspection of `~/.claude/`, `~/.codex/`, and `~/.gemini/` directories outside the repository workspace was restricted by system protection boundaries; audit conclusions rely on repository source files (`hosts/*.conf`, `hooks/*.txt`, `setup.sh`).
3. **Semantic Search Provider Sunset**: The sunset of Voyage-4 embedding provider documented in `ROADMAP.md` (2026-09-04) has passed; keyword recall operates as designed.

---

## 5. Conclusion & Prioritized Action Roadmap

The Kineti local harness exhibits strong conceptual design and clean modularity. However, critical gaps between CLI binary implementations, test suites, and documentation currently block full automated pipeline execution.

### Immediate Hardening Priorities (P0):
1. Apply Finding 1 fix to `bin/kineti-state.ts` to restore `get gate.<name>` functionality.
2. Apply Finding 2 fix to `bin/kineti-memory-job.ts` and `tests/memory-job.test.ts` to bring `bun test` to 100% pass.
3. Apply Finding 3 fix to `bin/kineti-spend.ts` to eliminate duplicate `.kineti/.kineti/` path creation.
4. Apply Finding 4 fix to `bin/kineti-evidence.ts` and `.gitignore` to prevent `.agents` and `design/screenshots/` from invalidating proof records.
5. Apply Finding 5 fix to `bin/kineti-state.ts` to support `pending` gate status.

### Pipeline & Documentation Realignment (P1):
6. Apply Finding 6 fix to `bin/kineti-saga.ts` to allow rollback of committed runs.
7. Implement Finding 7 (`scripts/clean-check.sh`) to fulfill ETHOS Rule 8.
8. Align `skills/officehours/SKILL.md` and `skills/design/SKILL.md` with ETHOS Rule 1.3 UX Blueprint requirements (Finding 8).
9. Align record types in `kineti.config.json` (`dossier`) and fix `journal.md` link in `officehours/SKILL.md` (Findings 9 & 10).

---

## 6. Verification Method

To independently verify all findings:

```sh
# 1. Typecheck baseline
bun run typecheck

# 2. Test suite baseline (reveals Finding 2 failure)
bun test tests/

# 3. Setup installer smoke test
bash tests/test-setup.sh

# 4. Gate status get bug (Finding 1)
bun -e '
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
const dir = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-verify-"));
fs.mkdirSync(path.join(dir, ".kineti"));
const p = path.resolve("bin/kineti-state.ts");
spawnSync("bun", [p, "init", "--project", "test"], { cwd: dir });
spawnSync("bun", [p, "set", "gate.spec", "pass"], { cwd: dir });
const res = spawnSync("bun", [p, "get", "gate.spec"], { cwd: dir, encoding: "utf8" });
console.log("Gate get status:", res.status, "Error:", res.stderr.trim());
fs.rmSync(dir, { recursive: true });
'

# 5. Spend double path bug (Finding 3)
bun -e '
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
const dir = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-verify-"));
fs.mkdirSync(path.join(dir, ".kineti"));
const p = path.resolve("bin/kineti-spend.ts");
spawnSync("bun", [p, "log", "--stage", "build", "--tokens-in", "10", "--tokens-out", "10"], { cwd: dir });
console.log("Nested path exists:", fs.existsSync(path.join(dir, ".kineti", ".kineti")));
fs.rmSync(dir, { recursive: true });
'

# 6. Fingerprint invalidation by .agents (Finding 4)
bun -e '
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
const dir = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-verify-"));
fs.mkdirSync(path.join(dir, "src"));
fs.writeFileSync(path.join(dir, "src", "a.ts"), "export const a = 1;");
const p = path.resolve("bin/kineti-evidence.ts");
const fp1 = spawnSync("bun", [p, "fingerprint"], { cwd: dir, encoding: "utf8" }).stdout.trim();
fs.mkdirSync(path.join(dir, ".agents"));
fs.writeFileSync(path.join(dir, ".agents", "progress.md"), "test");
const fp2 = spawnSync("bun", [p, "fingerprint"], { cwd: dir, encoding: "utf8" }).stdout.trim();
console.log("Fingerprint changed by .agents:", fp1 !== fp2);
fs.rmSync(dir, { recursive: true });
'
```
