#!/usr/bin/env bun
// Public CI initializer. Creates only missing runtime files and never replaces
// an existing locked goal or spend ledger.

import fs from "node:fs";
import path from "node:path";

interface Config {
  name?: string;
  root_goal?: string;
  settings?: {
    ci_stage?: number;
    spend_limit_usd?: { global?: number };
  };
}

interface State {
  version?: number;
  project?: string;
  root_goal?: string | { description?: string } | null;
  root_goal_locked_at?: string | null;
  stage?: number | string;
  gates?: Record<string, "pass" | "fail" | "pending">;
  history?: { at: string; event: string }[];
}

interface Spend {
  total_microcents?: number;
  total_usd?: number;
  tripped?: boolean;
  limit_microcents?: number;
}

export interface SeedResult {
  project: string;
  stage: number | string;
  stateCreated: boolean;
  spendCreated: boolean;
  spendLimitUsd: number;
}

function readJson<T>(file: string, label: string): T | null {
  if (!fs.existsSync(file)) return null;
  try {
    return JSON.parse(fs.readFileSync(file, "utf8")) as T;
  } catch (error) {
    throw new Error(`${label} is not valid JSON: ${(error as Error).message}`);
  }
}

function writeJsonAtomic(file: string, value: unknown): void {
  const temp = `${file}.${process.pid}.${Date.now()}.tmp`;
  try {
    fs.writeFileSync(temp, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600, flag: "wx" });
    fs.renameSync(temp, file);
    try {
      fs.chmodSync(file, 0o600);
    } catch {
      // Best effort on platforms without POSIX file modes.
    }
  } catch (error) {
    try { fs.rmSync(temp, { force: true }); } catch { /* preserve original error */ }
    throw error;
  }
}

function hasLockedGoal(state: State): boolean {
  if (typeof state.root_goal === "string") return state.root_goal.trim().length > 0;
  return Boolean(state.root_goal && typeof state.root_goal === "object" && state.root_goal.description?.trim());
}

function validateExistingSpend(spend: Spend): void {
  const total = spend.total_microcents ?? (typeof spend.total_usd === "number" ? spend.total_usd * 1_000_000 : NaN);
  if (!Number.isFinite(total) || total < 0) throw new Error("existing .kineti/spend.json has an invalid total");
  if (typeof spend.tripped !== "boolean") throw new Error("existing .kineti/spend.json has no valid tripped flag");
  if (spend.limit_microcents !== undefined && (!Number.isFinite(spend.limit_microcents) || spend.limit_microcents <= 0)) {
    throw new Error("existing .kineti/spend.json has an invalid limit");
  }
}

export function seedCIState(workspaceRoot: string = process.cwd(), stageOverride?: number): SeedResult {
  const root = path.resolve(workspaceRoot);
  if (stageOverride !== undefined && (!Number.isInteger(stageOverride) || stageOverride < 1 || stageOverride > 13)) {
    throw new Error("--stage must be an integer from 1 to 13");
  }

  const config = readJson<Config>(path.join(root, "kineti.config.json"), "kineti.config.json");
  if (!config) throw new Error("kineti.config.json not found");
  if (typeof config.root_goal !== "string" || !config.root_goal.trim()) {
    throw new Error("kineti.config.json must contain a non-empty root_goal");
  }

  const configuredStage = config.settings?.ci_stage ?? 9;
  if (!Number.isInteger(configuredStage) || configuredStage < 1 || configuredStage > 13) {
    throw new Error("settings.ci_stage must be an integer from 1 to 13");
  }
  const spendLimitUsd = config.settings?.spend_limit_usd?.global ?? 50;
  if (!Number.isFinite(spendLimitUsd) || spendLimitUsd <= 0 || spendLimitUsd > 1000) {
    throw new Error("settings.spend_limit_usd.global must be greater than 0 and no more than 1000");
  }

  const pkg = readJson<{ name?: string }>(path.join(root, "package.json"), "package.json");
  const project = pkg?.name || config.name || path.basename(root) || "kineti";
  const stage = stageOverride ?? configuredStage;
  const kdir = path.join(root, ".kineti");
  const statePath = path.join(kdir, "state.json");
  const spendPath = path.join(kdir, "spend.json");

  const state = readJson<State>(statePath, ".kineti/state.json");
  if (state && !hasLockedGoal(state)) {
    throw new Error(".kineti/state.json exists without a locked root_goal; set the goal explicitly with `kineti state set root_goal ...`");
  }
  const spend = readJson<Spend>(spendPath, ".kineti/spend.json");
  if (spend) validateExistingSpend(spend);
  if (state && stageOverride !== undefined && state.stage !== stageOverride) {
    throw new Error("existing .kineti/state.json was kept unchanged; update its stage explicitly before using --stage");
  }

  const stateCreated = !state;
  const spendCreated = !spend;
  const effectiveProject = typeof state?.project === "string" ? state.project : project;
  if (!stateCreated && !spendCreated) {
    return { project: effectiveProject, stage: state?.stage ?? stage, stateCreated, spendCreated, spendLimitUsd };
  }

  fs.mkdirSync(kdir, { recursive: true, mode: 0o700 });
  const now = new Date().toISOString();
  if (stateCreated) {
    const seeded: State = {
      version: 1,
      project,
      root_goal: config.root_goal.trim(),
      root_goal_locked_at: now,
      stage,
      gates: {},
      history: [{ at: now, event: `kineti seed stage=${stage}` }],
    };
    writeJsonAtomic(statePath, seeded);
  }
  if (spendCreated) {
    writeJsonAtomic(spendPath, {
      total_microcents: 0,
      tripped: false,
      limit_microcents: Math.round(spendLimitUsd * 1_000_000),
    });
  }

  try { fs.chmodSync(kdir, 0o700); } catch { /* Best effort on platforms without POSIX modes. */ }
  return { project: effectiveProject, stage: state?.stage ?? stage, stateCreated, spendCreated, spendLimitUsd };
}

export function main(argv: string[] = process.argv.slice(2)): void {
  if (argv.length === 1 && (argv[0] === "--help" || argv[0] === "-h")) {
    console.log("Usage: kineti seed [--stage <1-13>]\nCreate only missing .kineti/state.json and .kineti/spend.json from kineti.config.json.");
    return;
  }
  let stage: number | undefined;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--stage") {
      const value = argv[++i];
      if (value === undefined) {
        console.error("kineti seed: --stage needs a value");
        process.exitCode = 2;
        return;
      }
      stage = Number(value);
    } else {
      console.error(`kineti seed: unknown option ${argv[i]}`);
      process.exitCode = 2;
      return;
    }
  }

  try {
    const result = seedCIState(process.cwd(), stage);
    console.log(`kineti seed: ${result.stateCreated ? "created" : "kept"} .kineti/state.json; ${result.spendCreated ? "created" : "kept"} .kineti/spend.json (project=${result.project}, stage=${result.stage})`);
  } catch (error) {
    console.error(`kineti seed: ${(error as Error).message}`);
    process.exitCode = 2;
  }
}

if (import.meta.main) main();
