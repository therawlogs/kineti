#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { fingerprint, hashFileChunked, verifyReceiptSignature, type EvidenceRecord } from "./kineti-evidence.ts";
import { loadVerifyCommand, readJson, readJsonl, sha256, splitLegacyCommand, writeJson } from "./lib.ts";

export class GateFailure extends Error { constructor(message: string, readonly exitCode = 1) { super(message); } }

export function applyProtectedWrite(input: { label: string; runId: string; destination: string; arguments: unknown; content: string }, root = process.cwd()): string {
  const rec = readJsonl<EvidenceRecord>(path.join(root, ".kineti", "evidence.jsonl")).find((r) => r.label === input.label && r.run_id === input.runId);
  if (!rec || rec.receipt_version !== 2 || rec.exit_code !== 0 || rec.verification?.status !== "valid") throw new GateFailure("proof is missing, legacy, unsupported, or invalid");
  if (!verifyReceiptSignature(rec)) throw new GateFailure("receipt issuer signature is invalid");
  const configured = loadVerifyCommand(root);
  const expectedCommand = configured ? splitLegacyCommand(configured) : [];
  if (!configured || JSON.stringify(rec.manifest?.command) !== JSON.stringify(expectedCommand)) throw new GateFailure("receipt command does not match the configured verification command");
  if (rec.fingerprint !== fingerprint(root) || rec.manifest?.input_digest !== rec.fingerprint) throw new GateFailure("workspace or test inputs changed after proof");
  if (rec.manifest.parser_sha256 !== hashFileChunked(path.join(import.meta.dir, "kineti-evidence.ts"))) throw new GateFailure("receipt parser changed after proof");
  for (const dep of rec.manifest.dependencies) if (!fs.existsSync(path.join(root, dep.path)) || hashFileChunked(path.join(root, dep.path)) !== dep.sha256) throw new GateFailure(`dependency changed: ${dep.path}`);
  for (const test of rec.verification.inventory) if (!fs.existsSync(path.join(root, test.path)) || hashFileChunked(path.join(root, test.path)) !== test.sha256) throw new GateFailure(`required test changed or missing: ${test.path}`);
  for (const artifact of rec.manifest.artifact_digests) if (!fs.existsSync(path.join(root, artifact.path)) || hashFileChunked(path.join(root, artifact.path)) !== artifact.sha256) throw new GateFailure(`artifact changed or missing: ${artifact.path}`);
  const effect = rec.manifest.protected_effect;
  const bytes = Buffer.from(input.content, "utf8");
  const rel = input.destination.replaceAll("\\", "/");
  if (!effect || effect.destination !== rel || JSON.stringify(effect.arguments) !== JSON.stringify(input.arguments) || effect.bytes_sha256 !== sha256(bytes) || effect.bytes_base64 !== bytes.toString("base64")) throw new GateFailure("destination, arguments, or bytes do not match the receipt");
  const target = path.resolve(root, rel);
  if (path.isAbsolute(rel) || rel.split("/").includes("..") || !target.startsWith(path.resolve(root) + path.sep)) throw new GateFailure("destination must be inside the workspace", 2);
  let parent = path.dirname(target);
  while (parent.startsWith(path.resolve(root))) { if (fs.lstatSync(parent).isSymbolicLink()) throw new GateFailure("symlink destination is not allowed"); if (parent === path.resolve(root)) break; parent = path.dirname(parent); }
  if (fs.existsSync(target) && fs.lstatSync(target).isSymbolicLink()) throw new GateFailure("symlink destination is not allowed");
  const lock = path.join(root, ".kineti", "protected-write.lock");
  const usedFile = path.join(root, ".kineti", "protected-write-runs.json");
  fs.mkdirSync(path.dirname(lock), { recursive: true });
  let lockFd: number;
  try { lockFd = fs.openSync(lock, "wx", 0o600); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new GateFailure("another protected write is already running"); throw error; }
  const temp = path.join(root, ".kineti", `protected-write.${process.pid}.${Math.random().toString(36).slice(2)}.tmp`);
  try {
    if (rec.fingerprint !== fingerprint(root)) throw new GateFailure("workspace changed during authorization");
    const used = readJson<string[]>(usedFile) ?? [];
    if (used.includes(input.runId)) throw new GateFailure("proof has already been used");
    used.push(input.runId);
    writeJson(usedFile, used);
    if (!fs.existsSync(path.dirname(target)) || !fs.statSync(path.dirname(target)).isDirectory()) throw new GateFailure("destination directory must already exist", 2);
    fs.writeFileSync(temp, bytes, { flag: "wx", mode: 0o600 });
    if (rec.fingerprint !== fingerprint(root)) throw new GateFailure("workspace changed before protected write");
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
    const input = JSON.parse(raw);
    if (!input || typeof input !== "object" || typeof input.label !== "string" || typeof input.runId !== "string" || typeof input.destination !== "string" || typeof input.content !== "string" || !Object.hasOwn(input, "arguments")) throw new GateFailure("input JSON requires label, runId, destination, arguments, and content", 2);
    console.log(applyProtectedWrite(input));
    process.exit(0);
  } catch (error) {
    const code = error instanceof GateFailure ? error.exitCode : 3;
    console.error(`kineti: ${code === 1 ? "blocked" : code === 2 ? "invalid input" : "write failure"}: ${(error as Error).message}`);
    process.exit(code);
  }
}
