#!/usr/bin/env bun
// bin/kineti-audit.ts
// Kineti write-only audit log with hash chain.
// File lives outside the repo so repo wipes do not erase history.
// No delete or edit command exists by design. View-only for users.

import fs from "node:fs";
import path from "node:path";
import {
  computeDelimitedHash, die, ensureDir, machineDir, nowIso, ok, readJsonl,
} from "./lib.ts";

interface AuditEntry {
  seq: number;
  at: string;
  actor: string;
  action: string;
  detail: string;
  prev_hash: string;
  hash: string;
}

function auditFile(): string {
  return path.join(machineDir(), "audit.log.jsonl");
}

function loadChain(): AuditEntry[] {
  return readJsonl<AuditEntry>(auditFile());
}

/** Append one entry to the hash-chained audit log. Exported for spend/companion use. */
export function appendAudit(actor: string, action: string, detail: string): AuditEntry {
  const chain = loadChain();
  const prev = chain.length > 0 ? chain[chain.length - 1].hash : "GENESIS";
  const seq = chain.length;
  const at = nowIso();
  const cleanActor = (actor || "unknown").slice(0, 128);
  const cleanAction = (action || "unknown").slice(0, 128);
  const cleanDetail = (detail || "").slice(0, 2000);
  const hash = computeDelimitedHash([
    String(seq), at, cleanActor, cleanAction, cleanDetail, prev,
  ]);
  const entry: AuditEntry = {
    seq, at, actor: cleanActor, action: cleanAction,
    detail: cleanDetail, prev_hash: prev, hash,
  };
  ensureDir(machineDir());
  fs.appendFileSync(auditFile(), JSON.stringify(entry) + "\n");
  try {
    fs.chmodSync(auditFile(), 0o600);
  } catch { /* ignore chmod failures on some systems */ }
  return entry;
}

/** Verify the full chain. Returns true when intact. */
export function verifyAudit(): { ok: boolean; count: number; badSeq: number | null } {
  const chain = loadChain();
  let prev = "GENESIS";
  for (const e of chain) {
    const expect = computeDelimitedHash([
      String(e.seq), e.at, e.actor, e.action, e.detail, e.prev_hash,
    ]);
    if (e.prev_hash !== prev || e.hash !== expect) {
      return { ok: false, count: chain.length, badSeq: e.seq };
    }
    prev = e.hash;
  }
  return { ok: true, count: chain.length, badSeq: null };
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  if (cmd === "log") {
    let actor = process.env.USER || process.env.LOGNAME || "human";
    let action = "";
    let detail = "";
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--actor") actor = rest[++i] ?? actor;
      else if (rest[i] === "--action") action = rest[++i] ?? "";
      else if (rest[i] === "--detail") detail = rest[++i] ?? "";
    }
    if (!action) die("log requires --action A [--actor W] [--detail D]", 2);
    const e = appendAudit(actor, action, detail);
    ok(`audit ${e.seq} ${e.action} ${e.hash.slice(0, 12)}`);
    return;
  }
  if (cmd === "verify") {
    const r = verifyAudit();
    if (!r.ok) {
      console.error(`kineti: AUDIT BROKEN at seq ${r.badSeq} (${r.count} entries)`);
      process.exit(3);
    }
    ok(`audit intact: ${r.count} entries`);
    return;
  }
  if (cmd === "show") {
    let n = 20;
    const ni = rest.indexOf("--n");
    if (ni !== -1) n = Number(rest[ni + 1]) || 20;
    const chain = loadChain();
    for (const e of chain.slice(-n)) {
      console.log(`${e.seq} ${e.at} ${e.actor} ${e.action} ${e.detail} [${e.hash.slice(0, 12)}]`);
    }
    return;
  }
  die("unknown command: use log | verify | show", 2);
}

if (import.meta.main) main();
