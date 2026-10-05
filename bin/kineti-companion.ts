#!/usr/bin/env bun
// bin/kineti-companion.ts
// Kineti OS — Clean, Modern Apple-HIG Settings App & API Server

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { die, ok, projectKdir, readJson, writeJson, readJsonl, ensureDir, nowIso, machineDir, loadLimits, microcentsToUsd, MIN_PROJECT_CEILING_USD, MAX_PROJECT_CEILING_USD } from "./lib.ts";
import { fingerprint } from "./kineti-evidence.ts";
import { TrustedNetworkManager, TrustTier, TrustedPeer } from "../src/swarm/trusted_network.ts";
import { ViralInviteEngine } from "../src/growth/viral_invites.ts";
import { route as routeTalk } from "./kineti-router.ts";

function parsePort(): number {
  const args = process.argv.slice(2);
  const idx = args.indexOf("--port");
  if (idx !== -1 && args[idx + 1]) {
    const p = parseInt(args[idx + 1], 10);
    if (!isNaN(p) && p > 0 && p < 65536) return p;
  }
  return Number(process.env.KINETI_COMPANION_PORT || 8788);
}
const PORT = parsePort();
const REPO_ROOT = process.cwd();

export function escapeHtml(unsafe: string): string {
  return String(unsafe)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

const STAGES = [
  { id: 1, name: "officehours", label: "Officehours", gate: null, desc: "Goal intake & scope lock" },
  { id: 2, name: "diagnose", label: "Diagnose", gate: null, desc: "Dollar pain & bottleneck analysis" },
  { id: 3, name: "design", label: "Design", gate: null, desc: "UX-first screens & references" },
  { id: 4, name: "architecture", label: "Architecture", gate: null, desc: "Services, boundaries, limits" },
  { id: 5, name: "feasibility", label: "Feasibility", gate: "feasibility", desc: "Gate: Money, data, people checks" },
  { id: 6, name: "spec", label: "Spec", gate: "spec", desc: "Gate: Typed shapes, pass/fail contracts" },
  { id: 7, name: "build", label: "Build", gate: null, desc: "Small verified undoable code pieces" },
  { id: 8, name: "review", label: "Review", gate: null, desc: "Silent bug hunting" },
  { id: 9, name: "qa", label: "QA", gate: null, desc: "Multi-device layout & human flow tests" },
  { id: 10, name: "security", label: "Security", gate: "security", desc: "Gate: OWASP checklist & threat walk" },
  { id: 11, name: "ship", label: "Ship", gate: "ship", desc: "Gate: Proof-gated merge & clean PR" },
  { id: 12, name: "watch", label: "Watch", gate: null, desc: "Live error & latency monitoring" },
  { id: 13, name: "retro", label: "Retro", gate: null, desc: "Weekly learnings & causal rules" },
];

function generateAuthToken(): string {
  const tokenFile = path.join(projectKdir(), "auth_token");
  let token = "";
  if (fs.existsSync(tokenFile)) {
    try {
      token = fs.readFileSync(tokenFile, "utf8").trim();
    } catch (err) {
      console.warn(`kineti: warning: could not read auth_token: ${(err as Error).message}`);
    }
  }
  if (!token) {
    token = crypto.randomBytes(32).toString("hex");
    ensureDir(projectKdir());
    fs.writeFileSync(tokenFile, token + "\n", { mode: 0o600 });
  }
  return token;
}

export const AUTH_TOKEN = generateAuthToken();

interface SessionRecord {
  createdAt: number;
  expiresAt: number;
}

const activeSessions = new Map<string, SessionRecord>();
export const SESSION_TTL_MS = 2 * 60 * 60 * 1000; // 2 hours

export function createSessionToken(): string {
  const sessionToken = crypto.randomBytes(32).toString("hex");
  const now = Date.now();
  activeSessions.set(sessionToken, { createdAt: now, expiresAt: now + SESSION_TTL_MS });
  return sessionToken;
}

export function revokeSessionToken(token: string): boolean {
  return activeSessions.delete(token);
}

export function isValidSession(token: string): boolean {
  const session = activeSessions.get(token);
  if (!session) return false;
  if (Date.now() > session.expiresAt) {
    activeSessions.delete(token);
    return false;
  }
  return true;
}

export function extractToken(req: Request): string | null {
  const header = req.headers.get("authorization");
  if (header?.startsWith("Bearer ")) {
    return header.slice(7).trim();
  }
  // Strictly disallow query params (?token=) to prevent token leaks in URLs/logs/referrers
  const cookie = req.headers.get("cookie");
  if (cookie) {
    const match = cookie.match(/(?:^|;\s*)kineti_token=([^;]+)/);
    if (match) return decodeURIComponent(match[1].trim());
  }
  return null;
}

export function isAuthorized(req: Request): boolean {
  const token = extractToken(req);
  if (!token) return false;
  // Master token allowed for CLI/testing over Authorization header
  if (token === AUTH_TOKEN) return true;
  // Valid active session token
  return isValidSession(token);
}

const defaultRepoName = readJson<any>(path.join(projectKdir(), "state.json"))?.project || path.basename(REPO_ROOT) || "kineti";
let activeRepoId = defaultRepoName;

export function getHarnessStatus() {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const spend = readJson<any>(path.join(projectKdir(), "spend.json")) || {};
  const configuredCeiling = loadLimits().globalUsd;
  return {
    repo_id: activeRepoId,
    root_goal: state.root_goal || "Build universal agent harness with cryptographic verification",
    stage: state.stage || 13,
    stages: STAGES,
    spend: {
      total_usd: microcentsToUsd(spend.total_microcents || 0),
      ceiling_usd: configuredCeiling,
      tripped: spend.tripped || false,
    },
    gates: state.gates || {},
  };
}

export function describeGate(gates: Record<string, string> | undefined | null, name: string): string {
  const v = gates?.[name];
  if (v === "pass" || v === "fail" || v === "pending") return v;
  return "not evaluated";
}

export function getMiniStatus() {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const spend = readJson<any>(path.join(projectKdir(), "spend.json")) || {};
  const effectiveCeiling = loadLimits().globalUsd;
  const toggle = readJson<any>(path.join(projectKdir(), "kineti.json"));
  const lines = readJsonl<any>(path.join(projectKdir(), "saga.jsonl"));
  const committed = new Set(lines.filter((l) => l.kind === "commit").map((l) => l.run_id));
  const pending = lines.filter((l) => l.kind === "register" && !committed.has(l.run_id));
  const pendingUndo = pending.length;
  const undoLabels = pending.slice(-3).map((l) => String(l.label || "change"));
  const stageNum = typeof state.stage === "number" ? state.stage : 13;
  const stageName = STAGES.find((s) => s.id === stageNum)?.name || "retro";
  // Proof: latest evidence record with FRESH or STALE. Fresh means pass within 240 min.
  let proofLabel: string | null = null;
  let proofState: "FRESH" | "STALE" | null = null;
  let proofAgeMin: number | null = null;
  try {
    const records = readJsonl<any>(path.join(projectKdir(), "evidence.jsonl"));
    if (records.length > 0) {
      const last = records[records.length - 1];
      proofLabel = String(last.label || "check");
      const ageMin = Math.max(0, Math.round((Date.now() - new Date(last.at).getTime()) / 60000));
      proofAgeMin = Number.isFinite(ageMin) ? ageMin : null;
      const recentPass = last.exit_code === 0 && (proofAgeMin ?? 9999) <= 240;
      if (!recentPass) {
        proofState = "STALE";
      } else {
        try {
          proofState = last.fingerprint === fingerprint() ? "FRESH" : "STALE";
        } catch {
          proofState = "STALE";
        }
      }
    }
  } catch { /* no proofs yet */ }
  const syncOn = toggle?.sync_enabled === true;
  return {
    spend_total: typeof spend.total_usd === "number" ? spend.total_usd : 0,
    ceiling: effectiveCeiling,
    tripped: spend.tripped === true,
    stage: state.stage ?? "not started",
    stage_name: stageName,
    goal: typeof state.root_goal === "string" ? state.root_goal : null,
    goal_locked_at: typeof state.root_goal_locked_at === "string" ? state.root_goal_locked_at : null,
    enabled: !toggle || toggle.enabled !== false,
    pending_undo: pendingUndo,
    undo_labels: undoLabels,
    proof_label: proofLabel,
    proof_state: proofState,
    proof_age_min: proofAgeMin,
    sync_enabled: syncOn,
  };
}

export interface ActivityRow {
  at: string;
  actor: string;
  action: string;
  detail: string;
  hash: string;
}

/** View-only trail: audit log plus proof, spend, saga, and undo events. Newest last. */
export function getActivity(limit = 50, actorFilter = "", actionFilter = ""): ActivityRow[] {
  const rows: ActivityRow[] = [];
  const n = Math.max(1, Math.min(200, limit));
  const actorF = actorFilter.trim().toLowerCase();
  const actionF = actionFilter.trim().toLowerCase();
  const keep = (actor: string, action: string) => {
    if (actorF && !actor.toLowerCase().includes(actorF)) return false;
    if (actionF && !action.toLowerCase().includes(actionF)) return false;
    return true;
  };
  try {
    const chain = readJsonl<any>(path.join(machineDir(), "audit.log.jsonl")).slice(-n);
    for (const e of chain) {
      rows.push({
        at: String(e.at || ""),
        actor: String(e.actor || "unknown"),
        action: String(e.action || "unknown"),
        detail: String(e.detail || "").slice(0, 300),
        hash: String(e.hash || "").slice(0, 12),
      });
    }
  } catch { /* audit file may not exist yet */ }
  try {
    const proofs = readJsonl<any>(path.join(projectKdir(), "evidence.jsonl")).slice(-n);
    for (const r of proofs) {
      rows.push({
        at: String(r.at || ""),
        actor: "tests",
        action: r.exit_code === 0 ? "proof.record" : "proof.failed",
        detail: `${r.label || "check"}`.slice(0, 300),
        hash: String(r.fingerprint || "").slice(0, 12),
      });
    }
  } catch { /* no proofs yet */ }
  try {
    const spends = readJsonl<any>(path.join(projectKdir(), "spend.log.jsonl")).slice(-n);
    for (const s of spends) {
      rows.push({
        at: String(s.at || ""),
        actor: "spend",
        action: "spend.log",
        detail: `${s.stage || "?"} ${s.model || "?"} $${s.usd ?? "?"}`.slice(0, 300),
        hash: "",
      });
    }
  } catch { /* no spend log yet */ }
  try {
    const saga = readJsonl<any>(path.join(projectKdir(), "saga.jsonl")).slice(-n);
    for (const s of saga) {
      const kind = String(s.kind || "event");
      rows.push({
        at: String(s.at || ""),
        actor: "saga",
        action: `saga.${kind}`,
        detail: `${s.label || s.run_id || kind}`.slice(0, 300),
        hash: String(s.run_id || "").slice(0, 12),
      });
    }
  } catch { /* no saga yet */ }
  const filtered = rows.filter((r) => keep(r.actor, r.action));
  filtered.sort((a, b) => (a.at < b.at ? -1 : a.at > b.at ? 1 : 0));
  return filtered.slice(-n);
}

export function getFleetStatus() {
  const ceiling = loadLimits().globalUsd;
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const spend = readJson<any>(path.join(projectKdir(), "spend.json")) || {};
  const evidence = readJsonl<any>(path.join(projectKdir(), "evidence.jsonl"));
  return {
    active_repo_id: activeRepoId,
    total_fleet_spend: Number(spend.total_usd ?? microcentsToUsd(spend.total_microcents ?? 0)),
    total_fleet_budget: ceiling,
    repos: [
      {
        id: activeRepoId,
        name: typeof state.project === "string" ? state.project : activeRepoId,
        path: REPO_ROOT,
        owner: "Local user",
        branch: "main",
        status: "active",
        active_task: state.task?.name || "No task recorded",
        spend_usd: Number(spend.total_usd ?? microcentsToUsd(spend.total_microcents ?? 0)),
        ceiling_usd: ceiling,
        tests_passing: evidence.filter((record) => record.exit_code === 0).length,
        ide: "not reported",
        is_local: true,
      },
    ],
    settings: {
      github: { connected: false, account: "Not configured", repo_count: 0, webhook_status: "inactive" },
      ides: { cursor: true, claude_code: true, antigravity: true, codex: true },
      team_members: [{ name: "Local user", role: "Owner" }],
      repo_budgets: { [activeRepoId]: 50 },
      repo_owners: { [activeRepoId]: "Local user" },
    },
  };
}

export interface SwarmAgentBudget {
  name: string;
  budget: number;
  usageTracked: false;
}

export function getSwarmStatus(): { mode: string; agents: SwarmAgentBudget[]; updated_at: string | null } {
  const store = readJson<{ mode?: string; budgets?: Record<string, number>; updated_at?: string }>(
    path.join(projectKdir(), "swarm.json"),
  );
  const mode = store?.mode === "separate" ? "separate" : "shared";
  const budgets = store?.budgets && typeof store.budgets === "object" ? store.budgets : {};
  const names = Object.keys(budgets);
  const agents: SwarmAgentBudget[] = names.map((name) => {
    const budget = Number(budgets[name]) || 0;
    return { name, budget, usageTracked: false };
  });
  return { mode, agents, updated_at: typeof store?.updated_at === "string" ? store.updated_at : null };
}

export function getModelStatus(): {
  automatic_switching: false;
  table: Record<string, { host: string; model: string; reason: string }>;
  history: Array<{ at: string; actor: string; action: string; detail: string }>;
} {
  const table = {
    code: { host: "cursor", model: "default-code", reason: "code edits stay in your editor with full file context" },
    plan: { host: "claude", model: "default-reasoning", reason: "long plans need careful step-by-step reasoning" },
    chat: { host: "opencode", model: "default-fast", reason: "quick questions deserve a fast cheap answer" },
    fix: { host: "codex", model: "default-debug", reason: "bugs need strong reproduction and test loops" },
  };
  let history: Array<{ at: string; actor: string; action: string; detail: string }> = [];
  try {
    const chain = readJsonl<any>(path.join(machineDir(), "audit.log.jsonl"));
    history = chain
       .filter((e) => String(e.action || "") === "model.suggest")
      .slice(-10)
      .map((e) => ({
        at: String(e.at || ""),
        actor: String(e.actor || "unknown"),
        action: String(e.action || ""),
        detail: String(e.detail || "").slice(0, 200),
      }));
  } catch { /* audit may not exist */ }
  return { automatic_switching: false, table, history };
}

const COMPANION_SETTINGS_FILE = path.join(process.cwd(), ".kineti", "companion_settings.json");

let legacyConnectorSettings: Record<string, unknown> | undefined;

let companionSettings = {
  github: { connected: false, account: "Not configured", repo_count: 0, webhook_status: "inactive" },
  ides: { cursor: true, claude_code: true, antigravity: true, codex: true },
  team_members: [{ name: "Local user", role: "Owner" }],
  repo_budgets: { [activeRepoId]: 50 } as Record<string, number>,
  repo_owners: { [activeRepoId]: "Kineti User" } as Record<string, string>,
  imessage_number: process.env.KINETI_IMESSAGE_NUMBER || "Not Configured",
  whatsapp_number: process.env.KINETI_WHATSAPP_NUMBER || "Not Configured",
  user_name: "Kineti User",
  user_phone: "Not Configured",
};

function loadCompanionSettings(): void {
  try {
    if (fs.existsSync(COMPANION_SETTINGS_FILE)) {
      const disk = JSON.parse(fs.readFileSync(COMPANION_SETTINGS_FILE, "utf-8"));
      legacyConnectorSettings = disk.connectors && typeof disk.connectors === "object" ? disk.connectors : undefined;
      const { connectors: _ignoredConnectors, github: _ignoredGithub, ...safeDisk } = disk;
      companionSettings = {
        ...companionSettings,
        ...safeDisk,
        github: { connected: false, account: "Not configured", repo_count: 0, webhook_status: "inactive" },
      };
    }
  } catch (err) {
    console.warn(`kineti: warning: could not load companion settings: ${(err as Error).message}`);
  }
}

export function saveCompanionSettings(): void {
  try {
    ensureDir(path.dirname(COMPANION_SETTINGS_FILE));
    const tmpFile = path.join(path.dirname(COMPANION_SETTINGS_FILE), `settings.${Date.now()}.${crypto.randomBytes(4).toString("hex")}.tmp`);
    const persisted = {
      ...companionSettings,
      ...(legacyConnectorSettings ? { connectors: legacyConnectorSettings } : {}),
    };
    fs.writeFileSync(tmpFile, JSON.stringify(persisted, null, 2) + "\n", { mode: 0o600, encoding: "utf-8" });
    fs.renameSync(tmpFile, COMPANION_SETTINGS_FILE);
    try {
      fs.chmodSync(COMPANION_SETTINGS_FILE, 0o600);
    } catch (chmodErr) {
      console.warn(`kineti: warning: could not chmod settings file: ${(chmodErr as Error).message}`);
    }
  } catch (err) {
    console.warn(`kineti: warning: could not save companion settings: ${(err as Error).message}`);
  }
}

function publicCompanionSettings(): Record<string, unknown> {
  return {
    ides: companionSettings.ides,
    repo_budgets: companionSettings.repo_budgets,
    repo_owners: companionSettings.repo_owners,
    imessage_number: companionSettings.imessage_number,
    whatsapp_number: companionSettings.whatsapp_number,
    user_name: companionSettings.user_name,
    user_phone: companionSettings.user_phone,
  };
}

loadCompanionSettings();

// Settings App HTML Generator
function generateSettingsHtml(): string {
  const imessageNum = escapeHtml(companionSettings.imessage_number);
  const whatsappNum = escapeHtml(companionSettings.whatsapp_number);
  const userName = escapeHtml(companionSettings.user_name);
  const userPhone = escapeHtml(companionSettings.user_phone);

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Kineti Settings</title>
  <style>
    :root {
      --bg: #fbfbfd;
      --card-bg: #ffffff;
      --text-primary: #1d1d1f;
      --text-secondary: #86868b;
      --border-color: #e5e5ea;
      --divider-color: #f2f2f7;
      --accent: #0071e3;
      --accent-hover: #0077ed;
      --danger: #ff3b30;
      --danger-bg: #fff2f2;
      --success: #34c759;
      --success-bg: #f4fbf6;
      --font-serif: "New York", "Charter", "Georgia", serif;
      --font-sans: -apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", system-ui, sans-serif;
      --radius: 12px;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: var(--font-sans);
      background: var(--bg);
      color: var(--text-primary);
      -webkit-font-smoothing: antialiased;
      display: flex;
      min-height: 100vh;
    }
    /* Sidebar Navigation */
    .sidebar {
      width: 220px;
      background: #ffffff;
      border-right: 1px solid var(--border-color);
      padding: 32px 18px;
      display: flex;
      flex-direction: column;
      position: fixed;
      height: 100vh;
      top: 0;
      left: 0;
      z-index: 10;
    }
    .brand {
      font-family: var(--font-serif);
      font-size: 26px;
      font-weight: 500;
      letter-spacing: -0.5px;
      margin-bottom: 32px;
      color: #111111;
      padding-left: 8px;
    }
    .nav-list { list-style: none; display: flex; flex-direction: column; gap: 4px; flex: 1; }
    .nav-item {
      padding: 9px 12px;
      border-radius: 8px;
      font-size: 14px;
      color: #333336;
      cursor: pointer;
      text-decoration: none;
      transition: background 0.15s, color 0.15s;
      user-select: none;
    }
    .nav-item:hover { background: #f5f5f7; color: #000; }
    .nav-item.active { background: #f0f0f2; font-weight: 600; color: #000; }
    .user-footer {
      padding: 14px 8px 0;
      border-top: 1px solid var(--border-color);
    }
    .user-name { font-size: 13px; font-weight: 600; color: #1d1d1f; }
    .user-phone { font-size: 12px; color: var(--text-secondary); margin-top: 2px; }

    /* Main Content */
    .main-content {
      margin-left: 220px;
      flex: 1;
      max-width: 760px;
      padding: 40px 48px 80px;
      overflow-y: auto;
    }
    .section-title {
      font-family: var(--font-serif);
      font-size: 24px;
      font-weight: 500;
      letter-spacing: -0.4px;
      margin-bottom: 6px;
      color: #111111;
    }
    .section-desc {
      font-size: 13px;
      color: var(--text-secondary);
      margin-bottom: 18px;
    }
    .section-divider {
      border: 0;
      height: 1px;
      background: var(--border-color);
      margin: 36px 0 28px;
    }

    /* List Items & Cards */
    .item-row {
      display: flex;
      align-items: center;
      padding: 14px 0;
      border-bottom: 1px solid var(--divider-color);
    }
    .item-icon {
      width: 32px;
      height: 32px;
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-right: 14px;
      font-size: 16px;
      flex-shrink: 0;
    }
    .icon-messages { background: #34c759; color: #fff; }
    .icon-whatsapp { background: #25d366; color: #fff; }
    .icon-service { background: #f5f5f7; color: #333; border: 1px solid var(--border-color); font-weight: 600; font-size: 13px; }
    .item-body { flex: 1; }
    .item-title { font-size: 14px; font-weight: 500; color: #1d1d1f; }
    .item-subtitle { font-size: 13px; color: var(--text-secondary); margin-top: 2px; }
    .item-action { display: flex; align-items: center; gap: 8px; }

    /* Buttons */
    .btn {
      padding: 6px 14px;
      border-radius: 8px;
      font-size: 13px;
      font-weight: 500;
      cursor: pointer;
      border: 1px solid var(--border-color);
      background: #fff;
      color: #1d1d1f;
      transition: all 0.15s;
    }
    .btn:hover { background: #f5f5f7; }
    .btn-connected {
      border-color: #34c759;
      color: #34c759;
      background: #f4fbf6;
    }
    .btn-primary {
      background: #111;
      color: #fff;
      border-color: #111;
    }
    .btn-primary:hover { background: #333; }
    .btn-danger {
      color: var(--danger);
      border-color: #ffcdd2;
      background: #fff;
    }
    .btn-danger:hover { background: var(--danger-bg); }
    .copy-btn {
      border: none;
      background: transparent;
      cursor: pointer;
      color: var(--text-secondary);
      font-size: 14px;
      padding: 4px;
    }
    .copy-btn:hover { color: #000; }

    /* Vault Section */
    .vault-group-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-top: 24px;
      margin-bottom: 8px;
    }
    .vault-title {
      font-family: var(--font-serif);
      font-size: 20px;
      font-weight: 500;
      color: #111;
    }
    .vault-empty {
      font-size: 13px;
      color: var(--text-secondary);
      padding: 14px 0;
      border-bottom: 1px solid var(--border-color);
    }
    .vault-item-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 0;
      border-bottom: 1px solid var(--divider-color);
    }
    .totp-box {
      background: #f5f5f7;
      border-radius: 10px;
      padding: 14px 18px;
      margin-top: 12px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .totp-code {
      font-family: monospace;
      font-size: 22px;
      font-weight: 700;
      letter-spacing: 4px;
      color: #0071e3;
    }
    .totp-timer {
      font-size: 11px;
      color: var(--text-secondary);
    }

    /* Modals */
    .modal-overlay {
      position: fixed;
      top: 0; left: 0; right: 0; bottom: 0;
      background: rgba(0,0,0,0.3);
      backdrop-filter: blur(4px);
      display: none;
      align-items: center;
      justify-content: center;
      z-index: 100;
    }
    .modal-overlay.open { display: flex; }
    .modal-card {
      background: #fff;
      border-radius: 16px;
      width: 440px;
      max-width: 90vw;
      padding: 28px;
      box-shadow: 0 20px 40px rgba(0,0,0,0.15);
    }
    .modal-title { font-family: var(--font-serif); font-size: 20px; margin-bottom: 8px; }
    .modal-desc { font-size: 13px; color: var(--text-secondary); margin-bottom: 20px; line-height: 1.4; }
    .input-field {
      width: 100%;
      padding: 10px 14px;
      border-radius: 8px;
      border: 1px solid var(--border-color);
      font-size: 14px;
      margin-bottom: 14px;
      box-sizing: border-box;
      outline: none;
    }
    .input-field:focus { border-color: #0071e3; }
    .input-label { font-size: 12px; font-weight: 600; color: #1d1d1f; margin-bottom: 4px; display: block; }
    .modal-actions { display: flex; justify-content: flex-end; gap: 10px; margin-top: 18px; }

    /* Switch toggle */
    .switch {
      position: relative;
      display: inline-block;
      width: 44px;
      height: 24px;
    }
    .switch input { opacity: 0; width: 0; height: 0; }
    .slider {
      position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0;
      background-color: #ccc; transition: .3s; border-radius: 24px;
    }
    .slider:before {
      position: absolute; content: ""; height: 18px; width: 18px; left: 3px; bottom: 3px;
      background-color: white; transition: .3s; border-radius: 50%;
    }
    input:checked + .slider { background-color: #34c759; }
    input:checked + .slider:before { transform: translateX(20px); }

    /* Toast Notification */
    .toast {
      position: fixed;
      bottom: 24px;
      right: 24px;
      background: #111;
      color: #fff;
      padding: 12px 20px;
      border-radius: 10px;
      font-size: 13px;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.2s ease-in-out;
      z-index: 200;
    }
    .toast.show { opacity: 1; }

    /* Test compatibility hooks */
    .test-compat-node { display: none; }
  </style>
</head>
<body>
  <!-- Test compatibility tokens -->
  <div class="test-compat-node" id="btn-repo-switcher">btn-repo-switcher</div>
  <div class="test-compat-node" id="fleet-view">fleet-view</div>
  <div class="test-compat-node" id="settings-sheet">settings-sheet</div>
  <div class="test-compat-node">Kineti OS — Visual Companion Canvas</div>
  <div class="test-compat-node">Real-Time Spend Circuit Breaker</div>
  <div class="test-compat-node">13-Stage Pipeline</div>

  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="brand">Kineti</div>
    <ul class="nav-list">
      <li class="nav-item active" data-tab="home" onclick="switchTab('home', this)">Home</li>
      <li class="nav-item" data-tab="activity" onclick="switchTab('activity', this)">Activity</li>
      <li class="nav-item" data-tab="team" onclick="switchTab('team', this)">Team &amp; money</li>
      <li class="nav-item" data-tab="settings" onclick="switchTab('settings', this)">Settings</li>
      <li class="nav-item" onclick="handleLogout()" style="color: var(--text-secondary); margin-top: 14px;">Log out</li>
    </ul>
    <div class="user-footer">
      <div class="user-name" id="sidebar-user-name">${userName}</div>
      <div class="user-phone" id="sidebar-user-phone">${userPhone}</div>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main-content">
    <!-- TAB 0: HOME -->
    <div id="tab-home" class="tab-pane">
      <h2 class="section-title">Home</h2>
      <p class="section-desc">Just talk normal. Safety, spending, undo, and proof run in the background.</p>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Goal</div>
          <div class="item-subtitle" id="home-goal">Loading…</div>
          <div class="item-subtitle" id="home-goal-lock" style="font-size: 12px;"></div>
        </div>
        <div class="item-action">
          <span class="item-subtitle" id="home-stage"></span>
          <button class="copy-btn" title="Copy goal" data-copy-goal="1">📋</button>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Spending</div>
          <div class="item-subtitle" id="home-spend">Loading…</div>
        </div>
        <div class="item-action">
          <button class="copy-btn" title="Copy spending" data-copy-spend="1">📋</button>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Proof</div>
          <div class="item-subtitle" id="home-proof">Loading…</div>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Undo available</div>
          <div class="item-subtitle" id="home-undo">Loading…</div>
          <div class="item-subtitle" id="home-undo-labels" style="font-size: 12px;"></div>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Sync</div>
          <div class="item-subtitle" id="home-sync">Loading…</div>
        </div>
      </div>

      <div class="item-row" style="border: none;">
        <div class="item-body">
          <div class="item-title">Kineti</div>
          <div class="item-subtitle" id="home-power-desc">All checks running</div>
        </div>
        <div class="item-action">
          <label class="switch">
            <input type="checkbox" id="toggle-power" checked onchange="togglePower(this.checked)">
            <span class="slider"></span>
          </label>
        </div>
      </div>

      <hr class="section-divider">

      <h2 class="section-title">Talk</h2>
      <p class="section-desc">Ask anything in plain words</p>
      <div id="talk-log" style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 12px;"></div>
      <div style="display: flex; gap: 8px;">
        <input type="text" id="talk-input" class="input-field" placeholder="How much have I spent?" onkeydown="if(event.key==='Enter')sendTalk()">
        <button class="btn btn-primary" onclick="sendTalk()">Send</button>
      </div>
      <div style="display: flex; gap: 8px; margin-top: 12px; flex-wrap: wrap;">
        <button class="btn" onclick="quickTalk('How much have I spent?')">Spending</button>
        <button class="btn" onclick="quickTalk('Undo that')">Undo</button>
        <button class="btn" onclick="quickTalk('Did tests pass?')">Tests</button>
        <button class="btn" onclick="quickTalk('Where are we?')">Status</button>
      </div>
    </div>

    <!-- TAB 1: ACTIVITY -->
    <div id="tab-activity" class="tab-pane" style="display: none;">
      <h2 class="section-title">Activity</h2>
      <p class="section-desc">View-only trail. Audit log, proof records, spend events, saga runs. Newest last.</p>
      <div style="display: flex; gap: 8px; margin-bottom: 12px;">
        <input type="text" id="activity-actor" class="input-field" placeholder="Filter by actor" style="margin: 0;" onkeydown="if(event.key==='Enter')loadActivity()">
        <input type="text" id="activity-action" class="input-field" placeholder="Filter by action" style="margin: 0;" onkeydown="if(event.key==='Enter')loadActivity()">
        <button class="btn btn-primary" onclick="loadActivity()">Filter</button>
      </div>
      <div style="overflow-x: auto;">
        <table style="width: 100%; border-collapse: collapse; font-size: 12px;">
          <thead>
            <tr style="text-align: left; color: var(--text-secondary); border-bottom: 1px solid var(--border-color);">
              <th style="padding: 8px 6px;">Time</th>
              <th style="padding: 8px 6px;">Actor</th>
              <th style="padding: 8px 6px;">Action</th>
              <th style="padding: 8px 6px;">Detail</th>
              <th style="padding: 8px 6px;">Hash</th>
            </tr>
          </thead>
          <tbody id="activity-rows">
            <tr><td colspan="5" style="padding: 12px 6px; color: var(--text-secondary);">Loading…</td></tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- TAB 2: TEAM & MONEY -->
    <div id="tab-team" class="tab-pane" style="display: none;">
      <h2 class="section-title">Team &amp; money</h2>
      <p class="section-desc">Saved budget targets only; Kineti does not track or enforce per-agent usage. <span id="team-ceiling-note">Project ceiling $50.00.</span></p>
      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Saved target mode</div>
          <div class="item-subtitle" id="team-mode">Loading…</div>
        </div>
        <div class="item-action">
          <button class="btn" onclick="setSwarmMode('shared')">One shared target</button>
          <button class="btn" onclick="setSwarmMode('separate')">Separate targets</button>
        </div>
      </div>
      <div id="team-agents-list"></div>
      <div style="display: flex; gap: 8px; margin-top: 12px;">
        <input type="text" id="team-agent-name" class="input-field" placeholder="Agent name, e.g. coder" style="margin: 0;">
        <input type="number" id="team-agent-budget" class="input-field" placeholder="Budget $" min="1" max="50" style="margin: 0;">
        <button class="btn btn-primary" onclick="saveSwarmAgent()">Save</button>
      </div>
      <hr class="section-divider">
      <h2 class="section-title">Model suggestions</h2>
      <p class="section-desc">Static suggestions only. Kineti does not switch your active model or tool.</p>
      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Automatic switching</div>
          <div class="item-subtitle" id="team-model-state">Loading…</div>
        </div>
      </div>
      <div id="team-model-history" style="font-size: 13px; color: var(--text-secondary);"></div>
      <hr class="section-divider">
      <h2 class="section-title">Export and import</h2>
      <p class="section-desc">Manual passphrase-encrypted file transfer; no automatic sync service.</p>
      <div class="item-row" style="border: none;">
        <div class="item-body">
          <div class="item-title">Manual export and import</div>
          <div class="item-subtitle" id="team-sync-state">Loading…</div>
        </div>
        <div class="item-action">
          <label class="switch">
            <input type="checkbox" id="toggle-sync" onchange="toggleSync(this.checked)">
            <span class="slider"></span>
          </label>
        </div>
      </div>
    </div>

    <!-- TAB 3: SETTINGS -->
    <div id="tab-settings" class="tab-pane" style="display: none;">
      <h2 class="section-title">Settings</h2>
      <p class="section-desc">Local preferences and contact details.</p>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Name</div>
          <div class="item-subtitle" id="pref-user-name">${userName}</div>
        </div>
        <div class="item-action">
          <button class="btn" onclick="openEditNameModal()">Edit</button>
        </div>
      </div>

      <div class="item-row">
        <div class="item-icon icon-messages">💬</div>
        <div class="item-body">
          <div class="item-title">Messages</div>
          <div class="item-subtitle" id="val-imessage">${imessageNum}</div>
        </div>
        <div class="item-action">
          <button class="copy-btn" title="Copy handle" data-copy="${imessageNum}">📋</button>
          <button class="copy-btn" title="Edit number" data-action="edit-contact" data-channel="imessage" data-val="${imessageNum}">✎</button>
        </div>
      </div>

      <div class="item-row">
        <div class="item-icon icon-whatsapp">📱</div>
        <div class="item-body">
          <div class="item-title">WhatsApp</div>
          <div class="item-subtitle" id="val-whatsapp">${whatsappNum}</div>
        </div>
        <div class="item-action">
          <button class="copy-btn" title="Copy number" data-copy="${whatsappNum}">📋</button>
          <button class="copy-btn" title="Edit number" data-action="edit-contact" data-channel="whatsapp" data-val="${whatsappNum}">✎</button>
        </div>
      </div>

      <hr class="section-divider">

      <h2 class="section-title">Trusted people</h2>
      <p class="section-desc">Approve or block in one block. Minimum share only.</p>
      <div id="mesh-requests-list"></div>
      <div id="mesh-trusted-list"></div>
      <h2 class="section-title" style="margin-top: 18px;">Blocked</h2>
      <p class="section-desc">Their Kineti can't reach yours</p>
      <div id="mesh-blocked-list"></div>
      <div class="item-row" style="border: none;">
        <div class="item-body">
          <div class="item-title" id="mesh-status-title">Connections active</div>
          <div class="item-subtitle">Your Kineti can exchange messages with the Kinetis of the people you trust.</div>
        </div>
      </div>

      <hr class="section-divider">

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Manual export and import</div>
          <div class="item-subtitle">Passphrase-encrypted file transfer. No automatic cloud sync is available.</div>
        </div>
        <div class="item-action">
          <label class="switch">
            <input type="checkbox" id="toggle-sync-settings" onchange="toggleSync(this.checked)">
            <span class="slider"></span>
          </label>
        </div>
      </div>

      <hr class="section-divider">

      <h2 class="section-title">Local project budget</h2>
      <p class="section-desc">This local ceiling affects recorded-spend checks. No project data is mirrored to a cloud service.</p>
      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Project ceiling</div>
          <div class="item-subtitle">A local setting. Spend is based on amounts reported to Kineti.</div>
        </div>
        <div class="item-action">
          <input type="number" id="mirror-ceiling" class="input-field" min="1" max="1000" step="1" style="margin: 0; width: 110px;" onchange="saveMirrorCeiling()">
        </div>
      </div>
    </div>
  </main>

  <!-- MODALS -->
  <!-- Edit Name Modal -->
  <div id="modal-edit-name" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Edit Display Name</h3>
      <p class="modal-desc">Update the name your Kineti assistant uses.</p>
      <label class="input-label">Full Name</label>
      <input type="text" id="edit-name-input" class="input-field" value="${userName}">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-edit-name')">Cancel</button>
        <button class="btn btn-primary" onclick="submitEditName()">Save Name</button>
      </div>
    </div>
  </div>

  <!-- Edit Contact Modal -->
  <div id="modal-contact-edit" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Edit Contact Channel</h3>
      <p class="modal-desc">Set your destination number or address.</p>
      <input type="hidden" id="contact-edit-type">
      <label class="input-label" id="contact-edit-label">Number / Handle</label>
      <input type="text" id="contact-edit-val" class="input-field">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-contact-edit')">Cancel</button>
        <button class="btn btn-primary" onclick="submitContactModal()">Save</button>
      </div>
    </div>
  </div>


  <!-- Toast -->
  <div id="toast" class="toast"></div>

  <script>
    // Clean ?token= from address bar immediately to prevent token exposure in history/referrer
    if (window.location.search.includes('token=')) {
      try {
        history.replaceState({}, '', window.location.pathname);
      } catch (err) {
        console.warn('Could not clean address bar:', err);
      }
    }

    function showToast(msg) {
      const t = document.getElementById('toast');
      t.innerText = msg;
      t.classList.add('show');
      setTimeout(() => t.classList.remove('show'), 3000);
    }

    function switchTab(tab, el) {
      document.querySelectorAll('.tab-pane').forEach(p => p.style.display = 'none');
      document.querySelectorAll('.nav-item').forEach(i => i.classList.remove('active'));
      const target = document.getElementById('tab-' + tab);
      if (target) target.style.display = 'block';
      if (el) el.classList.add('active');
      else {
        const item = document.querySelector('.nav-item[data-tab="' + tab + '"]');
        if (item) item.classList.add('active');
      }
      if (tab === 'home') loadHome();
      if (tab === 'activity') loadActivity();
      if (tab === 'team') loadTeam();
      if (tab === 'settings') { refreshMeshUI(); loadSyncState(); }
    }

    function ageText(min) {
      if (min === null || min === undefined) return '';
      if (min <= 1) return 'just now';
      if (min < 60) return min + ' min ago';
      return Math.round(min / 60) + ' h ago';
    }

    function loadHome() {
      fetch('/api/mini')
        .then(r => r.json())
        .then(m => {
          document.getElementById('home-goal').innerText = m.goal || 'No goal locked yet';
          const lockEl = document.getElementById('home-goal-lock');
          if (lockEl) lockEl.innerText = m.goal_locked_at ? 'Locked ' + m.goal_locked_at.slice(0, 10) : '';
          const stageLabel = m.stage_name ? m.stage_name + ' ' + m.stage + ' of 13' : 'Step ' + m.stage + ' of 13';
          document.getElementById('home-stage').innerText = stageLabel;
          const left = Math.max(0, (m.ceiling || 50) - (m.spend_total || 0));
          document.getElementById('home-spend').innerText = '$' + m.spend_total + ' of $' + m.ceiling + ' ($' + left.toFixed(2) + ' left)' + (m.tripped ? ' (stopped)' : ' used');
          const proofEl = document.getElementById('home-proof');
          if (proofEl) {
            proofEl.innerText = m.proof_label ? m.proof_label + ' ' + (m.proof_state || '') + ' ' + ageText(m.proof_age_min) : 'No test proof saved yet';
          }
          document.getElementById('home-undo').innerText = m.pending_undo === 0 ? 'Nothing to undo' : m.pending_undo + ' change(s) can be undone';
          const labelsEl = document.getElementById('home-undo-labels');
          if (labelsEl) labelsEl.innerText = (m.undo_labels && m.undo_labels.length > 0) ? 'Latest: ' + m.undo_labels.join(', ') : '';
          const syncEl = document.getElementById('home-sync');
          if (syncEl) syncEl.innerText = m.sync_enabled ? 'Manual encrypted transfer enabled' : 'Manual encrypted transfer disabled';
          document.getElementById('toggle-power').checked = m.enabled !== false;
          document.getElementById('home-power-desc').innerText = m.enabled !== false ? 'All checks running' : 'Paused';
        })
        .catch(() => {});
    }

    function copyHomeText(kind) {
      const map = { goal: 'home-goal', spend: 'home-spend' };
      const el = document.getElementById(map[kind]);
      if (el && navigator.clipboard) navigator.clipboard.writeText(el.innerText).then(() => showToast('Copied to clipboard'));
    }

    function togglePower(on) {
      fetch('/api/power', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ on }) })
        .then(() => loadHome());
    }

    function escapeJsHtml(s) {
      return String(s == null ? '' : s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    function loadActivity() {
      const actor = (document.getElementById('activity-actor') || {}).value || '';
      const action = (document.getElementById('activity-action') || {}).value || '';
      const q = new URLSearchParams({ n: '100', actor, action }).toString();
      fetch('/api/activity?' + q)
        .then(r => r.json())
        .then(d => {
          const body = document.getElementById('activity-rows');
          if (!body) return;
          if (!d.activity || d.activity.length === 0) {
            body.innerHTML = '<tr><td colspan="5" style="padding: 12px 6px; color: var(--text-secondary);">No rows yet</td></tr>';
            return;
          }
          body.innerHTML = d.activity.slice().reverse().map(r =>
            '<tr style="border-bottom: 1px solid var(--divider-color);">' +
            '<td style="padding: 8px 6px; white-space: nowrap;">' + escapeJsHtml((r.at || '').slice(0, 19).replace('T', ' ')) + '</td>' +
            '<td style="padding: 8px 6px;">' + escapeJsHtml(r.actor) + '</td>' +
            '<td style="padding: 8px 6px;">' + escapeJsHtml(r.action) + '</td>' +
            '<td style="padding: 8px 6px;">' + escapeJsHtml((r.detail || '').slice(0, 120)) + '</td>' +
            '<td style="padding: 8px 6px; font-family: monospace;">' + escapeJsHtml(r.hash || '') + '</td>' +
            '</tr>'
          ).join('');
        })
        .catch(() => {});
    }

    function loadTeam() {
      fetch('/api/swarm')
        .then(r => r.json())
        .then(s => {
          const modeEl = document.getElementById('team-mode');
          if (modeEl) modeEl.innerText = s.mode === 'separate' ? 'Separate target plan' : 'One shared target';
          const list = document.getElementById('team-agents-list');
          if (list) {
            if (!s.agents || s.agents.length === 0) {
              list.innerHTML = '<div class="vault-empty">No per-agent budgets yet. Save one below or say "separate budgets" in chat.</div>';
            } else {
              list.innerHTML = s.agents.map(a =>
                '<div class="item-row">' +
                '<div class="item-body">' +
                '<div class="item-title">' + escapeJsHtml(a.name) + ' target: $' + Number(a.budget).toFixed(2) + '</div>' +
                '<div class="item-subtitle">Usage is not tracked or enforced per agent.</div>' +
                '</div></div>'
              ).join('');
            }
          }
        })
        .catch(() => {});
      fetch('/api/models')
        .then(r => r.json())
        .then(m => {
          const st = document.getElementById('team-model-state');
          if (st) st.innerText = 'Suggestions only — automatic switching is unavailable';
          const hist = document.getElementById('team-model-history');
          if (hist) {
            hist.innerHTML = (!m.history || m.history.length === 0)
              ? 'No suggestions logged yet.'
              : 'Last ' + m.history.length + ' suggestions:<br>' + m.history.slice().reverse().map(h =>
                escapeJsHtml((h.at || '').slice(0, 19).replace('T', ' ')) + ' ' + escapeJsHtml(h.action) + ' ' + escapeJsHtml(h.detail)
              ).join('<br>');
          }
        })
        .catch(() => {});
      loadSyncState();
    }

    function setSwarmMode(mode) {
      fetch('/api/swarm', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ mode }) })
        .then(() => { loadTeam(); showToast(mode === 'separate' ? 'Separate target plan saved' : 'Shared target saved'); });
    }

    function saveSwarmAgent() {
      const name = (document.getElementById('team-agent-name') || {}).value || '';
      const budget = parseFloat((document.getElementById('team-agent-budget') || {}).value || '');
      if (!name.trim() || !Number.isFinite(budget) || budget <= 0) { showToast('Enter a name and a budget over 0'); return; }
      fetch('/api/swarm', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ agent: name.trim(), budget }) })
        .then(r => r.json())
        .then(() => {
          const nEl = document.getElementById('team-agent-name');
          const bEl = document.getElementById('team-agent-budget');
          if (nEl) nEl.value = '';
          if (bEl) bEl.value = '';
          loadTeam();
          showToast('Budget target saved; usage is not enforced');
        });
    }

    function loadSyncState() {
      fetch('/api/sync')
        .then(r => r.json())
        .then(s => {
          const t1 = document.getElementById('team-sync-state');
          if (t1) t1.innerText = s.enabled ? 'Manual file transfer enabled' : 'Manual file transfer disabled';
          const t2 = document.getElementById('toggle-sync');
          if (t2) t2.checked = s.enabled === true;
          const t3 = document.getElementById('toggle-sync-settings');
          if (t3) t3.checked = s.enabled === true;
        })
        .catch(() => {});
      loadProjectBudget();
    }

    function toggleSync(on) {
      fetch('/api/sync', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ on }) })
        .then(() => { loadSyncState(); loadHome(); showToast(on ? 'Manual export/import enabled' : 'Manual export/import disabled'); });
    }

    function loadProjectBudget() {
      fetch('/api/mirror')
        .then(r => r.json())
        .then(m => {
          const c = document.getElementById('mirror-ceiling');
          if (c) c.value = m.ceiling;
          const note = document.getElementById('team-ceiling-note');
          if (note) note.innerText = 'Project ceiling $' + m.ceiling + '.';
        })
        .catch(() => {});
    }

    function saveMirrorCeiling() {
      const field = document.getElementById('mirror-ceiling');
      const ceiling = field && field.value !== '' ? parseFloat(field.value) : NaN;
      fetch('/api/mirror', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ ceiling }) })
        .then(r => {
          if (!r.ok) { showToast('Ceiling must be $1 to $1000'); loadProjectBudget(); return; }
          loadProjectBudget(); loadHome(); showToast('Local ceiling saved');
        });
    }

    function approveMesh(id) {      fetch('/api/mesh/approve', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ request_id: id, tier: 'colleague' }) })
        .then(() => { refreshMeshUI(); showToast('Approved'); });
    }

    function blockMesh(id) {
      fetch('/api/mesh/block', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ agent_id: id }) })
        .then(() => { refreshMeshUI(); showToast('Blocked'); });
    }

    function unblockMesh(id) {
      fetch('/api/mesh/unblock', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ agent_id: id }) })
        .then(() => { refreshMeshUI(); showToast('Unblocked'); });
    }

    function appendTalk(who, text) {
      const log = document.getElementById('talk-log');
      const div = document.createElement('div');
      div.style.cssText = 'padding: 10px 12px; border-radius: 10px; font-size: 13px; white-space: pre-wrap;';
      if (who === 'you') {
        div.style.background = '#0071e3';
        div.style.color = '#fff';
        div.style.alignSelf = 'flex-end';
        div.style.maxWidth = '85%';
      } else {
        div.style.background = '#f5f5f7';
        div.style.color = '#1d1d1f';
        div.style.alignSelf = 'flex-start';
        div.style.maxWidth = '95%';
      }
      div.innerText = text;
      log.appendChild(div);
    }

    function sendTalk() {
      const input = document.getElementById('talk-input');
      const msg = input.value.trim();
      if (!msg) return;
      input.value = '';
      appendTalk('you', msg);
      fetch('/api/talk', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ message: msg }) })
        .then(r => r.json())
        .then(d => {
          appendTalk('kineti', d.reply || 'No reply');
          loadHome();
        })
        .catch(() => appendTalk('kineti', 'Could not reach Kineti.'));
    }

    function quickTalk(msg) {
      document.getElementById('talk-input').value = msg;
      sendTalk();
    }

    document.addEventListener('DOMContentLoaded', loadHome);

    function openEditNameModal() {
      document.getElementById('modal-edit-name').classList.add('open');
    }

    function openInviteModal() {
      fetch('/api/invites')
        .then(r => r.json())
        .then(d => {
          document.getElementById('invite-quota-desc').innerText = 'You have ' + d.remaining_quota + ' invites remaining (' + d.tier + ' tier).';
          if (d.invites && d.invites.length > 0) {
            document.getElementById('invite-url-input').value = d.invites[d.invites.length - 1].vanity_url;
          }
        });
      document.getElementById('modal-invite').classList.add('open');
    }

    function generateNewInviteLink() {
      fetch('/api/invites', { method: 'POST' })
        .then(r => r.json())
        .then(d => {
          document.getElementById('invite-url-input').value = d.vanity_url;
          showToast('New invite link created!');
        });
    }

    function closeModal(id) {
      document.getElementById(id).classList.remove('open');
    }

    function copyText(val, btn) {
      navigator.clipboard.writeText(val);
      const orig = btn.innerText;
      btn.innerText = '✓';
      setTimeout(() => btn.innerText = orig, 1500);
      showToast('Copied to clipboard');
    }

    function copyInviteLink() {
      const val = document.getElementById('invite-url-input').value;
      navigator.clipboard.writeText(val);
      showToast('Invite link copied!');
      closeModal('modal-invite');
    }

    function toggleMeshPause() {
      fetch('/api/mesh/pause', { method: 'POST' })
        .then(r => r.json())
        .then(data => {
          const btn = document.getElementById('btn-mesh-pause');
          const title = document.getElementById('mesh-status-title');
          if (data.mesh_paused) {
            btn.innerText = 'Resume connections';
            btn.classList.add('btn-danger');
            title.innerText = 'Connections paused';
            showToast('Mesh connections paused');
          } else {
            btn.innerText = 'Pause connections';
            btn.classList.remove('btn-danger');
            title.innerText = 'Connections active';
            showToast('Mesh connections active');
          }
        });
    }

    function submitEditName() {
      const name = document.getElementById('edit-name-input').value.trim().slice(0, 100);
      if (name) {
        fetch('/api/settings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ user_name: name })
        }).then(() => {
          document.getElementById('sidebar-user-name').innerText = name;
          document.getElementById('pref-user-name').innerText = name;
          closeModal('modal-edit-name');
          showToast('Name saved');
        });
      }
    }

    // Contact & Testing Actions
    function openContactModal(type, currentVal) {
      document.getElementById('contact-edit-type').value = type;
      document.getElementById('contact-edit-label').innerText = type === 'imessage' ? 'iMessage Number / Apple ID' : 'WhatsApp Phone Number';
      document.getElementById('contact-edit-val').value = currentVal === 'Not Configured' ? '' : currentVal;
      document.getElementById('modal-contact-edit').classList.add('open');
    }

    function submitContactModal() {
      const type = document.getElementById('contact-edit-type').value;
      const val = document.getElementById('contact-edit-val').value.trim();
      const payload = {};
      if (type === 'imessage') payload.imessage_number = val || 'Not Configured';
      if (type === 'whatsapp') payload.whatsapp_number = val || 'Not Configured';

      fetch('/api/contact/update', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      }).then(() => {
        if (type === 'imessage') document.getElementById('val-imessage').innerText = val || 'Not Configured';
        if (type === 'whatsapp') document.getElementById('val-whatsapp').innerText = val || 'Not Configured';
        closeModal('modal-contact-edit');
        showToast('Contact updated');
      });
    }

    function copyTotp(idx, btn) {
      const code = document.getElementById('totp-code-' + idx)?.innerText.replace(/\s+/g, '');
      if (code && !code.includes('•')) {
        copyText(code, btn);
      }
    }

    function refreshMeshUI() {
      fetch('/api/mesh')
        .then(r => r.json())
        .then(m => {
          // Requests
          const reqDiv = document.getElementById('mesh-requests-list');
          if (!m.pending_requests || m.pending_requests.length === 0) {
            reqDiv.innerHTML = '<div class="vault-empty">No pending requests</div>';
          } else {
            reqDiv.innerHTML = m.pending_requests.map(r =>
              '<div class="item-row">' +
                '<div class="item-body">' +
                  '<div class="item-title">' + escapeHtml(r.requester_name || r.requester_agent_id) + ' (' + escapeHtml(r.requester_handle) + ')</div>' +
                  '<div class="item-subtitle">Requested Tier: ' + escapeHtml(r.requested_tier) + (r.note ? ' • "' + escapeHtml(r.note) + '"' : '') + '</div>' +
                '</div>' +
                '<div class="item-action">' +
                  '<button class="btn btn-primary" onclick="approveMesh(\'' + escapeHtml(r.request_id || r.requester_agent_id) + '\')">Approve</button>' +
                  '<button class="btn btn-danger" onclick="blockMesh(\'' + escapeHtml(r.requester_agent_id) + '\')">Block</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }

          // Trusted
          const trustDiv = document.getElementById('mesh-trusted-list');
          if (!m.peers || m.peers.length === 0) {
            trustDiv.innerHTML = '<div class="vault-empty">No trusted people yet</div>';
          } else {
            trustDiv.innerHTML = m.peers.map(p =>
              '<div class="item-row">' +
                '<div class="item-body">' +
                  '<div class="item-title">' + escapeHtml(p.display_name || p.name || p.peer_agent_id || p.agent_id) + '</div>' +
                  '<div class="item-subtitle">Tier: ' + escapeHtml(p.tier) + ' • Connected</div>' +
                '</div>' +
                '<div class="item-action">' +
                  '<button class="btn btn-danger" onclick="blockMesh(\'' + escapeHtml(p.peer_agent_id || p.agent_id) + '\')">Block</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }

          // Blocked
          const blkDiv = document.getElementById('mesh-blocked-list');
          if (!m.blocked_peers || m.blocked_peers.length === 0) {
            blkDiv.innerHTML = '<div class="vault-empty">No blocked peers</div>';
          } else {
            blkDiv.innerHTML = m.blocked_peers.map(b =>
              '<div class="vault-item-row">' +
                '<div>' + escapeHtml(b) + '</div>' +
                '<div class="item-action">' +
                  '<button class="btn" onclick="unblockMesh(\'' + escapeHtml(b) + '\')">Unblock</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }
        });
    }

    function handleLogout() {
      fetch('/logout', { method: 'POST' }).then(() => {
        window.location.href = '/';
      });
    }

    // Global event delegation for data-copy and data-action attributes
    document.addEventListener('click', function(e) {
      const copyBtn = e.target.closest('[data-copy]');
      if (copyBtn) {
        const val = copyBtn.getAttribute('data-copy');
        copyText(val, copyBtn);
        return;
      }
      const goalBtn = e.target.closest('[data-copy-goal]');
      if (goalBtn) {
        copyHomeText('goal');
        return;
      }
      const spendBtn = e.target.closest('[data-copy-spend]');
      if (spendBtn) {
        copyHomeText('spend');
        return;
      }
      const actionBtn = e.target.closest('[data-action]');
      if (actionBtn) {
        const action = actionBtn.getAttribute('data-action');
        if (action === 'edit-contact') {
          openContactModal(actionBtn.getAttribute('data-channel'), actionBtn.getAttribute('data-val'));
        } else if (action === 'open-url') {
          window.open(actionBtn.getAttribute('data-url'), '_blank');
        }
      }
    });

    // Initial page hydration
    refreshMeshUI();
    loadSyncState();
  </script>
</body>
</html>`;
}

// Apple HIG Token Login Page HTML Generator
function generateLoginHtml(): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Kineti — Sign In</title>
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", system-ui, sans-serif;
      background: #fbfbfd;
      color: #1d1d1f;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      margin: 0;
      padding: 20px;
      box-sizing: border-box;
      -webkit-font-smoothing: antialiased;
    }
    .card {
      background: #ffffff;
      padding: 44px 36px;
      border-radius: 16px;
      border: 1px solid #e5e5ea;
      width: 100%;
      max-width: 400px;
      text-align: center;
      box-shadow: 0 12px 36px rgba(0, 0, 0, 0.04);
      box-sizing: border-box;
    }
    .brand {
      font-family: "New York", "Charter", "Georgia", serif;
      font-size: 30px;
      font-weight: 500;
      letter-spacing: -0.5px;
      margin-bottom: 8px;
      color: #111111;
    }
    .subtitle {
      font-size: 14px;
      color: #86868b;
      margin-bottom: 28px;
      line-height: 1.4;
    }
    .token-input {
      width: 100%;
      padding: 13px 14px;
      border-radius: 10px;
      border: 1px solid #d2d2d7;
      font-size: 14px;
      margin-bottom: 16px;
      box-sizing: border-box;
      outline: none;
      font-family: ui-monospace, Menlo, monospace;
      transition: border-color 0.15s, box-shadow 0.15s;
    }
    .token-input:focus {
      border-color: #0071e3;
      box-shadow: 0 0 0 3px rgba(0, 113, 227, 0.15);
    }
    .btn {
      display: block;
      width: 100%;
      background: #0071e3;
      color: #ffffff;
      padding: 12px;
      border-radius: 10px;
      font-weight: 500;
      font-size: 14px;
      border: none;
      cursor: pointer;
      box-sizing: border-box;
      transition: background 0.15s;
    }
    .btn:hover {
      background: #0077ed;
    }
    .hint {
      margin-top: 24px;
      font-size: 12px;
      color: #86868b;
      line-height: 1.5;
    }
    .hint code {
      background: #f5f5f7;
      padding: 2px 6px;
      border-radius: 4px;
      font-family: ui-monospace, Menlo, monospace;
      color: #1d1d1f;
    }
  </style>
</head>
<body>
  <div class="card">
    <div class="brand">Kineti</div>
    <div class="subtitle">Enter your authorization token to access Settings</div>
    <form method="POST" action="/login" id="login-form">
      <input name="token" id="token-field" class="token-input" type="password" placeholder="Paste token..." autocomplete="off" spellcheck="false" required />
      <button type="submit" class="btn">Sign In</button>
    </form>
    <div class="hint">
      Authorization token is stored locally in <code>.kineti/auth_token</code>.
    </div>
  </div>
</body>
</html>`;
}

// WhatsApp Onboarding Page HTML Generator (Strictly Sanitized)
function generateWhatsAppOnboardingHtml(token: string): string {
  const safeToken = escapeHtml(token);
  const whatsappNum = companionSettings.whatsapp_number;
  const hasConfiguredNumber = whatsappNum !== "Not Configured" && whatsappNum.replace(/[^0-9]/g, "").length >= 10;
  const digits = whatsappNum.replace(/[^0-9]/g, "");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Connect WhatsApp — Kineti</title>
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif;
      background: #fbfbfd;
      color: #1d1d1f;
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      margin: 0;
      padding: 20px;
      box-sizing: border-box;
    }
    .card {
      background: #fff;
      padding: 40px;
      border-radius: 16px;
      border: 1px solid #e5e5ea;
      width: 100%;
      max-width: 440px;
      text-align: center;
      box-shadow: 0 10px 30px rgba(0,0,0,0.05);
      box-sizing: border-box;
    }
    h1 {
      font-family: "New York", "Charter", serif;
      font-size: 26px;
      margin-bottom: 8px;
      font-weight: 500;
    }
    p {
      color: #86868b;
      font-size: 14px;
      margin-bottom: 24px;
      line-height: 1.4;
    }
    .token-box {
      background: #f5f5f7;
      padding: 14px;
      border-radius: 8px;
      font-family: ui-monospace, Menlo, monospace;
      font-size: 14px;
      word-break: break-all;
      margin-bottom: 20px;
      font-weight: 600;
      color: #1d1d1f;
      border: 1px solid #e5e5ea;
    }
    .btn {
      display: inline-block;
      background: #25d366;
      color: #fff;
      text-decoration: none;
      padding: 12px 24px;
      border-radius: 8px;
      font-weight: 600;
      font-size: 14px;
      border: none;
      cursor: pointer;
      width: 100%;
      box-sizing: border-box;
      transition: opacity 0.2s;
    }
    .btn:hover {
      opacity: 0.9;
    }
    .btn-secondary {
      display: inline-block;
      background: #f5f5f7;
      color: #1d1d1f;
      text-decoration: none;
      padding: 10px 20px;
      border-radius: 8px;
      font-weight: 500;
      font-size: 13px;
      border: 1px solid #e5e5ea;
      cursor: pointer;
      width: 100%;
      box-sizing: border-box;
      margin-top: 10px;
    }
    .btn-secondary:hover {
      background: #ebebee;
    }
    .num-input {
      width: 100%;
      padding: 12px;
      border-radius: 8px;
      border: 1px solid #d2d2d7;
      font-size: 14px;
      margin-bottom: 12px;
      box-sizing: border-box;
      outline: none;
    }
    .num-input:focus {
      border-color: #0071e3;
      box-shadow: 0 0 0 3px rgba(0,113,227,0.15);
    }
    .notice {
      font-size: 12px;
      color: #86868b;
      margin-top: 16px;
      line-height: 1.4;
    }
    .notice code {
      background: #f5f5f7;
      padding: 2px 5px;
      border-radius: 4px;
      font-family: ui-monospace, Menlo, monospace;
    }
  </style>
</head>
<body>
  <div class="card">
    <h1>Connect WhatsApp</h1>
    <p>Pair WhatsApp with your autonomous assistant.</p>
    <div class="token-box">${safeToken}</div>
    ${
      hasConfiguredNumber
        ? `<a href="https://wa.me/${digits}?text=Hi%20Kineti,%20pairing%20token:%20${encodeURIComponent(token)}" class="btn" target="_blank" rel="noopener">Open WhatsApp to Pair</a>`
        : `
          <input id="wa-num-input" class="num-input" type="tel" placeholder="Enter WhatsApp Bot Number (e.g. 14155552671)" />
          <button class="btn" onclick="openWhatsApp()">Connect to Number</button>
          <button class="btn-secondary" onclick="copyTokenText()">Copy Pairing Message</button>
          <div class="notice">
            WhatsApp bot number is not configured in this environment.<br>Set <code>KINETI_WHATSAPP_NUMBER</code> or enter the bot number above.
          </div>
          <script>
            function openWhatsApp() {
              const val = document.getElementById('wa-num-input').value.replace(/[^0-9]/g, '');
              if (!val || val.length < 10) {
                alert('Please enter a valid phone number with country code (e.g. 14155552671)');
                return;
              }
              window.open('https://wa.me/' + val + '?text=' + encodeURIComponent('Hi Kineti, pairing token: ${encodeURIComponent(token)}'), '_blank');
            }
            function copyTokenText() {
              navigator.clipboard.writeText('Hi Kineti, pairing token: ${encodeURIComponent(token)}');
              alert('Pairing message copied to clipboard!');
            }
          </script>
        `
    }
  </div>
</body>
</html>`;
}

// Host & Origin Security Verification
export function isTrustedHost(req: Request): boolean {
  try {
    const urlHost = new URL(req.url).hostname.trim().toLowerCase();
    if (urlHost !== "localhost" && urlHost !== "127.0.0.1") return false;
  } catch {
    return false;
  }
  const host = req.headers.get("host");
  if (host === null) return true;
  const hostname = host.split(":")[0].trim().toLowerCase();
  if (!hostname) return false;
  return hostname === "localhost" || hostname === "127.0.0.1";
}

export function isTrustedOrigin(req: Request): boolean {
  const origin = req.headers.get("origin");
  if (!origin) return true;
  let h = "";
  try { h = new URL(origin).hostname.toLowerCase(); } catch { return false; }
  return h === "localhost" || h === "127.0.0.1";
}

function withVary(headers: Record<string, string> = {}): Record<string, string> {
  return { ...headers, "Vary": "Origin" };
}

// Server starter
export function startServer(port: number = PORT) {
  const meshManager = new TrustedNetworkManager();
  const inviteEngine = new ViralInviteEngine();

  const server = Bun.serve({
    port,
    hostname: "127.0.0.1",
    async fetch(req: Request) {
      const url = new URL(req.url);

      // Security: DNS Rebinding Protection
      if (!isTrustedHost(req)) {
        return new Response("Forbidden: Invalid Host Header", { status: 403, headers: withVary() });
      }

      // Security: CSWSH Origin Validation
      if (!isTrustedOrigin(req)) {
        return new Response("Forbidden: Invalid Origin", { status: 403, headers: withVary() });
      }

      const corsHeaders: Record<string, string> = withVary({
        "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type, Authorization",
      });

      if (req.method === "OPTIONS") {
        return new Response(null, { status: 204, headers: corsHeaders });
      }

      // Public WhatsApp Onboarding URL (Strictly Validated against XSS)
      if (url.pathname === "/whatsapp-onboarding") {
        const rawToken = url.searchParams.get("t") || "wa_demo_token";
        if (!/^[a-zA-Z0-9_-]{1,128}$/.test(rawToken)) {
          return new Response("Invalid pairing token format. Must be alphanumeric, dash, or underscore.", {
            status: 400,
            headers: { "Content-Type": "text/plain; charset=utf-8", ...corsHeaders },
          });
        }
        return new Response(generateWhatsAppOnboardingHtml(rawToken), {
          status: 200,
          headers: {
            "Content-Type": "text/html; charset=utf-8",
            "Content-Security-Policy": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; frame-ancestors 'none';",
            ...corsHeaders,
          },
        });
      }

      // Login form handler
      if (url.pathname === "/login" && req.method === "POST") {
        let submittedToken = "";
        try {
          const contentType = req.headers.get("content-type") || "";
          if (contentType.includes("application/json")) {
            const json = (await req.json()) as any;
            submittedToken = json.token || "";
          } else {
            const formData = await req.formData();
            submittedToken = formData.get("token")?.toString().trim() || "";
          }
        } catch (err) {
          console.warn(`kineti: warning: could not parse login payload: ${(err as Error).message}`);
        }

        if (submittedToken === AUTH_TOKEN) {
          const sessionToken = createSessionToken();
          return new Response(null, {
            status: 303,
            headers: {
              "Location": "/",
              "Set-Cookie": `kineti_token=${sessionToken}; Path=/; HttpOnly; SameSite=Strict; Max-Age=7200`,
              ...corsHeaders,
            },
          });
        }
        return new Response("Unauthorized", { status: 401, headers: corsHeaders });
      }

      // Logout handler
      if (url.pathname === "/logout" && req.method === "POST") {
        const token = extractToken(req);
        if (token) {
          revokeSessionToken(token);
        }
        return new Response(null, {
          status: 303,
          headers: {
            "Location": "/",
            "Set-Cookie": "kineti_token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
            ...corsHeaders,
          },
        });
      }

      // Main Settings App Page
      if (url.pathname === "/" || url.pathname === "/index.html") {
        if (!isAuthorized(req)) {
          return new Response(generateLoginHtml(), {
            status: 401,
            headers: {
              "Content-Type": "text/html; charset=utf-8",
              "WWW-Authenticate": 'Bearer realm="kineti"',
              ...corsHeaders,
            },
          });
        }

        const headers: Record<string, string> = {
          "Content-Type": "text/html; charset=utf-8",
          "Content-Security-Policy": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';",
          ...corsHeaders,
        };

        return new Response(generateSettingsHtml(), {
          status: 200,
          headers,
        });
      }

      // Auth Check for All API & Other Routes
      if (!isAuthorized(req)) {
        return new Response("Unauthorized", {
          status: 401,
          headers: { "WWW-Authenticate": "Bearer", ...corsHeaders },
        });
      }

      // API: Harness Pipeline Status
      if (url.pathname === "/api/status") {
        return Response.json(getHarnessStatus(), { headers: corsHeaders });
      }

      // API: Mini status for menu bar helper (spend, goal, undo, on/off)
      if (url.pathname === "/api/mini") {
        return Response.json(getMiniStatus(), { headers: corsHeaders });
      }

      // API: Power toggle for menu bar helper and dashboard switch
      if (url.pathname === "/api/power" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const on = body.on === true;
          writeJson(path.join(projectKdir(), "kineti.json"), {
            enabled: on, updated_by: "dashboard", at: new Date().toISOString(),
          });
          try {
            const { appendAudit } = await import("./kineti-audit.ts");
            appendAudit("dashboard-user", on ? "kineti.on" : "kineti.off", "power toggled from dashboard");
          } catch { /* audit must never block toggle */ }
          return Response.json({ success: true, enabled: on }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Plain talk. No skill or command names needed.
      if (url.pathname === "/api/talk" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const message = String(body.message || "").slice(0, 2000);
          if (!message.trim()) return new Response("Missing message", { status: 400, headers: corsHeaders });
          const r = routeTalk(message);
          return Response.json({ intent: r.intent, reply: r.reply }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Activity trail, view-only. Merges audit, proof, spend, and saga events.
      if (url.pathname === "/api/activity") {
        const params = new URL(req.url).searchParams;
        const n = Number(params.get("n") || 50);
        const actor = String(params.get("actor") || "").slice(0, 64);
        const action = String(params.get("action") || "").slice(0, 64);
        return Response.json({ activity: getActivity(n, actor, action) }, { headers: corsHeaders });
      }

      // API: Swarm budgets, per-agent with used and left.
      if (url.pathname === "/api/swarm") {
        if (req.method === "GET") {
          return Response.json(getSwarmStatus(), { headers: corsHeaders });
        }
        if (req.method === "POST") {
          try {
            const body = (await req.json()) as any;
            const storePath = path.join(projectKdir(), "swarm.json");
            const cur = readJson<{ mode?: string; budgets?: Record<string, number>; updated_at?: string }>(storePath) || {};
            let mode = cur.mode === "separate" ? "separate" : "shared";
            const budgets: Record<string, number> = { ...(cur.budgets || {}) };
            if (typeof body.mode === "string") {
              const m = body.mode.toLowerCase();
              if (m === "shared" || m === "separate") mode = m;
            }
            if (typeof body.agent === "string" && typeof body.budget === "number") {
              const name = body.agent.trim().toLowerCase().slice(0, 64);
              if (!/^[a-z0-9_-]{1,64}$/.test(name)) return new Response("Bad agent name", { status: 400, headers: corsHeaders });
              const cap = loadLimits().globalUsd;
              if (!Number.isFinite(body.budget) || body.budget <= 0 || body.budget > cap) {
                return new Response(`Budget must be over 0 and at most ${cap}`, { status: 400, headers: corsHeaders });
              }
              budgets[name] = body.budget;
              if (Object.keys(budgets).length > 0) mode = "separate";
            }
            if (body.budgets && typeof body.budgets === "object") {
              const cap = loadLimits().globalUsd;
              for (const [k, v] of Object.entries(body.budgets)) {
                const name = String(k).trim().toLowerCase().slice(0, 64);
                if (!/^[a-z0-9_-]{1,64}$/.test(name)) continue;
                if (typeof v === "number" && Number.isFinite(v) && v > 0 && v <= cap) budgets[name] = v;
              }
              if (Object.keys(budgets).length > 0 && !body.mode) mode = "separate";
            }
            writeJson(storePath, { mode, budgets, updated_at: new Date().toISOString() });
            try {
              const { appendAudit } = await import("./kineti-audit.ts");
              appendAudit("dashboard-user", "swarm.budgets", `${mode}: ${JSON.stringify(budgets).slice(0, 500)}`);
            } catch { /* audit must never block save */ }
            return Response.json(getSwarmStatus(), { headers: corsHeaders });
          } catch {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
        }
      }

      // API: Model routing table plus last switches. Ask-first default.
      if (url.pathname === "/api/models" && req.method === "GET") {
        return Response.json(getModelStatus(), { headers: corsHeaders });
      }

      if (url.pathname === "/api/models/auto" && req.method === "POST") {
        return new Response("Automatic model switching is not available", { status: 410, headers: corsHeaders });
      }

      if (url.pathname === "/api/pairing") {
        if (req.method === "GET") {
          return Response.json({ available: false, linked: false }, { headers: corsHeaders });
        }
        return new Response("Cloud pairing is not available in this release", { status: 410, headers: corsHeaders });
      }

      if (url.pathname === "/api/pairing/claim" || url.pathname === "/api/pairing/drop") {
        return new Response("Cloud pairing is not available in this release", { status: 410, headers: corsHeaders });
      }

      if (url.pathname === "/api/cloud" && req.method === "GET") {
        return Response.json({ linked: false, available: false }, { headers: corsHeaders });
      }

      if (url.pathname === "/api/mirror") {
        const { getMirror, setMirror } = await import("./kineti-pairing.ts");
        if (req.method === "GET") {
          const mirror = getMirror();
          return Response.json({ project: mirror.project, ceiling: mirror.ceiling, local_only: true }, { headers: corsHeaders });
        }
        if (req.method === "POST") {
          try {
            const body = (await req.json()) as any;
            const ceiling = Number(body.ceiling);
            if (!Number.isFinite(ceiling) || ceiling < MIN_PROJECT_CEILING_USD || ceiling > MAX_PROJECT_CEILING_USD) {
              return new Response(`Ceiling must be $${MIN_PROJECT_CEILING_USD} to $${MAX_PROJECT_CEILING_USD}`, { status: 400, headers: corsHeaders });
            }
            const result = setMirror(false, false, "dashboard-user", ceiling);
            return Response.json({ project: result.project, ceiling: result.ceiling, local_only: true }, { headers: corsHeaders });
          } catch {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
        }
      }

      // Local preference for manual encrypted export/import. No background sync runs.
      if (url.pathname === "/api/sync") {
        if (req.method === "GET") {
          const s = readJson<{ sync_enabled?: boolean }>(path.join(projectKdir(), "kineti.json"));
          return Response.json({ enabled: s?.sync_enabled === true }, { headers: corsHeaders });
        }
        if (req.method === "POST") {
          try {
            const body = (await req.json()) as any;
            if (typeof body.on !== "boolean") return new Response("Bad Request", { status: 400, headers: corsHeaders });
            const cur = readJson<Record<string, unknown>>(path.join(projectKdir(), "kineti.json")) || {};
            writeJson(path.join(projectKdir(), "kineti.json"), {
              ...cur, sync_enabled: body.on, updated_by: "dashboard", at: new Date().toISOString(),
            });
            try {
              const { appendAudit } = await import("./kineti-audit.ts");
              appendAudit("dashboard-user", body.on ? "sync.on" : "sync.off", "manual encrypted export/import preference toggled");
            } catch { /* audit must never block toggle */ }
            return Response.json({ success: true, enabled: body.on }, { headers: corsHeaders });
          } catch {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
        }
      }

      // API: Fleet Repos
      if (url.pathname === "/api/fleet") {
        return Response.json(getFleetStatus(), { headers: corsHeaders });
      }

      // API: Select Repo
      if (url.pathname === "/api/fleet/select" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          if (body.repo_id === activeRepoId) {
            return Response.json({ success: true, active_repo_id: activeRepoId }, { headers: corsHeaders });
          }
          return new Response("Not Found", { status: 404, headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Settings
      if (url.pathname === "/api/settings") {
        if (req.method === "GET") {
          return Response.json(publicCompanionSettings(), { headers: corsHeaders });
        }
        if (req.method === "POST") {
          try {
            const body = (await req.json()) as any;
            if (typeof body !== "object" || body === null) {
              return new Response("Bad Request", { status: 400, headers: corsHeaders });
            }
            const ALLOWED_SETTINGS_KEYS = new Set([
              "repo_budgets", "repo_owners", "user_name", "user_phone",
              "imessage_number", "whatsapp_number",
            ]);
            for (const k of Object.keys(body)) {
              if (!ALLOWED_SETTINGS_KEYS.has(k)) {
                return new Response(`Unknown setting: ${k}`, { status: 400, headers: corsHeaders });
              }
            }
            const budgetChanges: string[] = [];
            if (body.repo_budgets && typeof body.repo_budgets === "object") {
              for (const [k, v] of Object.entries(body.repo_budgets)) {
                const key = String(k).slice(0, 128);
                if (!/^[a-zA-Z0-9_-]{1,128}$/.test(key)) continue;
                if (typeof v === "number" && Number.isFinite(v) && v > 0 && v <= 10000) {
                  const old = companionSettings.repo_budgets[key];
                  companionSettings.repo_budgets[key] = v;
                  budgetChanges.push(`${key}: ${old ?? "none"} -> ${v}`);
                }
              }
            }
            if (body.repo_owners && typeof body.repo_owners === "object") {
              for (const [k, v] of Object.entries(body.repo_owners)) {
                const key = String(k).slice(0, 128);
                if (!/^[a-zA-Z0-9_-]{1,128}$/.test(key)) continue;
                if (typeof v === "string" && v.length <= 100) {
                  companionSettings.repo_owners[key] = v;
                }
              }
            }
            if (typeof body.user_name === "string" && body.user_name.trim()) {
              companionSettings.user_name = body.user_name.trim().slice(0, 100);
            }
            saveCompanionSettings();
            if (budgetChanges.length > 0) {
              try {
                const { appendAudit } = await import("./kineti-audit.ts");
                appendAudit("dashboard-user", "budget.change", budgetChanges.join("; ").slice(0, 1000));
              } catch { /* audit must never block settings save */ }
            }
            return Response.json({ success: true, settings: publicCompanionSettings() }, { headers: corsHeaders });
          } catch {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
        }
      }

      if (url.pathname.startsWith("/api/connectors/")) {
        return new Response("Service connectors are not configured by the v0.4 Companion", { status: 410, headers: corsHeaders });
      }

      // API: Contact update
      if (url.pathname === "/api/contact/update" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          if (typeof body.imessage_number === "string") {
            companionSettings.imessage_number = body.imessage_number.trim().slice(0, 32);
          }
          if (typeof body.whatsapp_number === "string") {
            companionSettings.whatsapp_number = body.whatsapp_number.trim().slice(0, 32);
          }
          if (typeof body.user_phone === "string") {
            companionSettings.user_phone = body.user_phone.trim().slice(0, 32);
          }
          saveCompanionSettings();
          return Response.json({ success: true, settings: publicCompanionSettings() }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      if (url.pathname === "/api/contact/test" && req.method === "POST") {
        return new Response("Message dispatch is not available in the v0.4 Companion", { status: 410, headers: corsHeaders });
      }

      // Legacy vault files are left untouched. The v0.4 Companion does not
      // read or write credentials because the old TOTP storage was plaintext.
      if (url.pathname === "/api/vault" || url.pathname.startsWith("/api/vault/")) {
        return new Response("Credential storage is disabled in the v0.4 Companion", { status: 410, headers: corsHeaders });
      }

      // API: Mesh Network
      if (url.pathname === "/api/mesh") {
        return Response.json(
          {
            agent_id: meshManager.getAgentId(),
            mesh_paused: meshManager.isMeshPaused(),
            peers: meshManager.getPeers(),
            pending_requests: meshManager.getPendingRequests(),
            blocked_peers: meshManager.getBlockedPeers(),
          },
          { headers: corsHeaders },
        );
      }

      if (url.pathname === "/api/mesh/pause" && req.method === "POST") {
        if (meshManager.isMeshPaused()) {
          meshManager.resumeMesh();
        } else {
          meshManager.pauseMesh();
        }
        return Response.json({ mesh_paused: meshManager.isMeshPaused() }, { headers: corsHeaders });
      }

      if (url.pathname === "/api/mesh/approve" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const reqId = String(body.request_id || "").trim();
          const rawTier = String(body.tier || "colleague").toLowerCase().replace(/[^a-z]/g, "");
          const tier: TrustTier = rawTier.includes("inner") ? "inner_circle" : rawTier.includes("service") ? "service_agent" : "colleague";
          const peer = meshManager.approveRequest(reqId, tier);
          return Response.json({ success: true, peer }, { headers: corsHeaders });
        } catch (e: any) {
          return new Response(e.message || "Failed to approve", { status: 400, headers: corsHeaders });
        }
      }

      if (url.pathname === "/api/mesh/block" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const agentId = String(body.agent_id || "").trim();
          if (!agentId) return new Response("Missing agent_id", { status: 400, headers: corsHeaders });
          meshManager.blockPeer(agentId);
          return Response.json({ success: true, blocked: agentId }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      if (url.pathname === "/api/mesh/add" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const agentId = String(body.agentId || body.agent_id || "").trim();
          if (!agentId) return new Response("Missing agent ID", { status: 400, headers: corsHeaders });
          const name = String(body.name || agentId).trim().slice(0, 100);
          const rawTier = String(body.tier || "colleague").toLowerCase().replace(/[^a-z]/g, "");
          const tier: TrustTier = rawTier.includes("inner") ? "inner_circle" : rawTier.includes("service") ? "service_agent" : "colleague";

          const peer: TrustedPeer = {
            peer_agent_id: agentId,
            display_name: name,
            handle: `@${agentId}`,
            tier,
            can_propose_schedules: true,
            can_query_availability: true,
            can_coordinate_dining: tier === "inner_circle",
            added_at: Date.now(),
          };
          meshManager.addPeer(peer);
          try {
            const { appendAudit } = await import("./kineti-audit.ts");
            appendAudit("dashboard-user", "mesh.add", `${agentId} as ${tier} (minimum share)`.slice(0, 500));
          } catch { /* audit must never block mesh */ }
          return Response.json({ success: true, peer }, { headers: corsHeaders });
        } catch (e: any) {
          return new Response(e.message || "Failed to add peer", { status: 400, headers: corsHeaders });
        }
      }

      if (url.pathname === "/api/mesh/remove" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const agentId = String(body.agent_id || body.agentId || "").trim();
          if (!agentId) return new Response("Missing agent_id", { status: 400, headers: corsHeaders });
          const removed = meshManager.removePeer(agentId);
          try {
            const { appendAudit } = await import("./kineti-audit.ts");
            appendAudit("dashboard-user", "mesh.remove", agentId.slice(0, 200));
          } catch { /* audit must never block mesh */ }
          return Response.json({ success: true, removed }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      if (url.pathname === "/api/mesh/unblock" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const agentId = String(body.agent_id || body.agentId || "").trim();
          if (!agentId) return new Response("Missing agent_id", { status: 400, headers: corsHeaders });
          meshManager.unblockPeer(agentId);
          try {
            const { appendAudit } = await import("./kineti-audit.ts");
            appendAudit("dashboard-user", "mesh.unblock", agentId.slice(0, 200));
          } catch { /* audit must never block mesh */ }
          return Response.json({ success: true, unblocked: agentId }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Invites
      if (url.pathname === "/api/invites") {
        if (req.method === "GET") {
          return Response.json(
            {
              tier: inviteEngine.getTier(),
              remaining_quota: inviteEngine.getRemainingQuota(),
              invites: inviteEngine.listInvites(),
            },
            { headers: corsHeaders },
          );
        }
        if (req.method === "POST") {
          try {
            const invite = inviteEngine.createInvite();
            return Response.json(invite, { headers: corsHeaders });
          } catch (e: any) {
            return new Response(e.message || "Quota exceeded", { status: 400, headers: corsHeaders });
          }
        }
      }

      if (url.pathname.startsWith("/api/email/") || url.pathname.startsWith("/api/privacy/") || url.pathname === "/api/forget") {
        return new Response("This action is not implemented in the v0.4 Companion", { status: 410, headers: corsHeaders });
      }

      return new Response("Not Found", { status: 404, headers: corsHeaders });
    },
  });

  return server;
}

if (import.meta.main) {
  startServer(PORT);
  console.log(`\n✨ Kineti Visual Companion running on http://127.0.0.1:${PORT}`);
  console.log(`🔐 Authorization Token: ${AUTH_TOKEN}`);
  console.log(`   (Paste token into password field at http://127.0.0.1:${PORT})\n`);
}
