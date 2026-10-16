import { afterEach, describe, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { applyProtectedWrite } from "../bin/kineti-protected-write.ts";
import { verifyReceiptSignature } from "../bin/kineti-evidence.ts";
import crypto from "node:crypto";

const root = path.join(process.cwd(), ".kineti", "protected-write-test");
afterEach(() => fs.rmSync(root, { recursive: true, force: true }));

describe("Cursor MCP protected write", () => {
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
    const base = { label: "release", runId: id!, destination: "result.txt", arguments: { mode: "release" }, content: "approved bytes\n" };
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

  test("denies a signed receipt whose command does not match the project verification command", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/other.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "required.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "release", "--required-test", "tests/required.test.ts", "--protected-write", JSON.stringify({ destination: "out.txt", arguments: {}, content: "x" }), "--", "bun", "test", "tests/required.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    const record = JSON.parse(fs.readFileSync(path.join(root, ".kineti", "evidence.jsonl"), "utf8").trim());
    expect(verifyReceiptSignature(record)).toBe(true);
    const wrongKey = crypto.generateKeyPairSync("ed25519").publicKey.export({ type: "spki", format: "pem" }).toString();
    expect(verifyReceiptSignature(record, wrongKey)).toBe(false);
    expect(verifyReceiptSignature({ ...record, issuer_key: wrongKey })).toBe(false);
    const id = record.run_id;
    expect(run.exitCode).toBe(0);
    expect(id).toBeTruthy();
    expect(() => applyProtectedWrite({ label: "release", runId: id!, destination: "out.txt", arguments: {}, content: "x" }, root)).toThrow();
  });

  test("denies forged runner output under a valid label and detects receipt tampering", () => {
    fs.mkdirSync(path.join(root, "tests"), { recursive: true });
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify({ settings: { verify_command: "bun test tests/other.test.ts" } }));
    fs.writeFileSync(path.join(root, "tests", "required.test.ts"), "import { test, expect } from 'bun:test'; test('one', () => expect(1).toBe(1));\n");
    const script = path.resolve("bin/kineti-evidence.ts");
    const run = Bun.spawnSync(["bun", script, "run", "--label", "release", "--required-test", "tests/required.test.ts", "--protected-write", JSON.stringify({ destination: "out.txt", arguments: {}, content: "x" }), "--", "bun", "test", "tests/required.test.ts"], { cwd: root, stdout: "pipe", stderr: "pipe" });
    expect(run.exitCode).toBe(0);
    const recordPath = path.join(root, ".kineti", "evidence.jsonl");
    const record = JSON.parse(fs.readFileSync(recordPath, "utf8").trim());
    expect(() => applyProtectedWrite({ label: "release", runId: record.run_id, destination: "out.txt", arguments: {}, content: "x" }, root)).toThrow("configured verification command");
    record.manifest.protected_effect.destination = "other.txt";
    fs.writeFileSync(recordPath, JSON.stringify(record) + "\n");
    expect(() => applyProtectedWrite({ label: "release", runId: record.run_id, destination: "other.txt", arguments: {}, content: "x" }, root)).toThrow("signature");
  });
});
