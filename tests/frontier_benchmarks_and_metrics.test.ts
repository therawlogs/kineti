// tests/frontier_benchmarks_and_metrics.test.ts
// Verification Suite for Context Integrity Layer (CIP) Frontier Benchmarks & Metrics
// References: Paper 3 (Beyond Vector Search) & Paper 5 (Outcome Engineering)
// Author: Praveen Kumar (therawlogs.com | Foundational AI Research)

import { describe, expect, it } from "bun:test";
import {
  PROVENANCE_KERNEL_ENTITIES,
  PROVENANCE_RELATION_TYPES,
  isKernelEntityType,
  isKernelRelationType,
  computeDnti,
  computeCostPerVerifiedOutcome,
  FRONTIER_BENCHMARKS,
} from "../src/harness/benchmark";
import {
  registerAgent,
  delegateTask,
  submitWorkerResult,
  reviewAndSignOutcome,
  verifyDualSignedTicket,
} from "../src/swarm/coordinator";

describe("Context Integrity Layer (CIP) — 20-Entity Provenance Kernel Primitives", () => {
  it("defines exactly 20 distinct provenance entities matching Paper 3 §3", () => {
    expect(PROVENANCE_KERNEL_ENTITIES.length).toBe(20);
    const uniqueSet = new Set(PROVENANCE_KERNEL_ENTITIES);
    expect(uniqueSet.size).toBe(20);

    const expectedEntities = [
      "actor",
      "role",
      "authority",
      "intent",
      "goal",
      "task",
      "action",
      "tool_call",
      "rollback_step",
      "observation",
      "evidence",
      "state_change",
      "metric",
      "decision",
      "dependency",
      "constraint",
      "approval",
      "exception",
      "outcome",
      "review_required",
    ] as const;

    for (const ent of expectedEntities) {
      expect(uniqueSet.has(ent)).toBe(true);
      expect(isKernelEntityType(ent)).toBe(true);
    }

    expect(isKernelEntityType("unknown_entity")).toBe(false);
    expect(isKernelEntityType("")).toBe(false);
  });

  it("defines exactly 13 canonical causal relation types matching Paper 3", () => {
    expect(PROVENANCE_RELATION_TYPES.length).toBe(13);
    const uniqueRelations = new Set(PROVENANCE_RELATION_TYPES);
    expect(uniqueRelations.size).toBe(13);

    const expectedRelations = [
      "CAUSED_BY",
      "RESOLVES",
      "IMPLEMENTS",
      "TRIGGERED",
      "AUTHORIZED_BY",
      "DEFINES",
      "DISPATCHES",
      "MUTATES",
      "YIELDS",
      "REPLACES",
      "RELATES_TO",
      "DERIVED_FROM",
      "COMPENSATES",
    ] as const;

    for (const rel of expectedRelations) {
      expect(uniqueRelations.has(rel)).toBe(true);
      expect(isKernelRelationType(rel)).toBe(true);
    }

    expect(isKernelRelationType("INVALID_RELATION")).toBe(false);
  });
});

describe("Directional Normalized Trust-Weighted Impact (DNTI) — Paper 5 §3.1", () => {
  it("calculates positive DNTI for latency minimization achieving target", () => {
    const res = computeDnti({
      metricName: "api_p99_latency_ms",
      metricBase: 1240.0,
      metricObs: 42.0,
      metricTarget: 500.0,
      direction: "minimization",
      semanticEntropy: 0.042,
      tau: 0.15,
      testExitCode: 0,
      assertionsPassed: 14,
      minAssertions: 1,
      diffCoveragePct: 100.0,
      minCoveragePct: 80,
      tamperDetected: false,
    });

    expect(res.passed).toBe(true);
    expect(res.testIntegrityPredicate).toBe(1);
    expect(res.directionalDelta).toBeGreaterThan(1.0); // (1240 - 42) / (1240 - 500) = 1198 / 740 = 1.6189
    expect(res.semanticEntropyAttenuation).toBeGreaterThan(0.7); // exp(-0.042 / 0.15) ≈ 0.7558
    expect(res.verifiedImpact).toBeGreaterThan(1.0);
  });

  it("calculates positive DNTI for throughput maximization", () => {
    const res = computeDnti({
      metricName: "req_per_sec",
      metricBase: 1000.0,
      metricObs: 2500.0,
      metricTarget: 2000.0,
      direction: "maximization",
      semanticEntropy: 0.01,
      tau: 0.15,
      testExitCode: 0,
      assertionsPassed: 8,
      diffCoveragePct: 92.0,
      tamperDetected: false,
    });

    expect(res.passed).toBe(true);
    expect(res.directionalDelta).toBeGreaterThan(1.0); // -1 * (1000 - 2500) / 1000 = 1.5
    expect(res.verifiedImpact).toBeGreaterThan(1.4);
  });

  it("zeroes verified impact if tests fail or tamper is detected", () => {
    const failedTests = computeDnti({
      metricName: "api_p99_latency_ms",
      metricBase: 1200,
      metricObs: 400,
      metricTarget: 500,
      direction: "minimization",
      semanticEntropy: 0.01,
      testExitCode: 1, // failing exit code
      assertionsPassed: 0,
      diffCoveragePct: 50,
      tamperDetected: false,
    });
    expect(failedTests.testIntegrityPredicate).toBe(0);
    expect(failedTests.verifiedImpact).toBe(0);
    expect(failedTests.passed).toBe(false);

    const tampered = computeDnti({
      metricName: "api_p99_latency_ms",
      metricBase: 1200,
      metricObs: 400,
      metricTarget: 500,
      direction: "minimization",
      semanticEntropy: 0.01,
      testExitCode: 0,
      assertionsPassed: 10,
      diffCoveragePct: 95,
      tamperDetected: true, // detected tampering!
    });
    expect(tampered.testIntegrityPredicate).toBe(0);
    expect(tampered.verifiedImpact).toBe(0);
    expect(tampered.passed).toBe(false);
  });

  it("penalizes high semantic entropy (hallucination attenuation)", () => {
    const lowEntropy = computeDnti({
      metricName: "coverage",
      metricBase: 50,
      metricObs: 90,
      metricTarget: 80,
      direction: "maximization",
      semanticEntropy: 0.02,
      tau: 0.15,
      testExitCode: 0,
      assertionsPassed: 10,
      diffCoveragePct: 90,
      tamperDetected: false,
    });

    const highEntropy = computeDnti({
      metricName: "coverage",
      metricBase: 50,
      metricObs: 90,
      metricTarget: 80,
      direction: "maximization",
      semanticEntropy: 0.60, // severe hallucination uncertainty
      tau: 0.15,
      testExitCode: 0,
      assertionsPassed: 10,
      diffCoveragePct: 90,
      tamperDetected: false,
    });

    expect(highEntropy.semanticEntropyAttenuation).toBeLessThan(lowEntropy.semanticEntropyAttenuation);
    expect(highEntropy.verifiedImpact).toBeLessThan(lowEntropy.verifiedImpact);
  });
});

describe("Cost Per Verified Outcome ($/Outcome) Economics — Paper 5 §4", () => {
  it("computes exact cost per outcome matching the $0.31 benchmark", () => {
    const res = computeCostPerVerifiedOutcome({
      costDirectTokensUsd: 0.22,
      costSamplingTokensUsd: 0.00,
      costToolsUsd: 0.05,
      costInfraUsd: 0.04,
      verifiedOutcomes: 1,
    });

    expect(res.totalWorkflowCostUsd).toBeCloseTo(0.31, 2);
    expect(res.costPerVerifiedOutcomeUsd).toBeCloseTo(0.31, 2);
  });

  it("avoids division-by-zero when verified outcomes is zero", () => {
    const res = computeCostPerVerifiedOutcome({
      costDirectTokensUsd: 1.50,
      costSamplingTokensUsd: 0.50,
      costToolsUsd: 0.20,
      costInfraUsd: 0.10,
      verifiedOutcomes: 0,
    });

    expect(res.costPerVerifiedOutcomeUsd).toBeCloseTo(2.30, 2);
    expect(Number.isFinite(res.costPerVerifiedOutcomeUsd)).toBe(true);
  });
});

describe("Frontier Benchmark Target Invariants — Paper 5 §6", () => {
  it("preserves ALE benchmark target thresholds", () => {
    expect(FRONTIER_BENCHMARKS.ale.overallPassRatePct).toBe(76.4);
    expect(FRONTIER_BENCHMARKS.ale.longHorizonPassRatePct).toBe(68.2);
    expect(FRONTIER_BENCHMARKS.ale.taskGamingRatePct).toBeLessThanOrEqual(0.1);
    expect(FRONTIER_BENCHMARKS.ale.totalBenchmarkTasks).toBeGreaterThanOrEqual(1000);
  });

  it("preserves SWE-bench Verified MTTR targets", () => {
    expect(FRONTIER_BENCHMARKS.sweBenchVerified.incidentMitigationMttrMinutes).toBe(4.2);
    expect(FRONTIER_BENCHMARKS.sweBenchVerified.mttrReductionFactor).toBe(9.1);
    expect(FRONTIER_BENCHMARKS.sweBenchVerified.totalResolvedIssues).toBe(500);
  });

  it("preserves economic unit target", () => {
    expect(FRONTIER_BENCHMARKS.economicTarget.costPerVerifiedOutcomeUsd).toBe(0.31);
  });
});

describe("VerificationTicket Integration with Outcome Metrics", () => {
  it("attaches DNTI, cost per outcome, and benchmark references to VerificationTicket", () => {
    const coord = registerAgent("Coordinator", "coordinator");
    const worker = registerAgent("Worker", "worker");
    const reviewer = registerAgent("Reviewer", "reviewer");

    const task = delegateTask(coord, worker, "M2 Recalibration", "Align harness with CIP canon", 5.0);
    const { evidenceHash, workerSignature } = submitWorkerResult(worker, task, "Evidence: all 20 kernel tests pass");

    const ticket = reviewAndSignOutcome(
      reviewer,
      task,
      worker,
      workerSignature,
      evidenceHash,
      true,
      {
        dntiScore: 1.62,
        costPerOutcomeUsd: 0.31,
        benchmarkRef: "ALE-76.4%/SWE-bench-4.2m",
        semanticEntropy: 0.042,
      }
    );

    expect(ticket.dntiScore).toBe(1.62);
    expect(ticket.costPerOutcomeUsd).toBe(0.31);
    expect(ticket.benchmarkRef).toBe("ALE-76.4%/SWE-bench-4.2m");
    expect(ticket.semanticEntropy).toBe(0.042);
    expect(verifyDualSignedTicket(ticket, worker, reviewer)).toBe(true);
  });
});
