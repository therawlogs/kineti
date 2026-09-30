#!/usr/bin/env bun
// scripts/ci-seed.ts
// Creates the runtime state a fresh CI runner is missing, so the verification
// gate in bin/kineti-ci.ts runs against a real stage + root goal instead of
// defaulting to stage 1 with no evidence requirement (a vacuous pass).

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
  version: 1;
  project: string;
  root_goal: string | null;
  root_goal_locked_at: string | null;
  stage: number | string;
  gates: Record<string, "pass" | "fail" | "pending">;
  history: { at: string; event: string }[];
}

interface Spend {
  total_microcents: number;
  tripped: boolean;
  limit_microcents: number;
}

function readJson<T>(p: string): T | null {
  if (!fs.existsSync(p)) return null;
  return JSON.parse(fs.readFileSync(p, "utf-8")) as T;
}

function writeJsonFile(p: string, value: unknown): void {
  fs.writeFileSync(p, JSON.stringify(value, null, 2) + "\n", { mode: 0o600 });
  try {
    fs.chmodSync(p, 0o600);
  } catch {
    /* best effort on platforms without POSIX modes */
  }
}

function main(): void {
  const argv = process.argv.slice(2);
  let stageOverride: number | null = null;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--stage") stageOverride = Number(argv[++i]);
  }
  if (stageOverride !== null && (!Number.isInteger(stageOverride) || stageOverride < 1 || stageOverride > 13)) {
    console.error("ci-seed: --stage must be an integer 1-13");
    process.exit(2);
  }

  const root = process.cwd();
  const config = readJson<Config>(path.join(root, "kineti.config.json"));
  if (!config) {
    console.error("ci-seed: kineti.config.json not found");
    process.exit(2);
  }

  const goal = config.root_goal ?? null;
  if (!goal) {
    console.error("ci-seed: kineti.config.json has no root_goal; lock one before running CI");
    process.exit(2);
  }

  const kdir = path.join(root, ".kineti");
  fs.mkdirSync(kdir, { recursive: true, mode: 0o700 });

  const now = new Date().toISOString();
  const pkg = readJson<{ name?: string }>(path.join(root, "package.json"));
  const project = pkg?.name ?? config.name ?? "kineti";
  const stage = stageOverride ?? config.settings?.ci_stage ?? 9;

  const statePath = path.join(kdir, "state.json");
  const state = readJson<State>(statePath);
  if (!state) {
    const seeded: State = {
      version: 1,
      project,
      root_goal: goal,
      root_goal_locked_at: now,
      stage,
      gates: {},
      history: [{ at: now, event: `ci-seed stage=${stage}` }],
    };
    writeJsonFile(statePath, seeded);
    console.log(`ci-seed: created .kineti/state.json (stage ${stage})`);
  } else if (!state.root_goal) {
    state.root_goal = goal;
    state.root_goal_locked_at = state.root_goal_locked_at ?? now;
    state.history.push({ at: now, event: "ci-seed backfilled root_goal" });
    writeJsonFile(statePath, state);
    console.log("ci-seed: backfilled root_goal in existing .kineti/state.json");
  } else {
    console.log("ci-seed: .kineti/state.json already present; left unchanged");
  }

  const spendPath = path.join(kdir, "spend.json");
  if (!readJson<Spend>(spendPath)) {
    const limitUsd = config.settings?.spend_limit_usd?.global ?? 50;
    const seededSpend: Spend = {
      total_microcents: 0,
      tripped: false,
      limit_microcents: Math.round(limitUsd * 1_000_000),
    };
    writeJsonFile(spendPath, seededSpend);
    console.log(`ci-seed: created .kineti/spend.json (limit $${limitUsd})`);
  }

  console.log(`ci-seed: ready (project=${project}, goal=${goal.slice(0, 60)})`);
}

if (import.meta.main) main();
