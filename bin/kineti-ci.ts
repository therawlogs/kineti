#!/usr/bin/env bun
// bin/kineti-ci.ts
// Context Integrity Layer (CIP) GitHub Actions CI PR Verification & Badging Utility
// Author: Kineti Research Team (therawlogs.com | Foundational AI Research)

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
  checksPassed: boolean;
  stageName: string;
  stageNumber: number;
  rootGoal: string;
  workspaceFingerprint: string;
  spendUsd: number;
  spendLimitUsd: number;
  spendTripped: boolean;
  evidenceCount: number;
  evidenceFresh: boolean;
  evidenceChecks: EvidenceCheck[];
  failures: string[];
  badgeUrl: string;
  markdownSummary: string;
  prComment: string;
}

export interface EvidenceCheck {
  label: string;
  status: "fresh" | "missing" | "failed" | "stale" | "fingerprint-mismatch" | "invalid-time";
  at?: string;
}

export interface CIOptions {
  requiredEvidenceLabels?: string[];
  maxEvidenceAgeMinutes?: number;
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

function markdownSafe(value: string): string {
  return value
    .replace(/[\r\n]+/g, " ")
    .replaceAll("|", "\\|")
    .replaceAll("`", "'")
    .replaceAll("@", "@​");
}

export function generateCIReport(workspaceRoot: string = process.cwd(), options: CIOptions = {}): CIReport {
  const kdir = path.join(workspaceRoot, ".kineti");
  const failures: string[] = [];

  // 1. Check state
  const statePath = path.join(kdir, "state.json");
  const state = fs.existsSync(statePath) ? readJson<KinetiState>(statePath) : null;
  if (!state) {
    failures.push("state.json missing: run `kineti seed` before the CI gate");
  }
  
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
  
  const requiredLabels = [...new Set((options.requiredEvidenceLabels ?? []).map((label) => label.trim()).filter(Boolean))];
  const maxEvidenceAgeMinutes = options.maxEvidenceAgeMinutes ?? 240;
  let evidenceChecks: EvidenceCheck[] = [];
  let evidenceFresh = true;

  if (requiredLabels.length > 0) {
    evidenceChecks = requiredLabels.map((label): EvidenceCheck => {
      const matching = evidenceRecords.filter((record) => record.label === label);
      if (matching.length === 0) return { label, status: "missing" };
      const latest = matching[matching.length - 1];
      const timestamp = Date.parse(latest.at);
      if (!Number.isFinite(timestamp) || timestamp > Date.now() + 60_000) {
        return { label, status: "invalid-time", at: latest.at };
      }
      if (latest.exit_code !== 0) return { label, status: "failed", at: latest.at };
      if (latest.fingerprint !== currentFp) return { label, status: "fingerprint-mismatch", at: latest.at };
      if ((Date.now() - timestamp) / 60_000 > maxEvidenceAgeMinutes) {
        return { label, status: "stale", at: latest.at };
      }
      return { label, status: "fresh", at: latest.at };
    });
    for (const check of evidenceChecks) {
      if (check.status === "fresh") continue;
      evidenceFresh = false;
      failures.push(`Required evidence '${check.label}' is ${check.status.replaceAll("-", " ")}`);
    }
  } else if (evidenceRecords.length > 0) {
    const latest = evidenceRecords[evidenceRecords.length - 1];
    const timestamp = Date.parse(latest.at);
    if (latest.exit_code !== 0) {
      evidenceFresh = false;
      failures.push(`Latest evidence record '${latest.label}' failed with exit code ${latest.exit_code}`);
    }
    if (latest.fingerprint !== currentFp) {
      evidenceFresh = false;
      failures.push("Workspace fingerprint mismatch: code modified since last verification record");
    }
    if (!Number.isFinite(timestamp) || timestamp > Date.now() + 60_000 || (Date.now() - timestamp) / 60_000 > maxEvidenceAgeMinutes) {
      evidenceFresh = false;
      failures.push(`Latest evidence record '${latest.label}' has an invalid or stale timestamp`);
    }
    evidenceChecks = [{
      label: latest.label,
      status: evidenceFresh ? "fresh" : latest.exit_code === 0 ? "stale" : "failed",
      at: latest.at,
    }];
  } else if (stageNum >= 11) {
    evidenceFresh = false;
    failures.push("No evidence records: run tests before ship");
  }

  const checksPassed = failures.length === 0;

  // 4. Generate Badge URL
  const badgeStatus = checksPassed ? "Checks--Passed" : "Checks--Blocked";
  const badgeColor = checksPassed ? "7c3aed" : "ef4444"; // violet-600 vs red-500
  const badgeUrl = `https://img.shields.io/badge/Kineti-${badgeStatus}-${badgeColor}?style=flat-square&logo=shield`;

  // 5. Generate Markdown Summary
  const shortFp = currentFp.slice(0, 12);
  const shortGoalHash = goalHash ? goalHash.slice(0, 12) : "none";

  const markdownSummary = `
## Kineti CI Checks

| Metric | Status / Value | Details |
| :--- | :--- | :--- |
| **CI result** | ${checksPassed ? "✅ **CHECKS PASSED**" : "❌ **BLOCKED**"} | ${checksPassed ? "Configured checks passed" : markdownSafe(failures.join("; "))} |
| **Pipeline Stage** | Stage ${stageNum}/13 (\`${markdownSafe(stageName)}\`) | Configured stage label |
| **Workspace Fingerprint** | \`${shortFp}\` | Delimited SHA-256 state hash |
| **Recorded spend** | \`$${spendUsd.toFixed(3)} / $${spendLimitUsd.toFixed(2)}\` | ${spendTripped ? "⚠️ Recorded limit tripped" : "Recorded limit not tripped"} |
| **Root Goal Hash** | \`${shortGoalHash}\` | Configured goal recorded in project state |
| **Evidence records** | ${evidenceRecords.length} record(s) | ${evidenceFresh ? "Current" : "Missing, failed, or stale"} |
${evidenceChecks.length ? `| **Required labels** | ${evidenceChecks.map((check) => `\`${markdownSafe(check.label)}\`: ${check.status}`).join("; ")} | Checked against this workspace fingerprint |` : ""}

${failures.length > 0 ? `### Gate Blocking Issues\n${failures.map(f => `- ${markdownSafe(f)}`).join("\n")}\n` : ""}
*Generated at ${nowIso()} by Kineti Context Integrity Layer (CIP) Runtime.*
`.trim();

  // 6. Generate PR Comment
  const prComment = `
[![Kineti CI Checks](${badgeUrl})](https://getkineti.com)

### Kineti CI Checks: ${checksPassed ? "**PASSED** ✅" : "**BLOCKED** ❌"}

> **Root goal hash**: \`${shortGoalHash}\`
> **Stage**: \`${stageNum}/13 (${markdownSafe(stageName)})\` &nbsp;|&nbsp; **Workspace fingerprint**: \`${shortFp}\` &nbsp;|&nbsp; **Recorded spend**: \`$${spendUsd.toFixed(3)}\`

${checksPassed
  ? "All required evidence checks passed for this workspace fingerprint. This report is not a signed safety certificate."
  : `**Failure details:**\n${failures.map(f => `- ❌ ${markdownSafe(f)}`).join("\n")}`
}
`.trim();

  return {
    timestamp: nowIso(),
    checksPassed,
    stageName,
    stageNumber: stageNum,
    rootGoal,
    workspaceFingerprint: currentFp,
    spendUsd,
    spendLimitUsd,
    spendTripped,
    evidenceCount: evidenceRecords.length,
    evidenceFresh,
    evidenceChecks,
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
  const requiredEvidenceLabels: string[] = [];
  let maxEvidenceAgeMinutes = 240;

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
      case "--require-evidence":
        if (!argv[i + 1]) {
          console.error("kineti ci: --require-evidence needs a label");
          process.exit(2);
        }
        requiredEvidenceLabels.push(argv[++i]);
        break;
      case "--max-evidence-age":
        maxEvidenceAgeMinutes = Number(argv[++i]);
        if (!Number.isFinite(maxEvidenceAgeMinutes) || maxEvidenceAgeMinutes <= 0) {
          console.error("kineti ci: --max-evidence-age must be a positive number of minutes");
          process.exit(2);
        }
        break;
    }
  }

  const report = generateCIReport(process.cwd(), { requiredEvidenceLabels, maxEvidenceAgeMinutes });

  if (badgeOnly) {
    console.log(report.badgeUrl);
    process.exit(report.checksPassed ? 0 : 1);
  }

  if (jsonOutput) {
    console.log(JSON.stringify(report, null, 2));
    process.exit(report.checksPassed ? 0 : 1);
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
  process.exit(report.checksPassed ? 0 : 1);
}

if (import.meta.main) {
  main();
}
