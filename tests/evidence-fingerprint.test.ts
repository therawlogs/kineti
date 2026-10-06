// tests/evidence-fingerprint.test.ts
// Workspace fingerprint: blind spots that used to fail unsafe, and kept behaviors.
// See docs/FINGERPRINT_LIMITATIONS.md.

import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { FULL_HASH_MAX_BYTES, fingerprint } from "../bin/kineti-evidence.ts";
import { computeDelimitedHash, sha256 } from "../bin/lib.ts";

describe("workspace fingerprint", () => {
  let dir: string;

  beforeEach(() => {
    const scratch = path.join(process.cwd(), ".kineti", "test-fingerprint");
    fs.mkdirSync(scratch, { recursive: true });
    dir = fs.mkdtempSync(path.join(scratch, "run-"));
  });

  afterEach(() => {
    // Restore permissions so cleanup can remove everything.
    for (const p of [path.join(dir, "locked.txt"), path.join(dir, "lockeddir")]) {
      try { fs.chmodSync(p, 0o700); } catch { /* not present */ }
    }
    fs.rmSync(dir, { recursive: true, force: true });
  });

  it("keeps the same value for ordinary files as the previous algorithm", () => {
    fs.writeFileSync(path.join(dir, "a.ts"), "export const a = 1;\n");
    const expected = computeDelimitedHash(["a.ts", sha256(Buffer.from("export const a = 1;\n"))]);
    expect(fingerprint(dir)).toBe(expected);
  });

  it("detects edits to files between 4 MiB and 50 MiB (previously skipped)", () => {
    const big = path.join(dir, "big.bin");
    fs.writeFileSync(big, Buffer.alloc(5 * 1024 * 1024, 1));
    const before = fingerprint(dir);
    const fd = fs.openSync(big, "r+");
    fs.writeSync(fd, Buffer.from([2]), 0, 1, 3 * 1024 * 1024);
    fs.closeSync(fd);
    expect(fingerprint(dir)).not.toBe(before);
  });

  it("files over 50 MiB: size or modified-time changes are detected", () => {
    const huge = path.join(dir, "huge.bin");
    fs.writeFileSync(huge, "");
    fs.truncateSync(huge, FULL_HASH_MAX_BYTES + 1);
    const before = fingerprint(dir);
    fs.truncateSync(huge, FULL_HASH_MAX_BYTES + 2);
    expect(fingerprint(dir)).not.toBe(before);
  });

  it("files over 50 MiB: an edit that keeps size and modified time is NOT detected (documented limit)", () => {
    const huge = path.join(dir, "huge.bin");
    fs.writeFileSync(huge, "");
    fs.truncateSync(huge, FULL_HASH_MAX_BYTES + 1);
    const t = new Date("2026-01-01T00:00:00Z");
    fs.utimesSync(huge, t, t);
    const before = fingerprint(dir);
    const fd = fs.openSync(huge, "r+");
    fs.writeSync(fd, Buffer.from([7]), 0, 1, 10);
    fs.closeSync(fd);
    fs.utimesSync(huge, t, t);
    expect(fingerprint(dir)).toBe(before);
  });

  it("detects a symlink pointing somewhere new (previously ignored)", () => {
    fs.writeFileSync(path.join(dir, "one.txt"), "1");
    fs.writeFileSync(path.join(dir, "two.txt"), "2");
    fs.symlinkSync("one.txt", path.join(dir, "link"));
    const before = fingerprint(dir);
    fs.unlinkSync(path.join(dir, "link"));
    fs.symlinkSync("two.txt", path.join(dir, "link"));
    expect(fingerprint(dir)).not.toBe(before);
  });

  it("detects adding a symlink", () => {
    fs.writeFileSync(path.join(dir, "one.txt"), "1");
    const before = fingerprint(dir);
    fs.symlinkSync("one.txt", path.join(dir, "link"));
    expect(fingerprint(dir)).not.toBe(before);
  });

  const isRoot = typeof process.getuid === "function" && process.getuid() === 0;

  it.skipIf(isRoot)("an unreadable file still changes the fingerprint (previously skipped)", () => {
    const before = fingerprint(dir);
    fs.writeFileSync(path.join(dir, "locked.txt"), "secret");
    fs.chmodSync(path.join(dir, "locked.txt"), 0o000);
    expect(fingerprint(dir)).not.toBe(before);
  });

  it.skipIf(isRoot)("an unreadable folder still changes the fingerprint", () => {
    const before = fingerprint(dir);
    fs.mkdirSync(path.join(dir, "lockeddir"));
    fs.writeFileSync(path.join(dir, "lockeddir", "x.txt"), "x");
    fs.chmodSync(path.join(dir, "lockeddir"), 0o000);
    expect(fingerprint(dir)).not.toBe(before);
  });

  it("normalizes text CRLF to LF: CRLF vs LF produces the exact same fingerprint", () => {
    fs.writeFileSync(path.join(dir, "a.ts"), "a\nb\n");
    const before = fingerprint(dir);
    fs.writeFileSync(path.join(dir, "a.ts"), "a\r\nb\r\n");
    expect(fingerprint(dir)).toBe(before);
  });

  it("binary files with CRLF bytes are not normalized", () => {
    fs.writeFileSync(path.join(dir, "a.bin"), Buffer.from([0, 13, 10, 1]));
    const before = fingerprint(dir);
    fs.writeFileSync(path.join(dir, "a.bin"), Buffer.from([0, 10, 1]));
    expect(fingerprint(dir)).not.toBe(before);
  });

  it("excluded folders are still skipped by design", () => {
    fs.writeFileSync(path.join(dir, "a.ts"), "a");
    const before = fingerprint(dir);
    fs.mkdirSync(path.join(dir, "node_modules"));
    fs.writeFileSync(path.join(dir, "node_modules", "x.js"), "x");
    expect(fingerprint(dir)).toBe(before);
  });
});
