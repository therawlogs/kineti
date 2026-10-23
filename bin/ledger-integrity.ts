import fs from "node:fs";
import path from "node:path";
import { sha256, writeJson } from "./lib.ts";

/** Local corruption detection, not protection against an owner rewriting all files. */
export function withFileLock<T>(file: string, action: () => T): T {
  fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
  const deadline = Date.now() + 15_000;
  let fd: number;
  while (true) {
    try { fd = fs.openSync(file, "wx", 0o600); break; }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      if (Date.now() >= deadline) throw new Error("ledger lock timed out; inspect interrupted operation before owner recovery");
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
    }
  }
  try { fs.writeFileSync(fd!, JSON.stringify({ pid: process.pid })); return action(); }
  finally { fs.closeSync(fd!); fs.unlinkSync(file); }
}

export function readStrictLines<T>(file: string): T[] {
  let bytes: string;
  try { bytes = fs.readFileSync(file, "utf8"); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === "ENOENT") return []; throw error; }
  if (!bytes) return [];
  if (!bytes.endsWith("\n")) throw new Error("ledger has an incomplete final line");
  return bytes.slice(0, -1).split("\n").map((line, index) => {
    if (!line.trim()) throw new Error(`empty ledger record at line ${index + 1}`);
    try { return JSON.parse(line) as T; }
    catch { throw new Error(`malformed ledger record at line ${index + 1}`); }
  });
}

/** Verify the last fully committed byte identity before reading or appending. */
export function checkCheckpoint(file: string): void {
  const checkpoint = `${file}.checkpoint.json`, initialized = `${file}.initialized`;
  if (!fs.existsSync(checkpoint)) {
    if (fs.existsSync(initialized)) throw new Error("initialized ledger checkpoint is missing; owner recovery required");
    return; // Legacy history is still checked by the caller's original hash rules.
  }
  let state: { version: number; bytes: number; sha256: string };
  try { state = JSON.parse(fs.readFileSync(checkpoint, "utf8")); }
  catch { throw new Error("ledger checkpoint is damaged; owner recovery required"); }
  if (!state! || state.version !== 1 || !Number.isSafeInteger(state.bytes) || state.bytes < 0 || !/^[a-f0-9]{64}$/.test(state.sha256)) throw new Error("invalid ledger checkpoint");
  if (!fs.existsSync(file)) throw new Error("initialized ledger is missing; owner recovery required");
  const bytes = fs.readFileSync(file);
  if (state.bytes !== bytes.length || state.sha256 !== sha256(bytes)) throw new Error("TAMPER: ledger changed or was truncated since its committed checkpoint");
}

/** Call only after chain validation and mutation, while holding the writer lock. */
export function commitCheckpoint(file: string): void {
  const bytes = fs.readFileSync(file);
  // Marker first: a crash cannot silently discard an initialized checkpoint.
  const marker = `${file}.initialized`;
  if (!fs.existsSync(marker)) fs.writeFileSync(marker, "1\n", { flag: "wx", mode: 0o600 });
  writeJson(`${file}.checkpoint.json`, { version: 1, bytes: bytes.length, sha256: sha256(bytes) });
}
