import { describe, expect, it } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { createAuthenticatedReviewSummary, createReviewSummary, canonicalJson, sha256, verifyAuthenticatedReviewSummary, verifyReviewSummary, type EvaluationReport } from "../bin/kineti-review-summary.ts";
import { EVALUATION_CONTRACT_VERSION, evaluationManifestDigest, signEvaluationResult, signOwnerManifest, type ApprovedEvaluationPins } from "../src/security/evaluation-contract.ts";

const h = (s: string) => sha256(s);
const report = () => ({
  evaluator: { id: "independent-evaluator", version: "2.1.0" },
  results: [
    { id: "tests", status: "pass" as const, reason: "Required checks ran", evidence_path: "evidence/tests.txt", evidence_digest: h("stdout") },
    { id: "other-host", status: "unsupported" as const, reason: "No second host was available" },
  ],
});

function trustedFixture() {
  const owner = crypto.generateKeyPairSync("ed25519"), evaluator = crypto.generateKeyPairSync("ed25519");
  const ownerPrivate = owner.privateKey.export({ type: "pkcs8", format: "pem" }).toString(), ownerPublic = owner.publicKey.export({ type: "spki", format: "pem" }).toString();
  const evaluatorPrivate = evaluator.privateKey.export({ type: "pkcs8", format: "pem" }).toString(), evaluatorPublic = evaluator.publicKey.export({ type: "spki", format: "pem" }).toString();
  const ownerIdentity = { id: "owner", key_id: "owner-key" }, evaluatorIdentity = { id: "independent-runner", key_id: "evaluator-key" };
  const pins: ApprovedEvaluationPins = {
    artifact_sha256: h("frozen artifact"), contract_sha256: h("approved contract"),
    policy: { id: "release", version: "1", sha256: h("policy") }, owner: ownerIdentity, evaluator: evaluatorIdentity,
    runner: { id: "isolated", version: "1", image_sha256: h("runner") },
    required_checks: [{ id: "tests", argv: ["bun", "test", "tests/"], input_sha256: [h("test bytes")] }],
  };
  const manifest = signOwnerManifest({ schema: EVALUATION_CONTRACT_VERSION, evaluation_id: "eval-1", ...pins }, ownerPrivate);
  const evaluation = signEvaluationResult({
    schema: EVALUATION_CONTRACT_VERSION, manifest_sha256: evaluationManifestDigest(manifest), artifact_sha256: pins.artifact_sha256,
    contract_sha256: pins.contract_sha256, policy_sha256: pins.policy.sha256, evaluator: evaluatorIdentity, runner: pins.runner,
    checks: [{ id: "tests", status: "passed", reason_code: "executed" }], decision: "accepted", reason_code: "all-required-checks-passed",
  }, evaluatorPrivate);
  const trust = { pins, trustedOwnerKeys: { [ownerIdentity.key_id]: ownerPublic }, trustedEvaluatorKeys: { [evaluatorIdentity.key_id]: evaluatorPublic } };
  return { pins, manifest, evaluation, trust, ownerPublic, evaluatorPublic };
}

describe("deterministic reviewer receipt summary", () => {
  it("creates an authenticated summary with owner inventory, evaluator, policy, and exact artifact identity", () => {
    const f = trustedFixture();
    const summary = createAuthenticatedReviewSummary(h("frozen artifact"), h("receipt"), f.manifest, f.evaluation, f.trust);
    expect(summary.schema_version).toBe(2);
    expect(summary.evaluated_sha256).toBe(f.pins.artifact_sha256);
    expect(summary.contract_sha256).toBe(f.pins.contract_sha256);
    expect(summary.policy).toEqual(f.pins.policy);
    expect(summary.required_inventory_sha256).toBeTruthy();
    expect(summary.evaluator).toEqual(f.evaluation.evaluator);
    expect(verifyAuthenticatedReviewSummary(summary, f.trust)).toBe(true);
    expect(verifyAuthenticatedReviewSummary(summary, f.trust, h("different expected summary"))).toBe(false);
    expect(() => createAuthenticatedReviewSummary(h("substituted artifact"), h("receipt"), f.manifest, f.evaluation, f.trust)).toThrow("do not match");

    const tampered = { ...summary, evaluation: { ...summary.evaluation, decision: "denied" as const } };
    expect(verifyAuthenticatedReviewSummary(tampered, f.trust)).toBe(false);
  });

  it("verifies authenticated summaries from files against separately supplied owner pins", () => {
    const f = trustedFixture(), root = path.resolve(import.meta.dir, "../.kineti/review-summary-auth-cli-test");
    fs.rmSync(root, { recursive: true, force: true }); fs.mkdirSync(root, { recursive: true });
    try {
      const artifact = path.join(root, "artifact.bin"), receipt = path.join(root, "receipt.json"), manifest = path.join(root, "manifest.json");
      const evaluation = path.join(root, "evaluation.json"), pins = path.join(root, "pins.json"), owner = path.join(root, "owner.pub"), evaluator = path.join(root, "evaluator.pub"), summaryPath = path.join(root, "summary.json");
      fs.writeFileSync(artifact, "frozen artifact"); fs.writeFileSync(receipt, "receipt bytes");
      fs.writeFileSync(manifest, JSON.stringify(f.manifest)); fs.writeFileSync(evaluation, JSON.stringify(f.evaluation)); fs.writeFileSync(pins, JSON.stringify(f.pins));
      fs.writeFileSync(owner, f.ownerPublic); fs.writeFileSync(evaluator, f.evaluatorPublic);
      const summary = createAuthenticatedReviewSummary(h("frozen artifact"), h("receipt bytes"), f.manifest, f.evaluation, f.trust);
      fs.writeFileSync(summaryPath, JSON.stringify(summary));
      const cli = path.resolve(import.meta.dir, "../bin/kineti-review-summary.ts");
      const args = ["bun", cli, "--verify", summaryPath, "--pins", pins, "--trusted-owner-key", owner, "--trusted-evaluator-key", evaluator, "--expected-digest", h(canonicalJson(summary))];
      const valid = Bun.spawnSync(args, { stdout: "pipe", stderr: "pipe" });
      expect(valid.exitCode).toBe(0);
      expect(valid.stdout.toString()).toContain("VALID AUTHENTICATED REVIEW SUMMARY");
      fs.writeFileSync(summaryPath, JSON.stringify({ ...summary, decision: "denied" }));
      const invalid = Bun.spawnSync(args, { stdout: "pipe", stderr: "pipe" });
      expect(invalid.exitCode).toBe(7);
    } finally { fs.rmSync(root, { recursive: true, force: true }); }
  });
  it("canonicalizes cases and produces byte-identical summaries", () => {
    const first = createReviewSummary(h("artifact"), h("receipt"), report());
    const reversed = createReviewSummary(h("artifact"), h("receipt"), { ...report(), results: [...report().results].reverse() });
    expect(canonicalJson(first)).toBe(canonicalJson(reversed));
    expect(first.results.map((r) => r.id)).toEqual(["other-host", "tests"]);
    expect(first.overall).toBe("unsupported");
  });

  it("binds the immutable artifact, receipt, and evaluator evidence", () => {
    const baseline = createReviewSummary(h("artifact"), h("receipt"), report());
    expect(createReviewSummary(h("changed artifact"), h("receipt"), report()).summary_sha256).not.toBe(baseline.summary_sha256);
    expect(createReviewSummary(h("artifact"), h("changed receipt"), report()).summary_sha256).not.toBe(baseline.summary_sha256);
    const changedEvidence: EvaluationReport = report(); changedEvidence.results[0] = { ...changedEvidence.results[0], evidence_digest: h("changed stdout") };
    expect(createReviewSummary(h("artifact"), h("receipt"), changedEvidence).summary_sha256).not.toBe(baseline.summary_sha256);
  });

  it("never promotes unsupported or failed cases to pass", () => {
    const unsupported = createReviewSummary(h("a"), h("r"), report());
    expect(unsupported.overall).toBe("unsupported");
    const failed = createReviewSummary(h("a"), h("r"), { ...report(), results: [...report().results, { id: "tamper", status: "fail", reason: "Digest mismatch", evidence_path: "evidence/tamper.txt", evidence_digest: h("tamper output") }] });
    expect(failed.overall).toBe("fail");
    expect(() => createReviewSummary(h("a"), h("r"), { ...report(), results: [{ id: "bad", status: "unsupported", reason: "Unavailable", evidence_digest: h("pretend pass") }] })).toThrow();
  });

  it("rejects pass and fail cases without both an evidence source path and digest", () => {
    for (const status of ["pass", "fail"] as const) {
      expect(() => createReviewSummary(h("a"), h("r"), { evaluator: report().evaluator, results: [{ id: status, status, reason: "A result without inspectable evidence" }] })).toThrow(/inspectable evidence_path and evidence_digest/);
      expect(() => createReviewSummary(h("a"), h("r"), { evaluator: report().evaluator, results: [{ id: status, status, reason: "Digest without source", evidence_digest: h("evidence") }] })).toThrow(/inspectable evidence_path and evidence_digest/);
    }
  });

  it("requires CLI evidence files to match the declared digest and stay inside the report folder", () => {
    const root = path.resolve(import.meta.dir, "../.kineti/review-summary-cli-test");
    fs.rmSync(root, { recursive: true, force: true });
    fs.mkdirSync(path.join(root, "evidence"), { recursive: true });
    try {
      fs.writeFileSync(path.join(root, "artifact.tar"), "frozen artifact");
      fs.writeFileSync(path.join(root, "receipt.json"), "{}\n");
      fs.writeFileSync(path.join(root, "evidence/tests.txt"), "test output\n");
      const evaluation = { evaluator: { id: "independent", version: "1" }, results: [{ id: "tests", status: "pass", reason: "Tests ran", evidence_path: "evidence/tests.txt", evidence_digest: h("test output\n") }] };
      fs.writeFileSync(path.join(root, "evaluation.json"), JSON.stringify(evaluation));
      const cli = path.resolve(import.meta.dir, "../bin/kineti-review-summary.ts");
      const args = ["bun", cli, "--artifact", path.join(root, "artifact.tar"), "--receipt", path.join(root, "receipt.json"), "--evaluation", path.join(root, "evaluation.json"), "--out", path.join(root, "summary.json")];
      expect(Bun.spawnSync(args).exitCode).toBe(0);
      const badReport = { ...evaluation, results: [{ ...evaluation.results[0], evidence_digest: h("forged output") }] };
      fs.writeFileSync(path.join(root, "evaluation.json"), JSON.stringify(badReport));
      expect(Bun.spawnSync([...args.slice(0, -1), path.join(root, "bad-summary.json")]).exitCode).toBe(7);
      const escapedReport = { ...evaluation, results: [{ ...evaluation.results[0], evidence_path: "../artifact.tar" }] };
      fs.writeFileSync(path.join(root, "evaluation.json"), JSON.stringify(escapedReport));
      expect(Bun.spawnSync([...args.slice(0, -1), path.join(root, "escape-summary.json")]).exitCode).toBe(7);
    } finally { fs.rmSync(root, { recursive: true, force: true }); }
  });

  it("detects changed contents and verifies against an independently retained digest", () => {
    const summary = createReviewSummary(h("artifact"), h("receipt"), report());
    expect(verifyReviewSummary(summary)).toBe(true);
    const tampered = { ...summary, evaluator: { ...summary.evaluator, id: "builder" } };
    expect(verifyReviewSummary(tampered)).toBe(false);
    expect(verifyReviewSummary(summary, h("wrong external anchor"))).toBe(false);
    expect(verifyReviewSummary(summary, h(canonicalJson(summary)))).toBe(true);
  });
});
