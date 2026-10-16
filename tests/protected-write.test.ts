import { afterEach, describe, expect, spyOn, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { applyProtectedWrite } from "../bin/kineti-protected-write.ts";
import { unsignedRecord, verifyReceiptSignature } from "../bin/kineti-evidence.ts";
import crypto from "node:crypto";

const root = path.join(process.cwd(), ".kineti", "protected-write-test");
const outsideRoot = root + "-outside";
afterEach(() => {
  fs.rmSync(root, { recursive: true, force: true });
  fs.rmSync(outsideRoot, { recursive: true, force: true });
});
function trustIssuer(issuerKey: string, revokedRunIds: string[] = [], revokedIssuerKeys: string[] = []) {
  fs.mkdirSync(path.join(root, ".kineti"), { recursive: true });
  fs.writeFileSync(path.join(root, ".kineti", "protected-write-policy.json"), JSON.stringify({ version: 1, trusted_issuer_key: issuerKey, revoked_run_ids: revokedRunIds, revoked_issuer_keys: revokedIssuerKeys }), { mode: 0o600 });
}

function fixtureProof(destination = "out.txt") {
  fs.rmSync(root, { recursive: true, force: true });
  fs.mkdirSync(path.join(root, "tests"), { recursive: true });
  fs.mkdirSync(path.dirname(path.join(root, destination)), { recursive: true });
  fs.mkdirSync(outsideRoot, { recursive: true });
  fs.writeFileSync(path.join(outsideRoot, "sentinel"), "unchanged");
  fs.writeFileSync(path.join(root, destination), "before");
  fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command_argv: ["bun", "test", "tests/one.test.ts"] } }));
  fs.writeFileSync(path.join(root, "tests/one.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
  const effect = { destination, arguments: {}, content: "approved bytes" };
  const run = Bun.spawnSync(["bun", path.resolve("bin/kineti-evidence.ts"), "run", "--label", "matrix", "--required-test", "tests/one.test.ts", "--protected-write", JSON.stringify(effect), "--", "bun", "test", "tests/one.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
  expect(run.exitCode).toBe(0);
  const ledger = path.join(root, ".kineti/evidence.jsonl");
  const record = JSON.parse(fs.readFileSync(ledger, "utf8").trim());
  trustIssuer(record.issuer_key);
  return { record, ledger, request: { label: record.label, runId: record.run_id, ...effect } };
}
function assertSentinels() {
  expect(fs.readFileSync(path.join(root, "out.txt"), "utf8")).toBe("before");
  expect(fs.readFileSync(path.join(outsideRoot, "sentinel"), "utf8")).toBe("unchanged");
}

describe("Cursor MCP protected write", () => {
  test("rejects a symlink destination parent and leaves outside sentinel unchanged", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.mkdirSync(outsideRoot, { recursive: true });
    fs.writeFileSync(path.join(outsideRoot, "sentinel.txt"), "untouched");
    fs.symlinkSync(outsideRoot, path.join(root, "linked"));
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/symlink.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "symlink.test.ts"), "import { test, expect } from 'bun:test'; test('symlink control', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const effect = { destination: "linked/out.txt", arguments: {}, content: "blocked\n" };
    const run = Bun.spawnSync(["bun", script, "run", "--label", "symlink", "--required-test", "tests/symlink.test.ts", "--protected-write", JSON.stringify(effect), "--", "bun", "test", "tests/symlink.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(run.exitCode).toBe(0);
    const record = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    trustIssuer(record.issuer_key);
    expect(() => applyProtectedWrite({ label: "symlink", runId: record.run_id, ...effect }, root)).toThrow("symlink destination is not allowed");
    expect(fs.existsSync(path.join(outsideRoot, "out.txt"))).toBe(false);
    expect(fs.readFileSync(path.join(outsideRoot, "sentinel.txt"), "utf8")).toBe("untouched");
  });

  test("preserves exact argv with spaces, quotes, Unicode, and backslashes", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    const testName = "argv with space.test.ts";
    fs.writeFileSync(path.join(root, "tests", testName), "import { test, expect } from 'bun:test'; test(\"argv 'quote' \\\\ ü\", () => expect(1).toBe(1));\n");
    const pattern = "argv 'quote' \\\\ ü";
    const exactArgv = ["bun", "test", "tests/" + testName, "--test-name-pattern", pattern];
    const config = { settings: { verify_command_argv: exactArgv } };
    const configPath = path.join(root, "kineti.config.json");
    fs.writeFileSync(configPath, JSON.stringify(config));
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "argv", "--required-test", "tests/" + testName, "--protected-write", JSON.stringify({ destination: "argv.txt", arguments: {}, content: "exact\n" }), "--", ...exactArgv], { cwd: root, stdout: "pipe", stderr: "pipe" });
    if (run.exitCode !== 0) throw new Error(new TextDecoder().decode(run.stderr) + new TextDecoder().decode(run.stdout));
    expect(run.exitCode).toBe(0);
    const record = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    expect(record.manifest.command).toEqual(exactArgv);
    expect(record.cmd).toBe(exactArgv.join(" "));
    const check = Bun.spawnSync(["bun", script, "check", "--label", "argv"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(check.exitCode).toBe(0);
    trustIssuer(record.issuer_key);
    const flattenedCollision = { settings: { verify_command_argv: [...exactArgv.slice(0, 4), "argv", "'quote'", "\\\\", "ü"] } };
    expect(flattenedCollision.settings.verify_command_argv.join(" ")).toBe(record.cmd);
    fs.writeFileSync(configPath, JSON.stringify(flattenedCollision));
    expect(() => applyProtectedWrite({ label: "argv", runId: record.run_id, destination: "argv.txt", arguments: {}, content: "exact\n" }, root)).toThrow("command does not match");
    const reordered = { settings: { verify_command_argv: ["bun", "test", exactArgv[3], exactArgv[2]] } };
    fs.writeFileSync(configPath, JSON.stringify(reordered));
    expect(() => applyProtectedWrite({ label: "argv", runId: record.run_id, destination: "argv.txt", arguments: {}, content: "exact\n" }, root)).toThrow("command does not match");
    fs.writeFileSync(configPath, JSON.stringify(config));
    expect(applyProtectedWrite({ label: "argv", runId: record.run_id, destination: "argv.txt", arguments: {}, content: "exact\n" }, root)).toContain("protected write applied");
    expect(fs.readFileSync(path.join(root, "argv.txt"), "utf8")).toBe("exact\n");
  });

  test("round trips empty and unusual arguments through checker and writer", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "tests", "required.test.ts"), "import { test, expect } from 'bun:test'; test('required', () => expect(1).toBe(1));\n");
    fs.writeFileSync(path.join(root, "runner.ts"), "const p = Bun.spawnSync(['bun', 'test', 'tests/required.test.ts']); process.stdout.write(p.stdout); process.stderr.write(p.stderr); process.exit(p.exitCode);\n");
    const exactArgv = ["bun", "run", "runner.ts", "", "spaced value", "'quoted'", "λ", "back\\slash"];
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command_argv: exactArgv } }));
    const effect = { destination: "argv.txt", arguments: { empty: "" }, content: "exact bytes" };
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "empty-argv", "--required-test", "tests/required.test.ts", "--protected-write", JSON.stringify(effect), "--", ...exactArgv], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(run.exitCode).toBe(0);
    const record = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    expect(record.manifest.command).toEqual(exactArgv);
    const check = Bun.spawnSync(["bun", script, "check", "--label", "empty-argv", "--expect-cmd", "run runner"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(check.exitCode).toBe(0); // Legacy --expect-cmd remains a substring check.
    const mismatch = Bun.spawnSync(["bun", script, "check", "--label", "empty-argv", "--expect-cmd", "not-the-command"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(mismatch.exitCode).toBe(4);
    trustIssuer(record.issuer_key);
    expect(applyProtectedWrite({ label: record.label, runId: record.run_id, ...effect }, root)).toContain("applied");
    expect(fs.readFileSync(path.join(root, "argv.txt"), "utf8")).toBe(effect.content);
  });

  test("checks declared environment values and current toolchain identity", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/env.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "env.test.ts"), "import { test, expect } from 'bun:test'; test('env', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const envRun = Bun.spawnSync(["bun", script, "run", "--label", "env", "--env-name", "KINETI_CONTRACT_INPUT", "--required-test", "tests/env.test.ts", "--protected-write", JSON.stringify({ destination: "env.txt", arguments: {}, content: "x" }), "--", "bun", "test", "tests/env.test.ts"], { cwd: root, env: { ...process.env, KINETI_CONTRACT_INPUT: "" }, stdout: "pipe", stderr: "pipe" });
    expect(envRun.exitCode).toBe(0);
    const envRecord = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    trustIssuer(envRecord.issuer_key);
    const originalValue = process.env.KINETI_CONTRACT_INPUT;
    delete process.env.KINETI_CONTRACT_INPUT;
    expect(() => applyProtectedWrite({ label: "env", runId: envRecord.run_id, destination: "env.txt", arguments: {}, content: "x" }, root)).toThrow("environment input changed");
    process.env.KINETI_CONTRACT_INPUT = "changed";
    expect(() => applyProtectedWrite({ label: "env", runId: envRecord.run_id, destination: "env.txt", arguments: {}, content: "x" }, root)).toThrow("environment input changed");
    if (originalValue === undefined) delete process.env.KINETI_CONTRACT_INPUT;
    else process.env.KINETI_CONTRACT_INPUT = originalValue;
    expect(fs.existsSync(path.join(root, "env.txt"))).toBe(false);

    const toolRun = Bun.spawnSync(["bun", script, "run", "--label", "tool", "--required-test", "tests/env.test.ts", "--protected-write", JSON.stringify({ destination: "tool.txt", arguments: {}, content: "x" }), "--", "bun", "test", "tests/env.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(toolRun.exitCode).toBe(0);
    const toolRecord = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim().split("\n").at(-1)!);
    trustIssuer(toolRecord.issuer_key);
    const execPathDescriptor = Object.getOwnPropertyDescriptor(process, "execPath");
    Object.defineProperty(process, "execPath", { value: "/kineti-changed-runtime", configurable: true });
    try {
      expect(() => applyProtectedWrite({ label: "tool", runId: toolRecord.run_id, destination: "tool.txt", arguments: {}, content: "x" }, root)).toThrow("toolchain identity changed");
    } finally {
      if (execPathDescriptor) Object.defineProperty(process, "execPath", execPathDescriptor);
    }
    expect(fs.existsSync(path.join(root, "tool.txt"))).toBe(false);
  });

  test("reproduces unsupported same-command forged success and a genuine execution control", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.mkdirSync(outsideRoot, { recursive: true });
    fs.writeFileSync(path.join(outsideRoot, "sentinel"), "unchanged");
    const exactArgv = ["bun", "test", "tests/"];
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command_argv: exactArgv } }));
    // The real Bun executable exits before registering or executing either required case.
    fs.writeFileSync(path.join(root, "tests/a.test.ts"), "console.log('tests/a.test.ts\\ntests/b.test.ts\\n1 pass\\n0 fail\\nRan 1 test across 1 file.'); process.exit(0);\n");
    fs.writeFileSync(path.join(root, "tests/b.test.ts"), "throw new Error('required case was reached');\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const effect = { destination: "out.txt", arguments: {}, content: "approved" };
    const prove = () => Bun.spawnSync(["bun", script, "run", "--label", "same-command", "--required-test", "tests", "--protected-write", JSON.stringify(effect), "--", ...exactArgv], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(prove().exitCode).toBe(0);
    const latest = () => JSON.parse(fs.readFileSync(path.join(root, ".kineti/evidence.jsonl"), "utf8").trim().split("\n").at(-1)!);
    const forged = latest();
    expect(forged.manifest.command).toEqual(exactArgv);
    expect(forged.verification.inventory.map((i: { path: string }) => i.path)).toEqual(["tests/a.test.ts", "tests/b.test.ts"]);
    trustIssuer(forged.issuer_key);
    // This assertion reproduces a LIMITATION, not acceptance of trusted test observation.
    expect(applyProtectedWrite({ label: forged.label, runId: forged.run_id, ...effect }, root)).toContain("applied");
    expect(fs.readFileSync(path.join(root, "out.txt"), "utf8")).toBe(effect.content);
    fs.rmSync(path.join(root, "out.txt"));
    for (const name of ["a", "b"]) fs.writeFileSync(path.join(root, `tests/${name}.test.ts`), `import { test, expect } from 'bun:test'; test('${name}', () => expect(1).toBe(1));\n`);
    expect(prove().exitCode).toBe(0);
    const genuine = latest();
    expect(genuine.verification.passed).toBe(2);
    expect(applyProtectedWrite({ label: genuine.label, runId: genuine.run_id, ...effect }, root)).toContain("applied");
    expect(fs.readFileSync(path.join(outsideRoot, "sentinel"), "utf8")).toBe("unchanged");
  });

  test("keeps same-label proof records separate across concurrent runs", async () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/one.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "one.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const args = ["bun", script, "run", "--label", "same", "--required-test", "tests/one.test.ts", "--", "bun", "test", "tests/one.test.ts"];
    const children = [Bun.spawn(args, { cwd: root, stdout: "pipe", stderr: "pipe" }), Bun.spawn(args, { cwd: root, stdout: "pipe", stderr: "pipe" })];
    const outputs = await Promise.all(children.map(async (child) => ({ status: await child.exited, stdout: await new Response(child.stdout).text() })));
    expect(outputs.map((item) => item.status)).toEqual([0, 0]);
    const ids = outputs.map((item) => item.stdout.match(/run=([0-9a-f-]{36})/)?.[1]);
    expect(new Set(ids).size).toBe(2);
    const ledgerCheck = Bun.spawnSync(["bun", path.resolve("bin/kineti-egress.ts"), "verify"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(ledgerCheck.exitCode).toBe(0);
    for (const id of ids) {
      const check = Bun.spawnSync(["bun", script, "check", "--label", "same", "--run-id", id!], { cwd: root, stdout: "pipe", stderr: "pipe" });
      expect(check.exitCode).toBe(0);
      expect(new TextDecoder().decode(check.stdout)).toContain("FRESH");
    }
  });

  test("binds the expected command, destination, arguments, and bytes; denies replay", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "package-output.tgz"), "candidate package bytes");
    fs.mkdirSync(path.join(root, ".kineti"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/one.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "one.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "release", "--required-test", "tests/one.test.ts", "--artifact", "package-output.tgz", "--protected-write", JSON.stringify({ destination: "result.txt", arguments: { mode: "release" }, content: "approved bytes\n" }), "--", "bun", "test", "tests/one.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(run.exitCode).toBe(0);
    const text = new TextDecoder().decode(run.stdout);
    const id = text.match(/run=([0-9a-f-]{36})/)?.[1];
    expect(id).toBeTruthy();
    const signedRecord = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    const base = { label: "release", runId: id!, destination: "result.txt", arguments: { mode: "release" }, content: "approved bytes\n" };
    expect(() => applyProtectedWrite(base, root)).toThrow("policy is missing");
    expect(fs.existsSync(path.join(root, "result.txt"))).toBe(false);
    trustIssuer(signedRecord.issuer_key);
    const forgedPair = crypto.generateKeyPairSync("ed25519");
    const forgedIssuer = forgedPair.publicKey.export({ type: "spki", format: "pem" }).toString();
    const forgedRecord = { ...signedRecord, issuer_key: forgedIssuer };
    forgedRecord.signature = crypto.sign(null, Buffer.from(unsignedRecord(forgedRecord)), forgedPair.privateKey).toString("base64");
    fs.writeFileSync(path.join(root, ".kineti", "evidence.jsonl"), JSON.stringify(forgedRecord) + "\n");
    expect(() => applyProtectedWrite(base, root)).toThrow("issuer is untrusted");
    expect(fs.existsSync(path.join(root, "result.txt"))).toBe(false);
    fs.writeFileSync(path.join(root, ".kineti", "evidence.jsonl"), JSON.stringify(signedRecord) + "\n");
    expect(() => applyProtectedWrite(base, root, Date.parse(signedRecord.at) - 60_001)).toThrow("future");
    expect(() => applyProtectedWrite(base, root, Date.parse(signedRecord.at) + 240 * 60_000 + 1)).toThrow("expired");
    trustIssuer(signedRecord.issuer_key, [id!]);
    expect(() => applyProtectedWrite(base, root)).toThrow("revoked");
    expect(fs.existsSync(path.join(root, "result.txt"))).toBe(false);
    trustIssuer(signedRecord.issuer_key, [], [signedRecord.issuer_key]);
    expect(() => applyProtectedWrite(base, root)).toThrow("issuer is untrusted, revoked");
    trustIssuer(signedRecord.issuer_key);
    fs.writeFileSync(path.join(root, ".kineti", "protected-write-policy.json"), JSON.stringify({ version: 1, trusted_issuer_key: signedRecord.issuer_key, revoked_run_ids: "malformed", revoked_issuer_keys: [] }));
    expect(() => applyProtectedWrite(base, root)).toThrow("policy has invalid fields");
    trustIssuer(signedRecord.issuer_key);
    const policyPath = path.join(root, ".kineti", "protected-write-policy.json");
    fs.chmodSync(policyPath, 0);
    try { expect(() => applyProtectedWrite(base, root)).toThrow("policy is missing, unreadable, or malformed"); }
    finally { fs.chmodSync(policyPath, 0o600); }
    expect(() => applyProtectedWrite({ ...base, destination: "other.txt" }, root)).toThrow("do not match");
    expect(() => applyProtectedWrite({ ...base, content: "altered\n" }, root)).toThrow("do not match");
    expect(() => applyProtectedWrite({ ...base, arguments: { mode: "debug" } }, root)).toThrow("do not match");
    fs.writeFileSync(path.join(root, "package-output.tgz"), "modified candidate package");
    expect(() => applyProtectedWrite(base, root)).toThrow("changed after proof");
    fs.writeFileSync(path.join(root, "package-output.tgz"), "candidate package bytes");
    expect(applyProtectedWrite(base, root)).toContain("protected write applied");
    expect(fs.readFileSync(path.join(root, "result.txt"), "utf8")).toBe("approved bytes\n");
    expect(() => applyProtectedWrite(base, root)).toThrow(/changed after proof|already been used/);
  });

  test("keeps wrong-key helper and wrong-configured-command checks", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/other.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "required.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "release", "--required-test", "tests/required.test.ts", "--protected-write", JSON.stringify({ destination: "out.txt", arguments: {}, content: "x" }), "--", "bun", "test", "tests/required.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    const record = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    trustIssuer(record.issuer_key);
    expect(verifyReceiptSignature(record)).toBe(true);
    const wrongKey = crypto.generateKeyPairSync("ed25519").publicKey.export({ type: "spki", format: "pem" }).toString();
    expect(verifyReceiptSignature(record, wrongKey)).toBe(false);
    expect(verifyReceiptSignature({ ...record, issuer_key: wrongKey })).toBe(false);
    const id = record.run_id;
    expect(run.exitCode).toBe(0);
    expect(id).toBeTruthy();
    expect(() => applyProtectedWrite({ label: "release", runId: id!, destination: "out.txt", arguments: {}, content: "x" }, root)).toThrow();
  });

  test("keeps configured-command mismatch and signature-tampering checks", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/other.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "required.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "release", "--required-test", "tests/required.test.ts", "--protected-write", JSON.stringify({ destination: "out.txt", arguments: {}, content: "x" }), "--", "bun", "test", "tests/required.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(run.exitCode).toBe(0);
    const recordPath = path.join(root, ".kineti", "evidence.jsonl");
    const record = JSON.parse(fs.readFileSync(recordPath, "utf8").trim());
    trustIssuer(record.issuer_key);
    expect(() => applyProtectedWrite({ label: "release", runId: record.run_id, destination: "out.txt", arguments: {}, content: "x" }, root)).toThrow("configured verification command");
    record.manifest.protected_effect.destination = "other.txt";
    fs.writeFileSync(recordPath, JSON.stringify(record) + "\n");
    expect(() => applyProtectedWrite({ label: "release", runId: record.run_id, destination: "other.txt", arguments: {}, content: "x" }, root)).toThrow("signature");
  });
  test("denies signed malformed proof fields for their intended reason in writer and checker", () => {
    const { record, request, ledger } = fixtureProof();
    const key = fs.readFileSync(path.join(root, ".kineti/evidence-issuer.pem.private"), "utf8");
    const cases: Array<[string, (r: any) => void, string]> = [
      ["missing issuer", (r) => delete r.issuer_key, "issuer is untrusted"],
      ["missing signature", (r) => delete r.signature, "signature is invalid"],
      ["malformed signature", (r) => r.signature = "%%%", "signature is invalid"],
      ["invalid time", (r) => r.at = "not-a-time", "invalid receipt timestamp"],
      ["unsupported version", (r) => r.receipt_version = 3, "unsupported receipt"],
      ["missing verification", (r) => delete r.verification, "verification is missing"],
      ["zero", (r) => r.verification.passed = 0, "verification evidence"],
      ["negative", (r) => r.verification.passed = -1, "verification evidence"],
      ["fraction", (r) => r.verification.passed = 1.5, "verification evidence"],
      ["failed", (r) => r.verification.failed = 1, "verification evidence"],
      ["skipped", (r) => r.verification.skipped = 1, "verification evidence"],
      ["missing counts", (r) => delete r.verification.passed, "verification evidence"],
      ["parser version", (r) => r.verification.parser = "unknown", "verification evidence"],
      ["missing inventory", (r) => delete r.verification.inventory, "inventory is malformed"],
      ["empty inventory", (r) => r.verification.inventory = [], "inventory is empty"],
      ["invalid input path", (r) => r.verification.inventory[0].path = "../outside", "inventory is malformed"],
      ["test digest", (r) => r.verification.inventory[0].sha256 = "0".repeat(64), "declared input changed"],
      ["dependency digest", (r) => r.manifest.dependencies = [{ path: "kineti.config.json", sha256: "0".repeat(64) }], "declared input changed"],
      ["parser digest", (r) => r.manifest.parser_sha256 = "0".repeat(64), "parser changed"],
      ["secret declaration", (r) => r.manifest.environment.API_TOKEN = "0".repeat(64), "secret-like"],
      ["missing manifest", (r) => delete r.manifest, "manifest or verification is missing"],
    ];
    for (const [name, mutate, reason] of cases) {
      const bad = structuredClone(record);
      mutate(bad);
      if (!name.includes("signature")) bad.signature = crypto.sign(null, Buffer.from(unsignedRecord(bad)), key).toString("base64");
      fs.writeFileSync(ledger, JSON.stringify(bad) + "\n");
      expect(() => applyProtectedWrite(request, root), name).toThrow(reason);
      const check = Bun.spawnSync(["bun", path.resolve("bin/kineti-evidence.ts"), "check", "--label", record.label], { cwd: root, stdout: "pipe", stderr: "pipe" });
      expect(check.exitCode, name).not.toBe(0);
      expect(new TextDecoder().decode(check.stderr), name).toContain(reason);
      assertSentinels();
    }
    fs.writeFileSync(ledger, JSON.stringify(record) + "\n");
    expect(applyProtectedWrite(request, root)).toContain("applied");
    expect(fs.readFileSync(path.join(root, "out.txt"), "utf8")).toBe(request.content);
  });

  test("does not create or enroll an issuer while checking a receipt", () => {
    const { record, request } = fixtureProof();
    fs.rmSync(path.join(root, ".kineti/evidence-issuer.pem"));
    fs.rmSync(path.join(root, ".kineti/evidence-issuer.pem.private"));
    const check = Bun.spawnSync(["bun", path.resolve("bin/kineti-evidence.ts"), "check", "--label", record.label], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(check.exitCode).toBe(7);
    expect(fs.existsSync(path.join(root, ".kineti/evidence-issuer.pem"))).toBe(false);
    // The writer uses the separately provisioned public policy, not signing-key availability.
    expect(applyProtectedWrite(request, root)).toContain("applied");
  });

  test("denies unreadable replay state and malformed bounded policy without consuming proof", () => {
    const { record, request } = fixtureProof();
    const replay = path.join(root, ".kineti/protected-write-runs.json");
    fs.writeFileSync(replay, "{broken");
    expect(() => applyProtectedWrite(request, root)).toThrow("replay state is unreadable or malformed");
    assertSentinels();
    fs.rmSync(replay);
    const policy = path.join(root, ".kineti/protected-write-policy.json");
    fs.writeFileSync(policy, JSON.stringify({ version: 1, trusted_issuer_key: record.issuer_key, revoked_run_ids: [17], revoked_issuer_keys: [] }));
    expect(() => applyProtectedWrite(request, root)).toThrow("policy has invalid fields");
    assertSentinels();
    trustIssuer(record.issuer_key);
    expect(applyProtectedWrite(request, root)).toContain("applied");
  });

  test("consumes proof on a deterministic failure immediately before destination rename", () => {
    const { record, request } = fixtureProof();
    const original = fs.renameSync;
    const fault = spyOn(fs, "renameSync").mockImplementation((from, to) => {
      if (String(to) === path.join(root, "out.txt")) throw Object.assign(new Error("injected pre-rename failure"), { code: "EIO" });
      return original(from, to);
    });
    try { expect(() => applyProtectedWrite(request, root)).toThrow("injected pre-rename failure"); }
    finally { fault.mockRestore(); }
    assertSentinels();
    expect(JSON.parse(fs.readFileSync(path.join(root, ".kineti/protected-write-runs.json"), "utf8"))).toEqual([record.run_id]);
    expect(() => applyProtectedWrite(request, root)).toThrow("already been used");
    expect(fs.readdirSync(path.join(root, ".kineti")).filter((f) => f.startsWith("protected-write.") || f === "protected-write.lock")).toEqual([]);
  });

  test("serializes a competing writer at the held lock with no duplicate effect", () => {
    const { record, request } = fixtureProof();
    const original = fs.writeFileSync;
    let competing = false;
    const pause = spyOn(fs, "writeFileSync").mockImplementation((...args: Parameters<typeof fs.writeFileSync>) => {
      if (path.basename(String(args[0])).startsWith("protected-write.")) {
        competing = true;
        expect(() => applyProtectedWrite(request, root)).toThrow("another protected write is already running");
        assertSentinels();
      }
      return original(...args);
    });
    try { expect(applyProtectedWrite(request, root)).toContain("applied"); }
    finally { pause.mockRestore(); }
    expect(competing).toBe(true);
    expect(JSON.parse(fs.readFileSync(path.join(root, ".kineti/protected-write-runs.json"), "utf8"))).toEqual([record.run_id]);
    expect(fs.readFileSync(path.join(root, "out.txt"), "utf8")).toBe(request.content);
  });

  test("blocks a checked input changed after consumption and preserves the destination", () => {
    const { record, request } = fixtureProof();
    const original = fs.writeFileSync;
    const source = path.join(root, "tests/one.test.ts");
    const bytes = fs.readFileSync(source);
    const fault = spyOn(fs, "writeFileSync").mockImplementation((...args: Parameters<typeof fs.writeFileSync>) => {
      const result = original(...args);
      if (path.basename(String(args[0])).startsWith("protected-write.")) original(source, "changed after check");
      return result;
    });
    try { expect(() => applyProtectedWrite(request, root)).toThrow("workspace changed before protected write"); }
    finally { fault.mockRestore(); fs.writeFileSync(source, bytes); }
    assertSentinels();
    expect(() => applyProtectedWrite(request, root)).toThrow("already been used");
    expect(JSON.parse(fs.readFileSync(path.join(root, ".kineti/protected-write-runs.json"), "utf8"))).toEqual([record.run_id]);
  });

  test("blocks a destination parent swapped for a symlink after the first check", () => {
    const { record, request } = fixtureProof(".kineti/destination/out.txt");
    const parent = path.join(root, ".kineti/destination");
    const saved = parent + "-saved";
    const original = fs.writeFileSync;
    const fault = spyOn(fs, "writeFileSync").mockImplementation((...args: Parameters<typeof fs.writeFileSync>) => {
      const result = original(...args);
      if (path.basename(String(args[0])).startsWith("protected-write.")) { fs.renameSync(parent, saved); fs.symlinkSync(outsideRoot, parent); }
      return result;
    });
    try { expect(() => applyProtectedWrite(request, root)).toThrow("symlink destination is not allowed"); }
    finally { fault.mockRestore(); fs.rmSync(parent); fs.renameSync(saved, parent); }
    expect(fs.existsSync(path.join(outsideRoot, "out.txt"))).toBe(false);
    expect(fs.readFileSync(path.join(parent, "out.txt"), "utf8")).toBe("before");
    expect(fs.readFileSync(path.join(outsideRoot, "sentinel"), "utf8")).toBe("unchanged");
    expect(JSON.parse(fs.readFileSync(path.join(root, ".kineti/protected-write-runs.json"), "utf8"))).toEqual([record.run_id]);
    expect(() => applyProtectedWrite(request, root)).toThrow("already been used");
  });

  test("keeps CLI input and execution result codes explicit", () => {
    const { request } = fixtureProof();
    const cli = path.resolve("bin/kineti-protected-write.ts");
    const run = (value: string) => Bun.spawnSync(["bun", cli], { cwd: root, stdin: Buffer.from(value), stdout: "pipe", stderr: "pipe" });
    expect(run("{broken").exitCode).toBe(2);
    expect(run(JSON.stringify({ ...request, content: null })).exitCode).toBe(2);
    expect(run(JSON.stringify({ ...request, content: "wrong" })).exitCode).toBe(1);
    // An existing directory cannot be replaced by a file; authorization otherwise matches.
    fs.rmSync(path.join(root, "out.txt"));
    fs.mkdirSync(path.join(root, "out.txt"));
    // The changed input must first be re-proved, so use a separate fixture for the effect.
    const fresh = fixtureProof(".kineti/existing-directory");
    fs.rmSync(path.join(root, ".kineti/existing-directory"));
    fs.mkdirSync(path.join(root, ".kineti/existing-directory"), { recursive: true });
    expect(run(JSON.stringify(fresh.request)).exitCode).toBe(3);
    expect(fs.statSync(path.join(root, ".kineti/existing-directory")).isDirectory()).toBe(true);
  });

  test("coordinates two real concurrent CLI requests using one proof", async () => {
    const { request, record } = fixtureProof(".kineti/out.txt");
    const cli = path.resolve("bin/kineti-protected-write.ts");
    const children = [0, 1].map(() => Bun.spawn(["bun", cli], { cwd: root, stdin: "pipe", stdout: "pipe", stderr: "pipe" }));
    for (const child of children) { child.stdin.write(JSON.stringify(request)); child.stdin.end(); }
    const results = await Promise.all(children.map(async (child) => ({ exit: await child.exited, stdout: await new Response(child.stdout).text(), stderr: await new Response(child.stderr).text() })));
    expect(results.map((r) => r.exit).sort()).toEqual([0, 1]);
    expect(results.find((r) => r.exit === 1)?.stderr).toMatch(/already running|already been used/);
    expect(fs.readFileSync(path.join(root, ".kineti/out.txt"), "utf8")).toBe(request.content);
    expect(JSON.parse(fs.readFileSync(path.join(root, ".kineti/protected-write-runs.json"), "utf8"))).toEqual([record.run_id]);
    expect(fs.readFileSync(path.join(outsideRoot, "sentinel"), "utf8")).toBe("unchanged");
  });

});
