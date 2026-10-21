import { legacyCanonicalJson } from "./canonical-json.ts";
import crypto from "node:crypto";

/** A signed handoff and evaluator result. This protocol does not execute checks. */
export const EVALUATION_CONTRACT_VERSION = "kineti-independent-evaluation/v1" as const;

export type CheckStatus = "passed" | "failed" | "skipped" | "unsupported" | "unrun" | "unknown";
export type EvaluationDecision = "accepted" | "denied" | "unknown";

export interface EvaluationSignature {
  algorithm: "Ed25519";
  key_id: string;
  value: string;
}

export interface EvaluationIssuer {
  id: string;
  key_id: string;
}

export interface RequiredCheck {
  id: string;
  argv: string[];
  /** SHA-256 digests of the exact required input bytes. */
  input_sha256: string[];
}

export interface CheckExecutionEnvironment {
  id: string;
  version: string;
  image_sha256: string;
}

export interface OwnerApprovedManifest {
  schema: typeof EVALUATION_CONTRACT_VERSION;
  evaluation_id: string;
  artifact_sha256: string;
  contract_sha256: string;
  policy: { id: string; version: string; sha256: string };
  owner: EvaluationIssuer;
  runner: CheckExecutionEnvironment;
  required_checks: RequiredCheck[];
  signature: EvaluationSignature;
}

export interface CheckResult {
  id: string;
  status: CheckStatus;
  reason_code: string;
}

export interface SignedEvaluationResult {
  schema: typeof EVALUATION_CONTRACT_VERSION;
  manifest_sha256: string;
  artifact_sha256: string;
  contract_sha256: string;
  policy_sha256: string;
  evaluator: EvaluationIssuer;
  runner: CheckExecutionEnvironment;
  checks: CheckResult[];
  decision: EvaluationDecision;
  reason_code: string;
  signature: EvaluationSignature;
}

export interface ApprovedEvaluationPins {
  artifact_sha256: string;
  contract_sha256: string;
  policy: { id: string; version: string; sha256: string };
  owner: EvaluationIssuer;
  evaluator: EvaluationIssuer;
  runner: CheckExecutionEnvironment;
  required_checks: RequiredCheck[];
}

export class EvaluationContractError extends Error {
  constructor(readonly code: string, message: string) { super(message); }
}

function fail(code: string, message: string): never { throw new EvaluationContractError(code, message); }

const canonical = legacyCanonicalJson;

export function evaluationDigest(value: unknown): string {
  return crypto.createHash("sha256").update(canonical(value)).digest("hex");
}

function withoutSignature<T extends { signature?: unknown }>(value: T): Omit<T, "signature"> {
  const { signature: _signature, ...unsigned } = value;
  return unsigned;
}

function validDigest(value: unknown): value is string { return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value); }
function issuerIs(value: unknown): value is EvaluationIssuer {
  const candidate = value as EvaluationIssuer | null;
  return !!candidate && typeof candidate.id === "string" && !!candidate.id.trim() && typeof candidate.key_id === "string" && !!candidate.key_id.trim();
}
function runnerIs(value: unknown): value is CheckExecutionEnvironment {
  const candidate = value as CheckExecutionEnvironment | null;
  return !!candidate && typeof candidate.id === "string" && !!candidate.id.trim() && typeof candidate.version === "string" && !!candidate.version.trim() && validDigest(candidate.image_sha256);
}
function checkIs(value: unknown): value is RequiredCheck {
  const candidate = value as RequiredCheck | null;
  return !!candidate && typeof candidate.id === "string" && !!candidate.id.trim()
    && Array.isArray(candidate.argv) && candidate.argv.length > 0
    && candidate.argv.every((part) => typeof part === "string" && !part.includes("\0"))
    && Array.isArray(candidate.input_sha256) && candidate.input_sha256.every(validDigest);
}
function publicKeyFingerprint(pem: string): string {
  try {
    return crypto.createHash("sha256").update(crypto.createPublicKey(pem).export({ type: "spki", format: "der" })).digest("hex");
  } catch { fail("invalid-public-key", "trusted public key is invalid"); }
}

export function signOwnerManifest(manifest: Omit<OwnerApprovedManifest, "signature">, privateKey: string | Buffer): OwnerApprovedManifest {
  const unsigned = withoutSignature(manifest as OwnerApprovedManifest);
  const signature = crypto.sign(null, Buffer.from(canonical(unsigned)), privateKey).toString("base64");
  return { ...unsigned, signature: { algorithm: "Ed25519", key_id: manifest.owner.key_id, value: signature } };
}

export function signEvaluationResult(result: Omit<SignedEvaluationResult, "signature">, privateKey: string | Buffer): SignedEvaluationResult {
  const unsigned = withoutSignature(result as SignedEvaluationResult);
  const signature = crypto.sign(null, Buffer.from(canonical(unsigned)), privateKey).toString("base64");
  return { ...unsigned, signature: { algorithm: "Ed25519", key_id: result.evaluator.key_id, value: signature } };
}

export function evaluationManifestDigest(manifest: OwnerApprovedManifest): string {
  return evaluationDigest(manifest);
}

function verifySignature(value: { signature: EvaluationSignature }, publicKey: string, expectedKeyId: string): boolean {
  try {
    return value.signature?.algorithm === "Ed25519"
      && value.signature.key_id === expectedKeyId
      && /^[A-Za-z0-9+/]+={0,2}$/.test(value.signature.value)
      && crypto.verify(null, Buffer.from(canonical(withoutSignature(value))), publicKey, Buffer.from(value.signature.value, "base64"));
  } catch { return false; }
}

function same(a: unknown, b: unknown): boolean { return canonical(a) === canonical(b); }

function derivedDecision(checks: CheckResult[]): { decision: EvaluationDecision; reason_code: string } {
  if (checks.some((check) => check.status === "failed")) return { decision: "denied", reason_code: "required-check-failed" };
  if (checks.some((check) => check.status === "skipped" || check.status === "unsupported" || check.status === "unrun")) return { decision: "denied", reason_code: "required-check-incomplete" };
  if (checks.some((check) => check.status === "unknown")) return { decision: "unknown", reason_code: "required-check-outcome-unknown" };
  return { decision: "accepted", reason_code: "all-required-checks-passed" };
}

/** Verify the signed handoff and result against owner-provided pins. */
export function validateTrustedEvaluation(
  manifest: OwnerApprovedManifest,
  result: SignedEvaluationResult,
  options: {
    pins: ApprovedEvaluationPins;
    trustedOwnerKeys: Record<string, string>;
    trustedEvaluatorKeys: Record<string, string>;
  },
): { decision: EvaluationDecision; reason_code: string; manifest_sha256: string } {
  if (!manifest || manifest.schema !== EVALUATION_CONTRACT_VERSION || typeof manifest.evaluation_id !== "string" || !manifest.evaluation_id.trim()) fail("invalid-manifest", "manifest schema or identity is invalid");
  if (!validDigest(manifest.artifact_sha256) || !validDigest(manifest.contract_sha256) || !manifest.policy || !validDigest(manifest.policy.sha256)
    || typeof manifest.policy.id !== "string" || !manifest.policy.id.trim() || typeof manifest.policy.version !== "string" || !manifest.policy.version.trim()) fail("invalid-manifest", "manifest artifact, contract, or policy identity is invalid");
  if (!issuerIs(manifest.owner) || !runnerIs(manifest.runner) || !Array.isArray(manifest.required_checks) || manifest.required_checks.length === 0 || !manifest.required_checks.every(checkIs)) fail("invalid-manifest", "manifest owner, runner, or required-check inventory is invalid");
  const manifestIds = manifest.required_checks.map((check) => check.id);
  if (new Set(manifestIds).size !== manifestIds.length) fail("invalid-manifest", "required-check ids must be unique");
  const ownerKey = options.trustedOwnerKeys[manifest.owner.key_id];
  if (!ownerKey || manifest.owner.key_id !== options.pins.owner.key_id || manifest.owner.id !== options.pins.owner.id
    || manifest.signature?.key_id !== manifest.owner.key_id || !verifySignature(manifest, ownerKey, manifest.owner.key_id)) fail("untrusted-owner", "manifest is not signed by the pinned owner key");

  const evaluatorKey = result && options.trustedEvaluatorKeys[result.evaluator?.key_id];
  if (!result || result.schema !== EVALUATION_CONTRACT_VERSION || !issuerIs(result.evaluator) || !evaluatorKey
    || result.evaluator.key_id !== options.pins.evaluator.key_id || result.evaluator.id !== options.pins.evaluator.id
    || result.signature?.key_id !== result.evaluator.key_id || !verifySignature(result, evaluatorKey, result.evaluator.key_id)) fail("untrusted-evaluator", "result is not signed by the pinned evaluator key");

  const ownerFingerprint = publicKeyFingerprint(ownerKey);
  if (ownerFingerprint === publicKeyFingerprint(evaluatorKey)) fail("shared-authority", "owner and evaluator must use separate signing keys");

  const manifestSha256 = evaluationManifestDigest(manifest);
  if (manifestSha256 !== result.manifest_sha256) fail("manifest-substitution", "evaluation result does not bind this signed manifest");
  if (manifest.artifact_sha256 !== options.pins.artifact_sha256 || result.artifact_sha256 !== manifest.artifact_sha256) fail("artifact-substitution", "evaluated artifact does not match the approved artifact");
  if (manifest.contract_sha256 !== options.pins.contract_sha256 || result.contract_sha256 !== manifest.contract_sha256) fail("contract-substitution", "evaluated contract does not match the approved contract");
  if (!same(manifest.policy, options.pins.policy) || result.policy_sha256 !== manifest.policy.sha256) fail("policy-substitution", "evaluation policy does not match the owner-approved policy");
  if (!same(manifest.runner, options.pins.runner) || !same(result.runner, manifest.runner)) fail("unsupported-runner", "evaluation runner does not match the declared environment");
  if (!same(manifest.required_checks, options.pins.required_checks)) fail("inventory-substitution", "required-check inventory does not match the owner-approved inventory");

  if (!Array.isArray(result.checks) || result.checks.length !== manifest.required_checks.length) fail("incomplete-results", "every required check must have exactly one result");
  const resultIds = result.checks.map((check) => check?.id);
  if (new Set(resultIds).size !== resultIds.length || manifestIds.some((id) => !resultIds.includes(id)) || resultIds.some((id) => !manifestIds.includes(id))) fail("incomplete-results", "check results do not match the required inventory");
  if (!result.checks.every((check) => check && typeof check.reason_code === "string" && !!check.reason_code.trim() && ["passed", "failed", "skipped", "unsupported", "unrun", "unknown"].includes(check.status))) fail("invalid-results", "check result status or reason code is invalid");
  const expectedDecision = derivedDecision(result.checks);
  if (result.decision !== expectedDecision.decision || result.reason_code !== expectedDecision.reason_code) fail("decision-mismatch", "signed decision does not match the required-check results");
  return { ...expectedDecision, manifest_sha256: manifestSha256 };
}
