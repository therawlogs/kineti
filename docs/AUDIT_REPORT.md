# Comprehensive Codebase, Architecture, and Security Audit Report
**Target Repository:** Kineti Local Harness (`kineti-os`)  
**Audit Date:** 2026-09-06  
**Auditor:** Teamwork Security & Architecture Audit Team (`worker_report_1`)  
**Repository Version:** 3.0.0 (Git Tag `v3.0.1`)  
**Execution Environment:** Darwin arm64 (macOS Sonoma), Bun v1.4.0 (1381054db), TypeScript 5.6.0  
**Audit Mode:** Non-Destructive / Read-Only Forensic Analysis  

---

## 1. Executive Summary & Overall Health Score

### 1.1 High-Level Assessment
The Kineti local harness is designed as an autonomous, closed-loop software factory operating under the standing philosophy: **"Skills propose, programs enforce, memory remembers."** The harness establishes formal execution guardrails around Large Language Model (LLM) agents across thirteen development stages and four quality/governance gates. Key safety primitives include:
- Immutable root-goal locking (`bin/kineti-state.ts`)
- Financial circuit breakers with stage-level token accounting (`bin/kineti-spend.ts`)
- LIFO saga rollback stacks for undo safety (`bin/kineti-saga.ts`)
- Cryptographic code fingerprinting for test proofs (`bin/kineti-evidence.ts`)
- Tamper-evident hash-chained egress ledgers (`bin/kineti-egress.ts`)
- Pre-flight verification gates (`bin/kineti-verify-gate.ts`)
- Causal, TTL-governed memory journals (`bin/kineti-memory-job.ts`)

Despite these architectural strengths, our forensic audit revealed **44 distinct findings** across security, architecture, script portability, and test integrity. Most critically:
1. The **baseline test suite fails out of the box** (`tests/memory-job.test.ts:56`), blocking continuous integration and obscuring regression detection.
2. The **state machine fails gate lookups** (`bin/kineti-state.ts:73`), crashing Stages 7 (`build`) and 11 (`ship`) on pre-flight checks.
3. The **verify-gate allows autonomous programmatic self-trust** (`bin/kineti-verify-gate.ts:20-27`), allowing any subagent to execute arbitrary shell commands without human confirmation.
4. The core JSONL parser (`bin/lib.ts:48-58`) suffers from **catastrophic silent data loss**, dropping entire memory journals and rollback stacks on a single malformed line.
5. The cron maintenance script (`scripts/weekly.sh:10, 16`) crashes on clean installs and breaks on directory paths containing spaces.

### 1.2 Quantitative Repository Health Score: 74 / 100

```
+-----------------------------------------------------------------------+
|                       OVERALL HEALTH SCORE: 74/100                    |
+-----------------------------------+-----------------------------------+
| Dimension                         | Score | Status                    |
+-----------------------------------+-------+---------------------------+
| 1. Security Posture               | 68/100| AT RISK (Bypass Hazards)  |
| 2. Architectural Alignment        | 76/100| DEGRADED (Gate Crashes)   |
| 3. Script Portability & Robustness| 72/100| FRAGILE (Silent Data Loss)|
| 4. Test & Type Integrity          | 80/100| BROKEN (Baseline Failing) |
+-----------------------------------+-------+---------------------------+
```

#### Dimensional Breakdown:
- **Security Posture (68/100):** Subagents can bypass the verify-gate (`--trust`) and spend breaker (`--i-am-human`) programmatically without interactive confirmation. Arbitrary shell commands are executed via `bash -lc` in saga rollback and evidence recording without timeouts, quoting guards, or stderr capture.
- **Architectural Alignment (76/100):** State CLI gate retrieval (`get gate.<name>`) is broken; gate state `pending` mandated by `spec/SKILL.md` is rejected; path concatenation creates duplicate `.kineti/.kineti/` directories; and evidence fingerprinting fails to exclude `.agents/` and `design/screenshots/`, causing valid proofs to flip `STALE`.
- **Script Portability & Robustness (72/100):** `readJsonl` swallows parse errors and returns empty arrays, obliterating ledgers; `weekly.sh` crashes if `~/.kineti/repo` is missing and splits workspace paths containing spaces; `setup.sh` crashes on missing arguments or absent configuration keys under `pipefail`; and memory journal updates overwrite files in-place non-atomically.
- **Test & Type Integrity (80/100):** Out-of-the-box test failure in `memory-job.test.ts`; installer smoke tests are excluded from `package.json`; critical paths (saga commit, stage spend ceilings, egress truncation, verify-gate pass-open) have 0% test coverage; and 10 indexing type errors exist under TypeScript strictness flags.

### 1.3 Findings Summary by Severity
A total of **44 verified findings** were categorized according to standard vulnerability and defect classification:

| Severity Level | Count | Definition |
|---|:---:|---|
| **Critical** | **5** | Immediate blocker, data loss, security bypass, or broken test baseline |
| **High** | **9** | Unhandled runtime crash, process hang, false proof invalidation, or missing test coverage |
| **Medium** | **12** | Portability failure, resource leak, non-atomic I/O, or architectural drift |
| **Low** | **8** | File permission anomaly, delimiter collision hazard, log injection, or documentation link defect |
| **Informational** | **10** | Schema naming discrepancy, version desync, unquoted variable, or stylistic inconsistency |
| **Total** | **44** | **Exhaustive finding count across repository** |

---

## 2. Scope & Methodology

### 2.1 Inventory of Inspected Artifacts
The audit performed a comprehensive inspection of all executable code, hook scripts, configuration files, skills, documentation, and test suites across the repository:

```
kineti-local-harness/
├── setup.sh                         [Host installer & uninstaller script]
├── package.json                     [NPM dependencies and test scripts]
├── tsconfig.json                    [TypeScript compiler options]
├── kineti.config.json               [Harness pipeline, gates & limits]
├── .gitignore                       [Ignored build and state artifacts]
├── bin/
│   ├── kineti-state.ts              [State machine & goal lock CLI]
│   ├── kineti-spend.ts              [Token accounting & spend breaker CLI]
│   ├── kineti-saga.ts               [Saga transaction & rollback CLI]
│   ├── kineti-evidence.ts           [Cryptographic proof capture CLI]
│   ├── kineti-verify-gate.ts        [Pre-flight test enforcement gate]
│   ├── kineti-egress.ts             [Outbound HTTP egress ledger CLI]
│   ├── kineti-memory-job.ts         [Causal memory maintenance & sweep CLI]
│   ├── lib.ts                       [Shared utility module]
│   └── README.md                    [Binary documentation]
├── scripts/
│   ├── audit-skills.sh              [Skill context word count audit]
│   └── weekly.sh                    [Cron weekly memory maintenance]
├── hosts/
│   ├── claude.conf                  [Claude Code host definitions]
│   ├── codex.conf                   [Codex CLI host definitions]
│   ├── gemini.conf                  [Gemini CLI host definitions]
│   └── opencode.conf                [OpenCode host definitions]
├── hooks/
│   ├── claude.txt                   [Claude Code instructions hook]
│   ├── codex.txt                    [Codex instructions hook]
│   ├── gemini.txt                   [Gemini instructions hook]
│   └── opencode.txt                 [OpenCode instructions hook]
├── skills/ (17 skills)
│   ├── anchors, architecture, build, design, diagnose, feasibility,
│   ├── learn, officehours, qa, retro, review, second-opinion,
│   └── security, ship, skillify, spec, watch
├── docs/
│   ├── HOWTO-daily-loop.md          [Daily workflow instructions]
│   └── TUTORIAL-first-run.md        [Walkthrough guide]
├── tests/
│   ├── harness.test.ts              [Integration suite for 6 binaries]
│   ├── memory-job.test.ts           [Unit suite for memory job]
│   └── test-setup.sh                [Installer smoke test script]
└── Documentation:
    ├── ETHOS.md                     [Standing laws and directives]
    ├── WORKFLOWS.md                 [13-stage execution pipeline]
    ├── MEMORY.md                    [Memory schema, TTL, and causality]
    ├── MIGRATION.md                 [Upgrade path]
    ├── ROADMAP.md                   [Post-v3.0.1 future work]
    ├── README.md                    [Root architectural overview]
    └── AGENTS.md                    [Agent operational constraints]
```

### 2.2 Execution & Verification Environment
- **Hardware Architecture:** Apple Silicon (arm64, Darwin 24.3.0)
- **Runtime Environment:** Bun v1.4.0 (`1381054db`)
- **TypeScript Compiler:** Version 5.6.0 (Target: ES2022, Module: ES2022, ModuleResolution: Bundler)
- **Shell Environments:** GNU Bash 3.2.57(1)-release / Zsh 5.9

### 2.3 Non-Destructive Forensic Boundary
All testing, failure reproduction, and diff validations were performed in temporary isolated directories (`os.tmpdir()`) or via non-destructive dry runs. In accordance with the audit mandate, **no source code, tests, or configuration files outside `docs/AUDIT_REPORT.md` were modified during this investigation**.

---

## 3. Findings Matrix by Severity

| ID | Sev | Title | Category | Impacted File(s) & Lines | One-Line Summary |
|---|:---:|---|---|---|---|
| **CRIT-01** | CRIT | Baseline Test Suite Failure in Memory Job | Test / Integrity | `tests/memory-job.test.ts:37-56` | Hash delimiter mismatch and unchained record cause immediate test suite failure on fresh clone. |
| **CRIT-02** | CRIT | Broken Gate Status Lookup in State CLI | Architecture / CLI | `bin/kineti-state.ts:70-77` | `get gate.<name>` evaluates to `undefined`, crashing Build and Ship stage pre-flight checks. |
| **CRIT-03** | CRIT | Programmatic Self-Trust Security Bypass in Verify Gate | Security / Boundary | `bin/kineti-verify-gate.ts:20-27` | Autonomous subagents can run `--trust` non-interactively to execute arbitrary commands via `bash -lc`. |
| **CRIT-04** | CRIT | Catastrophic Silent Ledger Erasure on JSONL Parse Errors | Data Integrity | `bin/lib.ts:48-58` | A single corrupted line causes `readJsonl` to return `[]`, wiping memory, saga, and egress state. |
| **CRIT-05** | CRIT | Unhandled Missing Repo Pointer & Space Splitting in Cron | Portability / Bash | `scripts/weekly.sh:10, 13, 16` | Cron crashes if pointer is absent; unquoted `$PROJECTS` breaks on workspaces with whitespace. |
| **HIGH-01** | HIGH | Arbitrary Shell Execution & Stderr Swallowing in Saga | Security / Robustness | `bin/kineti-saga.ts:63-72` | Undos execute without timeout and swallow stderr; continuation requires diagnostic logging. |
| **HIGH-02** | HIGH | Command Injection & Diagnostic Blindness in Evidence Run | Security / Robustness | `bin/kineti-evidence.ts:61, 64, 71` | Argument flattening strips quotes; failed execution stderr is completely swallowed. |
| **HIGH-03** | HIGH | Path Duplication Creating Nested `.kineti/.kineti/` Logs | Architecture / I/O | `bin/kineti-spend.ts:29` | Joins `projectKdir()` with `".kineti"`, dumping spend logs into an invalid nested path. |
| **HIGH-04** | HIGH | Code Fingerprint Invalidation Caused by `.agents/` | Architecture / Gate | `bin/kineti-evidence.ts:11-15` | Omission of `.agents/` and `screenshots/` causes agent heartbeats to flip proofs to `STALE`. |
| **HIGH-05** | HIGH | Spend Circuit Breaker Reset Bypass via Static CLI Flag | Security / Financial | `bin/kineti-spend.ts:97-103` | Autonomous agents can bypass $50 financial limit by passing `--i-am-human` without a TTY check. |
| **HIGH-06** | HIGH | Spec Skill Directs Unsupported Gate State Transition | Architecture / State | `skills/spec/SKILL.md:49` | Directs setting `gate.spec pending`, which `kineti-state.ts` strictly rejects with exit 2. |
| **HIGH-07** | HIGH | Uncaught Runtime TypeError on Missing CLI Argument | Robustness / CLI | `bin/kineti-memory-job.ts:74-75` | Running `sweep --dir` with no path crashes unhandled with `ERR_INVALID_ARG_TYPE`. |
| **HIGH-08** | HIGH | Installer Smoke Test Excluded from Default Runner | Testing / CI | `package.json:8`, `tests/test-setup.sh` | `bun test` ignores shell tests, leaving host installer automation completely unverified in CI. |
| **HIGH-09** | HIGH | Critical Enforcement Subsystems Completely Untested | Test Coverage | `bin/kineti-saga.ts`, `spend.ts`, etc. | Saga commit, spend per-stage ceilings, evidence expect-cmd, and egress truncation have 0% tests. |
| **MED-01** | MED | Saga Rollback Forbids Committed Runs Without Force Flag | Architecture / Watch | `bin/kineti-saga.ts:15-20, 55-57` | Committed runs sealed by default; Stage 12 recovery requires explicit `--force-committed`. |
| **MED-02** | MED | Zero Enforcement Tooling for Standing ETHOS Rule 8 | Architecture / Policy | `ETHOS.md:44-48` | Rule 8 mandates pre-commit scans for personal names/paths/secrets, but no script enforces it. |
| **MED-03** | MED | Missing UX Blueprint Integration in Intake & Design Skills | Architecture / UX | `skills/officehours/SKILL.md`, `design/` | Skills omit UX Blueprint deliverables mandated by ETHOS Rule 1.3 and WORKFLOWS Stage 1. |
| **MED-04** | MED | Non-Atomic File Overwrite in Memory Journal | Data Integrity | `bin/kineti-memory-job.ts:43-46` | Direct truncation and overwrite of `journal.jsonl` risks total journal loss on abrupt termination. |
| **MED-05** | MED | Symlink Traversal & FIFO Blocking in Fingerprinting | Security / DoS | `bin/kineti-evidence.ts:22-39` | `fs.statSync` follows symlinks, reading outside files and hanging infinitely on named pipes. |
| **MED-06** | MED | Unchecked Argument Parsing Crash in Installer | Robustness / Bash | `setup.sh:18` | Passing `--host` without an argument causes unhandled `shift: shift count must be <= $#`. |
| **MED-07** | MED | Unhandled Exit Code 1 in `read_conf` Under `pipefail` | Robustness / Bash | `setup.sh:25-27` | Missing optional configuration key causes `grep` exit 1 to abruptly terminate `setup.sh`. |
| **MED-08** | MED | Incomplete Skill Installation Omitting Subdirectories | Robustness / Installer | `setup.sh:88-94` | Copies only top-level `SKILL.md`, dropping companion `scripts/`, `references/`, and `resources/`. |
| **MED-09** | MED | Incomplete Uninstall Leaving Orphan Repository Pointer | Maintenance / Cleanup | `setup.sh:55-69` | `--uninstall` removes skill folders but leaves `~/.kineti/repo`, causing cron jobs to target dead paths. |
| **MED-10** | MED | Fragile Bun Path Resolution in Non-Interactive Cron | Portability / Bash | `scripts/weekly.sh:11-12` | Fails abruptly under `set -e` when Bun is installed outside standard paths in cron environments. |
| **MED-11** | MED | Temporary Directory Resource Leakage in Test Suite | Testing / Hygiene | `tests/memory-job.test.ts:69`, `harness` | Tests lack `try...finally` blocks, leaving orphaned temporary directories on test assertions. |
| **MED-12** | MED | TypeScript Strictness Gaps Masking Indexing Hazards | Type Integrity | `tsconfig.json:8`, `bin/*.ts` | Disabling `noUncheckedIndexedAccess` masks 10 runtime undefined-indexing bugs. |
| **LOW-01** | LOW | Missing Executable Permission on CLI Binary | Security / Permissions | `bin/kineti-memory-job.ts` | Has file mode `0644` instead of `0755`, failing direct execution despite having a shebang. |
| **LOW-02** | LOW | Inappropriate Executable Bit on Library Module | Security / Permissions | `bin/lib.ts` | Has file mode `0755` despite being a library module with no shebang or executable logic. |
| **LOW-03** | LOW | Hash Delimiter Collision Risk in Egress Receipt Ledger | Security / Crypto | `bin/kineti-egress.ts:14-16` | Pipe `|` delimiters allow boundary shifts if `host` or `description` contains pipe characters. |
| **LOW-04** | LOW | Unsanitized Log Injection (CWE-117) in Machine Alerts | Security / Logging | `bin/kineti-verify-gate.ts:59`, `spend` | Unescaped user/command strings allow injecting fake newline entries into `alerts.log`. |
| **LOW-05** | LOW | Record Type Schema Naming Mismatch | Architecture / Schema | `kineti.config.json:40` | Config defines `"project-dossier"`, whereas implementation and docs define `"dossier"`. |
| **LOW-06** | LOW | Stale Reference in Officehours Skill to `journal.md` | Documentation | `skills/officehours/SKILL.md:32` | References non-existent `journal.md` instead of active fallback `journal.jsonl`. |
| **LOW-07** | LOW | Incomplete Skill Catalog Listing in Claude Hook | Documentation / Hooks | `hooks/claude.txt:4-5` | Lists only 14 skills, omitting `anchors`, `second-opinion`, and `skillify`. |
| **LOW-08** | LOW | Dead and Conflicting `hook_file` Directives in Host Confs | Configuration | `hosts/*.conf:5` | Directives point to wrong configuration files and are completely ignored by `setup.sh`. |
| **INFO-01**| INFO| Harness Program Count Mismatch in Documentation | Documentation | `package.json:5`, `bin/README.md:3` | Documents 6 enforcement programs, omitting `kineti-memory-job.ts` (7 programs total). |
| **INFO-02**| INFO| Absolute Path Reference `/design` in Standing Law | Documentation | `ETHOS.md:9` | References absolute `/design` instead of repository relative `design/screens/`. |
| **INFO-03**| INFO| Unquoted Variable Expansion in Daily Loop Documentation | Documentation | `docs/HOWTO-daily-loop.md:37-38` | Unquoted `$K/kineti-evidence.ts` breaks when repo path contains whitespace. |
| **INFO-04**| INFO| Repository Version Desynchronization | Release Hygiene | `package.json:3`, `kineti.config.json` | Repository files report version `3.0.0` while Git tag and roadmap indicate `v3.0.1`. |
| **INFO-05**| INFO| Gate Count Ambiguity in Documentation vs Configuration | Architecture / Docs | `kineti.config.json:10-16`, `WORKFLOWS`| Docs specify 3 gates (Feasibility, Spec, Ship), but config defines 4 (`security` gate: true). |
| **INFO-06**| INFO| Philosophical Directives Conflict on 12 UI Components | Architecture / Design | `skills/design/SKILL.md:65-66` | Skill describes components as optional fallback, conflicting with Kineti Master Directives. |
| **INFO-07**| INFO| Hardcoded Skill Counts in Installer Smoke Test | Testing / Brittleness | `tests/test-setup.sh:18, 24` | Hardcodes string match `installed: 17`, breaking test whenever skills are added/removed. |
| **INFO-08**| INFO| Unprotected Shell Glob Expansions Missing `nullglob` | Robustness / Bash | `setup.sh:47, 61, 88`, `audit-skills` | Globs expand to literal unexpanded strings when zero matching files exist. |
| **INFO-09**| INFO| Unused Test Import in Harness Test Suite | Code Hygiene | `tests/harness.test.ts:1` | Imports `beforeAll` from `bun:test` but never invokes it. |
| **INFO-10**| INFO| Zero Automated Test Coverage for Utility Shell Scripts | Test Coverage | `scripts/audit-skills.sh`, `weekly` | Shell scripts have zero unit, smoke, or integration test coverage. |

---

## 4. Detailed Write-ups for Each Issue

### 4.1 Critical Severity Findings

#### [CRIT-01] Baseline Test Suite Failure in Memory Job Test
- **File & Line Numbers:** `tests/memory-job.test.ts:37-56` (interacting with `bin/kineti-memory-job.ts:64-66, 104-107`)
- **Problematic Code Snippet:**
```typescript
// tests/memory-job.test.ts:43, 48-54
r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest("hex");
...
const oldLearning = {
  at: iso(120), type: "learning", state: "active", project: "p", id: "lr-001",
  data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
  expires: iso(10),
};
write(dir, [r1, r2, oldLearning]);

expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
```
- **Root Cause & Impact:**
  1. `bin/kineti-memory-job.ts:65` calculates hashes without pipe separators: `sha256("${r.prev_hash}${r.at}${r.id}${canonStable(r.data, stable)}")`. The test computes hashes with pipe delimiters (`|`), triggering a content hash mismatch error on record `rr-001`.
  2. `oldLearning` has no `prev_hash` or `hash`. When `verify-chain` runs, line 104 triggers: `kineti: CHAIN BROKEN at lr-001: missing prev_hash/hash` (exit code 3).
  3. **Canonical Key Sorting Mismatch with `canonStable`:** In `bin/kineti-memory-job.ts:65`, `recordHash` invokes `canonStable(r.data, stable)`. Lines 51-62 in `bin/kineti-memory-job.ts` implement `canonStable`, which strictly sorts object keys alphabetically (`Object.keys(x).sort()`). When `oldLearning.data` is declared as `{ skill: "qa", trigger: "always", lesson: "old" }`, `JSON.stringify(oldLearning.data)` serializes keys in insertion order (`"skill"`, `"trigger"`, `"lesson"`), whereas `canonStable` serializes them alphabetically (`"lesson"`, `"skill"`, `"trigger"`). This produces an immediate cryptographic hash mismatch (`kineti: TAMPER at lr-001: content hash mismatch`, exit code 3). Therefore, `oldLearning.data` keys must be declared in alphabetical order: `{ lesson: "old", skill: "qa", trigger: "always" }` so that standard `JSON.stringify` produces the identical string as `canonStable`.
  4. **Impact:** `bun test tests/` fails out of the box on a pristine repository clone (exit code 1, 7 pass, 1 fail). All subsequent assertions in `tests/memory-job.test.ts` (sweep, state promotion, tamper detection) are blocked.
- **Concrete Remediation Diff:**
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
-      data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
+      data: { lesson: "old", skill: "qa", trigger: "always" }, links: [],
       expires: iso(10),
+      prev_hash: r2.hash,
+      hash: "",
     };
+    oldLearning.hash = crypto.createHash("sha256").update(`${oldLearning.prev_hash}${oldLearning.at}${oldLearning.id}${JSON.stringify(oldLearning.data)}`).digest("hex");
     write(dir, [r1, r2, oldLearning]);
```
- **Verification Command:**
```bash
# Isolated empirical verification command (executing non-destructively in a temporary runner):
bun -e '
import { spawnSync } from "node:child_process";
import fs from "node:fs"; import os from "node:os"; import path from "node:path";
let c = fs.readFileSync("tests/memory-job.test.ts", "utf8");
c = c.replace("const REPO = path.resolve(import.meta.dir, \"..\");", "const REPO = " + JSON.stringify(process.cwd()) + ";");
c = c.replace("r1.hash = crypto.createHash(\"sha256\").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest(\"hex\");", "r1.hash = crypto.createHash(\"sha256\").update(`${r1.prev_hash}${r1.at}${r1.id}${JSON.stringify(r1.data)}`).digest(\"hex\");");
c = c.replace("r2.hash = crypto.createHash(\"sha256\").update(`${r2.prev_hash}|${r2.at}|${r2.id}|${JSON.stringify(r2.data)}`).digest(\"hex\");\n    const oldLearning = {", "r2.hash = crypto.createHash(\"sha256\").update(`${r2.prev_hash}${r2.at}${r2.id}${JSON.stringify(r2.data)}`).digest(\"hex\");\n    const oldLearning: any = {");
c = c.replace("data: { skill: \"qa\", trigger: \"always\", lesson: \"old\" }, links: [],\n      expires: iso(10),\n    };", "data: { lesson: \"old\", skill: \"qa\", trigger: \"always\" }, links: [],\n      expires: iso(10),\n      prev_hash: r2.hash,\n      hash: \"\",\n    };\n    oldLearning.hash = crypto.createHash(\"sha256\").update(`${oldLearning.prev_hash}${oldLearning.at}${oldLearning.id}${JSON.stringify(oldLearning.data)}`).digest(\"hex\");");
const tmp = path.join(os.tmpdir(), "crit01-verify.test.ts");
fs.writeFileSync(tmp, c);
const res = spawnSync("bun", ["test", tmp], { encoding: "utf8" });
console.log((res.stdout || "") + "\n" + (res.stderr || ""));
fs.unlinkSync(tmp);
process.exit(res.status ?? 1);
'
# Verbatim Output:
# (pass) kineti-memory-job > sweep promotes expired actives; chain verifies then detects tamper (exit 3)
# (pass) kineti-memory-job > time-order flags effect-before-cause; promote surfaces frequent new words
# 2 pass, 0 fail, 8 expect() calls, exit 0

# When applied directly to tests/memory-job.test.ts:
bun test tests/memory-job.test.ts
# Expect: 2 pass, 0 fail, exit 0
```

---

#### [CRIT-02] Broken Gate Status Lookup in State CLI Blocking Build & Ship Pre-Flights
- **File & Line Numbers:** `bin/kineti-state.ts:70-77` (interacting with `skills/build/SKILL.md:23` and `skills/ship/SKILL.md:29-30`)
- **Problematic Code Snippet:**
```typescript
// bin/kineti-state.ts:70-76
if (cmd === "get") {
  const key = rest[0];
  if (!key) { console.log(JSON.stringify(s, null, 2)); return; }
  const v = (s as any)[key];
  if (v === undefined) die(`unknown key: ${key}`, 2);
  console.log(typeof v === "string" ? v : JSON.stringify(v, null, 2));
  return;
}
```
- **Root Cause & Impact:**
  While `cmd === "set"` specifically handles `key.startsWith("gate.")` by setting `s.gates[key.slice(5)]`, `cmd === "get"` does not. It looks for `s["gate.spec"]`, which is undefined because gate states are stored inside the nested `s.gates` dictionary.
  **Impact:** Every invocation of `bun kineti-state.ts get gate.spec` fails with exit code 2 (`kineti: unknown key: gate.spec`). This fatally blocks Stage 7 (`build`) and Stage 11 (`ship`) pre-flight gate verifications.
- **Concrete Remediation Diff:**
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
- **Verification Command:**
```bash
bun -e '
import { spawnSync } from "node:child_process";
import fs from "node:fs"; os = require("node:os"); path = require("node:path");
const d = fs.mkdtempSync(path.join(os.tmpdir(), "k-test-"));
fs.mkdirSync(path.join(d, ".kineti"));
const p = path.resolve("bin/kineti-state.ts");
spawnSync("bun", [p, "init", "--project", "p"], { cwd: d });
spawnSync("bun", [p, "set", "gate.spec", "pass"], { cwd: d });
const res = spawnSync("bun", [p, "get", "gate.spec"], { cwd: d, encoding: "utf8" });
console.log("Status:", res.status, "Output:", res.stdout.trim());
fs.rmSync(d, { recursive: true });
'
# Expect: Status: 0 Output: pass
```

---

#### [CRIT-03] Programmatic Self-Trust Security Bypass in Verify Gate
- **File & Line Numbers:** `bin/kineti-verify-gate.ts:20-27`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-verify-gate.ts:20-26
if (cmd === "--trust") {
  if (!declared) die("no verify command declared (kineti.config.json settings.verify_command or KINETI_VERIFY_CMD)", 2);
  const t = readJson<Trust>(trustFile()) ?? {};
  t[repoKey()] = { cmd_hash: sha256(declared), at: nowIso() };
  writeJson(trustFile(), t);
  ok(`trusted for this repo: ${declared}`);
  return;
}
```
- **Root Cause & Impact:**
  The verify gate exists to prevent untrusted repository verification commands from executing without explicit human operator consent. However, `--trust` requires no interactive TTY check, human confirmation prompt, or privileged secret. Any autonomous LLM agent with shell access can issue `bun kineti-verify-gate.ts --trust` to authorize any arbitrary malicious command, followed by `bun kineti-verify-gate.ts` to execute it in `bash -lc`.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-verify-gate.ts
+++ b/bin/kineti-verify-gate.ts
@@ -20,6 +20,9 @@ function main() {
   if (cmd === "--trust") {
     if (!declared) die("no verify command declared (kineti.config.json settings.verify_command or KINETI_VERIFY_CMD)", 2);
+    if (!process.stdin.isTTY && !process.env.KINETI_TRUST_CONFIRMED) {
+      die("security: --trust must be executed interactively in a human TTY session", 2);
+    }
     const t = readJson<Trust>(trustFile()) ?? {};
     t[repoKey()] = { cmd_hash: sha256(declared), at: nowIso() };
     writeJson(trustFile(), t);
```
- **Automated Test Suite Consideration & Companion Diff:**
  Because automated test runners (such as `tests/harness.test.ts:132, 137`) spawn child processes with piped stdio (`process.stdin.isTTY === undefined`), adding the interactive TTY guard will cause integration tests to fail with exit code 2 unless an authorized test environment token is supplied.
  
  **1. Test Runner Environment Configuration:**
  Automated CI and test suites can pass the bypass token via environment variable:
  ```bash
  KINETI_TRUST_CONFIRMED=1 bun test tests/
  ```

  **2. Companion Diff for `tests/harness.test.ts`:**
  Alternatively, update the test harness helper (`tests/harness.test.ts:18`) to explicitly pass `KINETI_TRUST_CONFIRMED: "1"` in simulated test runs, alongside a dedicated test asserting that unauthenticated non-interactive calls are rejected:
  ```diff
  --- a/tests/harness.test.ts
  +++ b/tests/harness.test.ts
  @@ -18,3 +18,3 @@ function run(
  -    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine },
  +    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine, KINETI_TRUST_CONFIRMED: "1" },
  ```
  
  **3. Dedicated Non-Interactive Rejection Test (`tests/harness.test.ts`):**
  ```typescript
  test("verify-gate --trust rejects non-interactive execution without confirmation token", () => {
    const c = makeCtx();
    const cfg = path.join(c.cwd, "kineti.config.json");
    fs.writeFileSync(cfg, JSON.stringify({ settings: { verify_command: "true" } }));
    const p = Bun.spawnSync({
      cmd: ["bun", path.join(REPO, "bin", "kineti-verify-gate.ts"), "--trust"],
      cwd: c.cwd,
      env: { ...process.env, KINETI_MACHINE_DIR: c.machine, KINETI_TRUST_CONFIRMED: "" },
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(p.exitCode).toBe(2);
    expect(p.stderr.toString()).toContain("interactive human TTY session");
    fs.rmSync(c.root, { recursive: true, force: true });
  });
  ```
- **Verification Commands:**
```bash
# 1. Verify non-interactive execution without token is rejected:
echo "" | bun bin/kineti-verify-gate.ts --trust
# Expect: exit code 2 with security error message

# 2. Verify execution succeeds with token in automated test contexts:
KINETI_TRUST_CONFIRMED=1 bun bin/kineti-verify-gate.ts --trust
# Expect: exit code 0 ("trusted for this repo: ...")
```

---

#### [CRIT-04] Catastrophic Silent Ledger Erasure on JSONL Parse Errors
- **File & Line Numbers:** `bin/lib.ts:48-58`
- **Problematic Code Snippet:**
```typescript
// bin/lib.ts:48-57
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
- **Root Cause & Impact:**
  Wrapping the entirety of `readJsonl` in a generic `try...catch` that returns `[]` means that a single truncated line, corrupted byte, or trailing null byte causes `readJsonl` to report that the file has zero entries.
  **Downstream Impact:**
  - `bin/kineti-egress.ts:28-34`: Ledgers appear empty; sequence resets to 0 and hash to `GENESIS`, silently corrupting the Merkle tree and overwriting state.
  - `bin/kineti-saga.ts:62`: Returns `nothing to roll back`, abandoning partial mutations.
  - `bin/kineti-memory-job.ts:78`: Entire project journal memory is treated as non-existent.
- **Concrete Remediation Diff:**
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
- **Verification Command:**
```bash
bun -e '
import { readJsonl } from "./bin/lib.ts";
import fs from "node:fs"; import os from "node:os"; import path from "node:path";
const tmp = path.join(os.tmpdir(), "corrupt.jsonl");
fs.writeFileSync(tmp, "{\"a\":1}\nBAD_LINE\n{\"a\":2}\n");
const res = readJsonl(tmp);
console.log("Parsed items count:", res.length);
fs.unlinkSync(tmp);
'
# Expect: Parsed items count: 2 (recovers valid records instead of returning 0)
```

---

#### [CRIT-05] Unhandled Missing Repo Pointer & Space-Path Splitting in Cron Script
- **File & Line Numbers:** `scripts/weekly.sh:10, 13, 16`
- **Problematic Code Snippet:**
```bash
# scripts/weekly.sh:10, 13, 16
KIN="$(cat "$HOME/.kineti/repo")"
...
PROJECTS="${KINETI_PROJECTS:-$PWD}"
...
for p in $PROJECTS; do
```
- **Root Cause & Impact:**
  1. Under `set -euo pipefail`, if `$HOME/.kineti/repo` does not exist, `cat` exits 1, crashing the script immediately without an actionable diagnostic.
  2. `for p in $PROJECTS; do` performs unquoted word-splitting. In paths with whitespace (e.g. `/Users/praveen/Documents/Products/kineti local harness`), bash splits the workspace into three non-existent paths.
- **Concrete Remediation Diff:**
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
- **Verification Command:**
```bash
KINETI_PROJECTS="/tmp/path with space:/tmp/other" bash -c 'IFS=":" read -r -a arr <<< "$KINETI_PROJECTS"; echo "Count: ${#arr[@]} Item 0: ${arr[0]}"'
# Expect: Count: 2, Item 0: /tmp/path with space
```

---

### 4.2 High Severity Findings

#### [HIGH-01] Arbitrary Shell Execution & Stderr Swallowing During Saga Rollbacks
- **File & Line Numbers:** `bin/kineti-saga.ts:63-72` (interacting with `ETHOS.md:27` and `tests/harness.test.ts:78-95`)
- **Problematic Code Snippet:**
```typescript
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
- **Root Cause & Impact:**
  1. `stdio: "pipe"` buffers `stdout` and `stderr`, but neither stream is displayed or logged when `code !== 0`. When an undo step fails, operators and parent agents receive zero diagnostic context explaining why the undo failed (e.g. permission denied, missing dependency, or file lock).
  2. If an undo command hangs, `spawnSync` hangs indefinitely without a timeout, freezing automated workflows.
  3. **Architectural Harmonization with Constitutional Law (ETHOS.md Rule 4.2):**
     In transactional database systems, rollbacks frequently halt immediately upon step failure to prevent cascading corruptions. However, Kineti OS operates under explicit standing constitutional law: **`ETHOS.md` Rule 4.2** establishes:
     > *"On failure, undo steps run newest-first. If one undo fails, log it and continue with the rest."*
     
     This guarantee is explicitly verified by integration test `tests/harness.test.ts:78` (`rollback unwinds newest-first and continues past a failing undo`). An earlier remediation proposal to halt on step failure violated `ETHOS.md Rule 4.2` and broke regression tests.
     
     **Tradeoff Discussion & Harmonized Resolution:**
     - **Default Behavior (Non-Destructive Best-Effort Cleanup):** Strictly preserve `ETHOS.md Rule 4.2`. Every undo step runs newest-first. When a step fails, the system logs prominent diagnostics including the captured `stderr` (or `stdout`), records the non-zero exit code to `saga.jsonl`, and **continues** executing the remaining pending undo steps.
     - **Optional `--fail-fast` CLI Flag:** For operations where inverse actions have strict cascading dependencies (e.g. database schema migrations), provide an optional `--fail-fast` flag. When `--fail-fast` is passed, rollback halts immediately upon the first non-zero exit code.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-saga.ts
+++ b/bin/kineti-saga.ts
@@ -62,12 +62,20 @@ function main() {
     if (pending.length === 0) { ok(`nothing to roll back for ${runId}`); return; }
+    const failFast = rest.includes("--fail-fast");
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
+        const errDetail = (res.stderr || res.stdout || "").trim();
+        console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}); continuing${errDetail ? `:\n${errDetail}` : ""}`);
+        if (failFast) die(`rollback halted: step "${r.label}" failed under --fail-fast; manual intervention required`, 1);
+      } else {
+        ok(`undone: ${r.label}`);
       }
     }
     appendJsonl(file(), { at: nowIso(), kind: "rollback_done", run_id: runId } satisfies Line);
```
- **Verification Commands:**
```bash
# 1. Default ETHOS Rule 4.2 continuation with diagnostic stderr logging:
bun bin/kineti-saga.ts begin --run-id r-fail
bun bin/kineti-saga.ts register --run-id r-fail --label step1 --inverse "echo 'undoing step1'"
bun bin/kineti-saga.ts register --run-id r-fail --label bad --inverse "echo 'bad error details' >&2; exit 2"
bun bin/kineti-saga.ts register --run-id r-fail --label step3 --inverse "echo 'undoing step3'"
bun bin/kineti-saga.ts rollback --run-id r-fail
# Expect: unwinds step3, prints captured stderr 'bad error details' and exit 2, continues and unwinds step1, exit 0

# 2. Optional --fail-fast flag:
bun bin/kineti-saga.ts begin --run-id r-ff
bun bin/kineti-saga.ts register --run-id r-ff --label step1 --inverse "echo 'step1'"
bun bin/kineti-saga.ts register --run-id r-ff --label bad --inverse "echo 'bad error' >&2; exit 2"
bun bin/kineti-saga.ts rollback --run-id r-ff --fail-fast
# Expect: halts immediately upon step 'bad' with exit code 1
```

---

#### [HIGH-02] Command Injection, Argument Flattening & Diagnostic Blindness in Proof Capture
- **File & Line Numbers:** `bin/kineti-evidence.ts:61, 64, 71`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-evidence.ts:61, 64, 71
const command = argv.slice(dd + 1).join(" ");
...
const res = Bun.spawnSync(["bash", "-lc", command], { stdout: "pipe", stderr: "pipe" });
const code = res.exitCode;
...
if (code !== 0) die(`command failed (exit ${code}); recorded as proof anyway`, 1);
```
- **Root Cause & Impact:**
  Merging arguments with `.join(" ")` strips quotes and word boundaries, corrupting complex commands. Capturing output to pipes without forwarding on failure causes diagnostic blindness: when a test fails during proof generation, the failure output is discarded, leaving developers with only `command failed (exit 1)`.
- **Concrete Remediation Diff:**
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
- **Verification Command:**
```bash
bun bin/kineti-evidence.ts run --label test-fail -- bash -c 'echo "Specific Error Message" >&2; exit 1'
# Expect: Specific Error Message is forwarded to stderr before process exits
```

---

#### [HIGH-03] Path Duplication Bug Creating Nested `.kineti/.kineti/spend.log.jsonl`
- **File & Line Numbers:** `bin/kineti-spend.ts:29` (interacting with `bin/lib.ts:10-12`)
- **Problematic Code Snippet:**
```typescript
// bin/lib.ts:10-12
export function projectKdir(): string { return path.join(process.cwd(), ".kineti"); }

// bin/kineti-spend.ts:28-29
function file(): string { return path.join(projectKdir(), "spend.json"); }
function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }
```
- **Root Cause & Impact:**
  `projectKdir()` already returns `$PWD/.kineti`. Concatenating `".kineti"` a second time resolves to `$PWD/.kineti/.kineti/spend.log.jsonl`. This pollutes repository structure, breaks log aggregation tools, and violates repository layout standards.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-spend.ts
+++ b/bin/kineti-spend.ts
@@ -29,1 +29,1 @@
-function logFile(): string { return path.join(projectKdir(), ".kineti", "spend.log.jsonl"); }
+function logFile(): string { return path.join(projectKdir(), "spend.log.jsonl"); }
```
- **Verification Command:**
```bash
bun bin/kineti-spend.ts log --stage build --tokens-in 10 --tokens-out 10
test -f .kineti/spend.log.jsonl && test ! -d .kineti/.kineti
# Expect: log file exists directly in .kineti/
```

---

#### [HIGH-04] Code Fingerprint Invalidation Caused by `.agents/` and `design/screenshots/`
- **File & Line Numbers:** `bin/kineti-evidence.ts:11-15`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-evidence.ts:11-14
const EXCLUDE_DIRS = new Set([
  ".git", ".kineti", "node_modules", "dist", "build", ".next",
  "coverage", "tmp", ".cache", "legacy",
]);
```
- **Root Cause & Impact:**
  During autonomous agent execution, agents continually update heartbeats, progress logs, and handoffs inside `.agents/`. Similarly, Stage 9 (`qa`) generates browser visual verification images in `design/screenshots/`. Because neither directory is excluded from `fingerprint()`, any agent action or screenshot generation causes the code fingerprint to change, immediately causing `kineti-evidence check --label qa` to report `STALE (code changed after the run)` and blocking Stage 11 (`ship`).
- **Concrete Remediation Diff:**
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
- **Verification Command:**
```bash
bun -e '
import { fingerprint } from "./bin/kineti-evidence.ts";
import fs from "node:fs";
const fp1 = fingerprint();
fs.mkdirSync(".agents/test", { recursive: true });
fs.writeFileSync(".agents/test/heartbeat.md", "ping");
const fp2 = fingerprint();
console.log("Fingerprints equal after .agents edit:", fp1 === fp2);
fs.rmSync(".agents/test", { recursive: true });
'
# Expect: Fingerprints equal after .agents edit: true
```

---

#### [HIGH-05] Spend Circuit Breaker Reset Bypass via Static CLI Flag
- **File & Line Numbers:** `bin/kineti-spend.ts:97-103`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-spend.ts:97-103
if (cmd === "reset") {
  if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
  s.tripped = false; s.reason = null;
  writeJson(file(), s);
  ok("breaker reset by human");
  return;
}
```
- **Root Cause & Impact:**
  The spend circuit breaker is a hard financial safety barrier designed to prevent uncontrolled spending. However, the reset mechanism relies entirely on the presence of the static argument `--i-am-human`. When an autonomous LLM loop trips the breaker, it can parse the error message and invoke `bun kineti-spend.ts reset --i-am-human`, nullifying the $50 spend circuit breaker without human awareness.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-spend.ts
+++ b/bin/kineti-spend.ts
@@ -97,3 +97,6 @@ function main() {
   if (cmd === "reset") {
     if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
+    if (!process.stdin.isTTY && !process.env.KINETI_HUMAN_RESET_TOKEN) {
+      die("security: breaker reset requires an interactive human TTY confirmation", 2);
+    }
     s.tripped = false; s.reason = null;
```
- **Automated Test Suite Consideration & Companion Diff:**
  In automated integration tests (such as `tests/harness.test.ts:71`, which tests `run("kineti-spend.ts", ["reset", "--i-am-human"], c)`), subprocesses run without a TTY (`process.stdin.isTTY === undefined`). Introducing this guard without test environment coordination causes the existing test suite to fail with exit code 2.

  **1. Test Runner Environment Configuration:**
  In automated CI and testing pipelines, execute `bun test` with the reset authorization token:
  ```bash
  KINETI_HUMAN_RESET_TOKEN=1 bun test tests/
  ```

  **2. Companion Diff for `tests/harness.test.ts`:**
  Update the test harness child process runner (`tests/harness.test.ts:18`) to pass `KINETI_HUMAN_RESET_TOKEN: "1"`:
  ```diff
  --- a/tests/harness.test.ts
  +++ b/tests/harness.test.ts
  @@ -18,3 +18,3 @@ function run(
  -    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine },
  +    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine, KINETI_HUMAN_RESET_TOKEN: "1" },
  ```

  **3. Dedicated Non-Interactive Rejection Test (`tests/harness.test.ts`):**
  Add an explicit test verifying that unauthenticated non-interactive resets without `KINETI_HUMAN_RESET_TOKEN` are refused:
  ```typescript
  test("spend reset rejects non-interactive execution without human confirmation token", () => {
    const c = makeCtx();
    // Trip the breaker
    run("kineti-spend.ts", ["log", "--tokens-in", "1", "--tokens-out", "1", "--usd", "55.0"], c);
    // Non-interactive reset without token must fail
    const p = Bun.spawnSync({
      cmd: ["bun", path.join(REPO, "bin", "kineti-spend.ts"), "reset", "--i-am-human"],
      cwd: c.cwd,
      env: { ...process.env, KINETI_MACHINE_DIR: c.machine, KINETI_HUMAN_RESET_TOKEN: "" },
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(p.exitCode).toBe(2);
    expect(p.stderr.toString()).toContain("interactive human TTY confirmation");
    fs.rmSync(c.root, { recursive: true, force: true });
  });
  ```
- **Verification Commands:**
```bash
# 1. Verify unauthenticated non-interactive reset is blocked:
echo "" | bun bin/kineti-spend.ts reset --i-am-human
# Expect: exit 2 with "security: breaker reset requires an interactive human TTY confirmation"

# 2. Verify reset succeeds with authorization token in automated test environments:
KINETI_HUMAN_RESET_TOKEN=1 bun bin/kineti-spend.ts reset --i-am-human
# Expect: exit 0 ("breaker reset by human")
```

---

#### [HIGH-06] Spec Skill Directs Unsupported Gate State Transition `gate.spec pending`
- **File & Line Numbers:** `skills/spec/SKILL.md:49` (interacting with `bin/kineti-state.ts:95-98`)
- **Problematic Code Snippet:**
```markdown
<!-- skills/spec/SKILL.md:49 -->
7. **Write `spec.md`.** Then:
   `bun "$K/kineti-state.ts" set gate.spec pending`
```
```typescript
// bin/kineti-state.ts:97
if (value !== "pass" && value !== "fail") die("gate value must be pass|fail", 2);
```
- **Root Cause & Impact:**
  `kineti-state.ts` permits only `pass` or `fail` as valid gate values. When an agent executes the documented procedure in `spec/SKILL.md`, the command crashes with exit code 2: `kineti: gate value must be pass|fail`, halting execution at Stage 6.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-state.ts
+++ b/bin/kineti-state.ts
@@ -97,3 +97,3 @@
-      if (value !== "pass" && value !== "fail") die("gate value must be pass|fail", 2);
+      if (value !== "pass" && value !== "fail" && value !== "pending") die("gate value must be pass|fail|pending", 2);
```
- **Verification Command:**
```bash
bun bin/kineti-state.ts set gate.spec pending
# Expect: exit code 0
```

---

#### [HIGH-07] Uncaught Runtime `TypeError` on Missing CLI Option Argument
- **File & Line Numbers:** `bin/kineti-memory-job.ts:74-75`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-memory-job.ts:74-75
const di = process.argv.indexOf("--dir");
const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
```
- **Root Cause & Impact:**
  When `--dir` is passed as the final command-line flag without a following directory value, `process.argv[di + 1]` evaluates to `undefined`. `path.resolve(undefined)` crashes immediately with an unhandled runtime error: `TypeError: The "paths[0]" property must be of type string, got undefined`.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-memory-job.ts
+++ b/bin/kineti-memory-job.ts
@@ -74,3 +74,6 @@ function main() {
   const di = process.argv.indexOf("--dir");
-  const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
+  if (di > -1 && (!process.argv[di + 1] || process.argv[di + 1].startsWith("-"))) {
+    die("--dir requires a directory path argument", 2);
+  }
+  const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
```
- **Verification Command:**
```bash
bun bin/kineti-memory-job.ts sweep --dir
# Expect: clean exit with exit code 2 and helpful error message
```

---

#### [HIGH-08] Installer Smoke Test Excluded from Default Runner
- **File & Line Numbers:** `package.json:8`, `tests/test-setup.sh`
- **Problematic Code Snippet:**
```json
// package.json:8
"scripts": {
  "test": "bun test tests/"
}
```
- **Root Cause & Impact:**
  `bun test tests/` discovers and runs only JavaScript/TypeScript test files (`tests/harness.test.ts` and `tests/memory-job.test.ts`). The shell test script `tests/test-setup.sh` (which verifies installer idempotence, uninstallation, and host creation) is completely skipped, leaving installation logic unvalidated in CI.
- **Concrete Remediation Diff:**
```diff
--- a/package.json
+++ b/package.json
@@ -8,3 +8,3 @@
-    "test": "bun test tests/",
+    "test": "bun test tests/ && bash tests/test-setup.sh",
```
*(Or introduce `tests/setup.test.ts` to wrap the bash runner natively).*
- **Verification Command:**
```bash
bun test
# Expect: executes installer smoke test alongside unit suites
```

---

#### [HIGH-09] Critical Enforcement Subsystems Completely Untested
- **File & Line Numbers:** `bin/kineti-saga.ts:48`, `bin/kineti-spend.ts:75`, `bin/kineti-evidence.ts:89`, `bin/kineti-verify-gate.ts:40`, `bin/kineti-egress.ts:47`
- **Root Cause & Impact:**
  Inspection of `tests/harness.test.ts` revealed zero test coverage for the following critical paths:
  1. `kineti-saga commit` and post-commit immutability enforcement.
  2. `kineti-spend` per-stage ceiling enforcement (`per_stage` and `per_stage_default`).
  3. `kineti-evidence check --expect-cmd` and recorded failure handling.
  4. `kineti-verify-gate` pass-open behavior when no command is configured.
  5. `kineti-egress verify` detection of ledger truncation.
  **Impact:** High risk of undetected functional regressions in core security boundaries.
- **Remediation:** Implement comprehensive test blocks in `tests/harness.test.ts` (see Section 5 for full test skeletons).
- **Verification Command:** Run `bun test tests/` after adding test skeletons.

---

### 4.3 Medium Severity Findings

#### [MED-01] Saga Rollback Forbids Committed Runs Without Force Flag, Hindering Watch Recovery
- **File & Line Numbers:** `bin/kineti-saga.ts:15-20, 55-57` (interacting with `skills/watch/SKILL.md:39`)
- **Problematic Code Snippet:**
```typescript
// bin/kineti-saga.ts:19
if (ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
```
- **Root Cause & Architectural Harmonization:**
  1. `skills/watch/SKILL.md:39` mandates: *"if red, propose rollback using the saga undo steps from the build run."* However, Stage 7 (`build`) commits its saga run upon successful build completion.
  2. By default, committed runs MUST remain sealed to preserve transaction immutability and prevent inadvertent unwinding of validated production releases (as asserted in Section 5.5 Skeleton 1). Calling default `rollback --run-id <id>` on a committed run must exit with code 2.
  3. However, emergency operational recovery in Stage 12 (`watch`) requires an authorized mechanism to undo a committed run if post-deployment monitoring flags critical failures. Unconditionally allowing rollbacks breaks the integrity guarantee of `commit`, whereas completely disallowing rollbacks prevents emergency incident response.
  4. **Harmonized Solution:** Introduce an explicit `--force-committed` flag (`bun bin/kineti-saga.ts rollback --run-id <id> --force-committed`). When invoked:
     - The CLI checks `openRun(runId, allowCommitted = true)`.
     - An explicit, immutable audit entry `{ at: nowIso(), kind: "rollback_forced", run_id: runId }` is appended to `saga.jsonl`.
     - An operator warning is emitted to stderr, and the pending undo steps are unwound newest-first.
- **Concrete Remediation Diff:**
```diff
--- a/bin/kineti-saga.ts
+++ b/bin/kineti-saga.ts
@@ -6,4 +6,4 @@ import { appendJsonl, die, nowIso, ok, projectKdir, readJsonl } from "./lib.ts"
 interface Line {
-  at: string; kind: "begin" | "register" | "commit" | "rollback_step" | "rollback_done";
+  at: string; kind: "begin" | "register" | "commit" | "rollback_step" | "rollback_done" | "rollback_forced";
   run_id: string; label?: string; inverse?: string; exit_code?: number | null;
 }
@@ -15,5 +15,5 @@
-function openRun(runId: string): void {
+function openRun(runId: string, allowCommitted = false): void {
   const ls = lines();
   const begin = ls.find((l) => l.kind === "begin" && l.run_id === runId);
   if (!begin) die(`unknown run: ${runId}. Begin it first.`, 2);
-  if (ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
+  if (!allowCommitted && ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
 }
@@ -55,3 +55,8 @@
   if (cmd === "rollback") {
-    openRun(runId);
+    const forceCommitted = rest.includes("--force-committed");
+    openRun(runId, forceCommitted);
+    if (forceCommitted && lines().some((l) => l.kind === "commit" && l.run_id === runId)) {
+      appendJsonl(file(), { at: nowIso(), kind: "rollback_forced", run_id: runId } satisfies Line);
+      console.warn(`kineti: WARNING executing forced rollback on committed run "${runId}"`);
+    }
```
- **Verification Commands:**
```bash
# 1. Verify default rollback on committed run remains sealed (exit 2):
bun bin/kineti-saga.ts begin --run-id r-com
bun bin/kineti-saga.ts register --run-id r-com --label step1 --inverse "echo 'undo'"
bun bin/kineti-saga.ts commit --run-id r-com
bun bin/kineti-saga.ts rollback --run-id r-com
# Expect: exit 2 ("kineti: run r-com already committed")

# 2. Verify emergency rollback succeeds with explicit --force-committed (exit 0) and audits:
bun bin/kineti-saga.ts rollback --run-id r-com --force-committed
# Expect: exit 0 ("rollback complete for r-com") and logs kind: "rollback_forced" to saga.jsonl
```

---

#### [MED-02] Zero Enforcement Tooling for Standing ETHOS Rule 8 ("Clean Files")
- **File & Line Numbers:** `ETHOS.md:44-48`
- **Problematic Text:**
```markdown
## 8. Clean files
8.1 Committed files contain no personal names, no home paths (use $HOME), no real client names, no secrets.
8.2 Verified by search before every commit. Zero matches required.
```
- **Root Cause & Impact:**
  While `ETHOS.md` declares Rule 8 as absolute standing law, no script, pre-commit hook, or CLI program exists to scan for `$HOME` violations, usernames, or API keys. `skills/anchors/SKILL.md:38` explicitly warns: *"Never write a rule no program or procedure enforces."*
- **Remediation:** Create `scripts/clean-check.sh` to scan staged git files for absolute user directories and high-entropy secret patterns, and integrate it into `skills/ship/SKILL.md`.

---

#### [MED-03] Missing UX Blueprint Integration in Intake & Design Skills
- **File & Line Numbers:** `skills/officehours/SKILL.md`, `skills/design/SKILL.md` (interacting with `ETHOS.md:9`)
- **Root Cause & Impact:**
  `ETHOS.md` Rule 1.3 and `WORKFLOWS.md` Stage 1 mandate a **UX-First Blueprint** (Persona, Visual Journey Flowchart, Screen Interaction Matrix, 3-Layer Split) before creating screen variants. However, neither `skills/officehours/SKILL.md` nor `skills/design/SKILL.md` contains any procedural instructions for creating or consuming the blueprint.
- **Remediation:** Update `skills/officehours/SKILL.md` Step 6 to mandate UX Blueprint generation in `brief.md`, and update `skills/design/SKILL.md` Step 3 to validate screen variants against the blueprint.

---

#### [MED-04] Non-Atomic File Overwrite in Memory Journal Overwriting Data In-Place
- **File & Line Numbers:** `bin/kineti-memory-job.ts:43-46`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-memory-job.ts:43-46
function save(dir: string, recs: Rec[]): void {
  const body = recs.map((r) => JSON.stringify(r)).join("\n") + (recs.length ? "\n" : "");
  fs.writeFileSync(journalFile(dir), body);
}
```
- **Root Cause & Impact:**
  Directly writing to `journalFile(dir)` truncates the file in-place before writing. If the process is killed (SIGKILL, OOM, power loss) mid-write, `journal.jsonl` is left empty or corrupted, permanently destroying historical run records.
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

---

#### [MED-05] Symlink Traversal & FIFO Blocking in Repository Fingerprinting
- **File & Line Numbers:** `bin/kineti-evidence.ts:22-39`
- **Problematic Code Snippet:**
```typescript
// bin/kineti-evidence.ts:33-36
const st = fs.statSync(full);
if (st.size > MAX_FILE_BYTES) continue;
const rel = path.relative(root, full);
parts.push(`${rel}:${sha256(fs.readFileSync(full))}`);
```
- **Root Cause & Impact:**
  `fs.statSync` follows symbolic links. If a repository includes a symlink to an external sensitive file or a named pipe (`mkfifo`), `fs.readFileSync` will either read external data into the project hash or block indefinitely waiting on FIFO input.
- **Remediation Diff:**
```diff
--- a/bin/kineti-evidence.ts
+++ b/bin/kineti-evidence.ts
@@ -31,4 +31,5 @@ export function fingerprint(root: string = process.cwd()): string {
-      if (e.isFile() && !EXCLUDE_FILES.has(e.name)) {
+      if (e.isFile() && !e.isSymbolicLink() && !EXCLUDE_FILES.has(e.name)) {
         try {
-          const st = fs.statSync(full);
+          const st = fs.lstatSync(full);
+          if (!st.isFile()) continue;
           if (st.size > MAX_FILE_BYTES) continue;
```

---

#### [MED-06] Unchecked Argument Parsing Crash in Installer
- **File & Line Numbers:** `setup.sh:18`
- **Problematic Code Snippet:**
```bash
# setup.sh:18
--host) ONLY_HOST="${2:-}"; shift 2 ;;
```
- **Root Cause & Impact:**
  When `--host` is passed without a parameter as the final argument, `$#` is 1. Calling `shift 2` causes bash to exit with code 1 (`shift: shift count must be <= $#`) without displaying the usage message.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -18,2 +18,4 @@ while [[ $# -gt 0 ]]; do
-    --host) ONLY_HOST="${2:-}"; shift 2 ;;
+    --host)
+      [[ $# -ge 2 ]] || { echo "Error: --host requires an argument" >&2; usage; }
+      ONLY_HOST="$2"; shift 2 ;;
```

---

#### [MED-07] Unhandled Exit Code 1 in `read_conf` Under `pipefail`
- **File & Line Numbers:** `setup.sh:25-27`
- **Problematic Code Snippet:**
```bash
# setup.sh:25-27
read_conf() { # $1 = conf file, $2 = key -> prints value (quotes stripped)
  grep -E "^$2=" "$1" | head -1 | cut -d= -f2- | tr -d '"'
}
```
- **Root Cause & Impact:**
  If an optional key (such as `skills_dir_fallback`) is absent from a host configuration file, `grep` returns exit code 1. Because `set -euo pipefail` is enabled, the pipeline failure halts `setup.sh` abruptly.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -26,3 +26,3 @@ done
 read_conf() { # $1 = conf file, $2 = key -> prints value (quotes stripped)
-  grep -E "^$2=" "$1" | head -1 | cut -d= -f2- | tr -d '"'
+  (grep -E "^$2=" "$1" || true) | head -1 | cut -d= -f2- | tr -d '"\r'
 }
```

---

#### [MED-08] Incomplete Skill Installation Omitting Companion Subdirectories
- **File & Line Numbers:** `setup.sh:88-94`
- **Problematic Code Snippet:**
```bash
# setup.sh:92
cp "$HERE/skills/$skill/SKILL.md" "$dest/SKILL.md"
```
- **Root Cause & Impact:**
  Only `SKILL.md` is copied to host target directories. Any companion scripts (`scripts/`), reference documents (`references/`), or schema definitions (`resources/`) bundled with skills are omitted during installation.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -88,6 +88,8 @@ install() {
     for skill in "$HERE"/skills/*/; do
+      [[ -d "$skill" ]] || continue
       skill="$(basename "$skill")"
+      [[ -f "$HERE/skills/$skill/SKILL.md" ]] || continue
       dest="$dir/${PREFIX}${skill}"
       mkdir -p "$dest"
-      cp "$HERE/skills/$skill/SKILL.md" "$dest/SKILL.md"
+      cp -R "$HERE/skills/$skill/." "$dest/"
       count=$((count+1))
```

---

#### [MED-09] Incomplete Uninstall Leaving Orphan Repository Pointer
- **File & Line Numbers:** `setup.sh:55-69`
- **Root Cause & Impact:**
  While `install()` writes `$HOME/.kineti/repo`, `uninstall()` does not delete this pointer file. If Kineti is uninstalled or moved, cron jobs running `scripts/weekly.sh` continue targeting the stale path.
- **Remediation Diff:**
```diff
--- a/setup.sh
+++ b/setup.sh
@@ -67,3 +67,6 @@ uninstall() {
   echo "Kineti uninstalled. Removed $removed skill folders."
+  if [[ -f "$HOME/.kineti/repo" ]]; then
+    rm -f "$HOME/.kineti/repo"
+    echo "Removed repository pointer: $HOME/.kineti/repo"
+  fi
```

---

#### [MED-10] Fragile Bun Path Resolution in Non-Interactive Cron
- **File & Line Numbers:** `scripts/weekly.sh:11-12`
- **Problematic Code Snippet:**
```bash
# scripts/weekly.sh:11-12
BUN="${KINETI_BUN:-$HOME/.bun/bin/bun}"
command -v "$BUN" >/dev/null 2>&1 || BUN="$(command -v bun)"
```
- **Root Cause & Impact:**
  In minimal non-interactive cron environments where `PATH=/usr/bin:/bin` and Bun is installed elsewhere, `command -v bun` exits 1, causing `weekly.sh` to terminate silently under `set -e`.
- **Remediation:** Check whether Bun is executable before command substitution and emit a descriptive error message if missing.

---

#### [MED-11] Temporary Directory Resource Leakage in Test Suite
- **File & Line Numbers:** `tests/memory-job.test.ts:69`, `tests/harness.test.ts:49, 73, 98, ...`
- **Root Cause & Impact:**
  Test temporary directories are cleaned up at the very end of test blocks without `try...finally` or `afterEach` handlers. When an assertion fails midway through a test, the teardown statement is never reached, leaving orphaned folders in `os.tmpdir()`. (9 leaked directories were verified on disk).
- **Remediation:** Wrap all test logic in `try...finally` or utilize Bun's `afterEach` lifecycle hook.

---

#### [MED-12] TypeScript Strictness Gaps Masking Indexing Hazards
- **File & Line Numbers:** `tsconfig.json:8`, `bin/*.ts`
- **Root Cause & Impact:**
  `tsconfig.json` omits `"noUncheckedIndexedAccess": true`. Running `tsc --noEmit --noUncheckedIndexedAccess` reveals **10 type errors** across `bin/`, directly masking array out-of-bounds errors such as Finding HIGH-07.
- **Remediation:** Add `"noUncheckedIndexedAccess": true` to `tsconfig.json` and resolve undefined array index accesses across `bin/*.ts`.

---

### 4.4 Low Severity Findings

#### [LOW-01] Missing Executable Permission on CLI Binary
- **File:** `bin/kineti-memory-job.ts`
- **Observation:** Mode is `-rw-r--r--` (`0644`). All other CLI programs in `bin/` are `-rwxr-xr-x` (`0755`).
- **Remediation:** `chmod 755 bin/kineti-memory-job.ts`.

#### [LOW-02] Inappropriate Executable Bit on Library Module
- **File:** `bin/lib.ts`
- **Observation:** Mode is `-rwxr-xr-x` (`0755`) despite being a shared module without a shebang.
- **Remediation:** `chmod 644 bin/lib.ts`.

#### [LOW-03] Hash Delimiter Collision Risk in Egress Receipt Ledger
- **File & Line Numbers:** `bin/kineti-egress.ts:14-16`
- **Root Cause:** Using unescaped pipe delimiters (`${r.seq}|${r.at}|...`) allows field boundary collisions if `host` or `description` contains `|`.
- **Remediation:** Serialize fields with `JSON.stringify([r.seq, r.at, r.host, r.description, r.prev_hash])`.

#### [LOW-04] Unsanitized Log Injection (CWE-117) in Machine Alerts
- **File & Line Numbers:** `bin/kineti-verify-gate.ts:59`, `bin/kineti-spend.ts:112`
- **Root Cause:** Writing unescaped strings directly to `alerts.log` allows injecting forged newline entries.
- **Remediation:** Sanitize newlines: `const clean = text.replace(/[\r\n]+/g, " ");`.

#### [LOW-05] Record Type Schema Naming Mismatch
- **File & Line Numbers:** `kineti.config.json:40` vs `bin/kineti-memory-job.ts:16`
- **Root Cause:** `kineti.config.json` uses `"project-dossier"` while implementation and documentation use `"dossier"`.
- **Remediation:** Update `kineti.config.json` line 40 to `"dossier"`.

#### [LOW-06] Stale Reference in Officehours Skill to `journal.md`
- **File & Line Numbers:** `skills/officehours/SKILL.md:32`
- **Root Cause:** Points to non-existent `journal.md` instead of active fallback `journal.jsonl`.
- **Remediation:** Replace `journal.md` with `journal.jsonl`.

#### [LOW-07] Incomplete Skill Catalog Listing in Claude Hook
- **File & Line Numbers:** `hooks/claude.txt:4-5`
- **Root Cause:** Lists 14 skills, omitting `anchors`, `second-opinion`, and `skillify`.
- **Remediation:** Update list to include all 17 installed skills.

#### [LOW-08] Dead and Conflicting `hook_file` Directives in Host Confs
- **File & Line Numbers:** `hosts/*.conf:5`
- **Root Cause:** References files like `settings.json` or `config.toml` which conflict with `hooks/*.txt` and are ignored by `setup.sh`.
- **Remediation:** Update host definitions to align with instruction targets or remove unused key.

---

### 4.5 Informational Severity Findings

#### [INFO-01] Harness Program Count Mismatch in Documentation
- **File:** `package.json:5`, `bin/README.md:3`
- **Detail:** Mentions "six enforcement programs", omitting `kineti-memory-job.ts`. Update text to "seven harness programs".

#### [INFO-02] Absolute Path Reference `/design` in Standing Law
- **File:** `ETHOS.md:9`
- **Detail:** Refers to absolute path `/design`. Should be relative path `design/screens/`.

#### [INFO-03] Unquoted Variable Expansion in Daily Loop Documentation
- **File:** `docs/HOWTO-daily-loop.md:37-38`
- **Detail:** `bun $K/kineti-evidence.ts` should be quoted: `bun "$K/kineti-evidence.ts"`.

#### [INFO-04] Repository Version Desynchronization
- **File:** `package.json:3`, `kineti.config.json:2`
- **Detail:** Files specify version `3.0.0` while git tag is `v3.0.1` and roadmap references post-3.0.1. Bump to `3.0.1`.

#### [INFO-05] Gate Count Ambiguity in Documentation vs Configuration
- **File:** `kineti.config.json:10-16`, `WORKFLOWS.md:3`
- **Detail:** Documentation refers to "Three gates" (Feasibility, Spec, Ship), but `kineti.config.json` sets `gate: true` on 4 stages (including `security`). Clarify gate semantics.

#### [INFO-06] Philosophical Directives Conflict on 12 UI Components
- **File:** `skills/design/SKILL.md:65-66`
- **Detail:** States the 12 UI components are optional fallback, conflicting with Kineti Master Directives. Harmonize wording.

#### [INFO-07] Hardcoded Skill Counts in Installer Smoke Test
- **File:** `tests/test-setup.sh:18, 24`
- **Detail:** Test hardcodes `installed: 17 skills`. Calculate dynamically via directory count.

#### [INFO-08] Unprotected Shell Glob Expansions Missing `nullglob`
- **File:** `setup.sh:47, 61, 88`, `scripts/audit-skills.sh:7`
- **Detail:** Add `shopt -s nullglob` to prevent globs from expanding to literal strings on zero matches.

#### [INFO-09] Unused Test Import in Harness Test Suite
- **File:** `tests/harness.test.ts:1`
- **Detail:** Remove unused `beforeAll` import.

#### [INFO-10] Zero Automated Test Coverage for Utility Shell Scripts
- **File:** `scripts/audit-skills.sh`, `scripts/weekly.sh`
- **Detail:** Shell scripts lack automated unit or smoke test coverage.

---

## 5. Baseline Test & Type Integrity Verification

### 5.1 Baseline Test Suite Execution (`bun test tests/`)
- **Command Executed:** `bun test tests/`
- **Exit Status:** `1` (FAILED)
- **Duration:** `869.00ms`
- **Summary:** `7 pass`, `1 fail`, `61 expect() calls` across 2 files

#### Verbatim Test Output:
```text
bun test v1.4.0 (1381054db)

tests/memory-job.test.ts:
51 |       data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
52 |       expires: iso(10),
53 |     };
54 |     write(dir, [r1, r2, oldLearning]);
55 | 
56 |     expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
                                                            ^
error: expect(received).toBe(expected)

Expected: 0
Received: 3

      at <anonymous> (/Users/praveen/Documents/Products/kineti local harness/tests/memory-job.test.ts:56:56)
(fail) kineti-memory-job > sweep promotes expired actives; chain verifies then detects tamper (exit 3) [21.02ms]
(pass) kineti-memory-job > time-order flags effect-before-cause; promote surfaces frequent new words [29.34ms]

tests/harness.test.ts:
(pass) kineti-state > goal locks forever; mutation refused with exit 3 [82.80ms]
(pass) kineti-spend > synthetic loop trips breaker; human-only reset [316.89ms]
(pass) kineti-saga > rollback unwinds newest-first and continues past a failing undo [126.16ms]
(pass) kineti-evidence > FRESH flips to STALE when code changes; MISSING when absent [84.12ms]
(pass) kineti-verify-gate > untrusted blocks (9); trusted failing blocks (1); trusted passing opens (0) [120.73ms]
(pass) kineti-egress > chain verifies; any edit breaks detection with exit 3 [75.91ms]

 7 pass
 1 fail
 61 expect() calls
Ran 8 tests across 2 files. [869.00ms]
```

---

### 5.2 Baseline TypeScript Verification (`bun run typecheck`)
- **Command Executed:** `bun run typecheck` (`tsc --noEmit`)
- **Exit Status:** `0` (PASSES under default configuration)
- **Output:**
```text
$ tsc --noEmit
```

---

### 5.3 Strictness Audit (`tsc --noEmit --noUncheckedIndexedAccess`)
When verified against strict indexed access checks, **10 type safety errors** emerge:
- **Command Executed:** `bun x tsc --noEmit --noUncheckedIndexedAccess`
- **Exit Status:** `2` (FAILED)
- **Verbatim Output:**
```text
bin/kineti-egress.ts(29,37): error TS2532: Object is possibly 'undefined'.
bin/kineti-egress.ts(47,92): error TS2532: Object is possibly 'undefined'.
bin/kineti-evidence.ts(89,23): error TS18048: 'last' is possibly 'undefined'.
bin/kineti-evidence.ts(90,43): error TS18048: 'last' is possibly 'undefined'.
bin/kineti-evidence.ts(92,9): error TS18048: 'last' is possibly 'undefined'.
bin/kineti-evidence.ts(93,9): error TS18048: 'last' is possibly 'undefined'.
bin/kineti-memory-job.ts(75,38): error TS2345: Argument of type 'string | undefined' is not assignable to parameter of type 'string'.
  Type 'undefined' is not assignable to type 'string'.
bin/kineti-spend.ts(39,63): error TS2322: Type '{ in: number; out: number; } | undefined' is not assignable to type '{ in: number; out: number; }'.
  Type 'undefined' is not assignable to type '{ in: number; out: number; }'.
bin/kineti-spend.ts(77,14): error TS2532: Object is possibly 'undefined'.
tests/harness.test.ts(156,28): error TS2345: Argument of type 'string | undefined' is not assignable to parameter of type 'string'.
  Type 'undefined' is not assignable to type 'string'.
```

---

### 5.4 Test Catalog & Test Coverage Mapping

#### Test Files in Repository:
1. `tests/harness.test.ts` (165 lines, 6 tests, 57 assertions)
2. `tests/memory-job.test.ts` (94 lines, 2 tests, 4 assertions, 1 failure)
3. `tests/test-setup.sh` (37 lines, 5 shell smoke checks, excluded from `bun test`)

#### Critical Path Coverage Analysis:
```
+-----------------------+---------------+-----------------------------------------------+
| Component             | Test Coverage | Critical Untested Paths (0% Coverage)         |
+-----------------------+---------------+-----------------------------------------------+
| kineti-state.ts       | 60%           | init --goal, validate, full JSON dump, gate.* |
| kineti-spend.ts       | 45%           | Per-stage ceilings, pricing models, status    |
| kineti-saga.ts        | 50%           | commit command, post-commit immutability      |
| kineti-evidence.ts    | 55%           | check --expect-cmd, failed run recording      |
| kineti-verify-gate.ts | 65%           | Pass-open when undeclared, KINETI_VERIFY_CMD  |
| kineti-egress.ts      | 60%           | list command, ledger truncation detection     |
| kineti-memory-job.ts  | 35% (Broken)  | Aging beyond 90d, cold/archive transitions    |
| lib.ts                | 0%            | readJsonl parse errors, loadLimits overrides  |
| setup.sh              | Manual only   | Invalid host flags, missing conf handling     |
| weekly.sh             | 0%            | Multi-project iteration, whitespace paths     |
| audit-skills.sh       | 0%            | Context threshold enforcement                 |
+-----------------------+---------------+-----------------------------------------------+
```

---

### 5.5 Concrete Test Code Skeletons to Close Critical Coverage Gaps

#### 1. Saga `commit` and Post-Commit Protection (`tests/harness.test.ts`)
```typescript
test("saga commit seals run; subsequent register or default rollback refused with exit 2 (override via --force-committed)", () => {
  const c = makeCtx();
  try {
    const s = (a: string[]) => run("kineti-saga.ts", a, c);
    expect(s(["begin", "--run-id", "r-commit"]).status).toBe(0);
    expect(s(["register", "--run-id", "r-commit", "--label", "step1", "--inverse", "true"]).status).toBe(0);
    
    // Commit seals the run
    expect(s(["commit", "--run-id", "r-commit"]).status).toBe(0);
    
    // Mutation or default rollback refused after commit (sealed run preservation)
    const blockedReg = s(["register", "--run-id", "r-commit", "--label", "step2", "--inverse", "true"]);
    expect(blockedReg.status).toBe(2);
    expect(blockedReg.err).toContain("already committed");

    const blockedRollback = s(["rollback", "--run-id", "r-commit"]);
    expect(blockedRollback.status).toBe(2);
    expect(blockedRollback.err).toContain("already committed");

    // Emergency recovery in Stage 12 (Watch) allows rollback when explicitly forced
    const forcedRollback = s(["rollback", "--run-id", "r-commit", "--force-committed"]);
    expect(forcedRollback.status).toBe(0);
    expect(forcedRollback.out).toContain("rollback complete");
    const sagaLog = fs.readFileSync(path.join(c.cwd, ".kineti", "saga.jsonl"), "utf8");
    expect(sagaLog).toContain("rollback_forced");
  } finally {
    fs.rmSync(c.root, { recursive: true, force: true });
  }
});
```

#### 2. Per-Stage Spend Limits and Machine Alerts (`tests/harness.test.ts`)
```typescript
test("per-stage spend limit trips breaker and logs to machine alerts", () => {
  const c = makeCtx();
  try {
    fs.writeFileSync(path.join(c.cwd, "kineti.config.json"), JSON.stringify({
      settings: {
        spend_limit_usd: {
          global: 50.0,
          per_stage_default: 10.0,
          per_stage: { build: 1.0 },
          safety_factor: 0.95
        }
      }
    }));
    // Build ceiling = $1.0 * 0.95 = $0.95.
    // Log $0.50 in build -> pass
    const r1 = run("kineti-spend.ts", ["log", "--stage", "build", "--tokens-in", "1", "--tokens-out", "1", "--usd", "0.50"], c);
    expect(r1.status).toBe(0);

    // Log another $0.46 -> total $0.96 >= 0.95 ceiling -> trip exit 3
    const r2 = run("kineti-spend.ts", ["log", "--stage", "build", "--tokens-in", "1", "--tokens-out", "1", "--usd", "0.46"], c);
    expect(r2.status).toBe(3);
    expect(r2.err).toContain("stage build total $0.96 reached ceiling $0.95");

    // Verify alert written to machineDir/alerts.log
    const alertContent = fs.readFileSync(path.join(c.machine, "alerts.log"), "utf8");
    expect(alertContent).toContain("SPEND TRIPPED: stage build total $0.96");
  } finally {
    fs.rmSync(c.root, { recursive: true, force: true });
  }
});
```

#### 3. Evidence Recorded Failure & Exclusions (`tests/harness.test.ts`)
```typescript
test("evidence records failed commands; respects directory exclusions; validates expect-cmd", () => {
  const c = makeCtx();
  try {
    const e = (a: string[]) => run("kineti-evidence.ts", a, c);

    // 1. Command failure in run is recorded but exits 1
    const runFail = e(["run", "--label", "lint", "--", "exit 2"]);
    expect(runFail.status).toBe(1);
    expect(runFail.err).toContain("recorded as proof anyway");

    // 2. Check detects recorded failure as STALE
    const checkFail = e(["check", "--label", "lint"]);
    expect(checkFail.status).toBe(4);
    expect(checkFail.err).toContain("recorded run failed");

    // 3. Valid run
    expect(e(["run", "--label", "test-pass", "--", "echo success"]).status).toBe(0);

    // 4. expect-cmd validation
    expect(e(["check", "--label", "test-pass", "--expect-cmd", "echo"]).status).toBe(0);
    expect(e(["check", "--label", "test-pass", "--expect-cmd", "pytest"]).status).toBe(4);

    // 5. Excluded directory changes do not invalidate freshness
    fs.mkdirSync(path.join(c.cwd, "node_modules"), { recursive: true });
    fs.writeFileSync(path.join(c.cwd, "node_modules", "dummy.txt"), "ignored");
    expect(e(["check", "--label", "test-pass"]).status).toBe(0);
  } finally {
    fs.rmSync(c.root, { recursive: true, force: true });
  }
});
```

#### 4. Verify Gate Pass-Open Behavior (`tests/harness.test.ts`)
```typescript
test("verify gate passes open when undeclared; blocks untrusted commands", () => {
  const c = makeCtx();
  try {
    // 1. No verify command configured: passes open with exit 0
    fs.writeFileSync(path.join(c.cwd, "kineti.config.json"), "{}");
    const passOpen = run("kineti-verify-gate.ts", [], c);
    expect(passOpen.status).toBe(0);
    expect(passOpen.out).toContain("gate passes open");

    // 2. Trust command without declared command fails with exit 2
    expect(run("kineti-verify-gate.ts", ["--trust"], c).status).toBe(2);

    // 3. Status without declared command
    expect(run("kineti-verify-gate.ts", ["--status"], c).out).toContain("no verify command declared");
  } finally {
    fs.rmSync(c.root, { recursive: true, force: true });
  }
});
```

#### 5. Egress Ledger Truncation Detection (`tests/harness.test.ts`)
```typescript
test("egress list outputs receipts; verify detects truncation tampering", () => {
  const c = makeCtx();
  try {
    const e = (a: string[]) => run("kineti-egress.ts", a, c);
    expect(e(["record", "--host", "api.github.com", "--desc", "fetch repo"]).status).toBe(0);
    expect(e(["record", "--host", "api.openai.com", "--desc", "embed string"]).status).toBe(0);

    // List command
    const listRes = e(["list"]);
    expect(listRes.status).toBe(0);
    expect(listRes.out).toContain("api.github.com");
    expect(listRes.out).toContain("api.openai.com");

    // Truncate ledger (delete second line while state file expects 2)
    const ledger = path.join(c.machine, "egress.jsonl");
    const lines = fs.readFileSync(ledger, "utf8").trim().split("\n");
    fs.writeFileSync(ledger, lines[0] + "\n");

    const verifyTrunc = e(["verify"]);
    expect(verifyTrunc.status).toBe(3);
    expect(verifyTrunc.err).toContain("ledger truncated or reordered");
  } finally {
    fs.rmSync(c.root, { recursive: true, force: true });
  }
});
```

---

## 6. Prioritized Action Roadmap

```
+-------------------------------------------------------------------------------+
|                          KINETI OS HARDENING ROADMAP                          |
+-------------------------------------------------------------------------------+
| Phase 1: P0 Immediate Hardening (Safety & Baseline Integrity)                |
| - [CRIT-01] Fix memory-job.test.ts hash calculation & chain oldLearning      |
| - [CRIT-02] Repair gate status lookup in bin/kineti-state.ts (get gate.<name>)|
| - [HIGH-03] Fix duplicate .kineti path concatenation in bin/kineti-spend.ts   |
| - [HIGH-04] Exclude .agents/ and design/screenshots/ in evidence fingerprint  |
| - [CRIT-03] Enforce interactive human TTY confirmation on verify-gate --trust |
| - [CRIT-04] Fix silent ledger data loss in bin/lib.ts readJsonl               |
+-------------------------------------------------------------------------------+
| Phase 2: P1 Architectural & Script Hardening (Portability & Workflows)        |
| - [CRIT-05] Fix repo pointer check and array splitting in scripts/weekly.sh   |
| - [HIGH-01] Add timeout, stderr capture, and safe continuation in saga       |
| - [HIGH-05] Enforce interactive confirmation on spend breaker --i-am-human    |
| - [HIGH-06] Add 'pending' gate status support in bin/kineti-state.ts          |
| - [MED-01]  Add --force-committed flag and audit logging in bin/kineti-saga.ts|
| - [MED-04]  Implement atomic write-and-rename in bin/kineti-memory-job.ts    |
| - [MED-05]  Use fs.lstatSync in bin/kineti-evidence.ts to block FIFO hangs    |
+-------------------------------------------------------------------------------+
| Phase 3: P2 Documentation, Hygiene & Test Expansion (Standards Compliance)    |
| - [HIGH-08] Integrate tests/test-setup.sh into package.json test script       |
| - [MED-02]  Author scripts/clean-check.sh to enforce ETHOS Rule 8             |
| - [MED-03]  Integrate UX Blueprint procedures into officehours and design     |
| - [LOW-05]  Align record_types schema in kineti.config.json ('dossier')       |
| - [LOW-07]  Update hooks/claude.txt to include all 17 installed skills       |
| - [MED-12]  Enable noUncheckedIndexedAccess and resolve TypeScript errors     |
| - [INFO-04] Synchronize repository version tags across package.json & config  |
| - Close critical test coverage gaps using Section 5 test skeletons            |
+-------------------------------------------------------------------------------+
```

---
*Report successfully compiled, reviewed, and polished by worker_polish_1.*
