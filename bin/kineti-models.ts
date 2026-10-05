#!/usr/bin/env bun
// bin/kineti-models.ts
// Static model suggestion table by task type. Keyword rules only; suggestions
// do not switch the host, active model, or agent session.

import { ok, die } from "./lib.ts";
import { appendAudit } from "./kineti-audit.ts";

/// Task types used by the static suggestion table.
export type TaskType = "code" | "plan" | "chat" | "fix";

export interface ModelPick {
  task: TaskType;
  host: string;
  model: string;
  reason: string;
}

/// Static keyword table. Entries are not based on live model pricing or benchmark results.
const TABLE: Record<TaskType, Omit<ModelPick, "task">> = {
  code: { host: "cursor", model: "default-code", reason: "code edits stay in your editor with full file context" },
  plan: { host: "claude", model: "default-reasoning", reason: "long plans need careful step-by-step reasoning" },
  chat: { host: "opencode", model: "default-fast", reason: "quick questions deserve a fast cheap answer" },
  fix: { host: "codex", model: "default-debug", reason: "bugs need strong reproduction and test loops" },
};

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
    ok(`${p.task}: suggest ${p.host}/${p.model} - ${p.reason}. Kineti does not switch your active tool or model.`);
    return;
  }
  die("unknown command: use recommend <words>; automatic model switching is not available", 2);
}

if (import.meta.main) cli();
