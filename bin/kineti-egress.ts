#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { appendJsonl, computeDelimitedHash, die, machineDir, nowIso, ok, readJson, readJsonl, sha256, writeJson } from "./lib.ts";

interface Receipt {
  seq: number; at: string; host: string; description: string;
  prev_hash: string; hash: string;
}

function file(): string { return path.join(machineDir(), "egress.jsonl"); }
function stateFile(): string { return path.join(machineDir(), "egress.state.json"); }

function computeHash(r: Omit<Receipt, "hash">): string {
  return computeDelimitedHash([String(r.seq), r.at, r.host, r.description, r.prev_hash]);
}

function computeHashLegacy(r: Omit<Receipt, "hash">): string {
  return sha256(`${r.seq}|${r.at}|${r.host}|${r.description}|${r.prev_hash}`);
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);

  if (cmd === "record") {
    let host = "", desc = "";
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--host") host = rest[++i] ?? "";
      else if (rest[i] === "--description" || rest[i] === "--desc") desc = rest[++i] ?? "";
    }
    if (!host || !desc) die("record requires --host H --description \"what is being sent and why\"");
    const chain = readJsonl<Receipt>(file());
    const prev = chain.length > 0 ? chain[chain.length - 1].hash : "GENESIS";
    const base = { seq: chain.length, at: nowIso(), host, description: desc, prev_hash: prev };
    const receipt: Receipt = { ...base, hash: computeHash(base) };
    appendJsonl(file(), receipt);
    writeJson(stateFile(), { count: chain.length + 1, last_hash: receipt.hash });
    ok(`receipt ${receipt.seq} recorded before send to ${host}`);
    return;
  }

  if (cmd === "verify") {
    const chain = readJsonl<Receipt>(file());
    const state = readJson<{ count: number; last_hash: string } | null>(stateFile());
    let prev = "GENESIS";
    for (const r of chain) {
      if (r.prev_hash !== prev) { console.error(`kineti: TAMPER at receipt ${r.seq}: broken parent link`); process.exit(3); }
      if (computeHash(r) !== r.hash && computeHashLegacy(r) !== r.hash) { console.error(`kineti: TAMPER at receipt ${r.seq}: content hash mismatch`); process.exit(3); }
      prev = r.hash;
    }
    if (state && (state.count !== chain.length || (chain.length > 0 && state.last_hash !== chain[chain.length - 1].hash))) {
      console.error("kineti: TAMPER: ledger truncated or reordered against recorded state");
      process.exit(3);
    }
    ok(`ledger intact: ${chain.length} receipts`);
    return;
  }

  if (cmd === "list") {
    for (const r of readJsonl<Receipt>(file())) {
      console.log(`${r.seq}\t${r.at}\t${r.host}\t${r.description}`);
    }
    return;
  }

  die(`unknown command: ${cmd}. Use record | verify | list`, 2);
}

main();
