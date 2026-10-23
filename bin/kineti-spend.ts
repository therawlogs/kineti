#!/usr/bin/env bun
// bin/kineti-spend.ts
// Context Integrity Layer (Context Integrity Protocol / CIP) Hardware Spend Circuit Breaker
// Enforces $50.00 hardware spending ceiling ($47.50 95% trip) and $/Outcome economics tracking.
// Reference: Paper 5 (Outcome Engineering)
// Author: Kineti Research Team (therawlogs.com | Foundational AI Research)

import fs from "node:fs";
import { appendAudit } from "./kineti-audit.ts";
import { validateReceipt } from "./kineti-evidence.ts";
import { checkedOptions } from "./strict-options.ts";
import { readSpending, saveSpending, spendingStop, withAccountingLock } from "./spending-state.ts";
import path from "node:path";
import {
  appendJsonl, ensureDir, loadLimits, machineDir, nowIso,
  ok, projectKdir, readJson, readJsonl, usdToMicrocents, writeJson,
} from "./lib.ts";

class SpendExit extends Error { constructor(message: string, readonly code: number) { super(message); } }
function die(message: string, code = 1): never { throw new SpendExit(message, code); }

interface Entry {
  at: string;
  stage: string;
  model: string;
  tokens_in: number;
  tokens_out: number;
  usd: number;
  microcents: number;
  run_id?: string;
  session_id?: string;
  pricing_mode?: "rate_card_estimate" | "override" | "custom_rate" | "estimate";
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

function file(): string { return path.join(projectKdir(), "spend.json"); }
function logFile(): string { return path.join(projectKdir(), "spend.log.jsonl"); }

function load(): SpendState { return readSpending(process.cwd(), true); }
export function inspectSpend(root = process.cwd()): SpendState { return withAccountingLock(root, () => readSpending(root, true)); }


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

function main() {
  const [cmd, ...raw] = process.argv.slice(2);
  const specs: Record<string, string[]> = { log: ["--stage", "--model", "--tokens-in", "--tokens-out", "--usd", "--rate-in", "--rate-out", "--run-id", "--session-id"], reset: ["--extend-usd"] };
  const rest = checkedOptions(raw, specs[cmd] ?? [], cmd === "log" ? ["--estimate"] : cmd === "reset" ? ["--i-am-human"] : ["--economics"], [], 0);
  if (rest.filter((v,i) => !v.startsWith("-") && (i === 0 || !rest[i-1].startsWith("--"))).length) die("unexpected positional argument", 2);

  if (!cmd || cmd === "--help" || cmd === "-h" || rest.includes("--help") || rest.includes("-h")) {
    printHelp();
    return;
  }

  return withAccountingLock(process.cwd(), () => {
  const limits = loadLimits();
  const s = load();

  if (cmd === "log") {
    const stopped = spendingStop(s, limits);
    if (stopped) die(`breaker is tripped: ${stopped}. Only a human may reset it.`, 3);
    let stage = "", model = "default", tin = NaN, tout = NaN, usdOverride: number | null = null;
    let usdGiven = false;
    let rateIn: number | null = null, rateOut: number | null = null;
    let allowEstimate = false;
    let runId: string | undefined, sessionId: string | undefined;
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
        case "--run-id": runId = rest[++i]; break;
        case "--session-id": sessionId = rest[++i]; break;
      }
    }
    if (Object.hasOwn(Object.prototype, stage) || stage === "prototype" || stage.includes("\0") || stage.trim() !== stage) die("invalid stage name", 2);
    if (!stage) die("log requires --stage S --tokens-in N --tokens-out N [--model M] [--usd X]", 2);
    if (!Number.isSafeInteger(tin) || tin < 0) die(`log requires safe integer --tokens-in >= 0 (got ${rest.join(" ")})`, 2);
    if (!Number.isSafeInteger(tout) || tout < 0) die(`log requires safe integer --tokens-out >= 0 (got ${rest.join(" ")})`, 2);
    if (usdGiven && (usdOverride === null || !Number.isFinite(usdOverride) || (usdOverride as number) < 0)) {
      die("log requires finite --usd >= 0", 2);
    }
    if (rateIn !== null && (!Number.isFinite(rateIn) || rateIn < 0)) die("log requires finite --rate-in >= 0", 2);
    if (rateOut !== null && (!Number.isFinite(rateOut) || rateOut < 0)) die("log requires finite --rate-out >= 0", 2);

    let usd: number;
    let pricingMode: "rate_card_estimate" | "override" | "custom_rate" | "estimate";

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
        pricingMode = "rate_card_estimate";
      } else if (allowEstimate) {
        usd = (tin / 1e6) * DEFAULT_PRICE.in + (tout / 1e6) * DEFAULT_PRICE.out;
        pricingMode = "estimate";
      } else {
        die(`unknown model '${model}': specify --usd override, --rate-in/--rate-out, or --estimate to proceed`, 2);
      }
    }

    for (const id of [runId, sessionId]) if (id && !/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(id)) die("invalid execution/session identity", 2);
    if ((rateIn === null) !== (rateOut === null)) die("both rate options are required", 2);
    const microcents = usdToMicrocents(usd);
    if (!Number.isSafeInteger(microcents) || !Number.isSafeInteger(s.total_microcents + microcents)) die("cost exceeds safe accounting range", 2);
    s.total_microcents = (s.total_microcents ?? usdToMicrocents(s.total_usd)) + microcents;
    s.total_usd = round(s.total_microcents / 1e6);

    s.by_stage_microcents[stage] = (s.by_stage_microcents[stage] ?? usdToMicrocents(s.by_stage[stage] ?? 0)) + microcents;
    s.by_stage[stage] = round(s.by_stage_microcents[stage] / 1e6);
    s.entries += 1;
    saveSpending(process.cwd(), s);
    appendJsonl(logFile(), {
      at: nowIso(),
      stage,
      model,
      tokens_in: tin,
      tokens_out: tout,
      usd: round(usd),
      microcents,
      pricing_mode: pricingMode,
      ...(runId ? { run_id: runId } : {}), ...(sessionId ? { session_id: sessionId } : {}),
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
      throw new SpendExit("spending stopped", 3);
    }
    const stop = spendingStop(s, limits);
    if (stop && cmd !== "status") die(stop, 3);
    let over = false;
    for (const [stg, spentMicro] of Object.entries(s.by_stage_microcents)) {
      if (spentMicro >= Math.round(stageLimit(limits, stg) * 1e6)) over = true;
    }
    if (s.total_microcents >= Math.round(limits.globalUsd * 1e6)) over = true;
    if (over && (cmd === "check" || cmd === "economics")) { console.error("kineti: over limit"); throw new SpendExit("spending stopped", 3); }

    const globalCeilingMicrocents = Math.round(limits.globalUsd * limits.safetyFactor * 1e6);
    const headroomMicro = Math.max(0, globalCeilingMicrocents - s.total_microcents);
    const headroomUsd = round(headroomMicro / 1e6);

    if (cmd === "economics" || rest.includes("--economics")) {
      const evidenceFile = path.join(projectKdir(), "evidence.jsonl");
      let verifiedOutcomes = 0;
      if (fs.existsSync(evidenceFile)) {
        try {
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
    saveSpending(process.cwd(), s);
    const refreshedLimits = loadLimits();
    const refreshedCeilingMicrocents = Math.round(refreshedLimits.globalUsd * refreshedLimits.safetyFactor * 1e6);
    const remainingHeadroom = Math.max(0, round((refreshedCeilingMicrocents - s.total_microcents) / 1e6));
    try {
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
  saveSpending(process.cwd(), s);
  try {
    ensureDir(machineDir());
    fs.appendFileSync(path.join(machineDir(), "alerts.log"), `${nowIso()} SPEND TRIPPED: ${reason}\n`);
  } catch {}
  die(`SPEND BREAKER TRIPPED: ${reason}`, 3);
}

function round(n: number): number { return Math.round(n * 1e4) / 1e4; }

if (import.meta.main) try { main(); } catch (error) { console.error(`kineti: ${(error as Error).message}`); process.exitCode = error instanceof SpendExit ? error.code : 3; }
