// tests/evidence-guards.test.ts
// Tests for test runner guard flags in kineti-evidence.ts:
// - zero-tests guard (default ON, opt-out with --allow-zero-tests)
// - skipped-tests guard (opt-in with --forbid-skipped)
// - Cargo workspace multi-crate safety (no false alarm on 0-test sub-crates)

import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { checkRunnerGuards, fingerprint, hashFileChunked, parseTestOutput, unsignedRecord, validateOperationalReceipt, validateReceipt, toolchainVersions, type EvidenceRecord } from "../bin/kineti-evidence.ts";

describe("test runner guards (checkRunnerGuards)", () => {
  it("recognizes supported pass summaries and rejects forged or incomplete text", () => {
    expect(parseTestOutput("4 pass\n0 fail\n")).toBeNull();
    expect(parseTestOutput("999 pass\n0 fail\nRan 999 tests across 1 file.\n1 pass\n0 fail\nRan 1 test across 1 file.")).toEqual({ passed: 1, failed: 0, skipped: 0 });
    expect(parseTestOutput("test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.1s")).toEqual({ passed: 4, failed: 0, skipped: 0 });
    expect(parseTestOutput("all tests passed")).toBeNull();
    expect(parseTestOutput("4 passed, 1 failed, 0 skipped in 0.01s")).toEqual({ passed: 4, failed: 1, skipped: 0 });
  });
  describe("zero-tests guard (default ON)", () => {
    it("catches Bun reporting 0 tests across files", () => {
      const output = "0 pass\n 0 fail\nRan 0 tests across 1 file. [110.00ms]";
      const res = checkRunnerGuards(output, { allowZeroTests: false });
      expect(res.guardFailed).toBe(true);
      expect(res.reason).toBe("zero-tests");
    });

    it("permits Bun reporting 0 tests when --allow-zero-tests is passed", () => {
      const output = "0 pass\n 0 fail\nRan 0 tests across 1 file. [110.00ms]";
      const res = checkRunnerGuards(output, { allowZeroTests: true });
      expect(res.guardFailed).toBe(false);
    });

    it("catches Jest reporting 0 total tests", () => {
      const output = "Test Suites: 1 passed, 1 total\nTests:       0 total\nSnapshots:   0 total";
      const res = checkRunnerGuards(output, { allowZeroTests: false });
      expect(res.guardFailed).toBe(true);
      expect(res.reason).toBe("zero-tests");
    });

    it("does NOT false-alarm on Cargo multi-crate workspace when one crate has 0 tests but another passes", () => {
      const cargoMixedOutput = `
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
`;
      const res = checkRunnerGuards(cargoMixedOutput, { allowZeroTests: false });
      expect(res.guardFailed).toBe(false);
    });

    it("catches Cargo workspace when ALL crates report 0 passed tests", () => {
      const cargoAllEmpty = `
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
`;
      const res = checkRunnerGuards(cargoAllEmpty, { allowZeroTests: false });
      expect(res.guardFailed).toBe(true);
      expect(res.reason).toBe("zero-tests");
    });

    it("passes a legitimate test run with passed tests", () => {
      const output = "15 pass\n 0 fail\nRan 15 tests across 2 files.";
      const res = checkRunnerGuards(output, { allowZeroTests: false });
      expect(res.guardFailed).toBe(false);
    });
  });

  describe("skipped-tests guard (opt-in with --forbid-skipped)", () => {
    it("ignores skips when --forbid-skipped is false (default)", () => {
      const output = "10 pass\n 2 skip\n 0 fail";
      const res = checkRunnerGuards(output, { forbidSkipped: false });
      expect(res.guardFailed).toBe(false);
    });

    it("catches Bun skip counts when --forbid-skipped is true", () => {
      const output = "10 pass\n 1 skip\n 0 fail";
      const res = checkRunnerGuards(output, { forbidSkipped: true });
      expect(res.guardFailed).toBe(true);
      expect(res.reason).toBe("skipped-tests");
    });

    it("catches Jest/Vitest/Pytest skipped counts when --forbid-skipped is true", () => {
      const output = "Tests: 4 skipped, 10 passed, 14 total";
      const res = checkRunnerGuards(output, { forbidSkipped: true });
      expect(res.guardFailed).toBe(true);
      expect(res.reason).toBe("skipped-tests");
    });

    it("catches Cargo ignored counts when --forbid-skipped is true", () => {
      const output = "test result: ok. 10 passed; 0 failed; 2 ignored";
      const res = checkRunnerGuards(output, { forbidSkipped: true });
      expect(res.guardFailed).toBe(true);
      expect(res.reason).toBe("skipped-tests");
    });

    it("does NOT false-alarm on 0 skipped or clean test runs", () => {
      const output = "10 pass\n 0 fail\nRan 10 tests across 1 file.";
      const res = checkRunnerGuards(output, { forbidSkipped: true });
      expect(res.guardFailed).toBe(false);
    });
  });
});

describe("operational evidence separation", () => {
  it("accepts signed typecheck evidence as a local result but never as test evidence", () => {
    const scratch = path.join(process.cwd(), ".kineti", "test-evidence-operational");
    fs.mkdirSync(scratch, { recursive: true });
    const root = fs.mkdtempSync(path.join(scratch, "run-"));
    try {
      fs.writeFileSync(path.join(root, "input.txt"), "fixture\n");
      const pair = crypto.generateKeyPairSync("ed25519");
      const publicKey = pair.publicKey.export({ type: "spki", format: "pem" }).toString();
      const privateKey = pair.privateKey.export({ type: "pkcs8", format: "pem" });
      const inputDigest = fingerprint(root);
      const receipt: EvidenceRecord = {
        receipt_version: 2,
        run_id: crypto.randomUUID(),
        at: new Date().toISOString(),
        label: "typecheck-fixture",
        cmd: "bun run typecheck",
        result_kind: "check",
        outcome: "passed",
        exit_code: 0,
        fingerprint: inputDigest,
        manifest: {
          command: ["bun", "run", "typecheck"],
          dependencies: [],
          toolchain: toolchainVersions(),
          environment: {},
          input_digest: inputDigest,
          artifact_digests: [],
          parser_sha256: hashFileChunked(path.join(import.meta.dir, "..", "bin", "kineti-evidence.ts")),
        },
        issuer_key: publicKey,
      };
      receipt.signature = crypto.sign(null, Buffer.from(unsignedRecord(receipt)), privateKey).toString("base64");

      expect(validateOperationalReceipt(receipt, { root, trustedPublicKey: publicKey }).outcome).toBe("passed");
      expect(() => validateReceipt(receipt, { root, trustedPublicKey: publicKey }))
        .toThrow("only test receipts can authorize the exact-input gate");
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
});
