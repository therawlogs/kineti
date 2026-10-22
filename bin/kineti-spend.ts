#!/usr/bin/env bun
// bin/kineti-spend.ts
// Context Integrity Layer (Context Integrity Protocol / CIP) Hardware Spend Circuit Breaker
// Enforces $50.00 hardware spending ceiling ($47.50 95% trip) and $/Outcome economics tracking.
// Reference: Paper 5 (Outcome Engineering)
// Author: Kineti Research Team (therawlogs.com | Foundational AI Research)

import fs from "node:fs";
import path from "node:path";
import { withFileLock } from "./ledger-integrity.ts";
import {
  appendJsonl, ensureDir, loadLimits, machineDir, nowIso,
  ok, projectKdir, readJson, readJsonl, usdToMicrocents, writeJson,
} from "./lib.ts";

interface Entry {
  at: string;
  stage: string;
  model: string;
  tokens_in: number;
  tokens_out: number;
  usd: number;
  microcents: number;
  pricing_mode?: "exact" | "override" | "custom_rate" | "estimate";
}
interface SpendState {
  tripped: boolean;
  reason: string | null;
  total_usd: number;
  total_microcents: number;
  by_stage: Record<string, number>;
  by_stage_microcents: Record<string, number>;
  entries: number;
}

const PRICES: Record<string, { in: number; out: number }> = {
  "opus": { in: 15, out: 75 },
  "sonnet": { in: 3, out: 15 },
  "haiku": { in: 0.8, out: 4 },
  "gpt-4": { in: 2.5, out: 10 },
  "gpt": { in: 2.5, out: 10 },
  "codex": { in: 2.5, out: 10 },
  "gemini": { in: 1.25, out: 10 },
  "claude": { in: 3, out: 15 },
  "default": { in: 3, out: 15 },
};
const DEFAULT_PRICE = { in: 3, out: 15 };

function priceFor(model: string): { in: number; out: number } | null {
  const m = model.toLowerCase().trim();
  for (const key of Object.keys(PRICES)) if (m.includes(key)) return PRICES[key];
  return null;
}

function file(root = process.cwd()): string { return path.join(root, ".kineti", "spend.json"); }
function logFile(root = process.cwd()): string { return path.join(root, ".kineti", "spend.log.jsonl"); }

class SpendError extends Error {
  constructor(message: string, readonly code: number) { super(message); }
}
function die(message: string, code = 1): never { throw new SpendError(message, code); }

// Hold one lock across validation, limits, accounting and writes. Never silently
// reclaim an abandoned lock: interrupted accounting requires owner inspection.
async function withSpendLock(fn: () => Promise<void>): Promise<void> {
  ensureDir(projectKdir());
  const lock = path.join(projectKdir(), "spend.lock");
  const deadline = Date.now() + 15_000;
  let fd: number;
  while (true) {
    try { fd = fs.openSync(lock, "wx", 0o600); break; }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
      if (Date.now() >= deadline) die("spend lock timed out; inspect interrupted accounting before recovery", 3);
      await new Promise(resolve => setTimeout(resolve, 10));
    }
  }
  try {
    fs.writeFileSync(fd!, JSON.stringify({ pid: process.pid, at: nowIso() }));
    await fn();
  } finally { fs.closeSync(fd!); fs.unlinkSync(lock); }
}

function load(root = process.cwd()): SpendState {
  const marker = path.join(root, ".kineti", "spend.initialized.json");
  const damaged = (): never => die("spend accounting is damaged or incomplete; owner inspection and explicit recovery required", 3);
  const exists = fs.existsSync(file(root));
  if (!exists) {
    if (fs.existsSync(logFile(root)) || fs.existsSync(marker)) damaged();
    return { tripped: false, reason: null, total_usd: 0, total_microcents: 0, by_stage: {}, by_stage_microcents: {}, entries: 0 };
  }
  let s: SpendState;
  try { s = JSON.parse(fs.readFileSync(file(root), "utf8")); } catch { return damaged(); }
  const money = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v) && v >= 0;
  const integer = (v: unknown): v is number => money(v) && Number.isSafeInteger(v);
  const amounts = (v: unknown): v is Record<string, number> => !!v && typeof v === "object" && !Array.isArray(v) && Object.values(v).every(money);
  if (!s! || typeof s !== "object" || typeof s.tripped !== "boolean" || !(s.reason === null || typeof s.reason === "string") || !money(s.total_usd) || !integer(s.entries) || !amounts(s.by_stage)) damaged();
  // Read legacy integer-less accounting without changing its stored identity.
  if (s.total_microcents === undefined) s.total_microcents = usdToMicrocents(s.total_usd);
  if (s.by_stage_microcents === undefined) s.by_stage_microcents = Object.fromEntries(Object.entries(s.by_stage).map(([k, v]) => [k, usdToMicrocents(v)]));
  if (!integer(s.total_microcents) || !amounts(s.by_stage_microcents) || !Object.values(s.by_stage_microcents).every(integer)) damaged();
  if (round(s.total_microcents / 1e6) !== round(s.total_usd)) damaged();
  let entries: Entry[] = [];
  if (fs.existsSync(logFile(root))) {
    const text = fs.readFileSync(logFile(root), "utf8");
    if (text && !text.endsWith("\n")) damaged();
    try { entries = text ? text.slice(0, -1).split("\n").map(line => JSON.parse(line)) : []; } catch { return damaged(); }
  }
  let total = 0;
  const stages: Record<string, number> = Object.create(null);
  for (const entry of entries) {
    if (!entry || typeof entry.stage !== "string" || !entry.stage || typeof entry.model !== "string" || !money(entry.tokens_in) || !money(entry.tokens_out) || !money(entry.usd)) damaged();
    const micro = entry.microcents === undefined ? usdToMicrocents(entry.usd) : entry.microcents;
    if (!integer(micro)) damaged();
    total += micro;
    stages[entry.stage] = (stages[entry.stage] ?? 0) + micro;
    if (!integer(total) || !integer(stages[entry.stage])) damaged();
  }
  if (entries.length !== s.entries || total !== s.total_microcents) damaged();
  for (const stage of new Set([...Object.keys(stages), ...Object.keys(s.by_stage_microcents), ...Object.keys(s.by_stage)])) {
    if ((stages[stage] ?? 0) !== (s.by_stage_microcents[stage] ?? 0) || round((stages[stage] ?? 0) / 1e6) !== round(s.by_stage[stage] ?? 0)) damaged();
  }
  return s;
}

export function inspectSpend(root = process.cwd()): SpendState {
  return withFileLock(path.join(root, ".kineti", "spend.lock"), () => load(root));
}


function stageLimit(limits: ReturnType<typeof loadLimits>, stage: string): number {
  return limits.perStage[stage] ?? limits.perStageDefaultUsd;
}

function printHelp(): void {
  console.log(`
kineti spend - CIP hardware spend circuit breaker & tracking

Commands:
  status                          Show spend totals vs budget and headroom
  check                           Verify spend is under limit (exits 3 if tripped/over)
  log [options]                   Log token usage and spend for an agent action
  economics                       Show spend outcome economics and $/outcome
  reset --i-am-human [options]    Reset tripped circuit breaker (human only)

Log Options:
  --stage <name>                  Stage name or number (required)
  --tokens-in <count>             Prompt/input token count (required)
  --tokens-out <count>            Completion/output token count (required)
  --model <name>                  Model identifier (e.g. sonnet, opus, gpt-4)
  --usd <amount>                  Direct USD cost override
  --rate-in <per_m>               Custom USD rate per million input tokens
  --rate-out <per_m>              Custom USD rate per million output tokens
  --estimate                      Allow fallback estimate rate for unknown models

Reset Options:
  --i-am-human                    Required confirmation flag
  --extend-usd <amount>           Increase project ceiling by specified amount
`);
}

async function main() {
  const [cmd, ...rest] = process.argv.slice(2);

  if (!cmd || cmd === "--help" || cmd === "-h" || rest.includes("--help") || rest.includes("-h")) {
    printHelp();
    return;
  }

  return withSpendLock(async () => {
  const limits = loadLimits();
  const s = load();

  if (cmd === "log") {
    if (s.tripped) die(`breaker is tripped: ${s.reason}. Only a human may reset it.`, 3);
    let stage = "", model = "default", tin = NaN, tout = NaN, usdOverride: number | null = null;
    let usdGiven = false;
    let rateIn: number | null = null, rateOut: number | null = null;
    let allowEstimate = false;
    for (let i = 0; i < rest.length; i++) {
      switch (rest[i]) {
        case "--stage": stage = rest[++i] ?? ""; break;
        case "--model": model = rest[++i] ?? "default"; break;
        case "--tokens-in": tin = Number(rest[++i]); break;
        case "--tokens-out": tout = Number(rest[++i]); break;
        case "--usd": usdOverride = Number(rest[++i]); usdGiven = true; break;
        case "--rate-in": rateIn = Number(rest[++i]); break;
        case "--rate-out": rateOut = Number(rest[++i]); break;
        case "--estimate": allowEstimate = true; break;
      }
    }
    if (!stage) die("log requires --stage S --tokens-in N --tokens-out N [--model M] [--usd X]", 2);
    if (!Number.isFinite(tin) || tin < 0) die(`log requires finite --tokens-in >= 0 (got ${rest.join(" ")})`, 2);
    if (!Number.isFinite(tout) || tout < 0) die(`log requires finite --tokens-out >= 0 (got ${rest.join(" ")})`, 2);
    if (usdGiven && (usdOverride === null || !Number.isFinite(usdOverride) || (usdOverride as number) < 0)) {
      die("log requires finite --usd >= 0", 2);
    }
    if (rateIn !== null && (!Number.isFinite(rateIn) || rateIn < 0)) die("log requires finite --rate-in >= 0", 2);
    if (rateOut !== null && (!Number.isFinite(rateOut) || rateOut < 0)) die("log requires finite --rate-out >= 0", 2);

    let usd: number;
    let pricingMode: "exact" | "override" | "custom_rate" | "estimate";

    if (usdGiven) {
      usd = usdOverride!;
      pricingMode = "override";
    } else if (rateIn !== null && rateOut !== null) {
      usd = (tin / 1e6) * rateIn + (tout / 1e6) * rateOut;
      pricingMode = "custom_rate";
    } else {
      const p = priceFor(model);
      if (p !== null) {
        usd = (tin / 1e6) * p.in + (tout / 1e6) * p.out;
        pricingMode = "estimate";
      } else if (allowEstimate) {
        usd = (tin / 1e6) * DEFAULT_PRICE.in + (tout / 1e6) * DEFAULT_PRICE.out;
        pricingMode = "estimate";
      } else {
        die(`unknown model '${model}': specify --usd override, --rate-in/--rate-out, or --estimate to proceed`, 2);
      }
    }

    s.by_stage = Object.assign(Object.create(null), s.by_stage);
    s.by_stage_microcents = Object.assign(Object.create(null), s.by_stage_microcents);
    const microcents = usdToMicrocents(usd);
    if (!Number.isSafeInteger(microcents) || microcents < 0 || !Number.isSafeInteger(s.total_microcents + microcents)) die("spend amount exceeds safe accounting range", 2);
    writeJson(path.join(projectKdir(), "spend.initialized.json"), { version: 1 });
    s.total_microcents = s.total_microcents + microcents;
    s.total_usd = round(s.total_microcents / 1e6);

    s.by_stage_microcents[stage] = (s.by_stage_microcents[stage] ?? usdToMicrocents(s.by_stage[stage] ?? 0)) + microcents;
    s.by_stage[stage] = round(s.by_stage_microcents[stage] / 1e6);
    s.entries += 1;
    writeJson(file(), s);
    appendJsonl(logFile(), {
      at: nowIso(),
      stage,
      model,
      tokens_in: tin,
      tokens_out: tout,
      usd: round(usd),
      microcents,
      pricing_mode: pricingMode,
    } satisfies Entry);

    const globalCeilingMicrocents = Math.round(limits.globalUsd * limits.safetyFactor * 1e6);
    const stageCeilingMicrocents = Math.round(stageLimit(limits, stage) * limits.safetyFactor * 1e6);
    if (s.total_microcents >= globalCeilingMicrocents) trip(s, `global total $${s.total_usd} reached ceiling $${round(limits.globalUsd * limits.safetyFactor)} of $${limits.globalUsd}`);
    else if (s.by_stage_microcents[stage] >= stageCeilingMicrocents) trip(s, `stage ${stage} total $${s.by_stage[stage]} reached ceiling $${round(stageLimit(limits, stage) * limits.safetyFactor)}`);
    else ok(`logged $${round(usd)} (stage ${stage}, ${pricingMode}); run total $${s.total_usd}`);
    return;
  }

  if (cmd === "check" || cmd === "status" || cmd === "economics") {
    if (s.tripped) {
      console.error(`kineti: TRIPPED: ${s.reason}`);
      throw new SpendError("spend limit reached", 3);
    }
    let over = false;
    for (const [stg, spentMicro] of Object.entries(s.by_stage_microcents)) {
      if (spentMicro >= Math.round(stageLimit(limits, stg) * limits.safetyFactor * 1e6)) over = true;
    }
    if (s.total_microcents >= Math.round(limits.globalUsd * limits.safetyFactor * 1e6)) over = true;
    if (over && (cmd === "check" || cmd === "economics")) { console.error("kineti: over limit"); throw new SpendError("spend limit reached", 3); }

    const globalCeilingMicrocents = Math.round(limits.globalUsd * limits.safetyFactor * 1e6);
    const headroomMicro = Math.max(0, globalCeilingMicrocents - s.total_microcents);
    const headroomUsd = round(headroomMicro / 1e6);

    if (cmd === "economics" || rest.includes("--economics")) {
      const evidenceFile = path.join(projectKdir(), "evidence.jsonl");
      let verifiedOutcomes = 0;
      if (fs.existsSync(evidenceFile)) {
        try {
          const { validateReceipt } = await import("./kineti-evidence.ts");
          const trustedPublicKey = fs.readFileSync(path.join(projectKdir(), "evidence-issuer.pem"), "utf8");
          const records = readJsonl<any>(evidenceFile);
          const latest = new Map(records.map(record => [record.label, record]));
          for (const record of latest.values()) {
            try { validateReceipt(record, { root: process.cwd(), trustedPublicKey }); verifiedOutcomes++; } catch { /* invalid or stale local receipts do not count */ }
          }
        } catch { /* missing evidence is not a verified outcome */ }
      }
      const costPerOutcome = verifiedOutcomes > 0 ? `$${(s.total_usd / verifiedOutcomes).toFixed(2)}` : "unknown";
      ok(`total $${s.total_usd} of $${limits.globalUsd}; entries ${s.entries}; tripped=${s.tripped}; headroom_to_trip=$${headroomUsd}; fresh_local_test_receipts=${verifiedOutcomes}; recorded_cost_per_receipt=${costPerOutcome}; provider_billing=unknown`);
      return;
    }

    ok(`total $${s.total_usd} of $${limits.globalUsd}; entries ${s.entries}; tripped=${s.tripped}; headroom_to_trip=$${headroomUsd}`);
    return;
  }

  if (cmd === "reset") {
    if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
    const actor = process.env.USER || process.env.LOGNAME || "human";
    let extendUsd = 0;
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--extend-usd") {
        extendUsd = Number(rest[++i]);
        if (!Number.isFinite(extendUsd) || extendUsd <= 0) die("reset requires finite --extend-usd > 0", 2);
      }
    }
    if (extendUsd > 0) {
      const mirrorPath = path.join(projectKdir(), "mirror.json");
      const mirror = readJson<{ ceiling?: number }>(mirrorPath) ?? {};
      const newCeiling = Math.min(1000, Math.round(((mirror.ceiling ?? limits.globalUsd) + extendUsd) * 100) / 100);
      writeJson(mirrorPath, { ...mirror, ceiling: newCeiling });
    }
    s.tripped = false; s.reason = null;
    writeJson(file(), s);
    const refreshedLimits = loadLimits();
    const refreshedCeilingMicrocents = Math.round(refreshedLimits.globalUsd * refreshedLimits.safetyFactor * 1e6);
    const remainingHeadroom = Math.max(0, round((refreshedCeilingMicrocents - s.total_microcents) / 1e6));
    try {
      const { appendAudit } = await import("./kineti-audit.ts");
      appendAudit(actor, "spend.reset", `breaker reset by human; total $${s.total_usd}${extendUsd > 0 ? ` (ceiling extended by $${extendUsd})` : ""}`);
    } catch { /* audit must never block reset */ }
    ok(`breaker reset by human (total $${s.total_usd}, headroom to trip: $${remainingHeadroom})`);
    return;
  }

  die(`unknown command: ${cmd}. Use log | check | status | economics | reset`, 2);
  });
}

function trip(s: SpendState, reason: string): never {
  s.tripped = true; s.reason = reason;
  writeJson(file(), s);
  try {
    ensureDir(machineDir());
    fs.appendFileSync(path.join(machineDir(), "alerts.log"), `${nowIso()} SPEND TRIPPED: ${reason}\n`);
  } catch {}
  die(`SPEND BREAKER TRIPPED: ${reason}`, 3);
}

function round(n: number): number { return Math.round(n * 1e4) / 1e4; }

if (import.meta.main) void main().catch(error => {
  console.error(`kineti: ${error instanceof Error ? error.message : String(error)}`);
  process.exitCode = error instanceof SpendError ? error.code : 3;
});
