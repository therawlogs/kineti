#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { die, ok, nowIso, projectKdir, readJson, writeJson } from "./lib.ts";

interface RunState {
  version: 1;
  project: string;
  root_goal: string | null;
  root_goal_locked_at: string | null;
  stage: number;
  gates: Record<string, "pass" | "fail">;
  history: { at: string; event: string }[];
}

const STAGE_IDS = [
  "officehours", "diagnose", "design", "architecture", "feasibility",
  "spec", "build", "review", "qa", "security", "ship", "watch", "retro",
];

function file(): string {
  return path.join(projectKdir(), "state.json");
}

function load(): RunState | null {
  return readJson<RunState>(file());
}

function validate(s: RunState): string[] {
  const errs: string[] = [];
  if (s.version !== 1) errs.push("version must be 1");
  if (typeof s.project !== "string" || s.project.length === 0) errs.push("project required");
  if (s.root_goal !== null && typeof s.root_goal !== "string") errs.push("root_goal must be string or null");
  if (!Number.isInteger(s.stage) || s.stage < 1 || s.stage > 13) errs.push("stage must be integer 1-13");
  if (STAGE_IDS[s.stage - 1] === undefined) errs.push("unknown stage");
  if (!Array.isArray(s.history)) errs.push("history must be array");
  return errs;
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  const f = file();

  if (cmd === "init") {
    if (load()) die(`state already exists at ${f}`);
    let project = "";
    let goal: string | null = null;
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--project") project = rest[++i] ?? "";
      else if (rest[i] === "--goal") goal = rest[++i] ?? null;
    }
    if (!project) die("init requires --project NAME");
    const s: RunState = {
      version: 1,
      project,
      root_goal: goal,
      root_goal_locked_at: goal ? nowIso() : null,
      stage: 1,
      gates: {},
      history: [{ at: nowIso(), event: `init project=${project}` }],
    };
    writeJson(f, s);
    ok(goal ? `state created; goal locked` : `state created; set the goal with: set root_goal "..."`);
    return;
  }

  const s = load();
  if (!s) die(`no state found; run: kineti-state init --project NAME`, 2);

  if (cmd === "get") {
    const key = rest[0];
    if (!key) { console.log(JSON.stringify(s, null, 2)); return; }
    const v = (s as any)[key];
    if (v === undefined) die(`unknown key: ${key}`, 2);
    console.log(typeof v === "string" ? v : JSON.stringify(v, null, 2));
    return;
  }

  if (cmd === "set") {
    const key = rest[0];
    const value = rest.slice(1).join(" ");
    if (!key || value.length === 0) die('set requires: set KEY VALUE');
    if (key === "root_goal") {
      if (s.root_goal !== null) {
        die("refused: root_goal is locked once set. Start a new run instead.", 3);
      }
      s.root_goal = value;
      s.root_goal_locked_at = nowIso();
      s.history.push({ at: nowIso(), event: "goal locked" });
    } else if (key === "stage") {
      const n = Number(value);
      if (!Number.isInteger(n) || n < 1 || n > 13) die("stage must be integer 1-13", 2);
      s.history.push({ at: nowIso(), event: `stage ${s.stage} -> ${n}` });
      s.stage = n;
    } else if (key.startsWith("gate.")) {
      const g = key.slice(5);
      if (value !== "pass" && value !== "fail") die("gate value must be pass|fail", 2);
      s.gates[g] = value;
      s.history.push({ at: nowIso(), event: `gate ${g}=${value}` });
    } else {
      die(`refused: only root_goal (once), stage, gate.* are settable`, 2);
    }
    const errs = validate(s);
    if (errs.length) die(`invalid state after write: ${errs.join("; ")}`, 2);
    writeJson(f, s);
    ok(`${key} = ${value}`);
    return;
  }

  if (cmd === "validate") {
    const errs = validate(s);
    if (errs.length) die(`invalid: ${errs.join("; ")}`, 1);
    ok("state valid");
    return;
  }

  die(`unknown command: ${cmd}. Use init | get [key] | set | validate`, 2);
}

main();
