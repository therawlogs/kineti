#!/usr/bin/env bun
import { legacyCanonicalJson } from "../src/security/canonical-json.ts";
/** Build and verify deterministic summaries of evaluator-owned review results. */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  evaluationDigest, validateTrustedEvaluation,
  type ApprovedEvaluationPins, type OwnerApprovedManifest, type SignedEvaluationResult,
} from "../src/security/evaluation-contract.ts";

export type ReviewStatus = "pass" | "fail" | "unsupported";
export interface ReviewCase { id: string; status: ReviewStatus; reason: string; evidence_path?: string; evidence_digest?: string; }
export interface EvaluationReport { evaluator: { id: string; version: string }; results: ReviewCase[]; }
export interface ReviewSummary {
  schema_version: 1;
  artifact_sha256: string;
  receipt_sha256: string;
  evaluator: { id: string; version: string };
  results: ReviewCase[];
  overall: "pass" | "fail" | "unsupported";
  summary_sha256: string;
}
export interface AuthenticatedReviewSummary {
  schema_version: 2;
  artifact_sha256: string;
  evaluated_sha256: string;
  contract_sha256: string;
  receipt_sha256: string;
  policy: OwnerApprovedManifest["policy"];
  required_inventory_sha256: string;
  evaluator: { id: string; key_id: string };
  runner: OwnerApprovedManifest["runner"];
  decision: "accepted" | "denied" | "unknown";
  reason_code: string;
  manifest: OwnerApprovedManifest;
  evaluation: SignedEvaluationResult;
  summary_sha256: string;
}
export interface EvaluationTrustOptions {
  pins: ApprovedEvaluationPins;
  trustedOwnerKeys: Record<string, string>;
  trustedEvaluatorKeys: Record<string, string>;
}

export const canonicalJson = legacyCanonicalJson;
export function sha256(data: Buffer | string): string { return crypto.createHash("sha256").update(data).digest("hex"); }
function digestInput(value: string, field: string): string {
  if (!/^[a-f0-9]{64}$/i.test(value)) throw new Error(`${field} must be a SHA-256 hex digest`);
  return value.toLowerCase();
}

export function createReviewSummary(artifactSha256: string, receiptSha256: string, report: EvaluationReport): ReviewSummary {
  const artifact = digestInput(artifactSha256, "artifact_sha256");
  const receipt = digestInput(receiptSha256, "receipt_sha256");
  if (!report || !report.evaluator || typeof report.evaluator.id !== "string" || !report.evaluator.id.trim() || typeof report.evaluator.version !== "string" || !report.evaluator.version.trim()) {
    throw new Error("evaluator id and version are required");
  }
  if (!Array.isArray(report.results) || report.results.length === 0) throw new Error("at least one review result is required");
  const ids = new Set<string>();
  const results = report.results.map((item) => {
    if (!item || typeof item.id !== "string" || !item.id.trim() || ids.has(item.id)) throw new Error("review case ids must be non-empty and unique");
    ids.add(item.id);
    if (item.status !== "pass" && item.status !== "fail" && item.status !== "unsupported") throw new Error(`invalid status for ${item.id}`);
    if (typeof item.reason !== "string" || !item.reason.trim()) throw new Error(`reason is required for ${item.id}`);
    if (item.status === "unsupported" && (item.evidence_digest || item.evidence_path)) throw new Error(`unsupported case ${item.id} cannot carry pass evidence`);
    if (item.status !== "unsupported" && (!item.evidence_digest || !item.evidence_path)) throw new Error(`${item.status} case ${item.id} requires an inspectable evidence_path and evidence_digest`);
    if (item.evidence_digest !== undefined) digestInput(item.evidence_digest, `evidence_digest for ${item.id}`);
    if (item.evidence_path !== undefined && (typeof item.evidence_path !== "string" || !item.evidence_path.trim() || path.isAbsolute(item.evidence_path) || item.evidence_path.includes("\\") || item.evidence_path.split("/").some((part) => !part || part === "." || part === ".."))) throw new Error(`evidence_path for ${item.id} must be a normalized relative path`);
    return { id: item.id, status: item.status, reason: item.reason, ...(item.evidence_path ? { evidence_path: item.evidence_path } : {}), ...(item.evidence_digest ? { evidence_digest: item.evidence_digest.toLowerCase() } : {}) };
  }).sort((a, b) => a.id.localeCompare(b.id, "en"));
  const overall: ReviewSummary["overall"] = results.some((r) => r.status === "fail") ? "fail" : results.some((r) => r.status === "unsupported") ? "unsupported" : "pass";
  const unsigned = { schema_version: 1 as const, artifact_sha256: artifact, receipt_sha256: receipt, evaluator: { id: report.evaluator.id, version: report.evaluator.version }, results, overall };
  return { ...unsigned, summary_sha256: sha256(canonicalJson(unsigned)) };
}

export function verifyReviewSummary(summary: ReviewSummary, expectedDigest?: string): boolean {
  try {
    const { summary_sha256, ...unsigned } = summary;
    if (summary.schema_version !== 1 || digestInput(summary.artifact_sha256, "artifact_sha256") !== summary.artifact_sha256 || digestInput(summary.receipt_sha256, "receipt_sha256") !== summary.receipt_sha256) return false;
    const rebuilt = createReviewSummary(summary.artifact_sha256, summary.receipt_sha256, { evaluator: summary.evaluator, results: summary.results });
    if (rebuilt.overall !== summary.overall || rebuilt.summary_sha256 !== summary_sha256 || canonicalJson(rebuilt) !== canonicalJson(summary)) return false;
    return expectedDigest === undefined || digestInput(expectedDigest, "expected summary digest") === sha256(canonicalJson(summary));
  } catch { return false; }
}

/** Include the signed owner inventory and signed evaluator result in a reviewer handoff. */
export function createAuthenticatedReviewSummary(
  artifactSha256: string,
  receiptSha256: string,
  manifest: OwnerApprovedManifest,
  evaluation: SignedEvaluationResult,
  trust: EvaluationTrustOptions,
): AuthenticatedReviewSummary {
  const artifact = digestInput(artifactSha256, "artifact_sha256");
  const receipt = digestInput(receiptSha256, "receipt_sha256");
  if (artifact !== manifest.artifact_sha256) throw new Error("immutable artifact bytes do not match owner-approved artifact digest");
  const validated = validateTrustedEvaluation(manifest, evaluation, trust);
  const unsigned = {
    schema_version: 2 as const,
    artifact_sha256: artifact,
    evaluated_sha256: manifest.artifact_sha256,
    contract_sha256: manifest.contract_sha256,
    receipt_sha256: receipt,
    policy: manifest.policy,
    required_inventory_sha256: evaluationDigest(manifest.required_checks),
    evaluator: { id: evaluation.evaluator.id, key_id: evaluation.evaluator.key_id },
    runner: evaluation.runner,
    decision: validated.decision,
    reason_code: validated.reason_code,
    manifest,
    evaluation,
  };
  return { ...unsigned, summary_sha256: sha256(canonicalJson(unsigned)) };
}

export function verifyAuthenticatedReviewSummary(
  summary: AuthenticatedReviewSummary,
  trust: EvaluationTrustOptions,
  expectedDigest?: string,
): boolean {
  try {
    if (summary.schema_version !== 2 || summary.artifact_sha256 !== summary.evaluated_sha256
      || summary.artifact_sha256 !== summary.manifest.artifact_sha256
      || summary.contract_sha256 !== summary.manifest.contract_sha256
      || summary.policy.sha256 !== summary.manifest.policy.sha256
      || summary.required_inventory_sha256 !== evaluationDigest(summary.manifest.required_checks)
      || summary.evaluator.id !== summary.evaluation.evaluator.id
      || summary.evaluator.key_id !== summary.evaluation.evaluator.key_id
      || canonicalJson(summary.runner) !== canonicalJson(summary.evaluation.runner)) return false;
    const validated = validateTrustedEvaluation(summary.manifest, summary.evaluation, trust);
    if (summary.decision !== validated.decision || summary.reason_code !== validated.reason_code) return false;
    const { summary_sha256, ...unsigned } = summary;
    if (sha256(canonicalJson(unsigned)) !== summary_sha256) return false;
    return expectedDigest === undefined || digestInput(expectedDigest, "expected summary digest") === sha256(canonicalJson(summary));
  } catch { return false; }
}

function usage(): never {
  console.error("usage: kineti-review-summary --artifact <immutable-file> --receipt <receipt-file> --evaluation <evaluator-result.json> --out <summary.json>\n       kineti-review-summary --artifact <immutable-file> --receipt <receipt-file> --manifest <signed-owner.json> --evaluation <signed-result.json> --pins <owner-pins.json> --trusted-owner-key <key.pem> --trusted-evaluator-key <key.pem> --out <summary.json>\n       kineti-review-summary --verify <summary.json> [--expected-digest <sha256>]");
  process.exit(2);
}
function arg(name: string): string | undefined { const i = process.argv.indexOf(name); return i >= 0 ? process.argv[i + 1] : undefined; }
if (import.meta.main) {
  try {
    const verifyPath = arg("--verify");
    if (verifyPath) {
      const parsed = JSON.parse(fs.readFileSync(verifyPath, "utf8")) as ReviewSummary | AuthenticatedReviewSummary;
      if (parsed.schema_version === 2) {
        const pinsPath = arg("--pins"), ownerKeyPath = arg("--trusted-owner-key"), evaluatorKeyPath = arg("--trusted-evaluator-key");
        if (!pinsPath || !ownerKeyPath || !evaluatorKeyPath) usage();
        const trust: EvaluationTrustOptions = {
          pins: JSON.parse(fs.readFileSync(pinsPath, "utf8")) as ApprovedEvaluationPins,
          trustedOwnerKeys: { [parsed.manifest.owner.key_id]: fs.readFileSync(ownerKeyPath, "utf8") },
          trustedEvaluatorKeys: { [parsed.evaluation.evaluator.key_id]: fs.readFileSync(evaluatorKeyPath, "utf8") },
        };
        if (!verifyAuthenticatedReviewSummary(parsed, trust, arg("--expected-digest"))) { console.error("INVALID: authenticated reviewer summary does not match owner pins or signatures"); process.exit(7); }
        console.log(`VALID AUTHENTICATED REVIEW SUMMARY: ${parsed.summary_sha256}; ${parsed.decision}`);
        process.exit(0);
      }
      if (!verifyReviewSummary(parsed, arg("--expected-digest"))) { console.error("INVALID: reviewer summary digest or contents do not match"); process.exit(7); }
      console.log(`VALID REVIEW SUMMARY: ${parsed.summary_sha256}; ${parsed.overall}`);
      process.exit(0);
    }
    const artifactPath = arg("--artifact"), receiptPath = arg("--receipt"), evaluationPath = arg("--evaluation"), outputPath = arg("--out");
    if (!artifactPath || !receiptPath || !evaluationPath || !outputPath) usage();
    // Read-only inputs: the evaluator report and artifact are never modified.
    const artifactDigest = sha256(fs.readFileSync(artifactPath));
    const receiptDigest = sha256(fs.readFileSync(receiptPath));
    const evaluationBytes = fs.readFileSync(evaluationPath, "utf8");
    const trustedManifestPath = arg("--manifest"), pinsPath = arg("--pins");
    if (trustedManifestPath || pinsPath || arg("--trusted-owner-key") || arg("--trusted-evaluator-key")) {
      if (!trustedManifestPath || !pinsPath || !arg("--trusted-owner-key") || !arg("--trusted-evaluator-key")) usage();
      const manifest = JSON.parse(fs.readFileSync(trustedManifestPath, "utf8")) as OwnerApprovedManifest;
      const signedEvaluation = JSON.parse(evaluationBytes) as SignedEvaluationResult;
      const pins = JSON.parse(fs.readFileSync(pinsPath, "utf8")) as ApprovedEvaluationPins;
      const ownerKey = fs.readFileSync(arg("--trusted-owner-key")!, "utf8"), evaluatorKey = fs.readFileSync(arg("--trusted-evaluator-key")!, "utf8");
      const summary = createAuthenticatedReviewSummary(artifactDigest, receiptDigest, manifest, signedEvaluation, {
        pins, trustedOwnerKeys: { [manifest.owner.key_id]: ownerKey }, trustedEvaluatorKeys: { [signedEvaluation.evaluator.key_id]: evaluatorKey },
      });
      fs.mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
      fs.writeFileSync(outputPath, `${canonicalJson(summary)}\n`, { flag: "wx", mode: 0o600 });
      console.log(`${summary.decision.toUpperCase()}: ${summary.summary_sha256}`);
      if (summary.decision === "denied") process.exit(1);
      if (summary.decision === "unknown") process.exit(4);
      process.exit(0);
    }
    const evaluation = JSON.parse(evaluationBytes) as EvaluationReport;
    const evidenceRoot = fs.realpathSync(path.dirname(path.resolve(evaluationPath)));
    for (const item of evaluation.results ?? []) {
      if (item.status === "unsupported") continue;
      if (!item.evidence_path) throw new Error(`missing evidence_path for ${item.id}`);
      const sourcePath = path.resolve(evidenceRoot, item.evidence_path);
      if (!sourcePath.startsWith(`${evidenceRoot}${path.sep}`)) throw new Error(`evidence source escapes evaluator report directory for ${item.id}`);
      const realSource = fs.realpathSync(sourcePath);
      if (!realSource.startsWith(`${evidenceRoot}${path.sep}`) || !fs.statSync(realSource).isFile()) throw new Error(`evidence source is not a regular file within evaluator report directory for ${item.id}`);
      if (sha256(fs.readFileSync(realSource)) !== item.evidence_digest) throw new Error(`evidence digest mismatch for ${item.id}`);
    }
    const summary = createReviewSummary(artifactDigest, receiptDigest, evaluation);
    fs.mkdirSync(path.dirname(path.resolve(outputPath)), { recursive: true });
    fs.writeFileSync(outputPath, `${canonicalJson(summary)}\n`, { flag: "wx", mode: 0o600 });
    console.log(`${summary.overall.toUpperCase()}: ${summary.summary_sha256}`);
    if (summary.overall === "fail") process.exit(1);
    if (summary.overall === "unsupported") process.exit(4);
  } catch (error) {
    console.error(`INVALID: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(7);
  }
}
