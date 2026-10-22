#!/usr/bin/env bun
/** Execute the shared, versioned local conformance corpus. This is not certification. */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
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

export type ConformanceDecision = "accepted" | "denied" | "unknown" | "unsupported";
export type ConformanceAdapter = "local-protocol" | "second-host-stub";
interface CorpusCase {
  id: string;
  scenario: string;
  expected: { decision: ConformanceDecision; reason_code: string };
}
interface Corpus { schema_version: 2; contract: "kineti-gate-conformance"; cases: CorpusCase[]; }
interface CaseResult {
  case_id: string;
  status: "passed" | "failed" | "unsupported";
  expected: CorpusCase["expected"];
  actual: CorpusCase["expected"];
}
export interface ConformanceReport {
  schema_version: 1;
  fixture_version: 2;
  fixture_sha256: string;
  adapter: ConformanceAdapter;
  outcome: "pass" | "fail" | "unsupported";
  results: CaseResult[];
}

const sha256 = (bytes: Uint8Array | string): string => crypto.createHash("sha256").update(bytes).digest("hex");
const scenarios = new Set([
  "authentic", "changed-artifact", "changed-contract", "changed-policy", "changed-inventory", "wrong-command",
  "wrong-issuer", "wrong-key", "unsupported-runner", "failed-check", "skipped-check", "unsupported-check",
  "unrun-check", "unknown-check", "zero-checks", "wrong-destination", "revoked-proof", "replayed-proof", "runner-fault",
]);

function parseCorpus(bytes: Uint8Array): Corpus {
  if (bytes.byteLength === 0 || bytes.byteLength > 1024 * 1024) throw new Error("conformance fixture must contain 1 byte to 1 MiB");
  let value: unknown;
  try { value = JSON.parse(Buffer.from(bytes).toString("utf8")); }
  catch { throw new Error("conformance fixture is not valid JSON"); }
  const corpus = value as Corpus;
  if (!corpus || corpus.schema_version !== 2 || corpus.contract !== "kineti-gate-conformance" || !Array.isArray(corpus.cases) || corpus.cases.length === 0 || corpus.cases.length > 500) {
    throw new Error("unsupported or malformed conformance fixture");
  }
  const ids = new Set<string>();
  for (const item of corpus.cases) {
    if (!item || typeof item.id !== "string" || !/^[a-z0-9][a-z0-9-]{2,80}$/.test(item.id) || ids.has(item.id)) throw new Error("conformance case ids must be unique lowercase identifiers");
    ids.add(item.id);
    if (!scenarios.has(item.scenario)) throw new Error(`unsupported conformance scenario in fixture: ${item.id}`);
    if (!item.expected || !["accepted", "denied", "unknown", "unsupported"].includes(item.expected.decision)
      || typeof item.expected.reason_code !== "string" || !/^[a-z0-9][a-z0-9-]{2,80}$/.test(item.expected.reason_code)) {
      throw new Error(`conformance case has invalid expected decision or reason: ${item.id}`);
    }
  }
  return corpus;
}

function keypair() {
  const pair = crypto.generateKeyPairSync("ed25519");
  return {
    privateKey: pair.privateKey.export({ type: "pkcs8", format: "pem" }).toString(),
    publicKey: pair.publicKey.export({ type: "spki", format: "pem" }).toString(),
  };
}

function protocolCase(scenario: string): CorpusCase["expected"] {
  if (["wrong-destination", "revoked-proof", "replayed-proof"].includes(scenario)) {
    const reason_code = scenario === "wrong-destination" ? "service-owned-action-unavailable"
      : scenario === "revoked-proof" ? "authoritative-revocation-unavailable" : "authoritative-replay-store-unavailable";
    return { decision: "unsupported", reason_code };
  }
  if (scenario === "runner-fault") return { decision: "unknown", reason_code: "runner-fault-outcome-unknown" };

  const owner = keypair(), evaluator = keypair();
  const ownerIdentity = { id: "owner-service", key_id: "owner-key-v1" };
  const evaluatorIdentity = { id: "isolated-evaluator", key_id: "evaluator-key-v1" };
  const runner = { id: "declared-runner", version: "1.0.0", image_sha256: sha256("runner image") };
  const checks = [{ id: "required-check", argv: ["bun", "test", "tests/"], input_sha256: [sha256("required input")] }];
  const pins: ApprovedEvaluationPins = {
    artifact_sha256: sha256("approved artifact"), contract_sha256: sha256("approved contract"),
    policy: { id: "release-policy", version: "1", sha256: sha256("approved policy") },
    owner: ownerIdentity, evaluator: evaluatorIdentity, runner, required_checks: checks,
  };
  let ownerForSigning = owner;
  let ownerForManifest = ownerIdentity;
  let evaluatorForSigning = evaluator;
  let evaluatorForResult = evaluatorIdentity;
  let artifact = pins.artifact_sha256;
  let contract = pins.contract_sha256;
  let policy = pins.policy;
  let manifestRunner = runner;
  let requiredChecks = checks;
  let checkResults: CheckResult[] = [{ id: "required-check", status: "passed", reason_code: "executed-and-passed" }];

  switch (scenario) {
    case "changed-artifact": artifact = sha256("substituted artifact"); break;
    case "changed-contract": contract = sha256("substituted contract"); break;
    case "changed-policy": policy = { ...pins.policy, sha256: sha256("substituted policy") }; break;
    case "changed-inventory": requiredChecks = [{ ...checks[0]!, id: "substituted-check" }]; break;
    case "wrong-command": requiredChecks = [{ ...checks[0]!, argv: ["bun", "test", "unapproved/command"] }]; break;
    case "wrong-issuer": {
      ownerForSigning = keypair(); ownerForManifest = { id: "other-owner", key_id: "other-owner-key" }; break;
    }
    case "wrong-key": {
      evaluatorForSigning = keypair(); evaluatorForResult = { id: "other-evaluator", key_id: "other-evaluator-key" }; break;
    }
    case "unsupported-runner": manifestRunner = { ...runner, version: "2.0.0" }; break;
    case "failed-check": checkResults = [{ id: "required-check", status: "failed", reason_code: "check-failed" }]; break;
    case "skipped-check": checkResults = [{ id: "required-check", status: "skipped", reason_code: "check-skipped" }]; break;
    case "unsupported-check": checkResults = [{ id: "required-check", status: "unsupported", reason_code: "check-unsupported" }]; break;
    case "unrun-check": checkResults = [{ id: "required-check", status: "unrun", reason_code: "check-unrun" }]; break;
    case "unknown-check": checkResults = [{ id: "required-check", status: "unknown", reason_code: "runner-lost" }]; break;
  }

  const manifestBody: Omit<OwnerApprovedManifest, "signature"> = {
    schema: EVALUATION_CONTRACT_VERSION, evaluation_id: `conformance-${scenario}`,
    artifact_sha256: artifact, contract_sha256: contract, policy, owner: ownerForManifest,
    runner: manifestRunner, required_checks: scenario === "zero-checks" ? [] : requiredChecks,
  };
  const manifest = signOwnerManifest(manifestBody, ownerForSigning.privateKey);
  const failed = checkResults.some((check) => check.status === "failed");
  const incomplete = checkResults.some((check) => ["skipped", "unsupported", "unrun"].includes(check.status));
  const unknown = checkResults.some((check) => check.status === "unknown");
  const decision: SignedEvaluationResult["decision"] = failed || incomplete ? "denied" : unknown ? "unknown" : "accepted";
  const reason_code = failed ? "required-check-failed" : incomplete ? "required-check-incomplete" : unknown ? "required-check-outcome-unknown" : "all-required-checks-passed";
  const result = signEvaluationResult({
    schema: EVALUATION_CONTRACT_VERSION, manifest_sha256: evaluationManifestDigest(manifest),
    artifact_sha256: artifact, contract_sha256: contract, policy_sha256: policy.sha256,
    evaluator: evaluatorForResult, runner: manifestRunner, checks: checkResults, decision, reason_code,
  }, evaluatorForSigning.privateKey);
  const options = {
    pins, trustedOwnerKeys: { [ownerIdentity.key_id]: owner.publicKey },
    trustedEvaluatorKeys: { [evaluatorIdentity.key_id]: evaluator.publicKey },
  };
  try {
    const outcome = validateTrustedEvaluation(manifest, result, options);
    return { decision: outcome.decision, reason_code: outcome.reason_code };
  } catch (error) {
    if (error instanceof EvaluationContractError) return { decision: "denied", reason_code: error.code };
    return { decision: "unknown", reason_code: "adapter-fault-outcome-unknown" };
  }
}

export function executeConformance(bytes: Uint8Array, adapter: ConformanceAdapter): ConformanceReport {
  if (adapter !== "local-protocol" && adapter !== "second-host-stub") throw new Error("adapter must be local-protocol or second-host-stub");
  const corpus = parseCorpus(bytes);
  const results: CaseResult[] = corpus.cases.map((item) => {
    if (adapter === "second-host-stub") {
      return { case_id: item.id, status: "unsupported", expected: item.expected, actual: { decision: "unsupported", reason_code: "adapter-unavailable" } };
    }
    const actual = protocolCase(item.scenario);
    if (actual.decision === "unsupported") return { case_id: item.id, status: "unsupported", expected: item.expected, actual };
    return { case_id: item.id, status: actual.decision === item.expected.decision && actual.reason_code === item.expected.reason_code ? "passed" : "failed", expected: item.expected, actual };
  });
  const outcome: ConformanceReport["outcome"] = results.some((item) => item.status === "failed") ? "fail"
    : results.some((item) => item.status === "unsupported") ? "unsupported" : "pass";
  return { schema_version: 1, fixture_version: corpus.schema_version, fixture_sha256: sha256(bytes), adapter, outcome, results };
}

function main(): void {
  const args = process.argv.slice(2);
  let adapter: ConformanceAdapter | undefined, fixture = "hosts/conformance.v2.json";
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--adapter") adapter = args[++i] as ConformanceAdapter;
    else if (args[i] === "--fixture") fixture = args[++i] ?? "";
    else if (args[i] === "--help" || args[i] === "-h") {
      console.log("kineti conformance --adapter <local-protocol|second-host-stub> [--fixture project/relative.json]"); return;
    } else throw new Error(`unknown argument: ${args[i]}`);
  }
  if (!adapter) throw new Error("--adapter is required");
  if (!fixture || path.isAbsolute(fixture) || fixture.split(/[\\/]/).includes("..")) throw new Error("fixture must be a project-relative path");
  const absolute = path.resolve(process.cwd(), fixture);
  const realRoot = fs.realpathSync(process.cwd()), realFile = fs.realpathSync(absolute);
  if (realFile !== realRoot && !realFile.startsWith(realRoot + path.sep)) throw new Error("fixture symlink escapes the project");
  const report = executeConformance(fs.readFileSync(realFile), adapter);
  console.log(JSON.stringify(report, null, 2));
  if (report.outcome === "fail") process.exitCode = 1;
  else if (report.outcome === "unsupported") process.exitCode = 4;
}

if (import.meta.main) {
  try { main(); }
  catch (error) { console.error(`kineti: ${error instanceof Error ? error.message : String(error)}`); process.exitCode = 2; }
}
