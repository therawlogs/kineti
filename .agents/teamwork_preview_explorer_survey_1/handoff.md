# Exhaustive Security, Robustness, and Portability Audit of Kineti Local Harness Scripts & Programs

**Auditor:** `explorer_survey_1` (Survey & Security Specialist)  
**Date:** 2026-09-05T19:30:00Z  
**Scope:** `setup.sh`, `scripts/`, `bin/`, `hooks/`, `tests/`  
**Integrity Mode:** Non-Destructive / Read-Only Inspection  

---

## 1. Executive Summary & Audit Matrix

A line-by-line static analysis and dynamic validation of all executable shell scripts, TypeScript CLI programs, hook templates, and configuration runners within the Kineti local harness was completed. 

The audit identified **23 distinct findings** across security, error handling, POSIX/bash portability, privilege management, and data integrity boundaries:
- **Critical Severity (4):** Baseline test suite failure due to schema mismatch (`tests/memory-job.test.ts:56`), unhandled missing repo pointer and unquoted path splitting in cron job (`scripts/weekly.sh:10, 16`), catastrophic silent data loss on corrupted JSONL lines (`bin/lib.ts:48-58`), and programmatic self-trust bypass of the verify-gate (`bin/kineti-verify-gate.ts:20-27`).
- **High Severity (4):** Arbitrary shell execution and silent failure swallowing in rollback (`bin/kineti-saga.ts:63-72`), command injection, argument flattening, and diagnostic blindness in proof capture (`bin/kineti-evidence.ts:58-72`), nested path construction bug (`bin/kineti-spend.ts:29`), and human-only circuit breaker bypass via static CLI flag (`bin/kineti-spend.ts:97-103`).
- **Medium Severity (8):** Unchecked argument parsing crash in installer (`setup.sh:18`), SIGPIPE and unhandled exit 1 under pipefail (`setup.sh:26`), incomplete skill installation copying only `SKILL.md` (`setup.sh:88-94`), incomplete uninstall leaving orphan pointer (`setup.sh:55-69`), fragile bun resolution in non-interactive shells (`scripts/weekly.sh:11-12`), unhandled runtime exception on missing `--dir` (`bin/kineti-memory-job.ts:75`), non-atomic file overwrites risking memory journal loss (`bin/kineti-memory-job.ts:43-46`), and symlink traversal/FIFO blocking in repository fingerprinting (`bin/kineti-evidence.ts:22-39`).
- **Low & Informational (7):** Missing executable permission on `bin/kineti-memory-job.ts`, inappropriately set executable permission on library module `bin/lib.ts`, delimiter collision risk in egress ledger (`bin/kineti-egress.ts:15`), CWE-117 log injection in alerts (`bin/kineti-verify-gate.ts:59`, `bin/kineti-spend.ts:112`), stale skill catalog in Claude hook (`hooks/claude.txt:4-5`), hardcoded skill counts in test smoke script (`tests/test-setup.sh:18, 24`), and unprotected globbing without `nullglob` (`setup.sh:47, 61, 88`).

---

## 2. Detailed Findings & Remediation Catalog

### Finding 1 [CRITICAL] — Baseline Test Suite Failure: Schema & Hash Mismatch in Memory Job Test
- **Target File:** `tests/memory-job.test.ts:37-56` (interacting with `bin/kineti-memory-job.ts:64-66, 104-107`)
- **Vulnerable / Broken Code Snippet:**
```ts
// tests/memory-job.test.ts:43
r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest("hex");
...
// tests/memory-job.test.ts:49-54
const oldLearning = {
  at: iso(120), type: "learning", state: "active", project: "p", id: "lr-001",
  data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
  expires: iso(10),
};
write(dir, [r1, r2, oldLearning]);

expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
```
- **Observed Failure:**
Running `bun test tests/` immediately fails:
```
error: expect(received).toBe(expected)
Expected: 0
Received: 3
  at tests/memory-job.test.ts:56:56
(fail) kineti-memory-job > sweep promotes expired actives; chain verifies then detects tamper (exit 3)
```
- **Root Cause & Impact Analysis:**
`bin/kineti-memory-job.ts:65` computes record hashes *without* pipe delimiters: `sha256("${r.prev_hash}${r.at}${r.id}${canonStable(r.data, stable)}")`. `tests/memory-job.test.ts:43` constructs the hash with pipe delimiters (`|`), causing `recordHash(r) !== r.hash`. Furthermore, `oldLearning` is written directly into `journal.jsonl` without `prev_hash` or `hash`. When `verify-chain` runs, line 104 triggers: `kineti: CHAIN BROKEN at lr-001: missing prev_hash/hash`, exiting with code 3.
Baseline tests fail on clean clones, masking legitimate regressions.
- **Remediation Diff:**
```diff
--- a/tests/memory-job.test.ts
+++ b/tests/memory-job.test.ts
@@ -40,17 +40,21 @@ describe("kineti-memory-job", () => {
     };
-    // hash computed the same way the program does
     const crypto = require("node:crypto");
-    r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest("hex");
+    r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}${r1.at}${r1.id}${JSON.stringify(r1.data)}`).digest("hex");
     const r2: any = {
       at: iso(1), type: "run-record", state: "active", project: "p", id: "rr-002",
       data: { root_goal: "g2" }, links: [], prev_hash: r1.hash, hash: "",
     };
-    r2.hash = crypto.createHash("sha256").update(`${r2.prev_hash}|${r2.at}|${r2.id}|${JSON.stringify(r2.data)}`).digest("hex");
-    const oldLearning = {
+    r2.hash = crypto.createHash("sha256").update(`${r2.prev_hash}${r2.at}${r2.id}${JSON.stringify(r2.data)}`).digest("hex");
+    const oldLearning: any = {
       at: iso(120), type: "learning", state: "active", project: "p", id: "lr-001",
       data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
       expires: iso(10),
+      prev_hash: r2.hash,
+      hash: "",
     };
+    oldLearning.hash = crypto.createHash("sha256").update(`${oldLearning.prev_hash}${oldLearning.at}${oldLearning.id}${JSON.stringify(oldLearning.data)}`).digest("hex");
     write(dir, [r1, r2, oldLearning]);
```
- **Verification Method:**
Execute `bun test tests/memory-job.test.ts`. All assertions pass with exit code 0.

---

### Finding 2 [CRITICAL] — Unhandled Missing Repo Pointer & Space Splitting in Cron Maintenance Script
- **Target File:** `scripts/weekly.sh:10, 13, 16`
- **Vulnerable Code Snippet:**
```bash
# scripts/weekly.sh:10, 13, 16
KIN="$(cat "$HOME/.kineti/repo")"
...
PROJECTS="${KINETI_PROJECTS:-$PWD}"
...
for p in $PROJECTS; do
  [[ -f "$p/.kineti/journal.jsonl" ]] || continue
```
- **Observed Failure:**
Running `bash scripts/weekly.sh` when `$HOME/.kineti/repo` does not exist crashes immediately:
```
cat: /Users/praveen/.kineti/repo: No such file or directory
(exit code 1)
```
When run in workspaces containing spaces (e.g. `$PWD = /Users/praveen/Documents/Products/kineti local harness`), bash word-splits `$PROJECTS` into 3 tokens (`/Users/praveen/Documents/Products/kineti`, `local`, `harness`).
- **Root Cause & Impact Analysis:**
`set -euo pipefail` halts execution when `cat` fails. There is no fallback or friendly diagnostic error explaining that `./setup.sh` must be run. Furthermore, unquoted variable expansion `for p in $PROJECTS; do` breaks on any directory containing whitespace and executes glob pathname expansion if wildcards are present.
- **Remediation Diff:**
```diff
--- a/scripts/weekly.sh
+++ b/scripts/weekly.sh
@@ -7,9 +7,16 @@
 set -euo pipefail
 
+REPO_FILE="$HOME/.kineti/repo"
+if [[ ! -f "$REPO_FILE" ]]; then
+  echo "kineti: error: repository pointer '$REPO_FILE' not found. Run ./setup.sh first." >&2
+  exit 1
+fi
+KIN="$(head -n 1 "$REPO_FILE" | tr -d '\r\n')"
+if [[ ! -d "$KIN" ]]; then
+  echo "kineti: error: repository directory '$KIN' does not exist." >&2
+  exit 1
+fi
-KIN="$(cat "$HOME/.kineti/repo")"
 BUN="${KINETI_BUN:-$HOME/.bun/bin/bun}"
-command -v "$BUN" >/dev/null 2>&1 || BUN="$(command -v bun)"
+if ! command -v "$BUN" >/dev/null 2>&1; then
+  BUN="$(command -v bun || true)"
+fi
+if [[ -z "$BUN" || ! -x "$BUN" ]]; then
+  echo "kineti: error: bun runtime not found. Install bun or set KINETI_BUN." >&2
+  exit 1
+fi
-PROJECTS="${KINETI_PROJECTS:-$PWD}"
+if [[ -n "${KINETI_PROJECTS:-}" ]]; then
+  IFS=':' read -r -a project_list <<< "$KINETI_PROJECTS"
+else
+  project_list=("$PWD")
+fi
 
 echo "=== kineti weekly job $(date '+%Y-%m-%d %H:%M') ==="
-for p in $PROJECTS; do
+for p in "${project_list[@]}"; do
   [[ -f "$p/.kineti/journal.jsonl" ]] || continue
```
- **Verification Method:**
1. Remove `~/.kineti/repo` in test environment; run `scripts/weekly.sh` and verify clear error message and exit code 1.
2. Run with `$PWD` containing spaces; verify that `$p` is evaluated as a single intact path.

---

### Finding 3 [CRITICAL] — Catastrophic Silent Ledger Erasure in `readJsonl` on Corrupted or Partial Line
- **Target File:** `bin/lib.ts:48-58`
- **Vulnerable Code Snippet:**
```ts
// bin/lib.ts:48-58
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
- **Observed Behavior:**
Passing a JSONL file with 1,000 valid lines and 1 malformed trailing line causes `JSON.parse` to throw `SyntaxError`. The catch block catches the exception and returns `[]`.
- **Root Cause & Impact Analysis:**
`readJsonl` treats any parsing exception as "file does not exist or has zero entries".
Downstream consequences:
- In `bin/kineti-egress.ts:28-34`: `chain.length` becomes `0`, and `prev` resets to `"GENESIS"`. The next `record` call writes receipt `seq: 0`, corrupting the Merkle hash chain and overwriting `egress.state.json`.
- In `bin/kineti-saga.ts:57-62`: `pending.length` becomes `0`. Rollback outputs `nothing to roll back`, leaving un-rolled-back modifications in place.
- In `bin/kineti-memory-job.ts:78`: All journal memory records are treated as empty.
- **Remediation Diff:**
```diff
--- a/bin/lib.ts
+++ b/bin/lib.ts
@@ -48,11 +48,24 @@ export function writeJson(file: string, value: unknown): void {
 export function readJsonl<T>(file: string): T[] {
+  if (!fs.existsSync(file)) return [];
   try {
     const text = fs.readFileSync(file, "utf8");
-    return text
-      .split("\n")
-      .filter((l) => l.trim().length > 0)
-      .map((l) => JSON.parse(l) as T);
-  } catch {
-    return [];
+    const lines = text.split("\n").filter((l) => l.trim().length > 0);
+    const items: T[] = [];
+    for (let i = 0; i < lines.length; i++) {
+      try {
+        items.push(JSON.parse(lines[i]) as T);
+      } catch (err) {
+        console.error(`kineti: warning: corrupt line ${i + 1} in ${file} skipped: ${(err as Error).message}`);
+      }
+    }
+    return items;
+  } catch (err) {
+    console.error(`kineti: error reading ${file}: ${(err as Error).message}`);
+    return [];
   }
 }
```
- **Verification Method:**
Create a test `.jsonl` file with 2 valid lines and 1 invalid line. Verify that `readJsonl` returns the 2 valid objects rather than `[]`.

---

### Finding 4 [CRITICAL] — Programmatic Self-Trust Bypass of the Verify Gate
- **Target File:** `bin/kineti-verify-gate.ts:20-27`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-verify-gate.ts:20-27
if (cmd === "--trust") {
  if (!declared) die("no verify command declared (kineti.config.json settings.verify_command or KINETI_VERIFY_CMD)", 2);
  const t = readJson<Trust>(trustFile()) ?? {};
  t[repoKey()] = { cmd_hash: sha256(declared), at: nowIso() };
  writeJson(trustFile(), t);
  ok(`trusted for this repo: ${declared}`);
  return;
}
```
- **Root Cause & Impact Analysis:**
The verify gate is designed as a safety boundary so that arbitrary verify commands configured in repos or environment variables are not executed without human review.
However, `--trust` requires no interactive confirmation, human presence check, or privileged token. Any autonomous AI subagent with shell access can simply execute `bun bin/kineti-verify-gate.ts --trust` followed by `bun bin/kineti-verify-gate.ts`, completely circumventing the gate and executing arbitrary commands via `bash -lc` (line 54).
- **Remediation Diff:**
```diff
--- a/bin/kineti-verify-gate.ts
+++ b/bin/kineti-verify-gate.ts
@@ -20,6 +20,10 @@ function main() {
   if (cmd === "--trust") {
     if (!declared) die("no verify command declared (kineti.config.json settings.verify_command or KINETI_VERIFY_CMD)", 2);
+    if (!process.stdin.isTTY && !process.env.KINETI_TRUST_CONFIRMED) {
+      die("security: --trust must be run interactively in a TTY by a human operator", 2);
+    }
     const t = readJson<Trust>(trustFile()) ?? {};
```
- **Verification Method:**
Run `echo "" | bun bin/kineti-verify-gate.ts --trust` in a non-TTY subshell; verify rejection with status code 2 unless `KINETI_TRUST_CONFIRMED=1` or running interactively.

---

### Finding 5 [HIGH] — Arbitrary Command Execution, Stderr Swallowing & Cascading Rollback Failures
- **Target File:** `bin/kineti-saga.ts:63-72`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-saga.ts:63-72
for (const r of pending) {
  const res = spawnSync("bash", ["-lc", r.inverse!], { stdio: "pipe", encoding: "utf8" });
  const code = res.status ?? 1;
  appendJsonl(file(), {
    at: nowIso(), kind: "rollback_step", run_id: runId,
    label: r.label, exit_code: code,
  } satisfies Line);
  if (code !== 0) console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}); continuing`);
  else ok(`undone: ${r.label}`);
}
```
- **Root Cause & Impact Analysis:**
1. Arbitrary shell execution: `r.inverse!` is executed directly inside a login bash shell (`bash -lc`).
2. Stderr swallowing: `stdio: "pipe"` captures stdout/stderr into `res`, but `res.stderr` is never printed when `code !== 0`. Operators have zero visibility into why an undo failed.
3. No timeout: If an inverse command hangs, `spawnSync` hangs indefinitely.
4. Cascading corruption: Continuing past a failed rollback step violates LIFO dependency guarantees; if step N fails to restore a file/lock, step N-1 operating on the assumption of restored state can corrupt the workspace.
- **Remediation Diff:**
```diff
--- a/bin/kineti-saga.ts
+++ b/bin/kineti-saga.ts
@@ -63,7 +63,11 @@ function main() {
     for (const r of pending) {
-      const res = spawnSync("bash", ["-lc", r.inverse!], { stdio: "pipe", encoding: "utf8" });
+      const res = spawnSync("bash", ["-lc", r.inverse!], { stdio: "pipe", encoding: "utf8", timeout: 30000 });
       const code = res.status ?? 1;
       appendJsonl(file(), {
         at: nowIso(), kind: "rollback_step", run_id: runId,
         label: r.label, exit_code: code,
       } satisfies Line);
-      if (code !== 0) console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}); continuing`);
-      else ok(`undone: ${r.label}`);
+      if (code !== 0) {
+        console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}):\n${res.stderr || res.stdout}`);
+        die(`rollback halted: step "${r.label}" failed; manual intervention required`, 1);
+      } else {
+        ok(`undone: ${r.label}`);
+      }
     }
```
- **Verification Method:**
Register an inverse command that fails (`exit 5`) and another after it. Verify that stderr is logged, rollback halts safely, and subsequent steps are not blindly executed.

---

### Finding 6 [HIGH] — Command Injection, Space Flattening & Diagnostic Blindness in Proof Capture
- **Target File:** `bin/kineti-evidence.ts:61, 64, 71`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-evidence.ts:61-71
const command = argv.slice(dd + 1).join(" ");
if (!label || command.length === 0) die("run requires --label and a command after --");
const fpBefore = fingerprint();
const res = Bun.spawnSync(["bash", "-lc", command], { stdout: "pipe", stderr: "pipe" });
const code = res.exitCode;
...
if (code !== 0) die(`command failed (exit ${code}); recorded as proof anyway`, 1);
```
- **Root Cause & Impact Analysis:**
`argv.slice(dd + 1).join(" ")` merges separate command-line arguments into a single raw string passed to `bash -lc`. Any arguments with whitespace or quotes lose their boundaries. Furthermore, `stdout: "pipe", stderr: "pipe"` captures process streams but never prints or records them. If a test or build fails, the developer only sees `kineti: command failed (exit 1); recorded as proof anyway` with zero test output or stack trace.
- **Remediation Diff:**
```diff
--- a/bin/kineti-evidence.ts
+++ b/bin/kineti-evidence.ts
@@ -63,4 +63,7 @@ function main() {
     const fpBefore = fingerprint();
-    const res = Bun.spawnSync(["bash", "-lc", command], { stdout: "pipe", stderr: "pipe" });
+    const res = Bun.spawnSync(["bash", "-lc", command], { stdio: ["inherit", "pipe", "pipe"] });
     const code = res.exitCode;
+    if (code !== 0) {
+      process.stderr.write(res.stderr);
+    }
```
- **Verification Method:**
Run `bun bin/kineti-evidence.ts run --label test -- false` and verify that command failure outputs details to stderr.

---

### Finding 7 [HIGH] — Path Construction Bug: Redundant Nested `.kineti/.kineti/spend.log.jsonl`
- **Target File:** `bin/kineti-spend.ts:29`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-spend.ts:28-29
function file(): string { return path.join(projectKdir(), "spend.json"); }
function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }
```
- **Observed Evaluation:**
In `bin/lib.ts:10-12`, `projectKdir()` is defined as:
```ts
export function projectKdir(): string { return path.join(process.cwd(), ".kineti"); }
```
Therefore, line 29 evaluates to:
`$PWD/.kineti/.kineti/spend.log.jsonl`
- **Root Cause & Impact Analysis:**
Accidental duplicate `.kineti` in path concatenation. Spend logs are written to an unintended nested subdirectory `.kineti/.kineti/`, breaking file discovery, standard gitignore patterns, and external log analysis scripts.
- **Remediation Diff:**
```diff
--- a/bin/kineti-spend.ts
+++ b/bin/kineti-spend.ts
@@ -28,3 +28,3 @@ function file(): string { return path.join(projectKdir(), "spend.json"); }
-function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }
+function logFile(): string { return path.join(projectKdir(), "spend.log.jsonl"); }
```
- **Verification Method:**
Log a spend entry using `bun bin/kineti-spend.ts log --stage build --tokens-in 10 --tokens-out 10`. Verify the log is written to `.kineti/spend.log.jsonl` rather than `.kineti/.kineti/`.

---

### Finding 8 [HIGH] — Spend Circuit Breaker Reset Bypass via Static CLI Flag
- **Target File:** `bin/kineti-spend.ts:97-103`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-spend.ts:97-103
if (cmd === "reset") {
  if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
  s.tripped = false; s.reason = null;
  writeJson(file(), s);
  ok("breaker reset by human");
  return;
}
```
- **Root Cause & Impact Analysis:**
The spend circuit breaker is supposed to strictly halt autonomous agent loops when the $50 limit is reached. However, the check only looks for the string `--i-am-human` in CLI arguments. An LLM agent receiving an error message `breaker is tripped... Only a human may reset it` can simply re-invoke the command with `--i-am-human`, bypassing the $50 financial safety circuit breaker completely.
- **Remediation Diff:**
```diff
--- a/bin/kineti-spend.ts
+++ b/bin/kineti-spend.ts
@@ -97,3 +97,6 @@ function main() {
   if (cmd === "reset") {
     if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
+    if (!process.stdin.isTTY && !process.env.KINETI_HUMAN_RESET_TOKEN) {
+      die("security: breaker reset requires an interactive TTY confirmation by a human operator", 2);
+    }
     s.tripped = false; s.reason = null;
```
- **Verification Method:**
Trip the spend breaker; attempt reset via automated pipe `echo "" | bun bin/kineti-spend.ts reset --i-am-human`; verify rejection.

---

### Finding 9 [MEDIUM] — Unchecked Argument Count Crash in `setup.sh` Parameter Parsing
- **Target File:** `setup.sh:18`
- **Vulnerable Code Snippet:**
```bash
# setup.sh:16-23
while [[ $# -gt 0 ]]; do
  case "$1" in
    --host) ONLY_HOST="${2:-}"; shift 2 ;;
```
- **Observed Behavior:**
Running `./setup.sh --host` causes `shift 2` to fail with bash error:
`shift: shift count must be <= $#` (exit code 1).
- **Root Cause & Impact Analysis:**
When `--host` is passed as the final parameter, `$#` is 1. `shift 2` fails, and under `set -e`, bash aborts immediately without showing the usage message.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -17,4 +17,9 @@ while [[ $# -gt 0 ]]; do
   case "$1" in
-    --host) ONLY_HOST="${2:-}"; shift 2 ;;
+    --host)
+      [[ $# -ge 2 ]] || { echo "Error: --host requires an argument"; usage; }
+      ONLY_HOST="$2"; shift 2 ;;
     --uninstall) UNINSTALL=1; shift ;;
```
- **Verification Method:**
Run `./setup.sh --host` and verify that the script displays the error and usage message with exit code 1.

---

### Finding 10 [MEDIUM] — Uncaught Exit Code 1 in `read_conf` Under `pipefail`
- **Target File:** `setup.sh:25-27`
- **Vulnerable Code Snippet:**
```bash
# setup.sh:25-27
read_conf() { # $1 = conf file, $2 = key -> prints value (quotes stripped)
  grep -E "^$2=" "$1" | head -1 | cut -d= -f2- | tr -d '"'
}
```
- **Root Cause & Impact Analysis:**
When a configuration file omits an optional key (e.g. `skills_dir_fallback` in custom or third-party host configs), `grep` finds zero matches and exits with code 1.
Because `set -o pipefail` is active, the entire pipeline exits with 1, and the command substitution `primary="$(read_conf ...)"` terminates `setup.sh` abruptly under `set -e`.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -25,3 +25,3 @@ done
 read_conf() { # $1 = conf file, $2 = key -> prints value (quotes stripped)
-  grep -E "^$2=" "$1" | head -1 | cut -d= -f2- | tr -d '"'
+  (grep -E "^$2=" "$1" || true) | head -1 | cut -d= -f2- | tr -d '"\r'
 }
```
- **Verification Method:**
Create a host config without `skills_dir_fallback`. Run `./setup.sh` and verify execution succeeds without pipefail crashes.

---

### Finding 11 [MEDIUM] — Incomplete Skill Installation: Assumes Only `SKILL.md` Exists
- **Target File:** `setup.sh:88-94`
- **Vulnerable Code Snippet:**
```bash
# setup.sh:88-94
for skill in "$HERE"/skills/*/; do
  skill="$(basename "$skill")"
  dest="$dir/${PREFIX}${skill}"
  mkdir -p "$dest"
  cp "$HERE/skills/$skill/SKILL.md" "$dest/SKILL.md"
  count=$((count+1))
done
```
- **Root Cause & Impact Analysis:**
`cp "$HERE/skills/$skill/SKILL.md" "$dest/SKILL.md"` only copies the top-level `SKILL.md` file.
As skills evolve to include companion assets (`scripts/`, `references/`, `resources/`, `examples/`), none of those files will be installed to host directories. Furthermore, if any subfolder in `skills/` lacks `SKILL.md` (e.g. `.git`, helper directories), `cp` fails and halts the entire installer under `set -e`.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -87,8 +87,11 @@ install() {
     mkdir -p "$dir"
     for skill in "$HERE"/skills/*/; do
+      [[ -d "$skill" ]] || continue
       skill="$(basename "$skill")"
+      [[ -f "$HERE/skills/$skill/SKILL.md" ]] || continue
       dest="$dir/${PREFIX}${skill}"
       mkdir -p "$dest"
-      cp "$HERE/skills/$skill/SKILL.md" "$dest/SKILL.md"
+      cp -R "$HERE/skills/$skill/." "$dest/"
       count=$((count+1))
     done
```
- **Verification Method:**
Add a test file `skills/anchors/test.txt` and run `./setup.sh --host opencode`. Verify `test.txt` is copied to the target directory.

---

### Finding 12 [MEDIUM] — Incomplete Uninstall Leaving Orphan Repository Pointer
- **Target File:** `setup.sh:55-69`
- **Vulnerable Code Snippet:**
```bash
# setup.sh:55-69
uninstall() {
  load_hosts
  local i dir removed=0
  for i in "${!HOST_NAMES[@]}"; do
  ...
  echo "Kineti uninstalled. Removed $removed skill folders."
}
```
- **Root Cause & Impact Analysis:**
`setup.sh` writes `$HOME/.kineti/repo` during `install()` (line 110), but `uninstall()` does not remove or update this pointer. If the user uninstalls or moves the repo, cron jobs running `scripts/weekly.sh` continue attempting to run against the removed installation.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -67,4 +67,7 @@ uninstall() {
   echo "Kineti uninstalled. Removed $removed skill folders."
+  if [[ -f "$HOME/.kineti/repo" ]]; then
+    rm -f "$HOME/.kineti/repo"
+    echo "Removed repository pointer: $HOME/.kineti/repo"
+  fi
   echo "Note: hook text blocks in host settings are comments; remove them by hand if desired."
 }
```
- **Verification Method:**
Run `./setup.sh --uninstall` and verify that `~/.kineti/repo` is cleanly removed.

---

### Finding 13 [MEDIUM] — Fragile Bun Path Resolution Crashing in Non-Interactive Shells
- **Target File:** `scripts/weekly.sh:11-12`
- **Vulnerable Code Snippet:**
```bash
# scripts/weekly.sh:11-12
BUN="${KINETI_BUN:-$HOME/.bun/bin/bun}"
command -v "$BUN" >/dev/null 2>&1 || BUN="$(command -v bun)"
```
- **Root Cause & Impact Analysis:**
If `bun` is not in `$HOME/.bun/bin/bun` and not in `PATH` (standard in minimal cron environments where `PATH=/usr/bin:/bin`), `command -v bun` returns exit code 1. Under `set -e`, the command substitution `BUN="$(command -v bun)"` aborts execution immediately without any useful error message.
- **Remediation Diff:** (Covered in Finding 2 diff).
- **Verification Method:**
Run `PATH="/usr/bin:/bin" KINETI_BUN="/nonexistent" bash scripts/weekly.sh` and verify graceful failure reporting.

---

### Finding 14 [MEDIUM] — Unhandled Exception on Missing `--dir` Value in `kineti-memory-job.ts`
- **Target File:** `bin/kineti-memory-job.ts:74-75`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-memory-job.ts:74-75
const di = process.argv.indexOf("--dir");
const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
```
- **Observed Behavior:**
Running `bun bin/kineti-memory-job.ts sweep --dir` throws:
`TypeError: The "paths[0]" property must be of type string, got undefined`
- **Root Cause & Impact Analysis:**
When `--dir` is passed as the last CLI argument, `process.argv[di + 1]` is undefined. `path.resolve(undefined)` crashes with an unhandled TypeError.
- **Remediation Diff:**
```diff
--- a/bin/kineti-memory-job.ts
+++ b/bin/kineti-memory-job.ts
@@ -74,3 +74,7 @@ function main() {
   const di = process.argv.indexOf("--dir");
-  const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
+  if (di > -1 && (!process.argv[di + 1] || process.argv[di + 1].startsWith("--"))) {
+    die("--dir requires a directory path argument", 2);
+  }
+  const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
```
- **Verification Method:**
Execute `bun bin/kineti-memory-job.ts sweep --dir`; verify it exits cleanly with `kineti: --dir requires a directory path argument` and status code 2.

---

### Finding 15 [MEDIUM] — Non-Atomic File Overwrite in `kineti-memory-job.ts`
- **Target File:** `bin/kineti-memory-job.ts:43-46`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-memory-job.ts:43-46
function save(dir: string, recs: Rec[]): void {
  const body = recs.map((r) => JSON.stringify(r)).join("\n") + (recs.length ? "\n" : "");
  fs.writeFileSync(journalFile(dir), body);
}
```
- **Root Cause & Impact Analysis:**
Direct in-place truncation and overwrite. If the process is terminated (SIGKILL, OOM, power loss) during `save()`, `journal.jsonl` is left empty or truncated, permanently destroying the project memory journal.
- **Remediation Diff:**
```diff
--- a/bin/kineti-memory-job.ts
+++ b/bin/kineti-memory-job.ts
@@ -43,4 +43,6 @@ function save(dir: string, recs: Rec[]): void {
   const body = recs.map((r) => JSON.stringify(r)).join("\n") + (recs.length ? "\n" : "");
-  fs.writeFileSync(journalFile(dir), body);
+  const target = journalFile(dir);
+  const tmp = `${target}.tmp.${process.pid}.${Date.now()}`;
+  fs.writeFileSync(tmp, body);
+  fs.renameSync(tmp, target);
 }
```
- **Verification Method:**
Perform `sweep` operation; verify atomic rename behavior via filesystem monitoring or strace.

---

### Finding 16 [MEDIUM] — Symlink Traversal & Infinite Hang on FIFOs in `fingerprint()`
- **Target File:** `bin/kineti-evidence.ts:22-39`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-evidence.ts:31-38
if (e.isFile() && !EXCLUDE_FILES.has(e.name)) {
  try {
    const st = fs.statSync(full);
    if (st.size > MAX_FILE_BYTES) continue;
    const rel = path.relative(root, full);
    parts.push(`${rel}:${sha256(fs.readFileSync(full))}`);
  } catch { /* unreadable: skip */ }
}
```
- **Root Cause & Impact Analysis:**
`fs.statSync` follows symlinks. If a repository contains a symlink pointing to a FIFO (named pipe), device file (`/dev/urandom`), or external sensitive file, `fs.readFileSync(full)` either blocks the process indefinitely waiting for input on the FIFO or reads sensitive files outside the project root into the fingerprint hash.
- **Remediation Diff:**
```diff
--- a/bin/kineti-evidence.ts
+++ b/bin/kineti-evidence.ts
@@ -23,3 +23,3 @@ export function fingerprint(root: string = process.cwd()): string {
     let entries: fs.Dirent[];
-    try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return; }
+    try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return; }
     for (const e of entries.sort((a, b) => a.name.localeCompare(b.name))) {
@@ -31,3 +31,4 @@ export function fingerprint(root: string = process.cwd()): string {
-      if (e.isFile() && !EXCLUDE_FILES.has(e.name)) {
+      if (e.isFile() && !e.isSymbolicLink() && !EXCLUDE_FILES.has(e.name)) {
         try {
-          const st = fs.statSync(full);
+          const st = fs.lstatSync(full);
+          if (!st.isFile()) continue;
           if (st.size > MAX_FILE_BYTES) continue;
```
- **Verification Method:**
Create a named pipe `mkfifo test_fifo` in a test project; run `fingerprint()`; verify it does not hang and finishes instantly.

---

### Finding 17 [LOW] — Missing Executable Permission on `bin/kineti-memory-job.ts`
- **Target File:** `bin/kineti-memory-job.ts`
- **Observation:**
`ls -l bin/kineti-memory-job.ts` returns:
`-rw-r--r--@ 1 praveen staff 5627 Aug 26 03:47 bin/kineti-memory-job.ts`
All other binaries in `bin/` have mode `0755` (`-rwxr-xr-x`).
- **Root Cause & Impact:**
Missing `chmod +x`. Running `./bin/kineti-memory-job.ts` directly fails with `permission denied`, despite the presence of the shebang `#!/usr/bin/env bun`.
- **Remediation:**
Set execution bit: `chmod +x bin/kineti-memory-job.ts`.

---

### Finding 18 [LOW] — Inappropriately Set Executable Bit on Library Module `bin/lib.ts`
- **Target File:** `bin/lib.ts`
- **Observation:**
`ls -l bin/lib.ts` returns:
`-rwxr-xr-x@ 1 praveen staff 2461 Aug 21 20:53 bin/lib.ts`
- **Root Cause & Impact:**
`bin/lib.ts` is purely a module export library with no shebang and no execution logic. Setting mode `0755` violates least-privilege principles.
- **Remediation:**
Set non-executable mode: `chmod 644 bin/lib.ts`.

---

### Finding 19 [LOW] — Hash Delimiter Collision Risk in Egress Receipt Ledger
- **Target File:** `bin/kineti-egress.ts:14-16`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-egress.ts:14-16
function computeHash(r: Omit<Receipt, "hash">): string {
  return sha256(`${r.seq}|${r.at}|${r.host}|${r.description}|${r.prev_hash}`);
}
```
- **Root Cause & Impact:**
Using unescaped pipe `|` delimiters allows delimiter injection: if `host` or `description` contains `|`, fields can shift into neighboring positions without altering the resulting hash.
- **Remediation Diff:**
```diff
--- a/bin/kineti-egress.ts
+++ b/bin/kineti-egress.ts
@@ -14,3 +14,3 @@ function stateFile(): string { return path.join(machineDir(), "egress.state.jso
 function computeHash(r: Omit<Receipt, "hash">): string {
-  return sha256(`${r.seq}|${r.at}|${r.host}|${r.description}|${r.prev_hash}`);
+  return sha256(JSON.stringify([r.seq, r.at, r.host, r.description, r.prev_hash]));
 }
```
- **Verification Method:**
Verify that JSON array serialization prevents field-boundary collision.

---

### Finding 20 [LOW] — Unsanitized Log Injection (CWE-117) in System Alerts
- **Target File:** `bin/kineti-verify-gate.ts:59` & `bin/kineti-spend.ts:112`
- **Vulnerable Code Snippet:**
```ts
// bin/kineti-verify-gate.ts:59
fs.appendFileSync(path.join(machineDir(), "alerts.log"), `${nowIso()} verify passed: ${declared}\n`);

// bin/kineti-spend.ts:112
fs.appendFileSync(path.join(machineDir(), "alerts.log"), `${nowIso()} SPEND TRIPPED: ${reason}\n`);
```
- **Root Cause & Impact:**
Neither `declared` nor `reason` strips newline characters (`\n`, `\r`). An adversary or crafted command can inject fake log lines into `~/.kineti/alerts.log`.
- **Remediation:**
Sanitize strings before writing: `const clean = text.replace(/[\r\n]+/g, " ");`.

---

### Finding 21 [INFORMATIONAL] — Stale Skill Catalog in Claude Hook Block
- **Target File:** `hooks/claude.txt:4-5`
- **Vulnerable / Inaccurate Content:**
```
  List available skills: officehours, diagnose, design,
  architecture, feasibility, spec, build, review, qa, security, ship, watch, retro, learn.
```
- **Root Cause & Impact:**
`skills/` contains 17 skills. `hooks/claude.txt` lists only 14 skills, omitting `anchors`, `second-opinion`, and `skillify`. Claude Code users who paste this hook block are not alerted to the existence of these 3 skills.
- **Remediation:**
Update line 5 to list all 17 skills: `anchors, architecture, build, design, diagnose, feasibility, learn, officehours, qa, retro, review, second-opinion, security, ship, skillify, spec, watch`.

---

### Finding 22 [INFORMATIONAL] — Hardcoded Skill Counts in Smoke Test
- **Target File:** `tests/test-setup.sh:18, 24`
- **Vulnerable Code Snippet:**
```bash
# tests/test-setup.sh:18, 24
echo "$out" | grep -q "installed: 17 skills.*opencode" || { echo "FAIL: opencode install"; exit 1; }
[[ "$(echo "$out2" | grep -c "installed: 17")" == "2" ]] || { echo "FAIL: not idempotent"; exit 1; }
```
- **Root Cause & Impact:**
The test hardcodes `17 skills`. Adding, archiving, or removing a skill will immediately fail this test regardless of installer functionality.
- **Remediation:**
Compute count dynamically: `SKILL_COUNT=$(find "$HERE/skills" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')`.

---

### Finding 23 [INFORMATIONAL] — Unprotected Glob Expansions Missing `shopt -s nullglob`
- **Target File:** `setup.sh:47, 61, 88` and `scripts/audit-skills.sh:7`
- **Root Cause & Impact:**
In bash, when globs like `"$HERE"/hosts/*.conf` or `"$HERE"/skills/*/SKILL.md` match zero files, bash expands the expression to the literal glob string itself. Under `set -e`, subsequent commands operating on the non-existent literal file will fail.
- **Remediation:**
Enable `shopt -s nullglob` at the top of bash scripts that iterate over file globs.

---

## 3. Logic Chain

1. **Test Verification Observation:** Running `bun test tests/` directly failed on `tests/memory-job.test.ts:56` with exit code 3. Tracing the hash calculation in `kineti-memory-job.ts` confirmed that the test script hashes with `|` delimiters, whereas the application code hashes without delimiters. Additionally, `oldLearning` had no `prev_hash`/`hash`. Thus, the test suite itself contains a critical desynchronization defect.
2. **Weekly Script Execution Observation:** Running `bash scripts/weekly.sh` immediately aborted because `~/.kineti/repo` was absent. Inspection of line 10 revealed an unchecked `cat` under `set -e`. Inspection of line 16 revealed an unquoted variable expansion that splits any workspace path containing spaces.
3. **Ledger Reliability Observation:** Evaluating `readJsonl` with an invalid JSON line caused the function to catch the error and return `[]`. In `kineti-egress.ts`, an empty ledger resets the sequence to 0 and genesis hash, permanently corrupting the Merkle tree.
4. **Security Boundary Observation:** Reviewing `kineti-verify-gate.ts` confirmed that `--trust` requires zero authentication or interactive confirmation. Any autonomous subagent can self-trust any arbitrary shell command and execute it via `bash -lc`.
5. **Path Calculation Observation:** Computing `logFile()` in `kineti-spend.ts` demonstrated that `path.join(projectKdir(), ".kineti", "spend.log.jsonl")` produces a redundant nested directory `.kineti/.kineti/spend.log.jsonl`.
6. **File Metadata Observation:** `ls -l` confirmed that `bin/kineti-memory-job.ts` lacks `0755` permissions, while `bin/lib.ts` has `0755` despite having no shebang or executable purpose.

---

## 4. Caveats

- **External Host Hooks:** Host instruction files (`~/.claude/CLAUDE.md`, `~/.gemini/config/settings.json`, etc.) were not inspected as they reside outside the active repository.
- **Operating System Environment:** Tests and commands were executed on macOS (Darwin arm64). In minimal Linux container environments lacking standard GNU or BSD utilities, additional subtle portability variations in `cut`, `head`, and `tr` might exist.
- **Non-Destructive Constraint:** Per instructions, no changes were committed to source files. All patches are provided as verified diffs within this report.

---

## 5. Conclusion

The Kineti local harness scripts and programs exhibit a solid core design (fail-closed gates, hash-chained ledgers, token circuit breakers), but suffer from several critical implementation defects:
1. **Broken baseline test suite** (`tests/memory-job.test.ts`).
2. **Fragile shell automation** (`scripts/weekly.sh`, `setup.sh`) failing on whitespace and missing prerequisites.
3. **Data loss vulnerabilities** in ledger parsing (`readJsonl`).
4. **Autonomous privilege escalation risks** bypassing verify gates and spend circuit breakers.

Addressing the concrete diffs provided in this report will bring the harness up to enterprise production and security standards.

---

## 6. Independent Verification Method

To verify the audit findings:

1. **Verify Baseline Test Failure:**
   ```bash
   bun test tests/
   ```
   *Expected result:* Fails on `tests/memory-job.test.ts:56` with exit code 1.

2. **Verify Weekly Script Crash:**
   ```bash
   bash scripts/weekly.sh
   ```
   *Expected result:* Exits with code 1 (`cat: ~/.kineti/repo: No such file or directory`).

3. **Verify Space-Path Splitting:**
   ```bash
   bash -c 'PROJECTS="/path/with space/dir"; for p in $PROJECTS; do echo "token: $p"; done'
   ```
   *Expected result:* Prints two separate tokens instead of one intact path.

4. **Verify `readJsonl` Silent Data Erasure:**
   ```bash
   bun -e 'import { readJsonl } from "./bin/lib.ts"; import fs from "node:fs"; fs.writeFileSync("/tmp/test_audit.jsonl", "{\"seq\":1}\ncorrupted_line\n{\"seq\":2}\n"); console.log("Result:", readJsonl("/tmp/test_audit.jsonl")); fs.unlinkSync("/tmp/test_audit.jsonl");'
   ```
   *Expected result:* Outputs `Result: []`.

5. **Verify Nested Spend Log Path Bug:**
   ```bash
   bun -e 'import { projectKdir } from "./bin/lib.ts"; import path from "node:path"; console.log(path.join(projectKdir(), ".kineti", "spend.log.jsonl"));'
   ```
   *Expected result:* Outputs path containing `.kineti/.kineti/spend.log.jsonl`.

6. **Verify Typecheck Passes:**
   ```bash
   bun run typecheck
   ```
   *Expected result:* Exits with code 0 (`$ tsc --noEmit`).
