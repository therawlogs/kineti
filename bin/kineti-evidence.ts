#!/usr/bin/env bun
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { appendJsonl, computeDelimitedHash, die, loadVerifyCommand, nowIso, ok, projectKdir, readJsonl, runSafeCommand, sha256 } from "./lib.ts";

export interface EvidenceRecord {
  receipt_version?: 2;
  run_id?: string;
  at: string;
  label: string;
  cmd: string;
  exit_code: number | null;
  fingerprint: string;
  guard_failure?: "zero-tests" | "skipped-tests" | "failed-tests" | "unknown-output" | "missing-tests" | "toolchain-changed" | "code-mutated";
  mutated_during_run?: boolean;
  manifest?: ExactInputManifest;
  verification?: { status: "valid" | "limited" | "invalid" | "unsupported"; parser: string; passed: number; failed: number; skipped: number; inventory: Array<{ path: string; sha256: string }> };
  issuer_key?: string;
  signature?: string;
}
export interface ExactInputManifest {
  command: string[];
  dependencies: Array<{ path: string; sha256: string }>;
  toolchain: { [name: string]: string };
  environment: { [name: string]: string };
  input_digest: string;
  artifact_digests: Array<{ path: string; sha256: string }>;
  parser_sha256: string;
  protected_effect?: { destination: string; arguments: unknown; bytes_sha256: string; bytes_base64: string };
}
type Record = EvidenceRecord;

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
function keyFile(): string { return path.join(projectKdir(), "evidence-issuer.pem"); }
function issuerKeypair() {
  const pubFile = keyFile(), privFile = `${pubFile}.private`;
  fs.mkdirSync(path.dirname(pubFile), { recursive: true });
  const lockFile = `${pubFile}.init.lock`;
  let lockFd: number | null = null;
  for (let attempt = 0; attempt < 500 && (!fs.existsSync(pubFile) || !fs.existsSync(privFile)); attempt++) {
    try { lockFd = fs.openSync(lockFile, "wx", 0o600); break; }
    catch { Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10); }
  }
  try {
    if (!fs.existsSync(pubFile) || !fs.existsSync(privFile)) {
      if (lockFd === null) throw new Error("timed out initializing receipt issuer key");
      const pair = crypto.generateKeyPairSync("ed25519");
      const nonce = `${process.pid}.${crypto.randomBytes(8).toString("hex")}`;
      const privateTemp = `${privFile}.${nonce}.tmp`, publicTemp = `${pubFile}.${nonce}.tmp`;
      fs.writeFileSync(privateTemp, pair.privateKey.export({ type: "pkcs8", format: "pem" }), { flag: "wx", mode: 0o600 });
      fs.writeFileSync(publicTemp, pair.publicKey.export({ type: "spki", format: "pem" }), { flag: "wx", mode: 0o644 });
      fs.renameSync(privateTemp, privFile);
      fs.renameSync(publicTemp, pubFile);
    }
  } finally {
    if (lockFd !== null) { fs.closeSync(lockFd); fs.rmSync(lockFile, { force: true }); }
  }
  const publicKey = fs.readFileSync(pubFile, "utf8");
  return { privateKey: fs.readFileSync(privFile, "utf8"), publicKey };
}
export function unsignedRecord(record: EvidenceRecord): string {
  const { signature: _sig, ...unsigned } = record;
  return JSON.stringify(unsigned);
}
export function verifyReceiptSignature(record: EvidenceRecord, trustedPublicKey?: string): boolean {
  if (!record.signature || !record.issuer_key || (trustedPublicKey && record.issuer_key !== trustedPublicKey)) return false;
  try { return crypto.verify(null, Buffer.from(unsignedRecord(record)), crypto.createPublicKey(record.issuer_key), Buffer.from(record.signature, "base64")); } catch { return false; }
}

/**
 * Hash a file in fixed-size chunks so large files do not load fully into memory.
 * For text files (no null bytes in the first 8 KB), CRLF (\r\n) is normalized to LF (\n)
 * across chunk streaming boundaries so Windows vs Unix line-ending flips do not alter hashes.
 * For binary files (containing null bytes), raw bytes are hashed verbatim.
 */
export function hashFileChunked(full: string): string {
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
      if (dir !== root) parts.push(path.relative(root, dir).split(path.sep).join("/") + "/", "unreadable");
      return;
    }
    for (const e of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const full = path.join(dir, e.name);
      const rel = path.relative(root, full);
      const relPosix = rel.split(path.sep).join("/");
      if (e.isDirectory()) {
        if (!EXCLUDE_DIRS.has(e.name)) walk(full);
        continue;
      }
      if (EXCLUDE_FILES.has(e.name)) continue;
      if (e.isSymbolicLink()) {
        try { parts.push(relPosix, `symlink:${fs.readlinkSync(full).split(path.sep).join("/")}`); } catch { parts.push(relPosix, "unreadable"); }
        continue;
      }
      if (e.isFile()) {
        try {
          const st = fs.statSync(full);
          if (st.size > FULL_HASH_MAX_BYTES) {
            parts.push(relPosix, `large:${st.size}:${Math.trunc(st.mtimeMs)}`);
          } else {
            parts.push(relPosix, hashFileChunked(full));
          }
        } catch { parts.push(relPosix, "unreadable"); }
      }
    }
  };
  walk(root);
  return parts.length ? computeDelimitedHash(parts) : sha256("EMPTY_WORKSPACE");
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

function printHelp(): void {
  console.log(`
kineti evidence / test - run commands and verify cryptographic proof receipts

Commands:
  fingerprint                     Print current workspace sha256 fingerprint
  run --label L [options] -- <cmd> Execute test and save proof receipt
  check --label L [options]       Verify receipt freshness and workspace state

Run Options:
  --label <name>                  Receipt label (required)
  --allow-shell                   Allow shell execution (interactive TTY only)
  --allow-zero-tests              Allow test runs with 0 reported tests
  --forbid-skipped                Treat skipped tests as failure
  --allow-workspace-mutation      Allow test run that mutates workspace files
  --timeout <seconds>             Command timeout (default 60s)

Check Options:
  --label <name>                  Receipt label to check (required)
  --max-age <minutes>             Maximum receipt age in minutes (default 240)
  --expect-cmd <substring>        Ensure command matches expected substring
  --allow-workspace-mutation      Allow receipts where workspace mutated during test
  --run-id <id>                   Select a unique receipt run
`);
}

const RECEIPT_PARSER_VERSION = "kineti-test-output-v1";
const SECRET_ENV_NAME = /(secret|token|password|passwd|credential|private.?key|api.?key|auth)/i;
function digestFile(p: string): string { return hashFileChunked(p); }
export function parseTestOutput(output: string): { passed: number; failed: number; skipped: number } | null {
  const cargo = Array.from(output.matchAll(/test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;/g));
  if (cargo.length) return cargo.reduce((a, m) => ({ passed: a.passed + +m[1], failed: a.failed + +m[2], skipped: a.skipped + +m[3] }), { passed: 0, failed: 0, skipped: 0 });
  const bunSummary = Array.from(output.matchAll(/^Ran (\d+) tests? across \d+ files?\./gim)).at(-1);
  if (bunSummary?.index !== undefined) {
    const beforeSummary = output.slice(0, bunSummary.index);
    const passMatches = Array.from(beforeSummary.matchAll(/^\s*(\d+) pass\s*$/gim));
    const failMatches = Array.from(beforeSummary.matchAll(/^\s*(\d+) fail\s*$/gim));
    const skipMatches = Array.from(beforeSummary.matchAll(/^\s*(\d+) skip(?:ped)?\s*$/gim));
    const bun = passMatches.at(-1), bunFail = failMatches.at(-1), bunSkip = skipMatches.at(-1);
    if (bun && bunFail && +bunSummary[1] === +bun[1] + +bunFail[1] + +(bunSkip?.[1] ?? 0)) return { passed: +bun[1], failed: +bunFail[1], skipped: +(bunSkip?.[1] ?? 0) };
    return null;
  }
  const pytest = Array.from(output.matchAll(/(?:=+\s*)?(\d+) passed(?:,\s*(\d+) failed)?(?:,\s*(\d+) skipped)?(?:\s+in\s+[\d.]+s)?(?:\s*=+)?/gi)).at(-1);
  if (pytest) return { passed: +pytest[1], failed: +(pytest[2] ?? 0), skipped: +(pytest[3] ?? 0) };
  return null;
}
function toolchainVersions(): { [name: string]: string } {
  const get = (cmd: string[]) => { try { const p = Bun.spawnSync(cmd, { stdout: "pipe", stderr: "pipe" }); return p.exitCode === 0 ? new TextDecoder().decode(p.stdout).trim().split("\n")[0] : "unavailable"; } catch { return "unavailable"; } };
  const resolve = (name: string) => { try { return Bun.which(name) ?? "unavailable"; } catch { return "unavailable"; } };
  const bunPath = resolve("bun"), nodePath = process.execPath, cargoPath = resolve("cargo"), rustcPath = resolve("rustc");
  return {
    kineti: JSON.parse(fs.readFileSync(path.join(import.meta.dir, "..", "package.json"), "utf8")).version,
    node: process.version, node_path: nodePath, node_sha256: fs.existsSync(nodePath) ? digestFile(nodePath) : "unavailable",
    bun: Bun.version, bun_path: bunPath, bun_sha256: bunPath !== "unavailable" ? digestFile(bunPath) : "unavailable",
    os: `${process.platform}-${process.arch}`, runner: "declared-command",
    bun_runner: get(["bun", "--version"]), rustc: get(["rustc", "--version"]), rustc_path: rustcPath,
    rustc_sha256: rustcPath !== "unavailable" ? digestFile(rustcPath) : "unavailable",
    cargo: get(["cargo", "--version"]), cargo_path: cargoPath, cargo_sha256: cargoPath !== "unavailable" ? digestFile(cargoPath) : "unavailable",
  };
}
export { toolchainVersions };

function main() {
  const argv = process.argv.slice(2);
  const cmd0 = argv[0];

  if (!cmd0 || cmd0 === "--help" || cmd0 === "-h") {
    printHelp();
    return;
  }

  if (cmd0 === "fingerprint") {
    ok(fingerprint());
    return;
  }

  if (cmd0 === "run") {
    const dd = argv.indexOf("--");
    const preArgs = dd === -1 ? argv.slice(1) : argv.slice(1, dd);
    if (preArgs.some((a) => a === "--help" || a === "-h")) {
      printHelp();
      return;
    }
    let label = "";
    let allowShell = false;
    let allowZeroTests = false;
    let forbidSkipped = false;
    let allowWorkspaceMutation = false;
    let timeoutSec = 60;
    const requiredTests: string[] = [], envNames: string[] = [], artifacts: string[] = [];
    let protectedWrite: { destination: string; arguments: unknown; content: string } | undefined;
    if (dd === -1) die("run requires: run --label L [--allow-shell] [--timeout <seconds>] [--allow-zero-tests] [--forbid-skipped] [--allow-workspace-mutation] -- <command...>");
    for (let i = 1; i < dd; i++) {
      if (argv[i] === "--label") label = argv[i + 1] ?? "";
      if (argv[i] === "--allow-shell") allowShell = true;
      if (argv[i] === "--allow-zero-tests") allowZeroTests = true;
      if (argv[i] === "--forbid-skipped") forbidSkipped = true;
      if (argv[i] === "--allow-workspace-mutation") allowWorkspaceMutation = true;
      if (argv[i] === "--timeout") timeoutSec = Number(argv[++i]);
      if (argv[i] === "--required-test") requiredTests.push(argv[++i] ?? "");
      if (argv[i] === "--env-name") envNames.push(argv[++i] ?? "");
      if (argv[i] === "--artifact") artifacts.push(argv[++i] ?? "");
      if (argv[i] === "--protected-write") { try { protectedWrite = JSON.parse(argv[++i] ?? "") as typeof protectedWrite; } catch { die("--protected-write must be JSON", 2); } }
    }
    if (!Number.isFinite(timeoutSec) || timeoutSec <= 0) die("--timeout must be a positive number of seconds");
    const cmdArgv = argv.slice(dd + 1);
    const command = cmdArgv.join(" ");
    if (!label || cmdArgv.length === 0) die("run requires --label and a command after --");
    const fpBefore = fingerprint();
    const toolchainBefore = toolchainVersions();
    const verifyCmd = loadVerifyCommand();
    const res = runSafeCommand(cmdArgv, { cwd: process.cwd(), workspaceRoot: process.cwd(), timeoutMs: timeoutSec * 1000, allowShell, verifyCmd });
    if (res.stdout) process.stdout.write(res.stdout);
    if (res.stderr) process.stderr.write(res.stderr + (res.stderr.endsWith("\n") ? "" : "\n"));
    let code = res.exitCode;
    let guardFailure: "zero-tests" | "skipped-tests" | "failed-tests" | "unknown-output" | "missing-tests" | "toolchain-changed" | "code-mutated" | undefined;

    let outputVerification: ReturnType<typeof parseTestOutput> = null;
    if (code === 0) {
      const combinedOutput = (res.stdout || "") + "\n" + (res.stderr || "");
      outputVerification = parseTestOutput(combinedOutput);
      if (!outputVerification) {
        code = 1;
        guardFailure = "unknown-output";
        process.stderr.write("kineti: guard failed: runner output is unknown or has no supported test summary\n");
      }
      const guardCheck = checkRunnerGuards(combinedOutput, { allowZeroTests, forbidSkipped });
      if (code === 0 && guardCheck.guardFailed) {
        code = 1;
        guardFailure = guardCheck.reason;
        process.stderr.write(`kineti: guard failed: ${guardCheck.message}\n`);
      }
      if (code === 0 && outputVerification && (outputVerification.passed === 0 || outputVerification.failed !== 0 || outputVerification.skipped !== 0)) {
        code = 1; guardFailure = outputVerification.passed === 0 ? "zero-tests" : outputVerification.failed > 0 ? "failed-tests" : "skipped-tests";
        process.stderr.write(`kineti: guard failed: tests passed=${outputVerification.passed}, failed=${outputVerification.failed}, skipped=${outputVerification.skipped}\n`);
      }
    }

    const inventoryPaths = requiredTests.flatMap((rel) => {
      const full = path.resolve(process.cwd(), rel);
      if (!full.startsWith(path.resolve(process.cwd()) + path.sep) || !fs.existsSync(full)) throw new Error(`required test missing or outside workspace: ${rel}`);
      if (fs.statSync(full).isFile()) return [rel];
      if (!fs.statSync(full).isDirectory()) throw new Error(`required test is not a file or directory: ${rel}`);
      const found: string[] = [];
      const walk = (dir: string) => { for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a,b) => a.name.localeCompare(b.name))) { const child = path.join(dir, entry.name); if (entry.isDirectory()) walk(child); else if (entry.isFile() && /\.(test|spec)\.[cm]?[jt]sx?$/.test(entry.name)) found.push(path.relative(process.cwd(), child)); } };
      walk(full);
      if (!found.length) throw new Error(`required test inventory is empty: ${rel}`);
      return found;
    });
    const inventory = inventoryPaths.map((rel) => ({ path: rel.split(path.sep).join("/"), sha256: digestFile(path.resolve(process.cwd(), rel)) }));
    if (outputVerification && cmdArgv[0] === "bun" && cmdArgv[1] === "test") {
      const combinedOutput = (res.stdout || "") + "\n" + (res.stderr || "");
      const missingOutput = inventory.filter((item) => !combinedOutput.includes(item.path));
      if (missingOutput.length) {
        code = 1; guardFailure = "missing-tests";
        process.stderr.write(`kineti: guard failed: test runner output did not name required tests: ${missingOutput.map((item) => item.path).join(", ")}\n`);
      }
    }
    for (const name of envNames) if (!/^[A-Z_][A-Z0-9_]*$/i.test(name) || SECRET_ENV_NAME.test(name)) die(`environment input is invalid or secret-like and cannot be recorded: ${name}`, 2);
    const envEvidence = Object.fromEntries(envNames.map((name) => [name, sha256(process.env[name] ?? "<unset>")]));
    const lockPaths = ["bun.lock", "bun.lockb", "package-lock.json", "npm-shrinkwrap.json", "yarn.lock", "pnpm-lock.yaml", "Cargo.lock", "requirements.txt", "poetry.lock", "uv.lock"].filter((p) => fs.existsSync(p) && fs.statSync(p).isFile());
    const dependencies = lockPaths.map((p) => ({ path: p, sha256: digestFile(p) }));
    const artifactDigests = artifacts.map((rel) => {
      const full = path.resolve(process.cwd(), rel);
      if (!full.startsWith(path.resolve(process.cwd()) + path.sep) || !fs.existsSync(full) || !fs.statSync(full).isFile()) throw new Error(`artifact missing or outside workspace: ${rel}`);
      return { path: rel.split(path.sep).join("/"), sha256: digestFile(full) };
    });
    if (protectedWrite && (!protectedWrite.destination || typeof protectedWrite.content !== "string" || !Object.hasOwn(protectedWrite, "arguments"))) die("protected write requires destination, arguments, and string content", 2);
    const protectedEffect = protectedWrite ? {
      destination: path.relative(process.cwd(), path.resolve(process.cwd(), protectedWrite.destination)).split(path.sep).join("/"),
      arguments: protectedWrite.arguments,
      bytes_sha256: sha256(Buffer.from(protectedWrite.content, "utf8")),
      bytes_base64: Buffer.from(protectedWrite.content, "utf8").toString("base64"),
    } : undefined;

    const fpAfter = fingerprint();
    if (JSON.stringify(toolchainBefore) !== JSON.stringify(toolchainVersions()) && code === 0) {
      code = 1; guardFailure = "toolchain-changed";
      process.stderr.write("kineti: guard failed: runner or toolchain changed during test execution\n");
    }
    const codeMutated = fpBefore !== fpAfter;
    if (codeMutated && !allowWorkspaceMutation && code === 0) {
      code = 1;
      guardFailure = "code-mutated";
      process.stderr.write("kineti: guard failed: workspace was modified during test execution (use --allow-workspace-mutation to allow)\n");
    }

    const runId = crypto.randomUUID();
    const parsed = outputVerification;
    const record: EvidenceRecord = {
      receipt_version: 2, run_id: runId,
      at: nowIso(), label, cmd: command, exit_code: code,
      fingerprint: fpAfter,
      manifest: { command: cmdArgv, dependencies, toolchain: toolchainBefore, environment: envEvidence, input_digest: fpBefore, artifact_digests: artifactDigests, parser_sha256: digestFile(path.join(import.meta.dir, "kineti-evidence.ts")), ...(protectedEffect ? { protected_effect: protectedEffect } : {}) },
      verification: { status: parsed && code === 0 && inventory.length > 0 ? "valid" : parsed && code === 0 ? "unsupported" : "invalid", parser: RECEIPT_PARSER_VERSION, passed: parsed?.passed ?? 0, failed: parsed?.failed ?? (code === 0 ? 1 : 0), skipped: parsed?.skipped ?? 0, inventory },
      ...(guardFailure ? { guard_failure: guardFailure } : {}),
      ...(codeMutated ? { mutated_during_run: true } : {}),
    } satisfies Record;
    const keys = issuerKeypair();
    record.issuer_key = keys.publicKey;
    record.signature = crypto.sign(null, Buffer.from(unsignedRecord(record)), keys.privateKey).toString("base64");
    appendJsonl(file(), record);
    if (code !== 0) die(`command failed (exit ${code}); recorded as proof anyway`, 1);
    ok(`proof recorded: ${label} run=${runId} ${!codeMutated ? "(code unchanged during run)" : "(note: code changed during run)"}`);
    return;
  }

  if (cmd0 === "check") {
    if (argv.slice(1).some((a) => a === "--help" || a === "-h")) {
      printHelp();
      return;
    }
    let label = "", runId: string | null = null, maxAgeMin = 240, expectCmd: string | null = null;
    let allowWorkspaceMutation = false;
    for (let i = 1; i < argv.length; i++) {
      switch (argv[i]) {
        case "--label": label = argv[++i] ?? ""; break;
        case "--run-id": runId = argv[++i] ?? ""; break;
        case "--max-age": {
          const raw = argv[++i];
          const parsed = Number(raw);
          if (!Number.isFinite(parsed) || parsed <= 0) {
            die("--max-age must be a positive finite number of minutes", 2);
          }
          maxAgeMin = parsed;
          break;
        }
        case "--expect-cmd": expectCmd = argv[++i] ?? null; break;
        case "--allow-workspace-mutation": allowWorkspaceMutation = true; break;
      }
    }
    if (!label) die("check requires --label L");
    const rs = records().filter((r) => r.label === label && (!runId || r.run_id === runId));
    if (rs.length === 0) { console.error("kineti: MISSING"); process.exit(5); }
    const last = rs[rs.length - 1];
    if (last.receipt_version === 2 && (!last.signature || !last.issuer_key)) { console.error("kineti: INVALID (receipt issuer signature is missing)"); process.exit(7); }
    if (last.receipt_version === 2 && last.signature && last.issuer_key) {
    try {
      const key = crypto.createPublicKey(last.issuer_key);
      const expected = crypto.createHash("sha256").update(key.export({ type: "spki", format: "der" })).digest("hex");
      const actual = crypto.createHash("sha256").update(crypto.createPublicKey(issuerKeypair().publicKey).export({ type: "spki", format: "der" })).digest("hex");
      if (expected !== actual || !verifyReceiptSignature(last)) throw new Error("signature mismatch");
    } catch { console.error("kineti: INVALID (wrong issuer key or receipt signature)"); process.exit(7); }
    }
    if (expectCmd && !last.cmd.includes(expectCmd)) { console.error("kineti: STALE (different command)"); process.exit(4); }
    const receiptTime = Date.parse(last.at);
    if (Number.isNaN(receiptTime)) {
      console.error("kineti: STALE (invalid receipt timestamp)");
      process.exit(4);
    }
    const now = Date.now();
    const ageMs = now - receiptTime;
    if (ageMs < -60000) {
      console.error("kineti: STALE (receipt timestamp is in the future)");
      process.exit(4);
    }
    const ageMin = Math.max(0, ageMs) / 60000;
    if (ageMin > maxAgeMin) { console.error(`kineti: STALE (${Math.round(ageMin)} min old)`); process.exit(4); }
    if (last.receipt_version !== 2 || !last.manifest || !last.verification) {
      console.error("kineti: LIMITED (legacy receipt is readable but cannot authorize the exact-input gate)"); process.exit(6);
    }
    if (last.verification.status !== "valid" || last.verification.parser !== RECEIPT_PARSER_VERSION || last.verification.passed === 0 || last.verification.failed !== 0 || last.verification.skipped !== 0) {
      console.error("kineti: INVALID (receipt verification evidence is missing, unknown, or failing)"); process.exit(7);
    }
    if (last.mutated_during_run && !allowWorkspaceMutation) {
      console.error("kineti: STALE (workspace changed during test execution)");
      process.exit(4);
    }
    if (last.fingerprint !== fingerprint()) { console.error("kineti: STALE (code changed after the run)"); process.exit(4); }
    if (last.manifest.input_digest !== last.fingerprint) { console.error("kineti: INVALID (workspace changed while tests were running)"); process.exit(7); }
    if (last.manifest.parser_sha256 !== digestFile(path.join(import.meta.dir, "kineti-evidence.ts"))) { console.error("kineti: INVALID (receipt parser changed)"); process.exit(7); }
    if (JSON.stringify(last.manifest.command) !== JSON.stringify(last.cmd.trim().split(/\s+/))) { console.error("kineti: INVALID (command manifest mismatch)"); process.exit(7); }
    for (const item of last.manifest.dependencies) if (!fs.existsSync(item.path) || digestFile(item.path) !== item.sha256) { console.error(`kineti: STALE (dependency input changed: ${item.path})`); process.exit(4); }
    for (const item of last.verification.inventory) if (!fs.existsSync(item.path) || digestFile(item.path) !== item.sha256) { console.error(`kineti: STALE (required test changed or missing: ${item.path})`); process.exit(4); }
    for (const item of last.manifest.artifact_digests) if (!fs.existsSync(item.path) || digestFile(item.path) !== item.sha256) { console.error(`kineti: STALE (artifact changed or missing: ${item.path})`); process.exit(4); }
    if (JSON.stringify(last.manifest.toolchain) !== JSON.stringify(toolchainVersions())) { console.error("kineti: STALE (runner or toolchain changed)"); process.exit(4); }
    for (const [name, valueHash] of Object.entries(last.manifest.environment)) if (sha256(process.env[name] ?? "<unset>") !== valueHash) { console.error(`kineti: STALE (declared environment input changed: ${name})`); process.exit(4); }
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
