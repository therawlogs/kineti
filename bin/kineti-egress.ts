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

const LOCK_TIMEOUT_MS = 15_000;
const STALE_LOCK_MS = 30_000;
class InvalidLedgerError extends Error {}

function pause(ms: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function lockAlive(pid: number): boolean {
  try { process.kill(pid, 0); return true; }
  catch (error) { return (error as NodeJS.ErrnoException).code === "EPERM"; }
}

/** Serialize ledger read/append/state updates across independent CLI processes. */
function withLedgerLock<T>(fn: () => T): T {
  fs.mkdirSync(path.dirname(file()), { recursive: true, mode: 0o700 });
  const lock = `${file()}.lock`;
  const ownerFile = path.join(lock, "owner.json");
  const deadline = Date.now() + LOCK_TIMEOUT_MS;
  while (true) {
    try {
      fs.mkdirSync(lock, { mode: 0o700 });
      fs.writeFileSync(ownerFile, JSON.stringify({ pid: process.pid, at: Date.now() }), { mode: 0o600 });
      break;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      try {
        const stat = fs.statSync(lock);
        let owner: { pid?: number } | null = null;
        try { owner = JSON.parse(fs.readFileSync(ownerFile, "utf8")); } catch { /* owner creation may still be in progress */ }
        const pid = Number(owner?.pid);
        if (Date.now() - stat.mtimeMs > STALE_LOCK_MS && (!Number.isInteger(pid) || pid <= 0 || !lockAlive(pid))) {
          const stale = `${lock}.stale-${process.pid}-${Date.now()}`;
          try { fs.renameSync(lock, stale); fs.rmSync(stale, { recursive: true, force: true }); } catch { /* another waiter recovered it */ }
          continue;
        }
      } catch (inspectError) {
        if ((inspectError as NodeJS.ErrnoException).code === "ENOENT") continue;
        throw inspectError;
      }
      if (Date.now() >= deadline) die("egress ledger lock timed out; no receipt was written", 4);
      pause(10);
    }
  }
  try { return fn(); }
  catch (error) {
    if (error instanceof InvalidLedgerError) {
      console.error(`kineti: ${error.message}`);
      process.exitCode = 3;
      return undefined as T;
    }
    throw error;
  }
  finally { fs.rmSync(lock, { recursive: true, force: true }); }
}

function assertValidChain(chain: Receipt[]): void {
  let prev = "GENESIS";
  for (const r of chain) {
    if (r.prev_hash !== prev || (computeHash(r) !== r.hash && computeHashLegacy(r) !== r.hash)) {
      throw new InvalidLedgerError(`TAMPER at receipt ${r.seq}: broken parent link or content hash; refusing ledger operation`);
    }
    prev = r.hash;
  }
}

function assertState(chain: Receipt[], state: { count: number; last_hash: string } | null): void {
  if (state && (state.count !== chain.length || (chain.length > 0 && state.last_hash !== chain[chain.length - 1].hash))) {
    throw new InvalidLedgerError("TAMPER: ledger truncated or reordered against recorded state");
  }
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
    withLedgerLock(() => {
      const chain = readJsonl<Receipt>(file());
      assertValidChain(chain);
      assertState(chain, readJson<{ count: number; last_hash: string } | null>(stateFile()));
      const prev = chain.length > 0 ? chain[chain.length - 1].hash : "GENESIS";
      const base = { seq: chain.length, at: nowIso(), host, description: desc, prev_hash: prev };
      const receipt: Receipt = { ...base, hash: computeHash(base) };
      appendJsonl(file(), receipt);
      writeJson(stateFile(), { count: chain.length + 1, last_hash: receipt.hash });
      ok(`receipt ${receipt.seq} recorded before send to ${host}`);
    });
    return;
  }

  if (cmd === "verify") {
    withLedgerLock(() => {
      const chain = readJsonl<Receipt>(file());
      const state = readJson<{ count: number; last_hash: string } | null>(stateFile());
      assertValidChain(chain);
      assertState(chain, state);
      ok(`ledger intact: ${chain.length} receipts`);
    });
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
