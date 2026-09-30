import { describe, test, expect } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import {
  splitLegacyCommand,
  hasShellMetachars,
  warnOnUnknownConfigKeys,
  loadLimits,
} from "../bin/lib.ts";

const REPO = path.resolve(import.meta.dir, "..");

function makeScratch(): string {
  const scratch = path.join(REPO, ".kineti", "scratch");
  fs.mkdirSync(scratch, { recursive: true });
  return fs.mkdtempSync(path.join(scratch, "lib-test-"));
}

describe("A4: splitLegacyCommand is quote-aware", () => {
  test("unquoted input splits on whitespace as before", () => {
    expect(splitLegacyCommand("git checkout -- file.txt")).toEqual(["git", "checkout", "--", "file.txt"]);
    expect(splitLegacyCommand("  bun   test  ")).toEqual(["bun", "test"]);
  });

  test("empty and blank input gives []", () => {
    expect(splitLegacyCommand("")).toEqual([]);
    expect(splitLegacyCommand("   ")).toEqual([]);
  });

  test("double-quoted path with spaces stays one token", () => {
    expect(splitLegacyCommand('git checkout -- "my dir/file.txt"')).toEqual([
      "git",
      "checkout",
      "--",
      "my dir/file.txt",
    ]);
  });

  test("single-quoted path with spaces stays one token", () => {
    expect(splitLegacyCommand("git checkout -- 'my dir/file.txt'")).toEqual([
      "git",
      "checkout",
      "--",
      "my dir/file.txt",
    ]);
  });

  test("backslash escapes a space", () => {
    expect(splitLegacyCommand("git checkout -- my\\ dir/file.txt")).toEqual([
      "git",
      "checkout",
      "--",
      "my dir/file.txt",
    ]);
  });

  test("quoted undo inverse passes the downstream metachar check", () => {
    const argv = splitLegacyCommand('git checkout -- "my dir/file.txt"');
    expect(hasShellMetachars(argv)).toBe(false);
  });
});

describe("A6: warnOnUnknownConfigKeys", () => {
  test("warns on unknown top-level and settings keys, never throws", () => {
    const dir = makeScratch();
    try {
      fs.writeFileSync(
        path.join(dir, "kineti.config.json"),
        JSON.stringify({
          version: "0.3.7",
          bogus_top: true,
          settings: { spend_limit_usd: { global: 50 }, ci_stage: 9, typo_key: 1 },
        }),
      );
      const warnings: string[] = [];
      const orig = console.warn;
      console.warn = (m: unknown) => {
        warnings.push(String(m));
      };
      try {
        expect(() => warnOnUnknownConfigKeys(dir)).not.toThrow();
        expect(() => loadLimits(dir)).not.toThrow();
      } finally {
        console.warn = orig;
      }
      const joined = warnings.join("\n");
      expect(joined).toContain("bogus_top");
      expect(joined).toContain("typo_key");
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  test("known keys produce no warning", () => {
    const dir = makeScratch();
    try {
      fs.writeFileSync(
        path.join(dir, "kineti.config.json"),
        JSON.stringify({
          version: "0.3.7",
          name: "Kineti OS",
          root_goal: "g",
          settings: { spend_limit_usd: { global: 50 }, verify_command: "bun test", ci_stage: 9 },
        }),
      );
      const warnings: string[] = [];
      const orig = console.warn;
      console.warn = (m: unknown) => {
        warnings.push(String(m));
      };
      try {
        warnOnUnknownConfigKeys(dir);
      } finally {
        console.warn = orig;
      }
      expect(warnings.join("\n")).not.toContain("unrecognized");
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  test("missing config file never throws", () => {
    const dir = makeScratch();
    try {
      expect(() => warnOnUnknownConfigKeys(dir)).not.toThrow();
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
});
