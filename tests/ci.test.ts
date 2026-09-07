// tests/ci.test.ts
// Tests for Kineti GitHub Actions CI verification & badging engine

import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { generateCIReport } from "../bin/kineti-ci.ts";
import { writeJson } from "../bin/lib.ts";

describe("Kineti GitHub Actions CI Verification & Badging", () => {
  let tmpDir: string;
  let kinetiDir: string;

  beforeEach(() => {
    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-ci-test-"));
    kinetiDir = path.join(tmpDir, ".kineti");
    fs.mkdirSync(kinetiDir, { recursive: true });
  });

  afterEach(() => {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  it("generates verified report on healthy workspace", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      current_stage: 7,
      root_goal: {
        description: "Build high-integrity microservice",
        locked_at: new Date().toISOString(),
        hash: "a".repeat(64),
      },
    });

    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 1250000, // $1.25
      tripped: false,
      limit_microcents: 50000000,
    });

    const report = generateCIReport(tmpDir);

    expect(report.verified).toBe(true);
    expect(report.stageNumber).toBe(7);
    expect(report.stageName).toBe("build");
    expect(report.spendUsd).toBe(1.25);
    expect(report.badgeUrl).toContain("Verified--Outcome");
    expect(report.badgeUrl).toContain("7c3aed");
    expect(report.failures).toHaveLength(0);
    expect(report.markdownSummary).toContain("Build high-integrity microservice");
    expect(report.prComment).toContain("PASSED");
  });

  it("detects tripped spend circuit breaker and blocks gate", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      current_stage: 8,
      root_goal: {
        description: "Refactor core kernel",
        locked_at: new Date().toISOString(),
        hash: "b".repeat(64),
      },
    });

    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 52000000, // $52.00
      tripped: true,
      limit_microcents: 50000000,
    });

    const report = generateCIReport(tmpDir);

    expect(report.verified).toBe(false);
    expect(report.spendTripped).toBe(true);
    expect(report.badgeUrl).toContain("Verification--Blocked");
    expect(report.badgeUrl).toContain("ef4444");
    expect(report.failures.some((f) => f.includes("circuit breaker"))).toBe(true);
    expect(report.markdownSummary).toContain("BLOCKED");
    expect(report.prComment).toContain("BLOCKED");
  });

  it("detects stale evidence when exit code is non-zero", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      current_stage: 9,
      root_goal: {
        description: "Run QA tests",
        locked_at: new Date().toISOString(),
        hash: "c".repeat(64),
      },
    });

    fs.writeFileSync(
      path.join(kinetiDir, "evidence.jsonl"),
      JSON.stringify({
        at: new Date().toISOString(),
        label: "qa-suite",
        cmd: "bun test",
        exit_code: 1,
        fingerprint: "abc123",
      }) + "\n"
    );

    const report = generateCIReport(tmpDir);

    expect(report.verified).toBe(false);
    expect(report.evidenceFresh).toBe(false);
    expect(report.failures.some((f) => f.includes("qa-suite"))).toBe(true);
  });

  it("supports flexible stage-agnostic task in CI verification", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      stage: "bugfix",
      task: { type: "bugfix", name: "Fix edge case" },
      root_goal: {
        description: "Emergency hotfix for memory leak",
        locked_at: new Date().toISOString(),
        hash: "b".repeat(64),
      },
    });

    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 500000,
      tripped: false,
      limit_microcents: 50000000,
    });

    const report = generateCIReport(tmpDir);
    expect(report.verified).toBe(true);
    expect(report.stageName).toBe("bugfix");
    expect(report.stageNumber).toBe(0);
    expect(report.markdownSummary).toContain("bugfix");
  });
});
