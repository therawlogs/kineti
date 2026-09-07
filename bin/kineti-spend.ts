#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import {
  appendJsonl, die, ensureDir, loadLimits, machineDir, nowIso,
  ok, projectKdir, readJson, usdToMicrocents, writeJson,
} from "./lib.ts";

interface Entry {
  at: string;
  stage: string;
  model: string;
  tokens_in: number;
  tokens_out: number;
  usd: number;
  microcents: number;
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
  "gpt": { in: 2.5, out: 10 },
  "codex": { in: 2.5, out: 10 },
  "gemini": { in: 1.25, out: 10 },
};
const DEFAULT_PRICE = { in: 3, out: 15 };

function file(): string { return path.join(projectKdir(), "spend.json"); }
function logFile(): string { return path.join(projectKdir(), "spend.log.jsonl"); }

function load(): SpendState {
  const loaded = readJson<SpendState>(file());
  if (loaded) {
    if (loaded.total_microcents === undefined) loaded.total_microcents = usdToMicrocents(loaded.total_usd);
    if (!loaded.by_stage_microcents) {
      loaded.by_stage_microcents = {};
      for (const [stg, amt] of Object.entries(loaded.by_stage || {})) {
        loaded.by_stage_microcents[stg] = usdToMicrocents(amt);
      }
    }
    return loaded;
  }
  return {
    tripped: false,
    reason: null,
    total_usd: 0,
    total_microcents: 0,
    by_stage: {},
    by_stage_microcents: {},
    entries: 0,
  };
}

function priceFor(model: string): { in: number; out: number } {
  const m = model.toLowerCase();
  for (const key of Object.keys(PRICES)) if (m.includes(key)) return PRICES[key];
  return DEFAULT_PRICE;
}

function stageLimit(limits: ReturnType<typeof loadLimits>, stage: string): number {
  return limits.perStage[stage] ?? limits.perStageDefaultUsd;
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  const limits = loadLimits();
  const s = load();

  if (cmd === "log") {
    if (s.tripped) die(`breaker is tripped: ${s.reason}. Only a human may reset it.`, 3);
    let stage = "", model = "default", tin = -1, tout = -1, usdOverride: number | null = null;
    for (let i = 0; i < rest.length; i++) {
      switch (rest[i]) {
        case "--stage": stage = rest[++i] ?? ""; break;
        case "--model": model = rest[++i] ?? "default"; break;
        case "--tokens-in": tin = Number(rest[++i]); break;
        case "--tokens-out": tout = Number(rest[++i]); break;
        case "--usd": usdOverride = Number(rest[++i]); break;
      }
    }
    if (!stage || tin < 0 || tout < 0) die("log requires --stage S --tokens-in N --tokens-out N [--model M] [--usd X]");
    const p = priceFor(model);
    const usd = usdOverride ?? (tin / 1e6) * p.in + (tout / 1e6) * p.out;

    const microcents = usdToMicrocents(usd);
    s.total_microcents = (s.total_microcents ?? usdToMicrocents(s.total_usd)) + microcents;
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
    } satisfies Entry);

    const globalCeilingMicrocents = Math.round(limits.globalUsd * limits.safetyFactor * 1e6);
    const stageCeilingMicrocents = Math.round(stageLimit(limits, stage) * limits.safetyFactor * 1e6);
    if (s.total_microcents >= globalCeilingMicrocents) trip(s, `global total $${s.total_usd} reached ceiling $${round(limits.globalUsd * limits.safetyFactor)} of $${limits.globalUsd}`);
    else if (s.by_stage_microcents[stage] >= stageCeilingMicrocents) trip(s, `stage ${stage} total $${s.by_stage[stage]} reached ceiling $${round(stageLimit(limits, stage) * limits.safetyFactor)}`);
    else ok(`logged $${round(usd)} (stage ${stage}); run total $${s.total_usd}`);
    return;
  }

  if (cmd === "check" || cmd === "status") {
    if (s.tripped) {
      console.error(`kineti: TRIPPED: ${s.reason}`);
      process.exit(3);
    }
    let over = false;
    for (const [stg, spentMicro] of Object.entries(s.by_stage_microcents)) {
      if (spentMicro >= Math.round(stageLimit(limits, stg) * 1e6)) over = true;
    }
    if (s.total_microcents >= Math.round(limits.globalUsd * 1e6)) over = true;
    if (over && cmd === "check") { console.error("kineti: over limit"); process.exit(3); }
    ok(`total $${s.total_usd} of $${limits.globalUsd}; entries ${s.entries}; tripped=${s.tripped}`);
    return;
  }

  if (cmd === "reset") {
    if (!rest.includes("--i-am-human")) die("reset requires --i-am-human (breakers are human-only)", 2);
    s.tripped = false; s.reason = null;
    writeJson(file(), s);
    ok("breaker reset by human");
    return;
  }

  die(`unknown command: ${cmd}. Use log | check | status | reset`, 2);
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

main();
