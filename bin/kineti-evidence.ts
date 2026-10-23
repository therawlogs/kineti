#!/usr/bin/env bun
import crypto from "node:crypto";
import { checkedOptions } from "./strict-options.ts";
import fs from "node:fs";
import path from "node:path";
import { appendJsonl, computeDelimitedHash, die, loadVerifyCommand, nowIso, ok, projectKdir, readJsonl, runSafeCommand, sha256 } from "./lib.ts";

export interface EvidenceRecord {
  receipt_version?: 2 | 3;
  session_id?: string;
  parent_run_id?: string;
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
  routing?: { [name: string]: unknown };
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
function keyFile(root = process.cwd()): string { return path.join(root, ".kineti/evidence-issuer.pem"); }
function issuerKeypair(root = process.cwd()) {
  const pubFile = keyFile(root), privFile = `${pubFile}.private`;
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
  if (typeof record.signature !== "string" || typeof record.issuer_key !== "string" || (trustedPublicKey && record.issuer_key !== trustedPublicKey) || !/^[A-Za-z0-9+/]{86}==$/.test(record.signature)) return false;
  try { const key = crypto.createPublicKey(trustedPublicKey ?? record.issuer_key); return key.asymmetricKeyType === "ed25519" && crypto.verify(null, Buffer.from(unsignedRecord(record)), key, Buffer.from(record.signature, "base64")); } catch { return false; }
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

export function readEvidence(root = process.cwd()): EvidenceRecord[] {
  const source = path.join(root, ".kineti/evidence.jsonl");
  if (!fs.existsSync(source)) return [];
  return fs.readFileSync(source,"utf8").split("\n").filter(line=>line.trim()).map(line=>JSON.parse(line));
}
function records(): Record[] { try { return readEvidence(); } catch { die("INVALID: corrupt evidence ledger",7); } }
export function issueEvidence(record: EvidenceRecord, root = process.cwd()): EvidenceRecord {
  const keys = issuerKeypair(root); record.issuer_key = keys.publicKey;
  record.signature = crypto.sign(null,Buffer.from(unsignedRecord(record)),keys.privateKey).toString("base64");
  appendJsonl(path.join(root,".kineti/evidence.jsonl"),record); return record;
}

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
  --expect-command <JSON argv>   Require exact verification arguments
  --session-id <uuid>            Correlate new signed receipts (run only)
  --parent-run-id <uuid>         Reference a parent execution (run only)
  --expect-cmd <substring>        Ensure command matches expected substring
  --allow-workspace-mutation      Allow receipts where workspace mutated during test
  --run-id <id>                   Select a unique receipt run
`);
}

export const RECEIPT_PARSER_VERSION = "kineti-test-output-v1";
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


export function environmentDigest(name: string): string {
  return sha256(JSON.stringify({ set: Object.hasOwn(process.env, name), value: process.env[name] ?? null }));
}

export class ReceiptValidationError extends Error {
  constructor(message: string, readonly kind: "invalid" | "stale" | "legacy" = "invalid") { super(message); }
}
export type ValidatedReceipt = EvidenceRecord & { receipt_version: 2 | 3; run_id: string; manifest: ExactInputManifest; verification: NonNullable<EvidenceRecord["verification"]>; issuer_key: string; signature: string };
const isObject = (v: unknown): v is { [key: string]: unknown } => !!v && typeof v === "object" && !Array.isArray(v);
const isDigest = (v: unknown): v is string => typeof v === "string" && /^[0-9a-f]{64}$/.test(v);
const isWorkspacePath = (v: unknown): v is string => typeof v === "string" && !!v && !v.includes("\0") && !path.isAbsolute(v) && !v.split(/[\\/]/).some((part) => part === ".." || part === "");

/** Shared checks for signed local evidence. Captured output is not independent test observation. */
export function parseExpectedCommand(raw: string): string[] {
  let value: unknown;
  try { value = JSON.parse(raw); } catch { throw new Error("expected command must be a JSON argument array"); }
  if (!Array.isArray(value) || !value.length || !value.every(v => typeof v === "string" && !v.includes("\0")) || !value[0]) throw new Error("expected command must be a nonempty JSON argument array");
  return value;
}
export function configuredCommand(root: string, label: string, requireMapping = false): string[] | undefined {
  const file = path.join(root, "kineti.config.json");
  if (!fs.existsSync(file)) return undefined;
  const settings = JSON.parse(fs.readFileSync(file, "utf8")).settings ?? {};
  const value = settings.verify_commands?.[label] ?? (requireMapping ? undefined : settings.verify_command_argv);
  return value === undefined ? undefined : parseExpectedCommand(JSON.stringify(value));
}

export function validateReceipt(value: unknown, options: { root: string; trustedPublicKey: string; now?: number; maxAgeMinutes?: number; allowWorkspaceMutation?: boolean; expectedCommand?: string[] }): ValidatedReceipt {
  const fail = (message: string, kind: "invalid" | "stale" | "legacy" = "invalid"): never => { throw new ReceiptValidationError(message, kind); };
  if (!isObject(value)) fail("receipt is malformed");
  const r = value as EvidenceRecord;
  const time = typeof r.at === "string" ? Date.parse(r.at) : NaN;
  const now = options.now ?? Date.now();
  if (!Number.isFinite(time) || !Number.isFinite(now) || new Date(time).toISOString() !== r.at) fail("invalid receipt timestamp", "stale");
  if (time > now + 60_000) fail("receipt timestamp is in the future", "stale");
  if (now - time > (options.maxAgeMinutes ?? 240) * 60_000) fail("receipt expired (maximum age exceeded)", "stale");
  if (r.receipt_version !== 2 && r.receipt_version !== 3) fail("legacy or unsupported receipt cannot authorize the exact-input gate", "legacy");
  if (!verifyReceiptSignature(r, options.trustedPublicKey)) fail("receipt issuer is untrusted or receipt signature is invalid");
  if (typeof r.run_id !== "string" || !/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(r.run_id) || typeof r.label !== "string" || !r.label || typeof r.cmd !== "string" || !isDigest(r.fingerprint)) fail("receipt identity or fingerprint is malformed");
  const uuid = (v: unknown) => typeof v === "string" && /^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(v);
  if (r.receipt_version === 3 && (!uuid(r.session_id) || (r.parent_run_id !== undefined && !uuid(r.parent_run_id)))) fail("signed session identity is malformed");
  const m = r.manifest, v = r.verification;
  if (!isObject(m) || !isObject(v)) fail("receipt manifest or verification is missing");
  if (v!.status !== "valid" || v!.parser !== RECEIPT_PARSER_VERSION || !Number.isSafeInteger(v!.passed) || v!.passed <= 0 || v!.failed !== 0 || v!.skipped !== 0 || r.exit_code !== 0 || r.guard_failure) fail("receipt verification evidence is missing, unknown, or failing");
  if (r.mutated_during_run !== undefined && typeof r.mutated_during_run !== "boolean") fail("workspace mutation flag is malformed");
  if (r.mutated_during_run && !options.allowWorkspaceMutation) fail("workspace changed during test execution", "stale");
  if (!Array.isArray(m!.command) || !m!.command.length || !m!.command[0] || !m!.command.every((part) => typeof part === "string" && !part.includes("\0"))) fail("command argv is missing or malformed");
  if (!isObject(m!.toolchain) || !Object.keys(m!.toolchain).length || !Object.values(m!.toolchain).every((v) => typeof v === "string") || !isObject(m!.environment) || !isDigest(m!.input_digest) || !isDigest(m!.parser_sha256)) fail("proof manifest is malformed");
  if (options.expectedCommand && (!options.expectedCommand.length || JSON.stringify(m!.command) !== JSON.stringify(options.expectedCommand))) fail(`label ${r.label}: expected arguments ${JSON.stringify(options.expectedCommand)}, recorded arguments ${JSON.stringify(m!.command)}; receipt command does not match the configured verification command`);
  for (const items of [m!.dependencies, v!.inventory, m!.artifact_digests]) {
    if (!Array.isArray(items) || new Set(items.map((item) => item?.path)).size !== items.length || !items.every((item) => isObject(item) && isWorkspacePath(item.path) && isDigest(item.sha256))) fail("proof input inventory is malformed");
  }
  if (!v!.inventory.length) fail("required test inventory is empty");
  for (const [name, digest] of Object.entries(m!.environment)) {
    if (!/^[A-Z_][A-Z0-9_]*$/i.test(name) || SECRET_ENV_NAME.test(name) || !isDigest(digest)) fail("proof has an invalid or secret-like environment declaration");
    if (environmentDigest(name) !== digest) fail("declared environment input changed: " + name, "stale");
  }
  if (JSON.stringify(m!.toolchain) !== JSON.stringify(toolchainVersions())) fail("declared toolchain identity changed after proof", "stale");
  if (r.fingerprint !== fingerprint(options.root)) fail("workspace or test inputs changed after proof", "stale");
  if (m!.input_digest !== r.fingerprint) fail("workspace changed while tests were running");
  if (m!.parser_sha256 !== digestFile(path.join(import.meta.dir, "kineti-evidence.ts"))) fail("receipt parser changed after proof");
  for (const items of [m!.dependencies, v!.inventory, m!.artifact_digests]) {
    for (const item of items) {
      const full = path.resolve(options.root, item.path);
      try { if (!fs.statSync(full).isFile() || digestFile(full) !== item.sha256) fail("declared input changed or missing: " + item.path, "stale"); }
      catch (error) { if (error instanceof ReceiptValidationError) throw error; fail("declared input changed or missing: " + item.path, "stale"); }
    }
  }
  if (m!.protected_effect !== undefined) {
    const effect = m!.protected_effect;
    if (!isObject(effect) || !isWorkspacePath(effect.destination) || !Object.hasOwn(effect, "arguments") || !isDigest(effect.bytes_sha256) || typeof effect.bytes_base64 !== "string" || Buffer.from(effect.bytes_base64, "base64").toString("base64") !== effect.bytes_base64 || sha256(Buffer.from(effect.bytes_base64, "base64")) !== effect.bytes_sha256) fail("protected effect is malformed");
  }
  return r as ValidatedReceipt;
}

function main() {
  const raw = process.argv.slice(2);
  const argv = checkedOptions(raw, raw[0] === "run" ? ["--label", "--timeout", "--required-test", "--env-name", "--artifact", "--protected-write", "--session-id", "--parent-run-id"] : ["--label", "--run-id", "--max-age", "--expect-cmd", "--expect-command"], raw[0] === "run" ? ["--allow-shell", "--allow-zero-tests", "--forbid-skipped", "--allow-workspace-mutation"] : ["--allow-workspace-mutation"], ["--required-test", "--env-name", "--artifact"], 1, raw[0] === "run");
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
    let sessionId = crypto.randomUUID() as string, parentRunId: string | undefined;
    let allowShell = false;
    let allowZeroTests = false;
    let forbidSkipped = false;
    let allowWorkspaceMutation = false;
    let timeoutSec = 60;
    const requiredTests: string[] = [], envNames: string[] = [], artifacts: string[] = [];
    let protectedWrite: { destination: string; arguments: unknown; content: string } | undefined;
    if (dd === -1) die("run requires: run --label L [--allow-shell] [--timeout <seconds>] [--allow-zero-tests] [--forbid-skipped] [--allow-workspace-mutation] -- <command...>");
    for (let i = 1; i < dd; i++) {
      if (argv[i] === "--label") label = argv[++i] ?? "";
      else if (argv[i] === "--session-id") sessionId = argv[++i];
      else if (argv[i] === "--parent-run-id") parentRunId = argv[++i];
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
    if (!Number.isFinite(timeoutSec) || timeoutSec <= 0) die("--timeout must be a positive number of seconds", 2);
    const cmdArgv = argv.slice(dd + 1);
    const command = cmdArgv.join(" ");
    if (!label || cmdArgv.length === 0) die("run requires --label and a command after --", 2);
    for (const id of [sessionId, parentRunId]) if (id && !/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(id)) die("invalid session/parent identity", 2);
    const inventoryPaths = [...new Set(requiredTests.flatMap((rel) => {
      const full = path.resolve(process.cwd(), rel);
      if (!full.startsWith(path.resolve(process.cwd()) + path.sep) || !fs.existsSync(full)) throw new Error(`required test missing or outside workspace: ${rel}`);
      if (fs.statSync(full).isFile()) return [rel];
      if (!fs.statSync(full).isDirectory()) throw new Error(`required test is not a file or directory: ${rel}`);
      const found: string[] = [];
      const walk = (dir: string) => { for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a,b) => a.name.localeCompare(b.name))) { const child = path.join(dir, entry.name); if (entry.isDirectory()) walk(child); else if (entry.isFile() && /\.(test|spec)\.[cm]?[jt]sx?$/.test(entry.name)) found.push(path.relative(process.cwd(), child)); } };
      walk(full);
      if (!found.length) throw new Error(`required test inventory is empty: ${rel}`);
      return found;
    }))];
    const inventory = inventoryPaths.map((rel) => ({ path: rel.split(path.sep).join("/"), sha256: digestFile(path.resolve(process.cwd(), rel)) }));
    for (const name of envNames) if (!/^[A-Z_][A-Z0-9_]*$/i.test(name) || SECRET_ENV_NAME.test(name)) die(`environment input is invalid or secret-like and cannot be recorded: ${name}`, 2);
    const envEvidence = Object.fromEntries(envNames.map((name) => [name, environmentDigest(name)]));
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

    const parserBefore = digestFile(path.join(import.meta.dir, "kineti-evidence.ts"));
    const fpBefore = fingerprint();
    const toolchainBefore = toolchainVersions();
    const verifyCmd = loadVerifyCommand();
    const res = runSafeCommand(cmdArgv, { cwd: process.cwd(), workspaceRoot: process.cwd(), timeoutMs: timeoutSec * 1000, allowShell, verifyCmd, literalTestArgv: true });
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

    if (outputVerification && cmdArgv[0] === "bun" && cmdArgv[1] === "test") {
      const combinedOutput = (res.stdout || "") + "\n" + (res.stderr || "");
      const missingOutput = inventory.filter((item) => !combinedOutput.includes(item.path));
      if (missingOutput.length) {
        code = 1; guardFailure = "missing-tests";
        process.stderr.write(`kineti: guard failed: test runner output did not name required tests: ${missingOutput.map((item) => item.path).join(", ")}\n`);
      }
    }
    const fpAfter = fingerprint();
    if (JSON.stringify(toolchainBefore) !== JSON.stringify(toolchainVersions()) && code === 0) {
      code = 1; guardFailure = "toolchain-changed";
      process.stderr.write("kineti: guard failed: runner or toolchain changed during test execution\n");
    }
    const declaredChanged = [...inventory, ...dependencies, ...artifactDigests].some((item) => !fs.existsSync(item.path) || digestFile(item.path) !== item.sha256)
      || envNames.some((name) => envEvidence[name] !== environmentDigest(name))
      || parserBefore !== digestFile(path.join(import.meta.dir, "kineti-evidence.ts"));
    const codeMutated = fpBefore !== fpAfter || declaredChanged;
    if (codeMutated && !allowWorkspaceMutation && code === 0) {
      code = 1;
      guardFailure = "code-mutated";
      process.stderr.write("kineti: guard failed: workspace was modified during test execution (use --allow-workspace-mutation to allow)\n");
    }

    const runId = crypto.randomUUID();
    const parsed = outputVerification;
    const record: EvidenceRecord = {
      receipt_version: 3, run_id: runId, session_id: sessionId, ...(parentRunId ? { parent_run_id: parentRunId } : {}),
      at: nowIso(), label, cmd: command, exit_code: code,
      fingerprint: fpAfter,
      manifest: { command: cmdArgv, dependencies, toolchain: toolchainBefore, environment: envEvidence, input_digest: fpBefore, artifact_digests: artifactDigests, parser_sha256: parserBefore, ...(protectedEffect ? { protected_effect: protectedEffect } : {}) },
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
    let expectedCommand: string[] | undefined;
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
        case "--expect-command": expectedCommand = parseExpectedCommand(argv[++i]); break;
        case "--allow-workspace-mutation": allowWorkspaceMutation = true; break;
      }
    }
    if (!label) die("check requires --label L");
    const rs = records().filter((r) => r.label === label && (!runId || r.run_id === runId));
    if (rs.length === 0) { console.error("kineti: MISSING"); process.exit(5); }
    const last = rs[rs.length - 1];
    if (expectCmd && (typeof last.cmd !== "string" || !last.cmd.includes(expectCmd))) { console.error("kineti: STALE (different command)"); process.exit(4); }
    try {
      // Checking never creates a key or enrolls the receipt's issuer.
      const trustedPublicKey = fs.readFileSync(keyFile(), "utf8");
      validateReceipt(last, { root: process.cwd(), trustedPublicKey, maxAgeMinutes: maxAgeMin, allowWorkspaceMutation, expectedCommand: expectedCommand ?? configuredCommand(process.cwd(), label) });
    } catch (error) {
      const kind = error instanceof ReceiptValidationError ? error.kind : "invalid";
      console.error(`kineti: ${kind === "stale" ? "STALE" : kind === "legacy" ? "LIMITED" : "INVALID"} (${error instanceof ReceiptValidationError ? error.message : "local issuer key is missing or unreadable"})`);
      process.exit(kind === "stale" ? 4 : kind === "legacy" ? 6 : 7);
    }
    ok(`FRESH: ${label}`);
    return;
  }

  die(`unknown command: ${cmd0}. Use fingerprint | run | check`, 2);
}

if (import.meta.main) { try { main(); } catch (error) { console.error(`kineti: ${(error as Error).message}`); process.exitCode = 2; } }
