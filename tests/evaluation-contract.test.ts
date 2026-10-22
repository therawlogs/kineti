import { describe, expect, test } from "bun:test";
import crypto from "node:crypto";
import {
  EVALUATION_CONTRACT_VERSION,
  EvaluationContractError,
  evaluationManifestDigest,
  signEvaluationResult,
  signOwnerManifest,
  validateTrustedEvaluation,
  type ApprovedEvaluationPins,
  type CheckResult,
  type OwnerApprovedManifest,
  type SignedEvaluationResult,
} from "../src/security/evaluation-contract.ts";

const digest = (text: string) => crypto.createHash("sha256").update(text).digest("hex");
function keypair() {
  const pair = crypto.generateKeyPairSync("ed25519");
  return { privateKey: pair.privateKey.export({ type: "pkcs8", format: "pem" }).toString(), publicKey: pair.publicKey.export({ type: "spki", format: "pem" }).toString() };
}

function fixture() {
  const owner = keypair();
  const evaluator = keypair();
  const ownerIdentity = { id: "owner-service", key_id: "owner-key-v1" };
  const evaluatorIdentity = { id: "isolated-evaluator", key_id: "evaluator-key-v1" };
  const runner = { id: "disposable-runner", version: "1.0.0", image_sha256: digest("runner image") };
  const requiredChecks = [
    { id: "unit-tests", argv: ["bun", "test", "tests/"], input_sha256: [digest("test bytes")] },
    { id: "typecheck", argv: ["bun", "run", "typecheck"], input_sha256: [digest("tsconfig bytes")] },
  ];
  const pins: ApprovedEvaluationPins = {
    artifact_sha256: digest("frozen artifact"),
    contract_sha256: digest("approved outcome contract"),
    policy: { id: "release-policy", version: "3", sha256: digest("policy bytes") },
    owner: ownerIdentity,
    evaluator: evaluatorIdentity,
    runner,
    required_checks: requiredChecks,
  };
  const unsignedManifest = {
    schema: EVALUATION_CONTRACT_VERSION,
    evaluation_id: "eval-2026-10-16-001",
    artifact_sha256: pins.artifact_sha256,
    contract_sha256: pins.contract_sha256,
    policy: pins.policy,
    owner: ownerIdentity,
    runner,
    required_checks: requiredChecks,
  };
  const manifest = signOwnerManifest(unsignedManifest, owner.privateKey);
  const checks: CheckResult[] = requiredChecks.map(({ id }) => ({ id, status: "passed", reason_code: "executed-and-passed" }));
  const unsignedResult: Omit<SignedEvaluationResult, "signature"> = {
    schema: EVALUATION_CONTRACT_VERSION,
    manifest_sha256: evaluationManifestDigest(manifest),
    artifact_sha256: pins.artifact_sha256,
    contract_sha256: pins.contract_sha256,
    policy_sha256: pins.policy.sha256,
    evaluator: evaluatorIdentity,
    runner,
    checks,
    decision: "accepted",
    reason_code: "all-required-checks-passed",
  };
  const result = signEvaluationResult(unsignedResult, evaluator.privateKey);
  const options = {
    pins,
    trustedOwnerKeys: { [ownerIdentity.key_id]: owner.publicKey },
    trustedEvaluatorKeys: { [evaluatorIdentity.key_id]: evaluator.publicKey },
  };
  return { owner, evaluator, ownerIdentity, evaluatorIdentity, pins, manifest, result, options, checks };
}

function reissue(f: ReturnType<typeof fixture>, manifest: OwnerApprovedManifest, checks: CheckResult[] = f.checks): SignedEvaluationResult {
  const failed = checks.some((check) => check.status === "failed");
  const incomplete = checks.some((check) => ["skipped", "unsupported", "unrun"].includes(check.status));
  const unknown = checks.some((check) => check.status === "unknown");
  const decision = failed || incomplete ? "denied" : unknown ? "unknown" : "accepted";
  const reason_code = failed ? "required-check-failed" : incomplete ? "required-check-incomplete" : unknown ? "required-check-outcome-unknown" : "all-required-checks-passed";
  return signEvaluationResult({
    ...f.result,
    manifest_sha256: evaluationManifestDigest(manifest),
    artifact_sha256: manifest.artifact_sha256,
    contract_sha256: manifest.contract_sha256,
    policy_sha256: manifest.policy.sha256,
    runner: manifest.runner,
    checks,
    decision,
    reason_code,
  }, f.evaluator.privateKey);
}

function expectCode(run: () => unknown, code: string) {
  let caught: unknown;
  try { run(); } catch (error) { caught = error; }
  expect(caught).toBeInstanceOf(EvaluationContractError);
  expect((caught as EvaluationContractError | undefined)?.code).toBe(code);
}

describe("independent evaluation protocol v1", () => {
  test("accepts an owner-approved inventory signed by a separate evaluator key", () => {
    const f = fixture();
    expect(validateTrustedEvaluation(f.manifest, f.result, f.options)).toEqual({ decision: "accepted", reason_code: "all-required-checks-passed", manifest_sha256: evaluationManifestDigest(f.manifest) });
  });

  test("rejects a substituted artifact, contract, policy, inventory, or runner even when re-signed", () => {
    const f = fixture();
    const mutations: Array<[string, (manifest: OwnerApprovedManifest) => Omit<OwnerApprovedManifest, "signature">]> = [
      ["artifact-substitution", (m) => ({ ...m, artifact_sha256: digest("different artifact") })],
      ["contract-substitution", (m) => ({ ...m, contract_sha256: digest("different contract") })],
      ["policy-substitution", (m) => ({ ...m, policy: { ...m.policy, sha256: digest("different policy") } })],
      ["inventory-substitution", (m) => ({ ...m, required_checks: m.required_checks.slice(0, 1) })],
      ["unsupported-runner", (m) => ({ ...m, runner: { ...m.runner, version: "2.0.0" } })],
    ];
    for (const [code, mutate] of mutations) {
      const unsigned = mutate(f.manifest);
      const manifest = signOwnerManifest(unsigned, f.owner.privateKey);
      const result = reissue(f, manifest);
      expectCode(() => validateTrustedEvaluation(manifest, result, f.options), code);
    }
  });

  test("rejects substituted manifests, invalid signatures, and untrusted issuers", () => {
    const f = fixture();
    const swapped = signOwnerManifest({ ...f.manifest, evaluation_id: "substituted-eval" }, f.owner.privateKey);
    expectCode(() => validateTrustedEvaluation(swapped, f.result, f.options), "manifest-substitution");
    const unsignedResult = { ...f.result, decision: "denied" as const };
    expectCode(() => validateTrustedEvaluation(f.manifest, unsignedResult, f.options), "untrusted-evaluator");
    expectCode(() => validateTrustedEvaluation(f.manifest, f.result, { ...f.options, trustedEvaluatorKeys: {} }), "untrusted-evaluator");
    const foreignOwner = keypair();
    const foreignManifest = signOwnerManifest({ ...f.manifest, owner: { ...f.ownerIdentity, key_id: "other-owner-key" } }, foreignOwner.privateKey);
    expectCode(() => validateTrustedEvaluation(foreignManifest, f.result, f.options), "untrusted-owner");
  });

  test("denies missing, duplicate, failed, skipped, unsupported, and unrun required checks", () => {
    const f = fixture();
    for (const status of ["failed", "skipped", "unsupported", "unrun"] as const) {
      const checks = f.checks.map((item, index) => index === 0 ? { ...item, status, reason_code: `case-${status}` } : item);
      const falselyAccepted = signEvaluationResult({ ...f.result, checks, decision: "accepted", reason_code: "all-required-checks-passed" }, f.evaluator.privateKey);
      expectCode(() => validateTrustedEvaluation(f.manifest, falselyAccepted, f.options), "decision-mismatch");
      expect(validateTrustedEvaluation(f.manifest, reissue(f, f.manifest, checks), f.options).decision).toBe("denied");
    }
    const missing = reissue(f, f.manifest, f.checks.slice(0, 1));
    expectCode(() => validateTrustedEvaluation(f.manifest, missing, f.options), "incomplete-results");
    const duplicate = reissue(f, f.manifest, [f.checks[0], f.checks[0]]);
    expectCode(() => validateTrustedEvaluation(f.manifest, duplicate, f.options), "incomplete-results");
  });

  test("returns unknown without accepting an unknown or zero-case result", () => {
    const f = fixture();
    const unknownChecks = f.checks.map((item, index) => index === 0 ? { ...item, status: "unknown" as const, reason_code: "runner-lost" } : item);
    const unknown = reissue(f, f.manifest, unknownChecks);
    expect(validateTrustedEvaluation(f.manifest, unknown, f.options).decision).toBe("unknown");
    expectCode(() => validateTrustedEvaluation({ ...f.manifest, required_checks: [] }, f.result, f.options), "invalid-manifest");
  });

  test("requires owner and evaluator to use distinct signing keys", () => {
    const f = fixture();
    const same = keypair();
    const owner = { id: "owner-service", key_id: "shared-key" };
    const evaluator = { id: "isolated-evaluator", key_id: "shared-key" };
    const manifest = signOwnerManifest({ ...f.manifest, owner }, same.privateKey);
    const result = signEvaluationResult({ ...f.result, manifest_sha256: evaluationManifestDigest(manifest), evaluator }, same.privateKey);
    expectCode(() => validateTrustedEvaluation(manifest, result, {
      pins: { ...f.pins, owner, evaluator },
      trustedOwnerKeys: { [owner.key_id]: same.publicKey },
      trustedEvaluatorKeys: { [evaluator.key_id]: same.publicKey },
    }), "shared-authority");
  });
});
