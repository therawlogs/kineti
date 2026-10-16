#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fingerprint, ReceiptValidationError, validateReceipt } from "./kineti-evidence.ts";
import { loadVerifyCommand, sha256, splitLegacyCommand, writeJson } from "./lib.ts";

export class GateFailure extends Error { constructor(message: string, readonly exitCode = 1) { super(message); } }
type ProtectedWritePolicy = { version: 1; trusted_issuer_key: string; revoked_run_ids: string[]; revoked_issuer_keys: string[] };
const POLICY_FILE = ".kineti/protected-write-policy.json";
const uuid = (v: unknown): v is string => typeof v === "string" && /^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(v);
function issuer(v: unknown): v is string {
  try { return typeof v === "string" && v.length <= 1024 && crypto.createPublicKey(v).asymmetricKeyType === "ed25519"; } catch { return false; }
}
function boundedJson(file: string, limit = 1024 * 1024): unknown {
  if (fs.statSync(file).size > limit) throw new Error("state exceeds size limit");
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function loadPolicy(root: string): ProtectedWritePolicy {
  let value: unknown;
  try { value = boundedJson(path.join(root, POLICY_FILE)); }
  catch { throw new GateFailure("protected-write policy is missing, unreadable, or malformed: " + POLICY_FILE, 2); }
  const p = value as Partial<ProtectedWritePolicy> | null;
  if (!p || p.version !== 1 || !issuer(p.trusted_issuer_key) || !Array.isArray(p.revoked_run_ids) || p.revoked_run_ids.length > 10000 || !p.revoked_run_ids.every(uuid) || !Array.isArray(p.revoked_issuer_keys) || p.revoked_issuer_keys.length > 1000 || !p.revoked_issuer_keys.every(issuer)) throw new GateFailure("protected-write policy has invalid fields: " + POLICY_FILE, 2);
  return p as ProtectedWritePolicy;
}
function destinationPath(root: string, rel: string): string {
  const target = path.resolve(root, rel);
  if (!rel || rel.includes("\0") || path.isAbsolute(rel) || rel.split(/[\\/]/).includes("..") || !target.startsWith(path.resolve(root) + path.sep)) throw new GateFailure("destination must be inside the workspace", 2);
  return target;
}
// Repeating this check narrows a race; separate path checks and rename are not an OS boundary.
function destinationParents(root: string, target: string): string {
  const identities: string[] = [];
  let parent = path.dirname(target);
  while (true) {
    let stat: fs.Stats;
    try { stat = fs.lstatSync(parent); }
    catch { throw new GateFailure("destination directory must already exist", 2); }
    if (stat.isSymbolicLink()) throw new GateFailure("symlink destination is not allowed");
    if (!stat.isDirectory()) throw new GateFailure("destination directory must already exist", 2);
    identities.push(`${parent}:${stat.dev}:${stat.ino}`);
    if (parent === path.resolve(root)) break;
    parent = path.dirname(parent);
  }
  try { if (fs.lstatSync(target).isSymbolicLink()) throw new GateFailure("symlink destination is not allowed"); }
  catch (error) { if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error; }
  return identities.join("\n");
}

export function applyProtectedWrite(input: unknown, root = process.cwd(), now = Date.now()): string {
  const request = input as { label: string; runId: string; destination: string; arguments: unknown; content: string };
  if (!request || typeof request !== "object" || typeof request.label !== "string" || !request.label || !uuid(request.runId) || typeof request.destination !== "string" || typeof request.content !== "string" || !Object.hasOwn(request, "arguments")) throw new GateFailure("input JSON requires label, runId, destination, arguments, and content", 2);
  const policy = loadPolicy(root);
  let matches: unknown[];
  try {
    const lines = fs.readFileSync(path.join(root, ".kineti/evidence.jsonl"), "utf8").trim().split("\n").filter(Boolean);
    matches = lines.map((line) => JSON.parse(line)).filter((r) => r && r.label === request.label && r.run_id === request.runId);
  } catch { throw new GateFailure("receipt ledger is missing, unreadable, or malformed", 2); }
  if (matches.length !== 1) throw new GateFailure("proof is missing or its run identity is ambiguous");
  if (policy.revoked_run_ids.includes(request.runId)) throw new GateFailure("proof run is revoked");
  if (policy.revoked_issuer_keys.includes(policy.trusted_issuer_key)) throw new GateFailure("receipt issuer is untrusted, revoked, or signature is invalid");
  let config: { settings?: { verify_command_argv?: unknown } } = {};
  try { config = boundedJson(path.join(root, "kineti.config.json")) as typeof config; }
  catch (error) { if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw new GateFailure("verification configuration is unreadable or malformed", 2); }
  if (!config || typeof config !== "object" || (config.settings !== undefined && (!config.settings || typeof config.settings !== "object" || Array.isArray(config.settings)))) throw new GateFailure("verification configuration is malformed", 2);
  const structured = config.settings?.verify_command_argv;
  if (Object.hasOwn(config.settings ?? {}, "verify_command_argv") && (!Array.isArray(structured) || !structured.length || !structured[0] || !structured.every((part: unknown) => typeof part === "string" && !part.includes("\0")))) throw new GateFailure("configured verification argv is malformed", 2);
  const configured = loadVerifyCommand(root);
  const expected = Array.isArray(structured) ? structured : configured ? splitLegacyCommand(configured) : [];
  let rec;
  try { rec = validateReceipt(matches[0], { root, trustedPublicKey: policy.trusted_issuer_key, now, expectedCommand: expected }); }
  catch (error) { if (error instanceof ReceiptValidationError) throw new GateFailure(error.message); throw error; }

  const effect = rec.manifest.protected_effect;
  const bytes = Buffer.from(request.content, "utf8");
  const rel = request.destination.replaceAll("\\", "/");
  if (!effect || effect.destination !== rel || JSON.stringify(effect.arguments) !== JSON.stringify(request.arguments) || effect.bytes_sha256 !== sha256(bytes) || effect.bytes_base64 !== bytes.toString("base64")) throw new GateFailure("destination, arguments, or bytes do not match the receipt");
  const target = destinationPath(root, rel);
  const parentsBefore = destinationParents(root, target);
  const lock = path.join(root, ".kineti/protected-write.lock");
  const usedFile = path.join(root, ".kineti/protected-write-runs.json");
  let lockFd: number;
  try { lockFd = fs.openSync(lock, "wx", 0o600); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new GateFailure("another protected write is already running"); throw error; }
  const temp = path.join(root, ".kineti", `protected-write.${crypto.randomUUID()}.tmp`);
  try {
    // Policy/replay state live outside the workspace fingerprint and are checked under the lock.
    const currentPolicy = loadPolicy(root);
    if (JSON.stringify(currentPolicy) !== JSON.stringify(policy)) throw new GateFailure("protected-write policy changed during authorization");
    if (rec.fingerprint !== fingerprint(root)) throw new GateFailure("workspace changed during authorization");
    let used: string[];
    try {
      const value = boundedJson(usedFile, 16 * 1024 * 1024);
      if (!Array.isArray(value) || !value.every(uuid)) throw new Error("invalid replay state");
      used = value;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") used = [];
      else throw new GateFailure("protected-write replay state is unreadable or malformed", 2);
    }
    if (used.includes(request.runId)) throw new GateFailure("proof has already been used");
    used.push(request.runId);
    writeJson(usedFile, used);
    fs.writeFileSync(temp, bytes, { flag: "wx", mode: 0o600 });
    if (rec.fingerprint !== fingerprint(root)) throw new GateFailure("workspace changed before protected write");
    if (parentsBefore !== destinationParents(root, target)) throw new GateFailure("destination parent changed during authorization");
    fs.renameSync(temp, target);
    return `protected write applied: ${rel}`;
  } finally {
    try { fs.rmSync(temp, { force: true }); } catch {}
    fs.closeSync(lockFd);
    fs.rmSync(lock, { force: true });
  }
}

if (import.meta.main) {
  try {
    const raw = await new Response(Bun.stdin.stream()).text();
    let input: unknown;
    try { input = JSON.parse(raw); } catch { throw new GateFailure("input must be valid JSON", 2); }
    console.log(applyProtectedWrite(input));
    process.exit(0);
  } catch (error) {
    const code = error instanceof GateFailure ? error.exitCode : 3;
    console.error(`kineti: ${code === 1 ? "blocked" : code === 2 ? "invalid input" : "write failure"}: ${(error as Error).message}`);
    process.exit(code);
  }
}
