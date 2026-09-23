#!/usr/bin/env bun
// bin/kineti-router.ts
// Phase 2 auto experience: plain-talk intent router, Kineti native.
// No outside model. Keyword rules only. User never sees skill or tool names.
// All replies use plain words plus numbered choices.

import path from "node:path";
import { projectKdir, readJson, readJsonl, writeJson, loadLimits } from "./lib.ts";
import { recommend, isAutoSwitch, setAutoSwitch } from "./kineti-models.ts";
import { appendAudit } from "./kineti-audit.ts";
import { livePairing, makePairing, minutesLeft, revokePairing, PAIR_LINK } from "./kineti-pairing.ts";

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
  // Model auto-switch phrases first: they contain on/off words of their own.
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
    `I found ${pending.length} change${pending.length === 1 ? "" : "s"} that can be undone, newest first. ` +
    `Latest: ${names}. Undo needs your yes in this chat.\n` +
    `Choices:\n1. Yes, undo the latest change.\n2. No, keep everything.\n3. Show me what would be undone first.`
  );
}

function proofReply(): string {
  const records = readJsonl<any>(path.join(projectKdir(), "evidence.jsonl"));
  if (records.length === 0) {
    return `No test proof saved yet.\nChoices:\n1. Run the checks now.\n2. Keep working, check later.`;
  }
  const last = records[records.length - 1];
  const ageMin = Math.round((Date.now() - new Date(last.at).getTime()) / 60000);
  const ageText = ageMin <= 1 ? "just now" : `${ageMin} minutes ago`;
  if (last.exit_code !== 0) {
    return (
      `Last check "${last.label}" ran ${ageText} and failed. Nothing shipped from it.\n` +
      `Choices:\n1. Fix it now.\n2. Show me the failure.\n3. Skip for now.`
    );
  }
  if (ageMin > 240) {
    return (
      `Last passing check "${last.label}" ran ${ageText}, which is old. I will re-run before shipping.\n` +
      `Choices:\n1. Re-run the checks now.\n2. Keep working.`
    );
  }
  return (
    `Tests passed ${ageText} ("${last.label}"). Proof is saved and fresh.\n` +
    `Choices:\n1. Keep going.\n2. Re-run the checks.`
  );
}

function statusReply(): string {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const goal = typeof state.root_goal === "string" && state.root_goal ? state.root_goal : "no goal locked yet";
  const stage = state.stage ?? "not started";
  return (
    `Goal: ${goal}\nStep: ${stage} of 13.\n` +
    `Choices:\n1. Continue from here.\n2. Change the goal in plain words.\n3. Show spending.`
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
    `Got it, yes. I will move ahead with ${name}. Safety, spending, and proof checks run in the background.\n` +
    `Choices:\n1. Go.\n2. Wait, show me the short plan first.`
  );
}

function approveFixReply(userText: string): string {
  const short = userText.slice(0, 200);
  return (
    `Got it, I will change course based on this: "${short}". Nothing is final until you say yes.\n` +
    `Choices:\n1. Show me the updated short plan.\n2. Keep the old plan.`
  );
}

function swarmBudgetReply(): string {
  return (
    `I see a team here. Each helper can get its own budget so one cannot spend it all.\n` +
    `Choices:\n1. Yes, give separate budgets.\n2. No, share one budget.\n3. Show current spending first.`
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
  const low = userText.toLowerCase();
  if (/\bauto.switch\s+on\b|\bauto switch on\b|\bturn auto on\b/.test(low)) {
    setAutoSwitch(true);
    try { appendAudit("user", "model.auto_on", "enabled from chat"); } catch {}
    return `Auto-switch is on. I will move between tools on my own and log every switch.\nChoices:\n1. Keep going.\n2. Turn auto-switch off.`;
  }
  if (/\bauto.switch\s+off\b|\bauto switch off\b|\bturn auto off\b/.test(low)) {
    setAutoSwitch(false);
    try { appendAudit("user", "model.auto_off", "disabled from chat"); } catch {}
    return `Auto-switch is off. I will always ask first.\nChoices:\n1. Keep going.\n2. Ask me which tool fits this task.`;
  }
  const pick = recommend(userText);
  const auto = isAutoSwitch();
  try { appendAudit("user", "model.suggest", `${pick.task}: ${pick.host}/${pick.model}`); } catch {}
  if (auto) {
    return (
      `This looks like ${pick.task} work. Moving to ${pick.host} (${pick.model}): ${pick.reason}.\n` +
      `Logged with reason.\nChoices:\n1. Go.\n2. Stay here instead.`
    );
  }
  return (
    `For this ${pick.task} work, ${pick.host} (${pick.model}) fits best: ${pick.reason}.\n` +
    `I will not move without your yes.\nChoices:\n1. Yes, switch.\n2. Stay here.\n3. Turn on auto-switch.`
  );
}

function helpReply(): string {
  return (
    `Just talk normal. I handle safety, spending, undo, and proof in the background. ` +
    `You only see results and advice. Say "Kineti off" any time to pause me.\n` +
    `Try:\n1. Tell me your idea in one sentence.\n2. Ask "how much have I spent?".\n3. Ask "where are we?".`
  );
}

function syncReply(): string {
  const s = readJson<{ sync_enabled?: boolean }>(path.join(projectKdir(), "kineti.json"));
  const on = s?.sync_enabled === true;
  if (!on) {
    return (
      `Device sync is off. When on, your notes move between your devices in encrypted form. ` +
      `Your goal is never overwritten by an import.\n` +
      `Choices:\n1. Turn sync on.\n2. Keep it off.`
    );
  }
  return (
    `Device sync is on. Exports are passphrase-encrypted, imports merge notes and never touch your goal.\n` +
    `Choices:\n1. Export my notes.\n2. Import on this device.\n3. Turn sync off.`
  );
}

function dashboardReply(userText: string): string {
  const low = userText.toLowerCase();
  if (/\brevoke|unmirror|drop|unlink\b/.test(low)) {
    const had = revokePairing("user");
    if (!had) return `There is no cloud link to drop. Local dashboard still works.\nChoices:\n1. Make a cloud link.\n2. Keep local only.`;
    return `Cloud link dropped. Stored tokens deleted on this device.\nChoices:\n1. Make a new link.\n2. Keep local only.`;
  }
  const live = livePairing();
  if (live) {
    return (
      `Your code is ${live.code}. Open ${PAIR_LINK} and log in with GitHub, then type the code. ` +
      `Valid ${minutesLeft(live)} more min, one use, project ${live.project}.\n` +
      `Choices:\n1. Make a fresh code.\n2. Drop the link.\n3. Keep local only.`
    );
  }
  if (/\b(yes|make|create|generate)\b/.test(low)) {
    const p = makePairing("user");
    return (
      `Your code is ${p.code}. Open ${PAIR_LINK} and log in with GitHub, then type the code. ` +
      `Valid ${minutesLeft(p)} min, one use, project ${p.project}. Nothing mirrors until you approve that project too.\n` +
      `Choices:\n1. Drop the link.\n2. Keep local only.`
    );
  }
  return (
    `Do you want a UI cloud link? The web dashboard then shows the same screens as here.\n` +
    `Choices:\n1. Yes, make a cloud link.\n2. No, local only.`
  );
}

function forgetReply(userText: string): string {
  const short = userText.slice(0, 160);
  return (
    `I will delete "${short}" everywhere: chat memory, vector index, and connected service caches. ` +
    `Your goal and identity stay. You get a written proof receipt with count and time.\n` +
    `Nothing is deleted until you say yes.\n` +
    `Choices:\n1. Yes, delete it and show proof.\n2. Show me what would be deleted first.\n3. Keep everything.`
  );
}

const RISKY_WORDS = /\b(pay|purchase|buy|send|email|transfer|delete|deploy|publish)\b/;

function taskReply(userText: string): string {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const hasGoal = typeof state.root_goal === "string" && state.root_goal.length > 0;
  const short = userText.slice(0, 160);
  if (!hasGoal) {
    return (
      `Noted: "${short}". I will treat that as your goal and lock it. ` +
      `Then I will ask only what is missing.\n` +
      `Choices:\n1. Yes, lock it.\n2. Let me rephrase.`
    );
  }
  if (RISKY_WORDS.test(userText.toLowerCase())) {
    return (
      `This looks costly or hard to undo: "${short}". I will check the risk first ` +
      `and nothing goes out until you say yes in plain words.\n` +
      `Choices:\n1. Check the risk and show me.\n2. Go ahead after the check.\n3. Stop.`
    );
  }
  return (
    `Noted: "${short}". I will fold it into your current goal and run safety and spending checks quietly.\n` +
    `Choices:\n1. Go ahead.\n2. Show me the short plan first.\n3. Stop, I will rephrase.`
  );
}

export function route(raw: string): RouterReply {
  const intent = classifyIntent(raw);
  if (intent === "kineti_on") {
    if (isEnabled()) return { intent, reply: `I am already on.\nChoices:\n1. Keep going.\n2. Turn me off.` };
    setEnabled(true);
    try { appendAudit("user", "kineti.on", "enabled from chat"); } catch {}
    return { intent, reply: `I am on now. Safety, spending, undo, and proof checks run again.\nChoices:\n1. Continue where we left off.\n2. Show status.` };
  }
  if (!isEnabled()) {
    return { intent, reply: `Kineti is off, so I am not running checks.\nChoices:\n1. Turn me back on.\n2. Keep me off.` };
  }
  switch (intent) {
    case "kineti_off":
      setEnabled(false);
      try { appendAudit("user", "kineti.off", "disabled from chat"); } catch {}
      return { intent, reply: `I am off now. No checks run until you say "Kineti on".\nChoices:\n1. Turn me back on.\n2. Keep me off.` };
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
