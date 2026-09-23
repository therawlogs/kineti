#!/usr/bin/env bun
// bin/kineti-models.ts
// Phase 4: native model table by task type. Kineti native, keyword rules only.
// Ask-first default. Auto-switch only when the user turns it on.
// Every switch is audit logged with reason.

import path from "node:path";
import { projectKdir, readJson, writeJson, ok, die } from "./lib.ts";
import { appendAudit } from "./kineti-audit.ts";

/// Task types Kineti routes between.
export type TaskType = "code" | "plan" | "chat" | "fix";

export interface ModelPick {
  task: TaskType;
  host: string;
  model: string;
  reason: string;
}

/// Native table: cheapest capable host first. No outside calls.
const TABLE: Record<TaskType, Omit<ModelPick, "task">> = {
  code: { host: "cursor", model: "default-code", reason: "code edits stay in your editor with full file context" },
  plan: { host: "claude", model: "default-reasoning", reason: "long plans need careful step-by-step reasoning" },
  chat: { host: "opencode", model: "default-fast", reason: "quick questions deserve a fast cheap answer" },
  fix: { host: "codex", model: "default-debug", reason: "bugs need strong reproduction and test loops" },
};

function switchFile(): string {
  return path.join(projectKdir(), "kineti.json");
}

export function isAutoSwitch(): boolean {
  const s = readJson<{ auto_switch?: boolean }>(switchFile());
  return s?.auto_switch === true;
}

export function setAutoSwitch(on: boolean, actor = "user"): void {
  const cur = readJson<Record<string, unknown>>(switchFile()) || {};
  writeJson(switchFile(), { ...cur, auto_switch: on, updated_by: actor, at: new Date().toISOString() });
}

/** Keyword task classification. No model call. */
export function classifyTask(raw: string): TaskType {
  const t = raw.toLowerCase();
  if (/\b(fix|bug|broken|error|fail|stack trace|crash|debug)\b/.test(t)) return "fix";
  if (/\b(plan|design|architect|roadmap|spec|strategy)\b/.test(t)) return "plan";
  if (/\b(code|implement|build|write|refactor|function|class|api|endpoint)\b/.test(t)) return "code";
  return "chat";
}

export function recommend(raw: string): ModelPick {
  const task = classifyTask(raw);
  return { task, ...TABLE[task] };
}

function cli(): void {
  const argv = process.argv.slice(2);
  const cmd = argv[0];
  if (cmd === "recommend") {
    const text = argv.slice(1).join(" ");
    if (!text) die("recommend requires words, e.g. fix the login bug", 2);
    const p = recommend(text);
    ok(`${p.task}: ${p.host}/${p.model} - ${p.reason}. Auto-switch is ${isAutoSwitch() ? "on" : "off"} (ask-first default).`);
    return;
  }
  if (cmd === "auto") {
    const on = argv[1] === "on";
    if (argv[1] !== "on" && argv[1] !== "off") die("auto requires on or off", 2);
    setAutoSwitch(on);
    try {
      appendAudit("user", on ? "model.auto_on" : "model.auto_off", "model auto-switch toggled");
    } catch { /* audit must never block toggle */ }
    ok(`model auto-switch is now ${on ? "on" : "off"}.`);
    return;
  }
  die("unknown command: use recommend <words> | auto <on|off>", 2);
}

if (import.meta.main) cli();
