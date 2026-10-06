#!/usr/bin/env bun
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { appendJsonl, computeDelimitedHash, die, loadVerifyCommand, nowIso, ok, projectKdir, readJsonl, runSafeCommand, sha256 } from "./lib.ts";

interface Record {
  at: string; label: string; cmd: string; exit_code: number | null;
  fingerprint: string;
}

const EXCLUDE_DIRS = new Set([
  ".git", ".kineti", ".agents", "node_modules", "dist", "build", ".next",
  "coverage", "tmp", ".cache", "legacy", "target",
]);
const EXCLUDE_FILES = new Set([".DS_Store"]);
// Files up to this size are hashed in full. Larger files record size + modified
// time instead (cheap, but an edit that keeps both the same is not detected).
export const FULL_HASH_MAX_BYTES = 50 * 1024 * 1024;
const CHUNK_BYTES = 1024 * 1024;

function file(): string { return path.join(projectKdir(), "evidence.jsonl"); }

/**
 * Hash a file in fixed-size chunks so large files do not load fully into memory.
 * For text files (no null bytes in the first 8 KB), CRLF (\r\n) is normalized to LF (\n)
 * across chunk streaming boundaries so Windows vs Unix line-ending flips do not alter hashes.
 * For binary files (containing null bytes), raw bytes are hashed verbatim.
 */
function hashFileChunked(full: string): string {
  const h = crypto.createHash("sha256");
  const fd = fs.openSync(full, "r");
  try {
    const buf = Buffer.allocUnsafe(CHUNK_BYTES);
    let n: number;
    let isBin = false;
    let first = true;
    let pendingCr = false;

    while ((n = fs.readSync(fd, buf, 0, CHUNK_BYTES, null)) > 0) {
      const chunk = buf.subarray(0, n);
      if (first) {
        first = false;
        for (let i = 0; i < Math.min(n, 8192); i++) {
          if (chunk[i] === 0) {
            isBin = true;
            break;
          }
        }
      }

      if (isBin) {
        h.update(chunk);
        continue;
      }

      // Handle a pending carriage return from the previous chunk boundary
      if (pendingCr) {
        pendingCr = false;
        if (chunk[0] !== 0x0A) {
          h.update(Buffer.from([0x0D]));
        }
      }

      const out: number[] = [];
      for (let i = 0; i < chunk.length; i++) {
        if (chunk[i] === 0x0D) {
          if (i === chunk.length - 1) {
            pendingCr = true;
          } else if (chunk[i + 1] === 0x0A) {
            // omit 0x0D immediately preceding 0x0A
          } else {
            out.push(0x0D);
          }
        } else {
          out.push(chunk[i]);
        }
      }
      if (out.length > 0) {
        h.update(Buffer.from(out));
      }
    }
    if (pendingCr) {
      h.update(Buffer.from([0x0D]));
    }
  } finally {
    fs.closeSync(fd);
  }
  return h.digest("hex");
}

/**
 * Workspace fingerprint. Every non-excluded entry contributes a (path, value) pair:
 * - regular file <= 50 MiB: sha256 (CRLF normalized to LF for non-binary files)
 * - regular file > 50 MiB:  "large:<size>:<mtimeMs>"
 * - symlink:                "symlink:<target>" (the link is not followed)
 * - unreadable file/folder: "unreadable"
 * Excluded folders and files are skipped by design. See docs/FINGERPRINT_LIMITATIONS.md.
 */
export function fingerprint(root: string = process.cwd()): string {
  const parts: string[] = [];
  const walk = (dir: string) => {
    let entries: fs.Dirent[];
    try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch {
      if (dir !== root) parts.push(path.relative(root, dir) + path.sep, "unreadable");
      return;
    }
    for (const e of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const full = path.join(dir, e.name);
      const rel = path.relative(root, full);
      if (e.isDirectory()) {
        if (!EXCLUDE_DIRS.has(e.name)) walk(full);
        continue;
      }
      if (EXCLUDE_FILES.has(e.name)) continue;
      if (e.isSymbolicLink()) {
        try { parts.push(rel, `symlink:${fs.readlinkSync(full)}`); } catch { parts.push(rel, "unreadable"); }
        continue;
      }
      if (e.isFile()) {
        try {
          const st = fs.statSync(full);
          if (st.size > FULL_HASH_MAX_BYTES) {
            parts.push(rel, `large:${st.size}:${Math.trunc(st.mtimeMs)}`);
          } else {
            parts.push(rel, hashFileChunked(full));
          }
        } catch { parts.push(rel, "unreadable"); }
      }
    }
  };
  walk(root);
  return parts.length ? computeDelimitedHash(parts) : sha256("EMPTY_WORKSPACE");
}

interface Record {
  at: string; label: string; cmd: string; exit_code: number | null;
  fingerprint: string;
  guard_failure?: "zero-tests" | "skipped-tests";
}

export function checkRunnerGuards(
  output: string,
  opts: { allowZeroTests?: boolean; forbidSkipped?: boolean } = {}
): { guardFailed: boolean; reason?: "zero-tests" | "skipped-tests"; message?: string } {
  if (!opts.allowZeroTests) {
    if (/^\s*Ran 0 tests across \d+ file/im.test(output) || /^\s*0 pass,\s+0 fail\b/im.test(output)) {
      return { guardFailed: true, reason: "zero-tests", message: "zero tests executed (bun reported 0 tests)" };
    }
    if (/^\s*Tests:\s+0 total\b/im.test(output)) {
      return { guardFailed: true, reason: "zero-tests", message: "zero tests executed (jest reported 0 total)" };
    }
    const cargoMatches = Array.from(output.matchAll(/test result: (?:ok|FAILED)\. (\d+) passed/g));
    if (cargoMatches.length > 0) {
      const totalPassed = cargoMatches.reduce((sum, m) => sum + Number(m[1]), 0);
      if (totalPassed === 0) {
        return { guardFailed: true, reason: "zero-tests", message: "cargo workspace executed 0 passed tests" };
      }
    } else if (output.includes("running 0 tests") && !/test result: ok\. [1-9]\d* passed/.test(output)) {
      return { guardFailed: true, reason: "zero-tests", message: "cargo executed 0 passed tests" };
    }
  }

  if (opts.forbidSkipped) {
    if (/^\s*.*\b[1-9]\d*\s+skip\b/im.test(output)) {
      return { guardFailed: true, reason: "skipped-tests", message: "skipped tests detected (bun reported skip)" };
    }
    if (/^\s*.*\b[1-9]\d*\s+skipped\b/im.test(output)) {
      return { guardFailed: true, reason: "skipped-tests", message: "skipped tests detected (jest/vitest/pytest reported skipped)" };
    }
    if (/^\s*.*\b[1-9]\d*\s+ignored\b/im.test(output)) {
      return { guardFailed: true, reason: "skipped-tests", message: "ignored tests detected (cargo reported ignored)" };
    }
  }

  return { guardFailed: false };
}

function records(): Record[] { return readJsonl<Record>(file()); }

function main() {
  const argv = process.argv.slice(2);
  const cmd0 = argv[0];

  if (cmd0 === "fingerprint") {
    ok(fingerprint());
    return;
  }

  if (cmd0 === "run") {
    let label = "";
    let allowShell = false;
    let allowZeroTests = false;
    let forbidSkipped = false;
    let timeoutSec = 60;
    const dd = argv.indexOf("--");
    if (dd === -1) die("run requires: run --label L [--allow-shell] [--timeout <seconds>] [--allow-zero-tests] [--forbid-skipped] -- <command...>");
    for (let i = 1; i < dd; i++) {
      if (argv[i] === "--label") label = argv[i + 1] ?? "";
      if (argv[i] === "--allow-shell") allowShell = true;
      if (argv[i] === "--allow-zero-tests") allowZeroTests = true;
      if (argv[i] === "--forbid-skipped") forbidSkipped = true;
      if (argv[i] === "--timeout") timeoutSec = Number(argv[++i]);
    }
    if (!Number.isFinite(timeoutSec) || timeoutSec <= 0) die("--timeout must be a positive number of seconds");
    const cmdArgv = argv.slice(dd + 1);
    const command = cmdArgv.join(" ");
    if (!label || cmdArgv.length === 0) die("run requires --label and a command after --");
    const fpBefore = fingerprint();
    const verifyCmd = loadVerifyCommand();
    const res = runSafeCommand(cmdArgv, { cwd: process.cwd(), workspaceRoot: process.cwd(), timeoutMs: timeoutSec * 1000, allowShell, verifyCmd });
    if (res.stdout) process.stdout.write(res.stdout);
    if (res.stderr) process.stderr.write(res.stderr + (res.stderr.endsWith("\n") ? "" : "\n"));
    let code = res.exitCode;
    let guardFailure: "zero-tests" | "skipped-tests" | undefined;

    if (code === 0) {
      const combinedOutput = (res.stdout || "") + "\n" + (res.stderr || "");
      const guardCheck = checkRunnerGuards(combinedOutput, { allowZeroTests, forbidSkipped });
      if (guardCheck.guardFailed) {
        code = 1;
        guardFailure = guardCheck.reason;
        process.stderr.write(`kineti: guard failed: ${guardCheck.message}\n`);
      }
    }

    const fpAfter = fingerprint();
    appendJsonl(file(), {
      at: nowIso(), label, cmd: command, exit_code: code,
      fingerprint: fpAfter,
      ...(guardFailure ? { guard_failure: guardFailure } : {}),
    } satisfies Record);
    if (code !== 0) die(`command failed (exit ${code}); recorded as proof anyway`, 1);
    ok(`proof recorded: ${label} ${fpBefore === fpAfter ? "(code unchanged during run)" : "(note: code changed during run)"}`);
    return;
  }

  if (cmd0 === "check") {
    let label = "", maxAgeMin = 240, expectCmd: string | null = null;
    for (let i = 1; i < argv.length; i++) {
      switch (argv[i]) {
        case "--label": label = argv[++i] ?? ""; break;
        case "--max-age": maxAgeMin = Number(argv[++i]); break;
        case "--expect-cmd": expectCmd = argv[++i] ?? null; break;
      }
    }
    if (!label) die("check requires --label L");
    const rs = records().filter((r) => r.label === label);
    if (rs.length === 0) { console.error("kineti: MISSING"); process.exit(5); }
    const last = rs[rs.length - 1];
    if (expectCmd && !last.cmd.includes(expectCmd)) { console.error("kineti: STALE (different command)"); process.exit(4); }
    const ageMin = (Date.now() - new Date(last.at).getTime()) / 60000;
    if (ageMin > maxAgeMin) { console.error(`kineti: STALE (${Math.round(ageMin)} min old)`); process.exit(4); }
    if (last.fingerprint !== fingerprint()) { console.error("kineti: STALE (code changed after the run)"); process.exit(4); }
    if (last.exit_code !== 0) {
      if (last.guard_failure) {
        console.error(`kineti: STALE (guard failed: ${last.guard_failure})`);
      } else {
        console.error("kineti: STALE (recorded run failed)");
      }
      process.exit(4);
    }
    ok(`FRESH: ${label}`);
    return;
  }

  die(`unknown command: ${cmd0}. Use fingerprint | run | check`, 2);
}

if (import.meta.main) main();
