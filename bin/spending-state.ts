/** Shared locked accounting snapshots. These records measure declared usage, not provider invoices. */
import fs from "node:fs";
import path from "node:path";
import { loadLimits } from "./lib.ts";

export interface SpendSnapshot {
  tripped: boolean; reason: string | null; total_microcents: number; total_usd: number;
  by_stage: Record<string, number>; by_stage_microcents: Record<string, number>; entries: number;
}
export function withAccountingLock<T>(root: string, fn: () => T): T {
  const dir = path.join(root, ".kineti");
  fs.mkdirSync(dir, { recursive: true });
  const lock = path.join(dir, "accounting.lock"), until = Date.now() + 10_000;
  while (true) {
    try { fs.mkdirSync(lock); fs.writeFileSync(path.join(lock, "owner"), String(process.pid)); break; }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      // Never delete an unowned lock: recovery must be explicit after an interrupted write.
      if (Date.now() >= until) throw new Error("accounting lock unavailable; inspect interrupted operation before recovery");
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
    }
  }
  try { return fn(); } finally { fs.rmSync(lock, { recursive: true }); }
}
export function readSpending(root: string, allowMissing = false): SpendSnapshot {
  const file = path.join(root, ".kineti/spend.json");
  if (!fs.existsSync(file) && allowMissing) return { tripped: false, reason: null, total_microcents: 0, total_usd: 0, by_stage: {}, by_stage_microcents: {}, entries: 0 };
  const s = JSON.parse(fs.readFileSync(file, "utf8"));
  const money = (v: unknown) => typeof v === "number" && Number.isSafeInteger(v) && v >= 0;
  if (!s || !money(s.total_microcents) || typeof s.tripped !== "boolean") throw new Error("invalid spending snapshot");
  const stages = s.by_stage_microcents ?? {};
  if (typeof stages !== "object" || stages === null || Array.isArray(stages) || !Object.values(stages).every(money)) throw new Error("invalid stage spending");
  if (Object.values(stages).reduce<number>((n, v) => n + (v as number), 0) > s.total_microcents) throw new Error("stage spending exceeds total");
  if (s.total_usd !== undefined && (!Number.isFinite(s.total_usd) || Math.abs(s.total_usd - s.total_microcents / 1e6) > 0.000051)) throw new Error("inconsistent spending total");
  if (s.entries !== undefined && !money(s.entries)) throw new Error("invalid spending entries");
  return { ...s, reason: s.reason ?? null, total_usd: s.total_microcents / 1e6, entries: s.entries ?? 0, by_stage_microcents: stages, by_stage: Object.fromEntries(Object.entries(stages).map(([k,v]) => [k, (v as number)/1e6])) };
}
export function spendingStop(s: SpendSnapshot, limits: ReturnType<typeof loadLimits>): string | null {
  if (s.tripped) return s.reason || "spending circuit breaker is tripped";
  if (s.total_microcents >= Math.round(limits.globalUsd * limits.safetyFactor * 1e6)) return "global spending stop threshold reached";
  for (const [stage, spent] of Object.entries(s.by_stage_microcents)) {
    if (spent >= Math.round((limits.perStage[stage] ?? limits.perStageDefaultUsd) * limits.safetyFactor * 1e6)) return `stage ${stage} spending stop threshold reached`;
  }
  return null;
}
