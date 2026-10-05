#!/usr/bin/env bun
// bin/kineti-router.ts
// Phase 2 auto experience: plain-talk intent router, Kineti native.
// No outside model. Keyword rules only. User never sees skill or tool names.
// All replies use plain words plus numbered choices.

import path from "node:path";
import { projectKdir, readJson, readJsonl, writeJson, loadLimits } from "./lib.ts";
import { recommend } from "./kineti-models.ts";
import { appendAudit } from "./kineti-audit.ts";
import { dropLink } from "./kineti-pairing.ts";

export type Intent =
  | "spend" | "undo" | "proof" | "status"
  | "approve_yes" | "approve_fix"
  | "kineti_on" | "kineti_off"
  | "swarm_budget" | "swarm_save" | "model" | "sync" | "dashboard" | "forget" | "help" | "task";

interface RouterReply {
  intent: Intent;
  reply: string;
}

function switchFile(): string {
  return path.join(projectKdir(), "kineti.json");
}

/** Kineti is on unless explicitly switched off. Missing file means on. */
export function isEnabled(): boolean {
  const s = readJson<{ enabled?: boolean }>(switchFile());
  if (!s || typeof s.enabled !== "boolean") return true;
  return s.enabled;
}

export function setEnabled(on: boolean, actor = "user"): void {
  writeJson(switchFile(), { enabled: on, updated_by: actor, at: new Date().toISOString() });
}

const YES_PATTERNS = [
  /^(yes|yeah|yep|y|sure|ok|okay|go ahead|do it|looks good|ship it|sounds good|approved|approve)\b/,
  /\b(go ahead|do it|looks good|ship it|sounds good)\b/,
];
const FIX_PATTERNS = [
  /^(no|not quite|change|fix|instead|actually)\b/,
  /\b(change|fix|instead|not quite|do not|don't)\b.*/,
];

export function classifyIntent(raw: string): Intent {
  const t = raw.toLowerCase().trim();
  if (!t) return "help";
  // Model suggestion phrases first: they also contain words used by other intents.
  if (/\bwhich model|best model|switch model|change model|faster model|stronger model|auto.switch|auto switch\b/.test(t)) return "model";
  if (/\brevoke cloud link|unmirror|drop cloud link|unlink device\b/.test(t)) return "dashboard";
  if (/\bkineti.dashboard|ui dashboard|cloud link|pair device|pairing code|app\.getkineti\b/.test(t)) return "dashboard";
  if (/\b(sync|sync my|export my|import my|other device|new phone|new laptop)\b/.test(t)) return "sync";
  // Swarm save phrases before spend: "share one budget" contains budget words.
  if (/\bshare one budget|shared? budgets?\b/.test(t)) return "swarm_save";
  if (/\bkineti\s+off\b|\bturn\s+off\b|\bswitch\s+off\b|\bpause\b|\bdisable\b/.test(t) && !/\bturn on\b/.test(t)) return "kineti_off";
  if (/\bkineti\s+on\b|\bturn\s+on\b|\bswitch\s+on\b|\benable\b|\bresume\b/.test(t)) return "kineti_on";
  if (/\b(separate budgets?|independent budgets?|per.agent budget|team budget|swarm)\b/.test(t)) {
    if (/\b(\d+\s*(usd|dollars?)|\bcoder\b.*\d+|\breviewer\b.*\d+|share one|shared? budget|split)\b/.test(t)) return "swarm_save";
    return "swarm_budget";
  }
  if (/\b(spend|spent|cost|budget|money|how much|ceiling|limit)\b/.test(t)) return "spend";
  if (/\b(undo|roll back|rollback|revert|restore)\b/.test(t)) return "undo";
  if (/\b(test|tests|proof|passing|pass|fresh|stale|evidence|verify|verification)\b/.test(t)) return "proof";
  if (/\b(forget|erase|purge|delete my|remove my|opt out|opt-out)\b/.test(t)) return "forget";
  if (/\b(where are we|status|current state|what.?s next|next step|progress|goal)\b/.test(t)) return "status";
  if (/\b(what can you do|help|how do i|how to)\b/.test(t)) return "help";
  if (YES_PATTERNS.some((p) => p.test(t)) && t.length < 60) return "approve_yes";
  if (FIX_PATTERNS.some((p) => p.test(t))) return "approve_fix";
  return "task";
}

function spendReply(): string {
  const spend = readJson<any>(path.join(projectKdir(), "spend.json")) || {};
  const total = typeof spend.total_usd === "number" ? spend.total_usd : 0;
  const tripped = spend.tripped === true;
  const ceiling = loadLimits().globalUsd;
  if (tripped) {
    return (
      `Spending is stopped. You have used $${total} of $${ceiling}. ` +
      `Only you can restart it.\nChoices:\n1. Keep it stopped.\n2. Restart it.`
    );
  }
  const left = Math.max(0, ceiling - total);
  return (
    `You have used $${total} of $${ceiling}. $${left.toFixed(2)} is left.\n` +
    `Choices:\n1. Keep going.\n2. Set a lower limit in plain words, for example "my limit is twenty dollars".`
  );
}

function undoReply(): string {
  const lines = readJsonl<any>(path.join(projectKdir(), "saga.jsonl"));
  const committed = new Set(lines.filter((l) => l.kind === "commit").map((l) => l.run_id));
  const pending = lines.filter((l) => l.kind === "register" && !committed.has(l.run_id));
  if (pending.length === 0) {
    return `There is nothing to undo right now.\nChoices:\n1. Keep working.\n2. Tell me what changed and I will check it.`;
  }
  const names = pending.slice(-3).map((l) => String(l.label || "change")).join(", ");
  return (
    `I found ${pending.length} registered undo step${pending.length === 1 ? "" : "s"}, newest first. ` +
    `Latest: ${names}. This reply helper does not execute the rollback.\n` +
    `Choices:\n1. Review the steps, then run kineti undo in a human terminal.\n2. Keep everything.\n3. Show the recorded steps.`
  );
}

function proofReply(): string {
  const records = readJsonl<any>(path.join(projectKdir(), "evidence.jsonl"));
  if (records.length === 0) {
    return `No local test receipt is saved yet.\nChoices:\n1. Run the checks with kineti test.\n2. Keep working, check later.`;
  }
  const last = records[records.length - 1];
  const ageMin = Math.round((Date.now() - new Date(last.at).getTime()) / 60000);
  const ageText = ageMin <= 1 ? "just now" : `${ageMin} minutes ago`;
  if (last.exit_code !== 0) {
    return (
      `The latest local check "${last.label}" failed ${ageText} ago.\n` +
      `Choices:\n1. Ask your coding agent to fix it.\n2. Show the failure.\n3. Skip for now.`
    );
  }
  if (ageMin > 240) {
    return (
      `Last passing receipt "${last.label}" ran ${ageText}, which is old. Re-run the check before shipping.\n` +
      `Choices:\n1. Re-run with kineti test.\n2. Keep working.`
    );
  }
  return (
    `The local receipt for "${last.label}" is passing and ${ageText}. It is not a signed certificate.\n` +
    `Choices:\n1. Keep going.\n2. Re-run with kineti test.`
  );
}

function statusReply(): string {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const goal = typeof state.root_goal === "string" && state.root_goal ? state.root_goal : "no goal locked yet";
  const stage = state.stage ?? "not started";
  return (
    `Goal: ${goal}\nStep: ${stage} of 13.\n` +
    `Choices:\n1. Continue in your coding agent.\n2. Set local state with kineti state set root_goal.\n3. Show recorded spending.`
  );
}

const STAGE_NAMES: Record<number, string> = {
  1: "idea", 2: "measure", 3: "look", 4: "parts map",
  5: "feasibility", 6: "plan", 7: "build", 8: "review",
  9: "test", 10: "safety", 11: "ship", 12: "watch", 13: "learn",
};

function approveYesReply(): string {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const n = Number(state.stage);
  const name = STAGE_NAMES[n] || "next step";
  return (
    `Got it, yes. The next workflow stage is ${name}. This reply helper does not run the work or checks.\n` +
    `Choices:\n1. Continue in your coding agent.\n2. Ask for the short plan first.`
  );
}

function approveFixReply(userText: string): string {
  const short = userText.slice(0, 200);
  return (
    `Got it. Your coding agent should use this correction: "${short}". This helper does not change files.\n` +
    `Choices:\n1. Ask your agent to show an updated plan.\n2. Keep the old plan.`
  );
}

function swarmBudgetReply(): string {
  return (
    `I can save proposed budget targets for a team, but Kineti does not enforce these against each agent's usage.\n` +
    `Choices:\n1. Save proposed targets.\n2. Use one shared target.\n3. Show recorded spending.`
  );
}

function swarmSaveReply(userText: string): string {
  const store = path.join(projectKdir(), "swarm.json");
  const budgets: Record<string, number> = {};
  const re = /([a-z_][a-z0-9_-]*)\s*(\d+(?:\.\d+)?)/gi;
  let m: RegExpExecArray | null;
  while ((m = re.exec(userText)) !== null) {
    const name = m[1].toLowerCase();
    if (["coder", "reviewer", "coordinator", "worker", "agent"].some((k) => name.includes(k))) {
      budgets[name] = Number(m[2]);
    }
  }
  const shared = /\bshare one|shared? budget\b/.test(userText.toLowerCase());
  const mode = shared || Object.keys(budgets).length === 0 ? "shared" : "separate";
  writeJson(store, { mode, budgets, updated_at: new Date().toISOString() });
  try {
    appendAudit("user", "swarm.budgets", `${mode}: ${JSON.stringify(budgets).slice(0, 500)}`);
  } catch { /* audit must never block save */ }
  if (mode === "shared") {
    return `Saved: one shared budget for the team.\nChoices:\n1. Keep going.\n2. Split into separate budgets instead.`;
  }
  const list = Object.entries(budgets).map(([k, v]) => `${k} $${v}`).join(", ");
  return `Saved separate budgets: ${list}.\nChoices:\n1. Keep going.\n2. Change them in plain words.`;
}

function modelReply(userText: string): string {
  const pick = recommend(userText);
  try { appendAudit("user", "model.suggest", `${pick.task}: ${pick.host}/${pick.model}`); } catch {}
  return (
    `Static suggestion for ${pick.task}: ${pick.host} (${pick.model}) — ${pick.reason}. ` +
    `Kineti does not switch your active tool or model.\nChoices:\n1. Use the suggestion yourself.\n2. Stay with your current tool.`
  );
}

function helpReply(): string {
  return (
    `This helper answers local status questions and offers suggestions. Your coding agent must call Kineti tools to run checks or make changes.\n` +
    `Try:\n1. Ask "how much have I spent?".\n2. Ask "where are we?".\n3. Ask "did the last test pass?".`
  );
}

function syncReply(): string {
  const s = readJson<{ sync_enabled?: boolean }>(path.join(projectKdir(), "kineti.json"));
  const on = s?.sync_enabled === true;
  return (
    `Automatic device sync is not available. The saved preference is ${on ? "on" : "off"}; this version supports manual encrypted export and import only.\n` +
    `Choices:\n1. Keep working locally.\n2. Use the manual export/import command.`
  );
}

function dashboardReply(userText: string): string {
  const low = userText.toLowerCase();
  if (/\brevoke|unmirror|drop|unlink\b/.test(low)) {
    const result = dropLink("user");
    return `${result.hadPairing || result.hadCloud ? "Local pairing data removed." : "No local pairing data was found."} No cloud service is connected.\nChoices:\n1. Keep using the local dashboard.\n2. Ask about another task.`;
  }
  return (
    `Cloud dashboard linking is not available in this release. The Companion stays on this device; no pairing code was made.\n` +
    `Choices:\n1. Keep using the local dashboard.\n2. Ask about another task.`
  );
}

function forgetReply(userText: string): string {
  const short = userText.slice(0, 160);
  return (
    `I cannot delete or verify deletion of stored data in this release. Nothing was changed for "${short}". ` +
    `Delete it directly in the service that stores it.\n` +
    `Choices:\n1. Keep the data unchanged.\n2. Tell me what you want to do instead.`
  );
}

const RISKY_WORDS = /\b(pay|purchase|buy|send|email|transfer|delete|deploy|publish)\b/;

function taskReply(userText: string): string {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const hasGoal = typeof state.root_goal === "string" && state.root_goal.length > 0;
  const short = userText.slice(0, 160);
  if (!hasGoal) {
    return (
      `Noted: "${short}". This helper does not lock a goal or start work. Set local state with kineti state init, or continue in your editor agent.\n` +
      `Choices:\n1. Set the goal in project state.\n2. Rephrase the task.`
    );
  }
  if (RISKY_WORDS.test(userText.toLowerCase())) {
    return (
      `This may be costly or hard to undo: "${short}". This helper does not execute or block the action. Review it in your coding agent before proceeding.\n` +
      `Choices:\n1. Review the risks.\n2. Continue in your coding agent.\n3. Stop.`
    );
  }
  return (
    `Noted: "${short}". This helper does not change files or run checks. Continue the task in your coding agent.\n` +
    `Choices:\n1. Continue in your coding agent.\n2. Ask for a short plan.\n3. Rephrase the task.`
  );
}

export function route(raw: string): RouterReply {
  const intent = classifyIntent(raw);
  if (intent === "kineti_on") {
    if (isEnabled()) return { intent, reply: `I am already on.\nChoices:\n1. Keep going.\n2. Turn me off.` };
    setEnabled(true);
    try { appendAudit("user", "kineti.on", "enabled from chat"); } catch {}
    return { intent, reply: `The plain-talk helper is on. Your coding agent still needs to call Kineti tools to run checks.\nChoices:\n1. Continue in your coding agent.\n2. Show local status.` };
  }
  if (!isEnabled()) {
    return { intent, reply: `The plain-talk helper is off. This does not stop your coding agent or other tools.\nChoices:\n1. Turn the helper back on.\n2. Keep it off.` };
  }
  switch (intent) {
    case "kineti_off":
      setEnabled(false);
      try { appendAudit("user", "kineti.off", "disabled from chat"); } catch {}
      return { intent, reply: `The plain-talk helper is off. This does not stop your coding agent or other tools.\nChoices:\n1. Turn the helper back on.\n2. Keep it off.` };
    case "spend": return { intent, reply: spendReply() };
    case "undo": return { intent, reply: undoReply() };
    case "proof": return { intent, reply: proofReply() };
    case "status": return { intent, reply: statusReply() };
    case "approve_yes": return { intent, reply: approveYesReply() };
    case "approve_fix": return { intent, reply: approveFixReply(raw) };
    case "swarm_budget": return { intent, reply: swarmBudgetReply() };
    case "swarm_save": return { intent, reply: swarmSaveReply(raw) };
    case "model": return { intent, reply: modelReply(raw) };
    case "dashboard": return { intent, reply: dashboardReply(raw) };
    case "sync": return { intent, reply: syncReply() };
    case "forget": return { intent, reply: forgetReply(raw) };
    case "help": return { intent, reply: helpReply() };
    case "task":
    default: return { intent, reply: taskReply(raw) };
  }
}

if (import.meta.main) {
  const argv = process.argv.slice(2);
  const asJson = argv.includes("--json");
  const text = argv.filter((a) => a !== "--json").join(" ").trim();
  if (!text) {
    console.error("kineti: give me words, for example: how much have I spent?");
    process.exit(2);
  }
  const r = route(text);
  if (asJson) console.log(JSON.stringify(r));
  else console.log(r.reply);
}
