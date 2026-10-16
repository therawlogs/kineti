#!/usr/bin/env bun
/** Build and verify deterministic summaries of evaluator-owned review results. */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

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

function sorted(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sorted);
  if (value && typeof value === "object") {
    const obj = value as Record<string, unknown>;
    return Object.fromEntries(Object.keys(obj).sort().map((key) => [key, sorted(obj[key])]));
  }
  return value;
}
export function canonicalJson(value: unknown): string { return JSON.stringify(sorted(value)); }
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

function usage(): never {
  console.error("usage: kineti-review-summary --artifact <immutable-file> --receipt <receipt-file> --evaluation <evaluator-result.json> --out <summary.json>\n       kineti-review-summary --verify <summary.json> [--expected-digest <sha256>]");
  process.exit(2);
}
function arg(name: string): string | undefined { const i = process.argv.indexOf(name); return i >= 0 ? process.argv[i + 1] : undefined; }
if (import.meta.main) {
  try {
    const verifyPath = arg("--verify");
    if (verifyPath) {
      const parsed = JSON.parse(fs.readFileSync(verifyPath, "utf8")) as ReviewSummary;
      if (!verifyReviewSummary(parsed, arg("--expected-digest"))) { console.error("INVALID: reviewer summary digest or contents do not match"); process.exit(7); }
      console.log(`VALID REVIEW SUMMARY: ${parsed.summary_sha256}; ${parsed.overall}`);
      process.exit(0);
    }
    const artifactPath = arg("--artifact"), receiptPath = arg("--receipt"), evaluationPath = arg("--evaluation"), outputPath = arg("--out");
    if (!artifactPath || !receiptPath || !evaluationPath || !outputPath) usage();
    // Read-only inputs: the evaluator report and artifact are never modified.
    const artifactDigest = sha256(fs.readFileSync(artifactPath));
    const receiptDigest = sha256(fs.readFileSync(receiptPath));
    const evaluation = JSON.parse(fs.readFileSync(evaluationPath, "utf8")) as EvaluationReport;
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
