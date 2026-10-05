// tests/ci.test.ts
// Tests for Kineti GitHub Actions CI verification & badging engine

import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { generateCIReport } from "../bin/kineti-ci.ts";
import { writeJson } from "../bin/lib.ts";
import { fingerprint } from "../bin/kineti-evidence.ts";

describe("Kineti GitHub Actions CI Verification & Badging", () => {
  let tmpDir: string;
  let kinetiDir: string;

  beforeEach(() => {
    const scratch = path.join(process.cwd(), ".kineti", "test-ci");
    fs.mkdirSync(scratch, { recursive: true });
    tmpDir = fs.mkdtempSync(path.join(scratch, "run-"));
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

    expect(report.checksPassed).toBe(true);
    expect(report.stageNumber).toBe(7);
    expect(report.stageName).toBe("build");
    expect(report.spendUsd).toBe(1.25);
    expect(report.badgeUrl).toContain("Checks--Passed");
    expect(report.badgeUrl).toContain("7c3aed");
    expect(report.failures).toHaveLength(0);
    expect(report.markdownSummary).toContain("Root Goal Hash");
    expect(report.prComment).not.toContain("Build high-integrity microservice");
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

    expect(report.checksPassed).toBe(false);
    expect(report.spendTripped).toBe(true);
    expect(report.badgeUrl).toContain("Checks--Blocked");
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

    expect(report.checksPassed).toBe(false);
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
    expect(report.checksPassed).toBe(true);
    expect(report.stageName).toBe("bugfix");
    expect(report.stageNumber).toBe(0);
    expect(report.markdownSummary).toContain("bugfix");
  });

  it("blocks when security gate failed", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      current_stage: 7,
      root_goal: {
        description: "Build feature",
        locked_at: new Date().toISOString(),
        hash: "d".repeat(64),
      },
      gates: { security: "fail" },
    });
    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 1000000,
      tripped: false,
      limit_microcents: 50000000,
    });
    const report = generateCIReport(tmpDir);
    expect(report.checksPassed).toBe(false);
    expect(report.failures.some((f) => f.includes("Security gate failed"))).toBe(true);
  });

  it("blocks ship without security pass", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      current_stage: 11,
      root_goal: {
        description: "Ship release",
        locked_at: new Date().toISOString(),
        hash: "e".repeat(64),
      },
      gates: { spec: "pass" },
    });
    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 1000000,
      tripped: false,
      limit_microcents: 50000000,
    });
    const report = generateCIReport(tmpDir);
    expect(report.checksPassed).toBe(false);
    expect(report.failures.some((f) => f.includes("Security gate must pass before ship"))).toBe(true);
  });

  it("blocks ship without evidence records", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      current_stage: 11,
      root_goal: {
        description: "Ship release",
        locked_at: new Date().toISOString(),
        hash: "f".repeat(64),
      },
      gates: { security: "pass", spec: "pass" },
    });
    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 1000000,
      tripped: false,
      limit_microcents: 50000000,
    });
    const report = generateCIReport(tmpDir);
    expect(report.checksPassed).toBe(false);
    expect(report.failures.some((f) => f.includes("No evidence records"))).toBe(true);
  });

  it("blocks when state.json is missing instead of passing vacuously", () => {
    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 0,
      tripped: false,
      limit_microcents: 50000000,
    });

    const report = generateCIReport(tmpDir);

    expect(report.checksPassed).toBe(false);
    expect(report.failures.some((f) => f.includes("state.json missing"))).toBe(true);
    expect(report.failures.some((f) => f.includes("kineti seed"))).toBe(true);
    expect(report.badgeUrl).toContain("Checks--Blocked");
  });

  it("requires each explicitly named evidence label and verifies its fingerprint", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      project: "test",
      stage: 9,
      root_goal: "Check all required evidence",
      gates: {},
      history: [],
    });
    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 0,
      tripped: false,
      limit_microcents: 50_000_000,
    });
    fs.writeFileSync(path.join(tmpDir, "source.ts"), "export const value = 1;\n");
    const fp = fingerprint(tmpDir);
    fs.writeFileSync(path.join(kinetiDir, "evidence.jsonl"), [
      { at: new Date().toISOString(), label: "typecheck", cmd: "bun run typecheck", exit_code: 0, fingerprint: fp },
    ].map((record) => JSON.stringify(record)).join("\n") + "\n");

    const report = generateCIReport(tmpDir, { requiredEvidenceLabels: ["typecheck", "unit-tests"] });

    expect(report.checksPassed).toBe(false);
    expect(report.evidenceChecks).toEqual([
      expect.objectContaining({ label: "typecheck", status: "fresh" }),
      expect.objectContaining({ label: "unit-tests", status: "missing" }),
    ]);
    expect(report.failures.some((failure) => failure.includes("unit-tests") && failure.includes("missing"))).toBe(true);
  });

  it("blocks required evidence with a failed result or a stale workspace fingerprint", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      project: "test",
      stage: 9,
      root_goal: "Check required evidence integrity",
      gates: {},
      history: [],
    });
    writeJson(path.join(kinetiDir, "spend.json"), {
      total_microcents: 0,
      tripped: false,
      limit_microcents: 50_000_000,
    });
    fs.writeFileSync(path.join(kinetiDir, "evidence.jsonl"), [
      { at: new Date().toISOString(), label: "failed-tests", cmd: "bun test", exit_code: 1, fingerprint: "old" },
      { at: new Date().toISOString(), label: "stale-lint", cmd: "bun run lint", exit_code: 0, fingerprint: "old" },
    ].map((record) => JSON.stringify(record)).join("\n") + "\n");

    const report = generateCIReport(tmpDir, { requiredEvidenceLabels: ["failed-tests", "stale-lint"] });

    expect(report.checksPassed).toBe(false);
    expect(report.evidenceChecks).toEqual([
      expect.objectContaining({ label: "failed-tests", status: "failed" }),
      expect.objectContaining({ label: "stale-lint", status: "fingerprint-mismatch" }),
    ]);
  });

  it("blocks evidence older than the configured maximum age", () => {
    writeJson(path.join(kinetiDir, "state.json"), {
      version: 1,
      project: "test",
      stage: 9,
      root_goal: "Check evidence age",
      gates: {},
      history: [],
    });
    fs.writeFileSync(path.join(tmpDir, "source.ts"), "const old = true;\n");
    const fp = fingerprint(tmpDir);
    fs.writeFileSync(path.join(kinetiDir, "evidence.jsonl"), JSON.stringify({
      at: new Date(Date.now() - 10 * 60_000).toISOString(),
      label: "old-test",
      cmd: "bun test",
      exit_code: 0,
      fingerprint: fp,
    }) + "\n");

    const report = generateCIReport(tmpDir, { requiredEvidenceLabels: ["old-test"], maxEvidenceAgeMinutes: 5 });

    expect(report.checksPassed).toBe(false);
    expect(report.evidenceChecks[0]).toMatchObject({ label: "old-test", status: "stale" });
  });

  it("enforces packaging boundary exclusions in .npmignore", () => {
    const npmignorePath = path.join(import.meta.dir, "..", ".npmignore");
    const content = fs.readFileSync(npmignorePath, "utf-8");
    const requiredPatterns = [
      ".agents/",
      ".cursor/",
      ".github/",
      "core-native/",
      "docs/archive.zip",
      "docs/archive/",
      ".kineti/",
      "*.bun-build",
      ".*.bun-build",
      "tests/",
    ];
    for (const pattern of requiredPatterns) {
      expect(content).toContain(pattern);
    }
  });
});
