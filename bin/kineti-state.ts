#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { die, ok, nowIso, projectKdir, readJson, writeJson, scaffoldRootHooks } from "./lib.ts";

export interface RunState {
  version: 1;
  project: string;
  root_goal: string | null;
  root_goal_locked_at: string | null;
  stage: number | string;
  task?: {
    type?: string;
    name?: string;
    step?: string;
  };
  gates: Record<string, "pass" | "fail" | "pending">;
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
  if (typeof s.stage === "number") {
    if (!Number.isInteger(s.stage) || s.stage < 1 || s.stage > 13) {
      errs.push("stage must be integer 1-13");
    }
  } else if (typeof s.stage === "string") {
    if (!s.stage.trim()) {
      errs.push("stage string cannot be empty");
    }
  } else {
    errs.push("stage must be integer 1-13 or non-empty string");
  }
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
    let initialStage: number | string = 1;
    let taskType: string | undefined = undefined;
    let taskName: string | undefined = undefined;
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--project") project = rest[++i] ?? "";
      else if (rest[i] === "--goal") goal = rest[++i] ?? null;
      else if (rest[i] === "--stage") {
        const val = rest[++i] ?? "1";
        if (/^-?\d+$/.test(val)) {
          const n = Number(val);
          if (n < 1 || n > 13) die("stage must be integer 1-13", 2);
          initialStage = n;
        } else {
          initialStage = val;
        }
      } else if (rest[i] === "--task") {
        taskType = rest[++i] ?? undefined;
      } else if (rest[i] === "--task-name") {
        taskName = rest[++i] ?? undefined;
      }
    }
    if (!project) die("init requires --project NAME");
    if (taskType && initialStage === 1) {
      initialStage = taskType;
    }
    const s: RunState = {
      version: 1,
      project,
      root_goal: goal,
      root_goal_locked_at: goal ? nowIso() : null,
      stage: initialStage,
      ...(taskType ? { task: { type: taskType, ...(taskName ? { name: taskName } : {}) } } : {}),
      gates: {},
      history: [{ at: nowIso(), event: `init project=${project} stage=${initialStage}${taskType ? ` task=${taskType}` : ""}` }],
    };
    writeJson(f, s);
    const hooked = scaffoldRootHooks();
    const hookMsg = hooked.length ? ` (installed root platform hooks: ${hooked.join(", ")})` : "";
    ok((goal ? `state created; goal locked` : `state created; set the goal with: set root_goal "..."`) + hookMsg);
    return;
  }

  const s = load();
  if (!s) die(`no state found; run: kineti-state init --project NAME`, 2);

  if (cmd === "get") {
    const key = rest[0];
    if (!key) { console.log(JSON.stringify(s, null, 2)); return; }
    let v = (s as any)[key];
    if (v === undefined && key.startsWith("gate.")) {
      v = (s.gates as any)?.[key.slice(5)];
    }
    if (v === undefined && key.startsWith("task.")) {
      v = (s.task as any)?.[key.slice(5)];
    }
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
      const trimmed = value.trim();
      if (/^-?\d+$/.test(trimmed)) {
        const n = Number(trimmed);
        if (n < 1 || n > 13) die("stage must be integer 1-13", 2);
        s.history.push({ at: nowIso(), event: `stage ${s.stage} -> ${n}` });
        s.stage = n;
      } else {
        const lower = trimmed.toLowerCase();
        const stdIdx = STAGE_IDS.indexOf(lower);
        if (stdIdx !== -1) {
          const n = stdIdx + 1;
          s.history.push({ at: nowIso(), event: `stage ${s.stage} -> ${n} (${lower})` });
          s.stage = n;
        } else {
          s.history.push({ at: nowIso(), event: `stage ${s.stage} -> ${trimmed}` });
          s.stage = trimmed;
        }
      }
    } else if (key === "task" || key.startsWith("task.")) {
      if (!s.task) s.task = {};
      if (key === "task") {
        s.task.type = value;
        s.history.push({ at: nowIso(), event: `task.type=${value}` });
      } else {
        const sub = key.slice(5);
        (s.task as any)[sub] = value;
        s.history.push({ at: nowIso(), event: `task.${sub}=${value}` });
      }
    } else if (key.startsWith("gate.")) {
      const g = key.slice(5);
      if (value !== "pass" && value !== "fail" && value !== "pending") die("gate value must be pass|fail|pending", 2);
      s.gates[g] = value;
      s.history.push({ at: nowIso(), event: `gate ${g}=${value}` });
    } else {
      die(`refused: only root_goal (once), stage, task, task.*, gate.* are settable`, 2);
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
