#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { die, nowIso, ok, readJsonl, sha256 } from "./lib.ts";

interface Link {
  word: string;
  from_id: string; from_at: string;
  to_id: string; to_at: string;
  status: "candidate" | "hypothesis" | "validated" | "rejected";
  proof_id?: string | null;
}

interface Rec {
  at: string;
  type: "run-record" | "learning" | "dossier";
  state: "active" | "warm" | "cold" | "archive";
  project: string;
  id: string;
  data: any;
  links?: Link[];
  expires?: string | null;
  prev_hash?: string;
  hash?: string;
}

const CORE_WORDS = new Set([
  "caused", "triggers", "blocks", "enables", "requires", "supports",
  "indicates", "contributes_to", "remediates", "contradicts",
  "supersedes", "resolves", "duplicates",
]);
const TIME_ORDERED = new Set(["caused", "triggers", "blocks"]);
const WARM_DAYS = 90, COLD_DAYS = 275;

function journalFile(dir: string): string {
  return path.join(dir, ".kineti", "journal.jsonl");
}

function load(dir: string): Rec[] {
  return readJsonl<Rec>(journalFile(dir));
}

function save(dir: string, recs: Rec[]): void {
  const body = recs.map((r) => JSON.stringify(r)).join("\n") + (recs.length ? "\n" : "");
  fs.writeFileSync(journalFile(dir), body);
}

function recordHash(r: Rec): string {
  return sha256(`${r.prev_hash}|${r.at}|${r.id}|${JSON.stringify(r.data)}`);
}

function ageDays(iso: string): number {
  return (Date.now() - new Date(iso).getTime()) / 86400000;
}

function main() {
  const [cmd] = process.argv.slice(2);
  const di = process.argv.indexOf("--dir");
  const dir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();

  if (cmd === "sweep") {
    const recs = load(dir);
    let moved = 0;
    for (const r of recs) {
      if (r.state === "archive") continue;
      const age = ageDays(r.at);
      let target: Rec["state"] = r.state;
      if (r.state === "active") {
        const expired = r.expires ? new Date(r.expires).getTime() < Date.now() : false;
        if (expired || (r.type === "learning" && age > 90)) target = "warm";
      }
      if (r.state === "warm" && age > WARM_DAYS + 90) target = "cold";
      if (r.state === "cold" && age > WARM_DAYS + COLD_DAYS) target = "archive";
      if (target !== r.state) { r.state = target; moved++; }
    }
    save(dir, recs);
    ok(`sweep complete: ${moved} record(s) moved`);
    return;
  }

  if (cmd === "verify-chain") {
    const chain = load(dir)
      .filter((r) => r.type === "run-record")
      .sort((a, b) => a.at.localeCompare(b.at));
    let prev = "GENESIS";
    for (const r of chain) {
      if (r.prev_hash !== prev) {
        console.error(`kineti: CHAIN BROKEN at ${r.id}: expected prev ${prev.slice(0, 8)}, found ${(r.prev_hash ?? "none").slice(0, 8)}`);
        process.exit(3);
      }
      if (recordHash(r) !== r.hash) {
        console.error(`kineti: TAMPER at ${r.id}: content hash mismatch`);
        process.exit(3);
      }
      prev = r.hash!;
    }
    ok(`chain intact: ${chain.length} run-record(s)`);
    return;
  }

  if (cmd === "time-order") {
    const byId = new Map(load(dir).map((r) => [r.id, r]));
    const bad: string[] = [];
    for (const r of load(dir)) {
      for (const l of r.links ?? []) {
        if (!TIME_ORDERED.has(l.word)) continue;
        const cause = byId.get(l.from_id), effect = byId.get(l.to_id);
        if (!cause || !effect) continue;
        if (new Date(effect.at) < new Date(cause.at)) {
          bad.push(`${l.word}: effect ${l.to_id} (${effect.at}) precedes cause ${l.from_id} (${cause.at})`);
        }
      }
    }
    if (bad.length) {
      console.error(`kineti: ${bad.length} time-order violation(s):`);
      for (const b of bad) console.error(`  - ${b}`);
      process.exit(1);
    }
    ok("time order holds for all caused/triggers/blocks links");
    return;
  }

  if (cmd === "promote") {
    const counts = new Map<string, number>();
    for (const r of load(dir)) {
      for (const l of r.links ?? []) {
        if (!CORE_WORDS.has(l.word)) counts.set(l.word, (counts.get(l.word) ?? 0) + 1);
      }
    }
    const candidates = [...counts.entries()].filter(([, n]) => n >= 3);
    if (candidates.length === 0) { ok("no promotion candidates (non-core words used fewer than 3 times)"); return; }
    for (const [w, n] of candidates.sort((a, b) => b[1] - a[1])) {
      console.log(`PROMOTE? "${w}" used ${n}x — add to core vocabulary in MEMORY.md and kineti.config.json`);
    }
    return;
  }

  die(`unknown command: ${cmd}. Use sweep | verify-chain | time-order | promote`, 2);
}

main();
