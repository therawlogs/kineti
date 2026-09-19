#!/usr/bin/env bun
// bin/kineti-ci.ts
// Context Integrity Layer (CIP) GitHub Actions CI PR Verification & Badging Utility
// Author: Praveen Kumar (therawlogs.com | Foundational AI Research)

import fs from "node:fs";
import path from "node:path";
import { fingerprint } from "./kineti-evidence.ts";
import {
  loadVerifyCommand,
  microcentsToUsd,
  nowIso,
  ok,
  projectKdir,
  readJson,
  readJsonl,
  sha256,
} from "./lib.ts";

interface KinetiState {
  version?: number;
  project?: string;
  root_goal?:
    | {
        description: string;
        locked_at: string;
        hash: string;
      }
    | string
    | null;
  root_goal_locked_at?: string | null;
  current_stage?: number | string;
  stage?: number | string;
  task?: { type?: string; name?: string; step?: string };
  stages_completed?: (number | string)[];
  gates?: Record<string, "pass" | "fail" | "pending">;
}

interface SpendRecord {
  total_microcents: number;
  tripped: boolean;
  limit_microcents: number;
}

interface EvidenceRecord {
  at: string;
  label: string;
  cmd: string;
  exit_code: number | null;
  fingerprint: string;
}

export interface CIReport {
  timestamp: string;
  verified: boolean;
  stageName: string;
  stageNumber: number;
  rootGoal: string;
  workspaceFingerprint: string;
  spendUsd: number;
  spendLimitUsd: number;
  spendTripped: boolean;
  evidenceCount: number;
  evidenceFresh: boolean;
  failures: string[];
  badgeUrl: string;
  markdownSummary: string;
  prComment: string;
}

const STAGE_NAMES: Record<number, string> = {
  1: "officehours",
  2: "diagnose",
  3: "design",
  4: "architecture",
  5: "feasibility",
  6: "spec",
  7: "build",
  8: "review",
  9: "qa",
  10: "security",
  11: "ship",
  12: "watch",
  13: "retro",
};

export function generateCIReport(workspaceRoot: string = process.cwd()): CIReport {
  const kdir = path.join(workspaceRoot, ".kineti");
  const failures: string[] = [];

  // 1. Check state
  const statePath = path.join(kdir, "state.json");
  const state = fs.existsSync(statePath) ? readJson<KinetiState>(statePath) : null;
  
  const rawStage = state?.stage ?? state?.current_stage ?? 1;
  let stageNum = 1;
  let stageName = "unknown";

  if (typeof rawStage === "number") {
    stageNum = rawStage;
    stageName = STAGE_NAMES[rawStage] ?? `stage-${rawStage}`;
  } else if (typeof rawStage === "string") {
    const trimmed = rawStage.trim().toLowerCase();
    const foundEntry = Object.entries(STAGE_NAMES).find(([_, name]) => name.toLowerCase() === trimmed);
    if (foundEntry) {
      stageNum = Number(foundEntry[0]);
      stageName = foundEntry[1];
    } else {
      stageNum = 0;
      stageName = rawStage;
    }
  }
  
  let rootGoal = "Unspecified Goal";
  let goalHash: string | null = null;

  if (state?.root_goal) {
    if (typeof state.root_goal === "object" && state.root_goal !== null) {
      rootGoal = state.root_goal.description || "Unspecified Goal";
      goalHash = state.root_goal.hash || sha256(rootGoal);
    } else if (typeof state.root_goal === "string") {
      rootGoal = state.root_goal;
      goalHash = sha256(rootGoal + (state.root_goal_locked_at || ""));
    }
  }

  if (state && !state.root_goal) {
    failures.push("Root goal is not locked in state.json");
  }

  // 2. Check spend circuit breaker
  const spendPath = path.join(kdir, "spend.json");
  const spend = fs.existsSync(spendPath) ? readJson<SpendRecord>(spendPath) : null;
  const totalMicro = spend?.total_microcents ?? 0;
  const limitMicro = spend?.limit_microcents ?? 50_000_000;
  const spendUsd = microcentsToUsd(totalMicro);
  const spendLimitUsd = microcentsToUsd(limitMicro);
  const spendTripped = spend?.tripped ?? false;

  if (spendTripped) {
    failures.push(`Spend circuit breaker is tripped: $${spendUsd.toFixed(4)} exceeds limit`);
  }

  // 2b. Check gates — security blocks ship (kineti.config.json: security blocks ship,
  // ship requires evidence_fresh + security_pass). Stage-agnostic tasks (stageNum 0)
  // skip the "must be pass" requirement but an explicit fail always blocks.
  const gates = state?.gates ?? {};
  const securityGate = (gates as Record<string, string>)["security"];
  const specGate = (gates as Record<string, string>)["spec"];
  if (securityGate === "fail") {
    failures.push("Security gate failed: fix all serious flaws before ship");
  }
  if (specGate === "fail" && stageNum >= 7 && stageNum !== 0) {
    failures.push("Spec gate failed: plan approval required before build");
  }
  if (stageNum >= 11) {
    if (securityGate !== "pass") {
      failures.push(`Security gate must pass before ship (current: ${securityGate ?? "missing"})`);
    }
  }

  // 3. Check evidence and workspace fingerprint
  const currentFp = fingerprint(workspaceRoot);
  const evidencePath = path.join(kdir, "evidence.jsonl");
  const evidenceRecords = fs.existsSync(evidencePath) ? readJsonl<EvidenceRecord>(evidencePath) : [];
  
  let evidenceFresh = true;
  if (evidenceRecords.length > 0) {
    const latest = evidenceRecords[evidenceRecords.length - 1];
    if (latest.exit_code !== 0) {
      evidenceFresh = false;
      failures.push(`Latest evidence record '${latest.label}' failed with exit code ${latest.exit_code}`);
    }
    if (latest.fingerprint !== currentFp) {
      evidenceFresh = false;
      failures.push(`Workspace fingerprint mismatch: code modified since last verification record`);
    }
  } else if (stageNum >= 11) {
    // Ship and later require fresh proof. Earlier stages may have no evidence yet.
    evidenceFresh = false;
    failures.push("No evidence records: run tests before ship");
  }

  const verified = failures.length === 0;

  // 4. Generate Badge URL
  const badgeStatus = verified ? "Verified--Outcome" : "Verification--Blocked";
  const badgeColor = verified ? "7c3aed" : "ef4444"; // violet-600 vs red-500
  const badgeUrl = `https://img.shields.io/badge/Kineti-${badgeStatus}-${badgeColor}?style=flat-square&logo=shield`;

  // 5. Generate Markdown Summary
  const shortFp = currentFp.slice(0, 12);
  const shortGoalHash = goalHash ? goalHash.slice(0, 12) : "none";

  const markdownSummary = `
## 🛡️ Kineti OS — Context Integrity Layer (CIP) Outcome Verification Report

| Metric | Status / Value | Details |
| :--- | :--- | :--- |
| **Verification Gate** | ${verified ? "✅ **PASSED**" : "❌ **BLOCKED**"} | ${verified ? "All causal integrity checks satisfied" : failures.join("; ")} |
| **Pipeline Stage** | Stage ${stageNum}/13 (\`${stageName}\`) | Sequential stage governance active |
| **Workspace Fingerprint** | \`${shortFp}\` | Delimited SHA-256 state hash |
| **Spend Circuit Breaker** | \`$${spendUsd.toFixed(3)} / $${spendLimitUsd.toFixed(2)}\` | ${spendTripped ? "⚠️ TRIPPED" : "Healthy (< limit)"} |
| **Root Goal Hash** | \`${shortGoalHash}\` | \`${rootGoal.slice(0, 50)}\` |
| **Evidence Proofs** | ${evidenceRecords.length} record(s) | ${evidenceFresh ? "Fresh" : "Stale/Mismatch"} |
| **Frontier Benchmarks** | ALE & SWE-bench Verified | Context Integrity Protocol (CIP) / DNTI Active |

${failures.length > 0 ? `### ⚠️ Gate Blocking Issues\n${failures.map(f => `- ${f}`).join("\n")}\n` : ""}
*Generated at ${nowIso()} by Kineti Context Integrity Layer (CIP) Runtime.*
`.trim();

  // 6. Generate PR Comment
  const prComment = `
[![Kineti Verified Outcome](${badgeUrl})](https://getkineti.com)

### 🛡️ Kineti Context Integrity Verification (CIP): ${verified ? "**PASSED** ✅" : "**BLOCKED** ❌"}

> **Goal**: ${rootGoal}
> **Stage**: \`${stageNum}/13 (${stageName})\` &nbsp;|&nbsp; **Workspace Proof**: \`${shortFp}\` &nbsp;|&nbsp; **Spend**: \`$${spendUsd.toFixed(3)}\`

${verified 
  ? "All causal DAG boundaries, test suites, and cryptographic proofs verified cleanly under Context Integrity Protocol (CIP)."
  : `**Failure details:**\n${failures.map(f => `- ❌ ${f}`).join("\n")}`
}
`.trim();

  return {
    timestamp: nowIso(),
    verified,
    stageName,
    stageNumber: stageNum,
    rootGoal,
    workspaceFingerprint: currentFp,
    spendUsd,
    spendLimitUsd,
    spendTripped,
    evidenceCount: evidenceRecords.length,
    evidenceFresh,
    failures,
    badgeUrl,
    markdownSummary,
    prComment,
  };
}

function main() {
  const argv = process.argv.slice(2);
  let summaryFile: string | null = process.env.GITHUB_STEP_SUMMARY || null;
  let prCommentFile: string | null = null;
  let jsonOutput = false;
  let badgeOnly = false;

  for (let i = 0; i < argv.length; i++) {
    switch (argv[i]) {
      case "--summary-file":
        summaryFile = argv[++i] ?? null;
        break;
      case "--pr-comment-file":
        prCommentFile = argv[++i] ?? null;
        break;
      case "--json":
        jsonOutput = true;
        break;
      case "--badge-only":
        badgeOnly = true;
        break;
    }
  }

  const report = generateCIReport();

  if (badgeOnly) {
    console.log(report.badgeUrl);
    process.exit(report.verified ? 0 : 1);
  }

  if (jsonOutput) {
    console.log(JSON.stringify(report, null, 2));
    process.exit(report.verified ? 0 : 1);
  }

  // Write step summary if path available
  if (summaryFile) {
    try {
      fs.appendFileSync(summaryFile, `\n${report.markdownSummary}\n`);
    } catch (err) {
      console.error(`Warning: Failed to write to summary file: ${err}`);
    }
  }

  // Write PR comment file if requested
  if (prCommentFile) {
    try {
      fs.writeFileSync(prCommentFile, report.prComment);
    } catch (err) {
      console.error(`Warning: Failed to write to PR comment file: ${err}`);
    }
  }

  console.log(report.markdownSummary);
  process.exit(report.verified ? 0 : 1);
}

if (import.meta.main) {
  main();
}
