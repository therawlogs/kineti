// src/harness/benchmark.ts
// Context Integrity Layer (Context Integrity Protocol / CIP)
// Frontier Benchmarks & Outcome Engineering Metrics
// Reference: Paper 3 (Beyond Vector Search) & Paper 5 (Outcome Engineering)
// Author: Praveen Kumar (therawlogs.com | Foundational AI Research)

import crypto from "node:crypto";

// ---------------------------------------------------------------------------
// 1. Universal 20-Entity Provenance Kernel Primitives (Paper 3 §3)
// ---------------------------------------------------------------------------

export const PROVENANCE_KERNEL_ENTITIES = [
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

export type KernelEntityType = (typeof PROVENANCE_KERNEL_ENTITIES)[number];

export function isKernelEntityType(value: string): value is KernelEntityType {
  return (PROVENANCE_KERNEL_ENTITIES as readonly string[]).includes(value);
}

export const PROVENANCE_RELATION_TYPES = [
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

export type KernelRelationType = (typeof PROVENANCE_RELATION_TYPES)[number];

export function isKernelRelationType(value: string): value is KernelRelationType {
  return (PROVENANCE_RELATION_TYPES as readonly string[]).includes(value);
}

export interface ProvenanceEntityRecord {
  id: string;
  type: KernelEntityType;
  payload: Record<string, unknown>;
  hlcTimestamp: string;
  parentIds: string[];
  canonicalHash: string;
}

// ---------------------------------------------------------------------------
// 2. Directional Normalized Trust-Weighted Impact (DNTI) (Paper 5 §3.1)
// ---------------------------------------------------------------------------

export type MetricDirection = "minimization" | "maximization";

export interface DntiInput {
  metricName: string;
  metricBase: number;
  metricObs: number;
  metricTarget: number;
  direction: MetricDirection;
  semanticEntropy: number; // SE >= 0
  tau?: number; // temperature scaling factor, default 0.15
  testExitCode: number; // 0 for pass
  assertionsPassed: number;
  minAssertions?: number; // default 1
  diffCoveragePct: number; // 0 - 100
  minCoveragePct?: number; // default 80
  tamperDetected: boolean;
}

export interface DntiResult {
  directionalDelta: number; // Phi
  semanticEntropyAttenuation: number; // sigma_tau(SE)
  testIntegrityPredicate: number; // Psi(T) in {0, 1}
  verifiedImpact: number; // Phi * sigma_tau(SE) * Psi(T)
  passed: boolean;
}

export function computeDnti(input: DntiInput): DntiResult {
  const tau = input.tau ?? 0.15;
  const minAssertions = input.minAssertions ?? 1;
  const minCoverage = input.minCoveragePct ?? 80;
  const epsilon = 1e-6;

  // 1. Directional Normalized Delta (Phi)
  // d = +1 for minimization (lower is better: latency, errors, MTTR)
  // d = -1 for maximization (higher is better: throughput, coverage, pass rate)
  const d = input.direction === "minimization" ? 1.0 : -1.0;
  const denom = Math.max(Math.abs(input.metricBase - input.metricTarget), epsilon);
  const directionalDelta = d * ((input.metricBase - input.metricObs) / denom);

  // 2. Bounded Exponential Semantic Entropy Attenuation sigma_tau(SE)
  // sigma_tau(SE) = exp(-SE / tau) in (0, 1]
  const clampedSe = Math.max(0, input.semanticEntropy);
  const semanticEntropyAttenuation = Math.exp(-clampedSe / Math.max(tau, 1e-6));

  // 3. Multi-Point Test Integrity Predicate Psi(T)
  // Psi(T) = I(ExitCode == 0) * I(Assertions >= K_min) * I(DiffCoverage >= theta_cov) * (1 - TamperFlag)
  const passExit = input.testExitCode === 0 ? 1 : 0;
  const passAssert = input.assertionsPassed >= minAssertions ? 1 : 0;
  const passCoverage = input.diffCoveragePct >= minCoverage ? 1 : 0;
  const passTamper = input.tamperDetected ? 0 : 1;

  const testIntegrityPredicate = passExit * passAssert * passCoverage * passTamper;

  // 4. Verified Impact
  const verifiedImpact = directionalDelta * semanticEntropyAttenuation * testIntegrityPredicate;
  const passed = testIntegrityPredicate === 1 && verifiedImpact > 0;

  return {
    directionalDelta,
    semanticEntropyAttenuation,
    testIntegrityPredicate,
    verifiedImpact,
    passed,
  };
}

// ---------------------------------------------------------------------------
// 3. Economic Unit Modeling: Cost Per Verified Outcome ($/Outcome) (Paper 5 §4)
// ---------------------------------------------------------------------------

export interface OutcomeEconomicsInput {
  costDirectTokensUsd: number;
  costSamplingTokensUsd: number;
  costToolsUsd: number;
  costInfraUsd: number;
  verifiedOutcomes: number;
}

export interface OutcomeEconomicsResult {
  totalWorkflowCostUsd: number;
  verifiedOutcomes: number;
  costPerVerifiedOutcomeUsd: number; // C_OVO
}

export function computeCostPerVerifiedOutcome(input: OutcomeEconomicsInput): OutcomeEconomicsResult {
  const totalWorkflowCostUsd =
    input.costDirectTokensUsd +
    input.costSamplingTokensUsd +
    input.costToolsUsd +
    input.costInfraUsd;

  const effectiveOutcomes = Math.max(1, input.verifiedOutcomes);
  const costPerVerifiedOutcomeUsd = totalWorkflowCostUsd / effectiveOutcomes;

  return {
    totalWorkflowCostUsd,
    verifiedOutcomes: input.verifiedOutcomes,
    costPerVerifiedOutcomeUsd,
  };
}

// ---------------------------------------------------------------------------
// 4. Frontier Benchmark Targets (Paper 5 §6)
// ---------------------------------------------------------------------------

export interface ALEBenchmarkTargets {
  benchmarkName: "Agents' Last Exam (ALE)";
  overallPassRatePct: number; // 76.4%
  longHorizonPassRatePct: number; // 68.2%
  taskGamingRatePct: number; // < 0.1%
  totalBenchmarkTasks: number; // 1000+
}

export interface SWEBenchVerifiedTargets {
  benchmarkName: "SWE-bench Verified Enterprise Incident Suite";
  incidentMitigationMttrMinutes: number; // 4.2 min
  mttrReductionFactor: number; // 9.1x
  totalResolvedIssues: number; // 500
}

export const FRONTIER_BENCHMARKS = {
  ale: {
    benchmarkName: "Agents' Last Exam (ALE)",
    overallPassRatePct: 76.4,
    longHorizonPassRatePct: 68.2,
    taskGamingRatePct: 0.1,
    totalBenchmarkTasks: 1000,
  } as ALEBenchmarkTargets,
  sweBenchVerified: {
    benchmarkName: "SWE-bench Verified Enterprise Incident Suite",
    incidentMitigationMttrMinutes: 4.2,
    mttrReductionFactor: 9.1,
    totalResolvedIssues: 500,
  } as SWEBenchVerifiedTargets,
  economicTarget: {
    costPerVerifiedOutcomeUsd: 0.31,
  },
};
