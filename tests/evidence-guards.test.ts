// tests/evidence-guards.test.ts
// Tests for test runner guard flags in kineti-evidence.ts:
// - zero-tests guard (default ON, opt-out with --allow-zero-tests)
// - skipped-tests guard (opt-in with --forbid-skipped)
// - Cargo workspace multi-crate safety (no false alarm on 0-test sub-crates)

import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { checkRunnerGuards } from "../bin/kineti-evidence.ts";
import { readJsonl, writeJson } from "../bin/lib.ts";

describe("test runner guards (checkRunnerGuards)", () => {
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
