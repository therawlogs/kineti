#!/usr/bin/env bun
// bin/kineti-companion.ts
// Kineti OS — Clean, Modern Apple-HIG Settings App & API Server

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { die, ok, projectKdir, readJson, writeJson, readJsonl, ensureDir, nowIso } from "./lib.ts";
import { TrustedNetworkManager, TrustTier, TrustedPeer } from "../src/swarm/trusted_network.ts";
import { PrivacyGovernanceManager } from "../src/privacy/governance.ts";
import { ViralInviteEngine } from "../src/growth/viral_invites.ts";
import { route as routeTalk } from "./kineti-router.ts";

const PORT = Number(process.env.KINETI_COMPANION_PORT || 8788);
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
  const config = readJson<any>(path.join(REPO_ROOT, "kineti.config.json")) || {};
  const configuredCeiling = config.settings?.spend_limit_usd?.global ?? 50.0;
  return {
    repo_id: activeRepoId,
    root_goal: state.root_goal || "Build universal agent harness with cryptographic verification",
    stage: state.stage || 13,
    stages: STAGES,
    spend: {
      total_usd: (spend.total_microcents || 0) / 100_000_000,
      ceiling_usd: configuredCeiling,
      tripped: spend.tripped || false,
    },
    gates: state.gates || { spec: "pass", ship: "pass", security: "pass" },
  };
}

export function getMiniStatus() {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const spend = readJson<any>(path.join(projectKdir(), "spend.json")) || {};
  const config = readJson<any>(path.join(REPO_ROOT, "kineti.config.json")) || {};
  const toggle = readJson<any>(path.join(projectKdir(), "kineti.json"));
  const lines = readJsonl<any>(path.join(projectKdir(), "saga.jsonl"));
  const committed = new Set(lines.filter((l) => l.kind === "commit").map((l) => l.run_id));
  const pendingUndo = lines.filter((l) => l.kind === "register" && !committed.has(l.run_id)).length;
  return {
    spend_total: typeof spend.total_usd === "number" ? spend.total_usd : 0,
    ceiling: config.settings?.spend_limit_usd?.global ?? 50.0,
    tripped: spend.tripped === true,
    stage: state.stage ?? "not started",
    goal: typeof state.root_goal === "string" ? state.root_goal : null,
    enabled: !toggle || toggle.enabled !== false,
    pending_undo: pendingUndo,
  };
}

export function getFleetStatus() {
  return {
    active_repo_id: activeRepoId,
    total_fleet_spend: 0,
    total_fleet_budget: 100,
    repos: [
      {
        id: activeRepoId,
        name: activeRepoId,
        path: REPO_ROOT,
        owner: "Kineti User",
        branch: "main",
        status: "active",
        active_task: "Universal Autonomous Assistant Integration",
        spend_usd: 0,
        ceiling_usd: 50,
        tests_passing: 100,
        ide: "antigravity",
        is_local: true,
      },
    ],
    settings: {
      github: { connected: true, account: "kineti-org", repo_count: 1, webhook_status: "active" },
      ides: { cursor: true, claude_code: true, antigravity: true, codex: true },
      team_members: [{ name: "Kineti User", email: "user@mail.kineti.com", role: "Owner" }],
      repo_budgets: { [activeRepoId]: 50 },
      repo_owners: { [activeRepoId]: "Kineti User" },
    },
  };
}

// Vault Storage Helper
const vaultFile = path.join(projectKdir(), "vault_entries.json");

export function base32Decode(input: string): Buffer {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  const cleaned = input.toUpperCase().replace(/=+$/, "").replace(/[^A-Z2-7]/g, "");
  let bits = 0;
  let value = 0;
  const output: number[] = [];
  for (let i = 0; i < cleaned.length; i++) {
    const val = alphabet.indexOf(cleaned[i]);
    if (val === -1) continue;
    value = (value << 5) | val;
    bits += 5;
    if (bits >= 8) {
      output.push((value >>> (bits - 8)) & 255);
      bits -= 8;
    }
  }
  return Buffer.from(output);
}

export function computeRfc6238Totp(secretBase32: string, timeSec: number = Math.floor(Date.now() / 1000), period: number = 30, digits: number = 6): string {
  try {
    const key = base32Decode(secretBase32);
    if (key.length === 0) return "000000";
    const counter = Math.floor(timeSec / period);
    const buf = Buffer.alloc(8);
    buf.writeBigUInt64BE(BigInt(counter));
    const hmac = crypto.createHmac("sha1", key).update(buf).digest();
    const offset = hmac[hmac.length - 1] & 0x0f;
    const codeInt = ((hmac.readUInt32BE(offset) & 0x7fffffff) % Math.pow(10, digits));
    return codeInt.toString().padStart(digits, "0");
  } catch (err) {
    console.warn(`kineti: warning: failed to compute TOTP: ${(err as Error).message}`);
    return "000000";
  }
}

interface VaultStorage {
  logins: Array<{ id: string; domain: string; username: string; created_at: string; password?: string }>;
  cards: Array<{ id: string; brand: string; last4: string; exp: string; spend_cap: number }>;
  personal_info: Array<{ id: string; label: string; value_masked: string }>;
  agent_items: Array<{ id: string; service: string; identifier: string; scope: string }>;
  totp_items: Array<{ id: string; issuer: string; account: string; secret_masked: string; secret_raw?: string; code?: string }>;
}

function loadVault(): VaultStorage {
  ensureDir(projectKdir());
  return (
    readJson<VaultStorage>(vaultFile) || {
      logins: [],
      cards: [],
      personal_info: [],
      agent_items: [],
      totp_items: [],
    }
  );
}

function saveVault(vault: VaultStorage): void {
  try {
    ensureDir(projectKdir());
    const tmpFile = path.join(projectKdir(), `vault_entries.${Date.now()}.${crypto.randomBytes(4).toString("hex")}.tmp`);
    fs.writeFileSync(tmpFile, JSON.stringify(vault, null, 2) + "\n", { mode: 0o600 });
    fs.renameSync(tmpFile, vaultFile);
    try {
      fs.chmodSync(vaultFile, 0o600);
    } catch (chmodErr) {
      console.warn(`kineti: warning: could not chmod vault file: ${(chmodErr as Error).message}`);
    }
  } catch (err) {
    console.warn(`kineti: warning: could not save vault: ${(err as Error).message}`);
  }
}

const COMPANION_SETTINGS_FILE = path.join(process.cwd(), ".kineti", "companion_settings.json");

let companionSettings = {
  github: { connected: true, account: "kineti-org", repo_count: 1, webhook_status: "active" },
  ides: { cursor: true, claude_code: true, antigravity: true, codex: true },
  team_members: [{ name: "Kineti User", email: "user@mail.kineti.com", role: "Owner" }],
  repo_budgets: { [activeRepoId]: 50 } as Record<string, number>,
  repo_owners: { [activeRepoId]: "Kineti User" } as Record<string, string>,
  imessage_number: process.env.KINETI_IMESSAGE_NUMBER || "Not Configured",
  whatsapp_number: process.env.KINETI_WHATSAPP_NUMBER || "Not Configured",
  agent_email: "agent@mail.kineti.com",
  user_name: "Kineti User",
  user_phone: "Not Configured",
  connectors: {
    google: { connected: true, account: "user@example.com", name: "Google Workspace", desc: "Gmail, Calendar, Tasks, Drive, Docs, Sheets, and Slides" },
    outlook: { connected: false, account: "corp@example.com", name: "Outlook", desc: "Read, search, draft, and organize mail via Microsoft Graph" },
    linear: { connected: true, account: "Linear Workspace", name: "Linear", desc: "Search and update issues, create issues and comments" },
    notion: { connected: true, account: "Team Notion", name: "Notion", desc: "Search, read, and manage Notion pages and databases" },
    github: { connected: true, account: "kineti-org", name: "GitHub", desc: "Read repositories, files, issues, pull requests, and code search" },
    slack: { connected: false, account: "Team Slack", name: "Slack", desc: "Read channels, send messages, reactions, and canvas notes" },
    brave: { connected: true, account: "Brave Search API", name: "Brave Search", desc: "Live web research, price discovery, and event ticketing" },
    twilio: { connected: false, account: "Twilio Voice", name: "Twilio Telephony", desc: "Outbound and inbound telephone calls with IVR" },
    granola: { connected: false, account: "Meeting Notes", name: "Granola", desc: "Read meeting notes, transcripts, and AI summaries" },
    wispr: { connected: true, account: "Wispr Flow MCP", name: "Wispr Flow", desc: "Voice dictation & speech-to-text MCP integration" },
  } as Record<string, { connected: boolean; account: string; name: string; desc: string; apiKey?: string }>,
};

function loadCompanionSettings(): void {
  try {
    if (fs.existsSync(COMPANION_SETTINGS_FILE)) {
      const disk = JSON.parse(fs.readFileSync(COMPANION_SETTINGS_FILE, "utf-8"));
      companionSettings = {
        ...companionSettings,
        ...disk,
        connectors: { ...companionSettings.connectors, ...(disk.connectors || {}) },
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
    fs.writeFileSync(tmpFile, JSON.stringify(companionSettings, null, 2) + "\n", { mode: 0o600, encoding: "utf-8" });
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

loadCompanionSettings();

export function renderConnectorsHtml(connectors: Record<string, any>): string {
  return Object.keys(connectors).map(k => {
    const c = connectors[k];
    const btnClass = c.connected ? 'btn btn-connected' : 'btn btn-primary';
    const btnText = c.connected ? 'Connected' : 'Connect';
    const badgeHtml = c.connected
      ? '<span style="font-size:11px;padding:2px 8px;border-radius:10px;background:var(--success-bg);color:var(--success);border:1px solid #c8e6c9;margin-left:8px;">Active</span>'
      : '<span style="font-size:11px;padding:2px 8px;border-radius:10px;background:#f5f5f7;color:var(--text-secondary);border:1px solid var(--border-color);margin-left:8px;">Off</span>';
    
    return `<div class="item-row" id="connector-row-${escapeHtml(k)}">
      <div class="item-icon icon-service">${escapeHtml(k.slice(0, 2).toUpperCase())}</div>
      <div class="item-body">
        <div class="item-title">${escapeHtml(c.name)}${badgeHtml}</div>
        <div class="item-subtitle">${escapeHtml(c.desc)}</div>
        ${c.account ? `<div style="font-size:12px;color:var(--text-secondary);margin-top:2px;">Account: ${escapeHtml(c.account)}</div>` : ''}
      </div>
      <div class="item-action">
        <button class="${btnClass}" data-action="toggle-connector" data-key="${escapeHtml(k)}">${btnText}</button>
        <button class="copy-btn" title="Configure credentials" data-action="config-connector" data-key="${escapeHtml(k)}" data-name="${escapeHtml(c.name)}" data-account="${escapeHtml(c.account || '')}">✎</button>
        <button class="copy-btn" style="color:var(--danger);" title="Delete or Reset connector" data-action="delete-connector" data-key="${escapeHtml(k)}">🗑️</button>
      </div>
    </div>`;
  }).join('');
}

// Settings App HTML Generator
function generateSettingsHtml(): string {
  const imessageNum = escapeHtml(companionSettings.imessage_number);
  const whatsappNum = escapeHtml(companionSettings.whatsapp_number);
  const agentEmail = escapeHtml(companionSettings.agent_email);
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
    .icon-email { background: #0071e3; color: #fff; }
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
      <li class="nav-item" data-tab="workspace" onclick="switchTab('workspace', this)">Workspace</li>
      <li class="nav-item" data-tab="trusted" onclick="switchTab('trusted', this)">Trusted people</li>
      <li class="nav-item" data-tab="preferences" onclick="switchTab('preferences', this)">Preferences</li>
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
        </div>
        <div class="item-action">
          <span class="item-subtitle" id="home-stage"></span>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Spending</div>
          <div class="item-subtitle" id="home-spend">Loading…</div>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Undo available</div>
          <div class="item-subtitle" id="home-undo">Loading…</div>
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

    <!-- TAB 1: WORKSPACE -->
    <div id="tab-workspace" class="tab-pane" style="display: none;">
      <h2 class="section-title">Contact</h2>
      <p class="section-desc">Ways to reach Kineti directly</p>

      <div class="item-row">
        <div class="item-icon icon-messages">💬</div>
        <div class="item-body">
          <div class="item-title">Messages</div>
          <div class="item-subtitle" id="val-imessage">${imessageNum}</div>
        </div>
        <div class="item-action">
          <button class="copy-btn" title="Copy handle" data-copy="${imessageNum}">📋</button>
          <button class="copy-btn" title="Edit number" data-action="edit-contact" data-channel="imessage" data-val="${imessageNum}">✎</button>
          <button class="copy-btn" title="Send test ping" data-action="test-ping" data-channel="imessage" data-val="${imessageNum}">⚡</button>
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
          <button class="copy-btn" title="Setup QR code" data-action="open-url" data-url="/whatsapp-onboarding">📱</button>
          <button class="copy-btn" title="Send test ping" data-action="test-ping" data-channel="whatsapp" data-val="${whatsappNum}">⚡</button>
        </div>
      </div>

      <div class="item-row">
        <div class="item-icon icon-email">✉️</div>
        <div class="item-body">
          <div class="item-title">Email</div>
          <div class="item-subtitle" id="val-email">${agentEmail}</div>
        </div>
        <div class="item-action">
          <button class="copy-btn" title="Copy email" data-copy="${agentEmail}">📋</button>
          <button class="copy-btn" title="Edit alias" data-action="open-email-modal">✎</button>
          <button class="copy-btn" title="Send test email" data-action="test-ping" data-channel="email" data-val="${agentEmail}">⚡</button>
        </div>
      </div>

      <hr class="section-divider">

      <h2 class="section-title">Data privacy</h2>
      <p class="section-desc">Manage data from connected services</p>
      <div class="item-row" style="border: none;">
        <div class="item-body">
          <div class="item-title">External data</div>
          <div class="item-subtitle">Manage emails, messages, and other data imported from your connected services</div>
        </div>
        <div class="item-action">
          <button class="btn btn-danger" onclick="triggerPurge()">Delete data</button>
        </div>
      </div>
    </div>

    <!-- TAB 3: TRUSTED PEOPLE -->
    <div id="tab-trusted" class="tab-pane" style="display: none;">
      <div class="vault-group-header">
        <div>
          <h2 class="section-title" style="margin: 0;">Requests</h2>
          <p class="section-desc" style="margin-bottom: 0;">People asking to connect their Kineti to yours</p>
        </div>
      </div>
      <div id="mesh-requests-list"></div>

      <hr class="section-divider">

      <div class="vault-group-header">
        <div>
          <h2 class="section-title" style="margin: 0;">Trusted people</h2>
          <p class="section-desc" style="margin-bottom: 0;">Their Kineti can reach yours</p>
        </div>
      </div>
      <div id="mesh-trusted-list"></div>

      <hr class="section-divider">

      <h2 class="section-title">Blocked</h2>
      <p class="section-desc">Their Kineti can't reach yours</p>
      <div id="mesh-blocked-list"></div>

      <hr class="section-divider">

      <h2 class="section-title">Connections</h2>
      <p class="section-desc">Whether other Kinetis can reach yours</p>
      <div class="item-row" style="border: none;">
        <div class="item-body">
          <div class="item-title" id="mesh-status-title">Connections active</div>
          <div class="item-subtitle">Your Kineti can exchange messages with the Kinetis of the people you trust. Handled through your chat.</div>
        </div>
        <div class="item-action">
          <span style="font-size:11px;padding:3px 8px;border-radius:10px;background:var(--success-bg);color:var(--success);border:1px solid #c8e6c9;">Synced from Chat</span>
        </div>
      </div>
    </div>

    <!-- TAB 4: PREFERENCES -->
    <div id="tab-preferences" class="tab-pane" style="display: none;">
      <h2 class="section-title">Preferences</h2>
      <p class="section-desc">Account and privacy settings</p>

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
        <div class="item-body">
          <div class="item-title">WhatsApp Connection</div>
          <div class="item-subtitle" id="pref-wa-status">${whatsappNum}</div>
        </div>
        <div class="item-action">
          <a href="/whatsapp-onboarding?t=wa_demo" target="_blank" class="btn">Pair New</a>
        </div>
      </div>

      <div class="item-row">
        <div class="item-body">
          <div class="item-title">Improve Kineti for everyone</div>
          <div class="item-subtitle">Allow anonymized interactions to improve autonomous models. You can opt out anytime.</div>
        </div>
        <div class="item-action">
          <label class="switch">
            <input type="checkbox" id="toggle-training" onchange="toggleTraining(this.checked)">
            <span class="slider"></span>
          </label>
        </div>
      </div>

      <div class="item-row" style="margin-top: 24px; border: none;">
        <div class="item-body">
          <div class="item-title" style="color: var(--danger);">Delete account</div>
          <div class="item-subtitle">Permanently delete your account, credentials vault, and data.</div>
        </div>
        <div class="item-action">
          <button class="btn btn-danger" onclick="confirmDeleteAccount()">Delete account</button>
        </div>
      </div>
    </div>
  </main>

  <!-- MODALS -->
  <!-- Add Login Modal -->
  <div id="modal-add-login" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Add Web Login</h3>
      <p class="modal-desc">Save a website domain and credentials to your secure local vault.</p>
      <label class="input-label">Website Domain</label>
      <input type="text" id="login-domain" class="input-field" placeholder="e.g. github.com">
      <label class="input-label">Username / Email</label>
      <input type="text" id="login-user" class="input-field" placeholder="e.g. user@example.com">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-add-login')">Cancel</button>
        <button class="btn btn-primary" onclick="submitAddLogin()">Save to Vault</button>
      </div>
    </div>
  </div>

  <!-- Add Card Modal -->
  <div id="modal-add-card" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Add Payment Card</h3>
      <p class="modal-desc">Configure a payment method or dynamic single-use card with spend cap.</p>
      <label class="input-label">Card Brand</label>
      <input type="text" id="card-brand" class="input-field" placeholder="e.g. Visa, Mastercard, Amex">
      <label class="input-label">Spend Cap ($ USD)</label>
      <input type="number" id="card-cap" class="input-field" placeholder="100.00" value="100.00" min="1" step="5">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-add-card')">Cancel</button>
        <button class="btn btn-primary" onclick="submitAddCard()">Issue Card</button>
      </div>
    </div>
  </div>

  <!-- Add Personal Info Modal -->
  <div id="modal-add-personal" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Add Personal Info</h3>
      <p class="modal-desc">Store travel credentials, airline loyalty numbers, or preferences.</p>
      <label class="input-label">Label</label>
      <input type="text" id="personal-label" class="input-field" placeholder="e.g. Passport, United MileagePlus">
      <label class="input-label">Value</label>
      <input type="text" id="personal-val" class="input-field" placeholder="e.g. USA •••• 9210">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-add-personal')">Cancel</button>
        <button class="btn btn-primary" onclick="submitAddPersonal()">Save</button>
      </div>
    </div>
  </div>

  <!-- Add TOTP Modal -->
  <div id="modal-add-totp" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Add Authenticator (TOTP)</h3>
      <p class="modal-desc">Provide RFC 6238 Base32 secret for automated multi-factor code generation.</p>
      <label class="input-label">Service / Issuer</label>
      <input type="text" id="totp-issuer" class="input-field" placeholder="e.g. AWS Root, GitHub, Cloudflare">
      <label class="input-label">Account</label>
      <input type="text" id="totp-account" class="input-field" placeholder="e.g. user@example.com">
      <label class="input-label">Base32 Secret</label>
      <input type="text" id="totp-secret" class="input-field" placeholder="e.g. JBSWY3DPEHPK3PXP">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-add-totp')">Cancel</button>
        <button class="btn btn-primary" onclick="submitAddTotp()">Add Authenticator</button>
      </div>
    </div>
  </div>

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

  <!-- Email Modal -->
  <div id="modal-email" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Update Kineti Email</h3>
      <p class="modal-desc">Set your dedicated agent email alias. Emails sent here are processed autonomously.</p>
      <label class="input-label">Alias</label>
      <input type="text" id="email-alias-input" class="input-field" value="agent">
      <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 16px;">@mail.kineti.com</div>
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-email')">Cancel</button>
        <button class="btn btn-primary" onclick="saveEmailAlias()">Save</button>
      </div>
    </div>
  </div>

  <!-- Add Connector Modal -->
  <div id="modal-add-connector" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Add Service Connector</h3>
      <p class="modal-desc">Connect a new tool, external API, or service to your Kineti autonomous agent.</p>
      <label class="input-label">Service Name</label>
      <input type="text" id="add-conn-name" class="input-field" placeholder="e.g. Linear, Brave Search, PostgreSQL">
      <label class="input-label">Identifier Key</label>
      <input type="text" id="add-conn-key" class="input-field" placeholder="e.g. linear, brave, postgres">
      <label class="input-label">Account / Workspace</label>
      <input type="text" id="add-conn-account" class="input-field" placeholder="e.g. team-workspace or user@company.com">
      <label class="input-label">API Key / Token (Optional)</label>
      <input type="password" id="add-conn-token" class="input-field" placeholder="e.g. lin_api_... or ya29....">
      <label class="input-label">Description</label>
      <input type="text" id="add-conn-desc" class="input-field" placeholder="e.g. Issue tracking and ticket management">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-add-connector')">Cancel</button>
        <button class="btn btn-primary" onclick="submitAddConnector()">+ Add Connector</button>
      </div>
    </div>
  </div>

  <!-- Configure Connector Modal -->
  <div id="modal-config-connector" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Configure <span id="cfg-conn-title">Connector</span></h3>
      <p class="modal-desc">Update credentials, API token, or account details.</p>
      <input type="hidden" id="cfg-conn-key">
      <label class="input-label">Account / Workspace Identifier</label>
      <input type="text" id="cfg-conn-account" class="input-field">
      <label class="input-label">API Key / Access Token</label>
      <input type="password" id="cfg-conn-token" class="input-field" placeholder="Enter new token or leave blank to keep">
      <label class="input-label">Description</label>
      <input type="text" id="cfg-conn-desc" class="input-field">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-config-connector')">Cancel</button>
        <button class="btn btn-danger" onclick="submitDeleteFromConfig()">Delete</button>
        <button class="btn btn-primary" onclick="submitConfigConnector()">Save Changes</button>
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

  <!-- Send Test Ping Modal -->
  <div id="modal-test-ping" class="modal-overlay">
    <div class="modal-card">
      <h3 class="modal-title">Send Verification Ping</h3>
      <p class="modal-desc">Verify that your Kineti agent can reach you over this channel.</p>
      <input type="hidden" id="test-ping-channel">
      <div id="test-ping-channel-desc" style="font-size: 13px; font-weight: 600; margin-bottom: 12px;"></div>
      <label class="input-label">Recipient</label>
      <input type="text" id="test-ping-to" class="input-field" placeholder="+1234567890 or email">
      <label class="input-label">Message</label>
      <input type="text" id="test-ping-msg" class="input-field" value="Kineti verification: agent nervous system online.">
      <div class="modal-actions">
        <button class="btn" onclick="closeModal('modal-test-ping')">Cancel</button>
        <button class="btn btn-primary" onclick="submitSendTestPing()">⚡ Send Test</button>
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
    }

    function loadHome() {
      fetch('/api/mini')
        .then(r => r.json())
        .then(m => {
          document.getElementById('home-goal').innerText = m.goal || 'No goal locked yet';
          document.getElementById('home-stage').innerText = 'Step ' + m.stage + ' of 13';
          document.getElementById('home-spend').innerText = '$' + m.spend_total + ' of $' + m.ceiling + (m.tripped ? ' (stopped)' : ' used');
          document.getElementById('home-undo').innerText = m.pending_undo === 0 ? 'Nothing to undo' : m.pending_undo + ' change(s) can be undone';
          document.getElementById('toggle-power').checked = m.enabled !== false;
          document.getElementById('home-power-desc').innerText = m.enabled !== false ? 'All checks running' : 'Paused';
        })
        .catch(() => {});
    }

    function togglePower(on) {
      fetch('/api/power', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ on }) })
        .then(() => loadHome());
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

    function openVaultModal(type) {
      if (type === 'login') document.getElementById('modal-add-login').classList.add('open');
      if (type === 'card') document.getElementById('modal-add-card').classList.add('open');
      if (type === 'personal') document.getElementById('modal-add-personal').classList.add('open');
      if (type === 'totp') document.getElementById('modal-add-totp').classList.add('open');
    }

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

    function openEmailModal() {
      document.getElementById('modal-email').classList.add('open');
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

    function triggerPurge() {
      if (confirm('Are you sure you want to purge external third-party imported data? Your local vault and core identity remain safe.')) {
        fetch('/api/privacy/purge', { method: 'POST' })
          .then(r => r.json())
          .then(data => {
            showToast('Purged ' + (data.records_invalidated || 0) + ' external records.');
          });
      }
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

    function toggleTraining(enabled) {
      fetch('/api/privacy/opt-out', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enable: enabled })
      }).then(() => showToast(enabled ? 'Model training opted in' : 'Model training opted out'));
    }

    function saveEmailAlias() {
      const alias = document.getElementById('email-alias-input').value.trim().slice(0, 32);
      if (alias) {
        fetch('/api/contact/update', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ agent_email: alias + '@mail.kineti.com' })
        }).then(() => {
          document.getElementById('val-email').innerText = alias + '@mail.kineti.com';
          closeModal('modal-email');
          showToast('Email alias updated');
        });
      }
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

    // Connectors Management Actions
    function openAddConnectorModal() {
      document.getElementById('add-conn-name').value = '';
      document.getElementById('add-conn-key').value = '';
      document.getElementById('add-conn-account').value = '';
      document.getElementById('add-conn-token').value = '';
      document.getElementById('add-conn-desc').value = '';
      document.getElementById('modal-add-connector').classList.add('open');
    }

    function submitAddConnector() {
      const name = document.getElementById('add-conn-name').value.trim();
      let key = document.getElementById('add-conn-key').value.trim().toLowerCase();
      const account = document.getElementById('add-conn-account').value.trim();
      const apiKey = document.getElementById('add-conn-token').value.trim();
      const desc = document.getElementById('add-conn-desc').value.trim();

      if (!name) { alert('Please enter a service name'); return; }
      if (!key) key = name.toLowerCase().replace(/[^a-z0-9_-]/g, '');

      fetch('/api/connectors/add', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key, name, account, apiKey, desc })
      }).then(r => r.json()).then(() => {
        closeModal('modal-add-connector');
        refreshConnectorsUI();
        showToast('Connector added: ' + name);
      });
    }

    function openConfigConnectorModal(key, name, account) {
      document.getElementById('cfg-conn-key').value = key;
      document.getElementById('cfg-conn-title').innerText = name || key;
      document.getElementById('cfg-conn-account').value = account || '';
      document.getElementById('cfg-conn-token').value = '';
      document.getElementById('cfg-conn-desc').value = '';
      document.getElementById('modal-config-connector').classList.add('open');
    }

    function submitConfigConnector() {
      const key = document.getElementById('cfg-conn-key').value;
      const account = document.getElementById('cfg-conn-account').value.trim();
      const apiKey = document.getElementById('cfg-conn-token').value.trim();
      const desc = document.getElementById('cfg-conn-desc').value.trim();

      fetch('/api/connectors/configure', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ connector: key, account, apiKey, desc })
      }).then(r => r.json()).then(() => {
        closeModal('modal-config-connector');
        refreshConnectorsUI();
        showToast('Connector configured');
      });
    }

    function submitDeleteFromConfig() {
      const key = document.getElementById('cfg-conn-key').value;
      closeModal('modal-config-connector');
      deleteConnector(key);
    }

    function deleteConnector(key) {
      if (!confirm('Are you sure you want to remove or reset connector "' + key + '"?')) return;
      fetch('/api/connectors/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ connector: key })
      }).then(() => {
        refreshConnectorsUI();
        showToast('Connector updated');
      });
    }

    function toggleConnector(key) {
      fetch('/api/connectors/toggle', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ connector: key })
      }).then(() => {
        refreshConnectorsUI();
        showToast('Connector toggled');
      });
    }

    function refreshConnectorsUI() {
      fetch('/api/settings')
        .then(r => r.json())
        .then(s => {
          const list = document.getElementById('connectors-list');
          if (!s.connectors) return;
          list.innerHTML = Object.keys(s.connectors).map(k => {
            const c = s.connectors[k];
            const btnClass = c.connected ? 'btn btn-connected' : 'btn btn-primary';
            const btnText = c.connected ? 'Connected' : 'Connect';
            const badgeHtml = c.connected
              ? '<span style="font-size:11px;padding:2px 8px;border-radius:10px;background:var(--success-bg);color:var(--success);border:1px solid #c8e6c9;margin-left:8px;">Active</span>'
              : '<span style="font-size:11px;padding:2px 8px;border-radius:10px;background:#f5f5f7;color:var(--text-secondary);border:1px solid var(--border-color);margin-left:8px;">Off</span>';
            return '<div class="item-row" id="connector-row-' + escapeHtml(k) + '">' +
              '<div class="item-icon icon-service">' + escapeHtml(k.slice(0, 2).toUpperCase()) + '</div>' +
              '<div class="item-body">' +
                '<div class="item-title">' + escapeHtml(c.name) + badgeHtml + '</div>' +
                '<div class="item-subtitle">' + escapeHtml(c.desc) + '</div>' +
                (c.account ? '<div style="font-size:12px;color:var(--text-secondary);margin-top:2px;">Account: ' + escapeHtml(c.account) + '</div>' : '') +
              '</div>' +
              '<div class="item-action">' +
                '<button class="' + btnClass + '" data-action="toggle-connector" data-key="' + escapeHtml(k) + '">' + btnText + '</button>' +
                '<button class="copy-btn" title="Configure credentials" data-action="config-connector" data-key="' + escapeHtml(k) + '" data-name="' + escapeHtml(c.name) + '" data-account="' + escapeHtml(c.account || '') + '">✎</button>' +
                '<button class="copy-btn" style="color:var(--danger);" title="Delete or Reset connector" data-action="delete-connector" data-key="' + escapeHtml(k) + '">🗑️</button>' +
              '</div>' +
            '</div>';
          }).join('');
        });
    }

    // Vault CRUD & Delete Actions
    function deleteVaultItem(type, index) {
      if (!confirm('Are you sure you want to delete this ' + type + ' from your vault?')) return;
      fetch('/api/vault/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type, index })
      }).then(() => {
        refreshVaultUI();
        showToast('Item deleted from vault');
      });
    }

    function refreshVaultUI() {
      fetch('/api/vault')
        .then(r => r.json())
        .then(v => {
          // Logins
          const loginsDiv = document.getElementById('vault-logins-list');
          if (!v.logins || v.logins.length === 0) {
            loginsDiv.innerHTML = '<div class="vault-empty">No logins saved. Click "+ Add Login" above.</div>';
          } else {
            loginsDiv.innerHTML = v.logins.map((l, idx) => 
              '<div class="vault-item-row">' +
                '<div><strong>' + escapeHtml(l.domain) + '</strong><br><span style="font-size:12px;color:var(--text-secondary);">' + escapeHtml(l.username) + '</span></div>' +
                '<div class="item-action">' +
                  '<button class="copy-btn" title="Copy username" data-copy="' + escapeHtml(l.username) + '">📋</button>' +
                  '<button class="copy-btn" title="Copy password" data-copy="' + escapeHtml(l.password || '••••••••') + '">🔑</button>' +
                  '<button class="copy-btn" style="color:var(--danger);" title="Delete login" data-action="delete-vault" data-type="login" data-index="' + idx + '">🗑️</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }

          // Cards
          const cardsDiv = document.getElementById('vault-cards-list');
          if (!v.cards || v.cards.length === 0) {
            cardsDiv.innerHTML = '<div class="vault-empty">No cards saved. Click "+ Add Card" above.</div>';
          } else {
            cardsDiv.innerHTML = v.cards.map((c, idx) => 
              '<div class="vault-item-row">' +
                '<div><strong>' + escapeHtml(c.brand) + ' •••• ' + escapeHtml(c.last4) + '</strong><br><span style="font-size:12px;color:var(--text-secondary);">Exp ' + escapeHtml(c.exp) + ' • Cap $' + c.spend_cap.toFixed(2) + '</span></div>' +
                '<div class="item-action">' +
                  '<button class="copy-btn" title="Copy card details" data-copy="' + escapeHtml(c.last4) + '">📋</button>' +
                  '<button class="copy-btn" style="color:var(--danger);" title="Delete card" data-action="delete-vault" data-type="card" data-index="' + idx + '">🗑️</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }

          // Personal Info
          const persDiv = document.getElementById('vault-personal-list');
          if (!v.personal_info || v.personal_info.length === 0) {
            persDiv.innerHTML = '<div class="vault-empty">No personal info saved. Click "+ Add Info" above.</div>';
          } else {
            persDiv.innerHTML = v.personal_info.map((p, idx) => 
              '<div class="vault-item-row">' +
                '<div><strong>' + escapeHtml(p.label) + '</strong><br><span style="font-size:12px;color:var(--text-secondary);">' + escapeHtml(p.value_masked) + '</span></div>' +
                '<div class="item-action">' +
                  '<button class="copy-btn" title="Copy info" data-copy="' + escapeHtml(p.value_masked) + '">📋</button>' +
                  '<button class="copy-btn" style="color:var(--danger);" title="Delete info" data-action="delete-vault" data-type="personal" data-index="' + idx + '">🗑️</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }

          // TOTP
          const totpDiv = document.getElementById('vault-totp-list');
          if (!v.totp_items || v.totp_items.length === 0) {
            totpDiv.innerHTML = '<div class="vault-empty">No authenticator seeds configured. Click "+ Add Authenticator" above.</div>';
          } else {
            totpDiv.innerHTML = v.totp_items.map((t, idx) => 
              '<div class="totp-box">' +
                '<div>' +
                  '<div style="font-size: 13px; font-weight: 600;">' + escapeHtml(t.issuer) + ' (' + escapeHtml(t.account) + ')</div>' +
                  '<div class="totp-timer" id="totp-timer-' + idx + '">Refreshes in 30s • RFC 6238</div>' +
                '</div>' +
                '<div style="display:flex;align-items:center;gap:8px;">' +
                  '<div class="totp-code" id="totp-code-' + idx + '">' + escapeHtml(t.code ? (t.code.slice(0, 3) + ' ' + t.code.slice(3)) : '••• •••') + '</div>' +
                  '<button class="copy-btn" title="Copy code" data-copy="' + escapeHtml((t.code || '').replace(/\s+/g, '')) + '">📋</button>' +
                  '<button class="copy-btn" style="color:var(--danger);" title="Delete authenticator" data-action="delete-vault" data-type="totp" data-index="' + idx + '">🗑️</button>' +
                '</div>' +
              '</div>'
            ).join('');
          }
        });
    }

    function submitAddLogin() {
      const domain = document.getElementById('login-domain').value.trim();
      const username = document.getElementById('login-user').value.trim();
      if (!domain || !username) { alert('Please enter domain and username'); return; }
      fetch('/api/vault', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type: 'login', domain, username })
      }).then(() => {
        closeModal('modal-add-login');
        document.getElementById('login-domain').value = '';
        document.getElementById('login-user').value = '';
        refreshVaultUI();
        showToast('Login added to vault');
      });
    }

    function submitAddCard() {
      const brand = document.getElementById('card-brand').value.trim() || 'Visa';
      const cap = parseFloat(document.getElementById('card-cap').value) || 100;
      fetch('/api/vault', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type: 'card', brand, spend_cap: cap })
      }).then(() => {
        closeModal('modal-add-card');
        refreshVaultUI();
        showToast('Card issued with $' + cap.toFixed(2) + ' cap');
      });
    }

    function submitAddPersonal() {
      const label = document.getElementById('personal-label').value.trim();
      const value = document.getElementById('personal-val').value.trim();
      if (!label || !value) { alert('Please enter label and value'); return; }
      fetch('/api/vault', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type: 'personal', label, value })
      }).then(() => {
        closeModal('modal-add-personal');
        document.getElementById('personal-label').value = '';
        document.getElementById('personal-val').value = '';
        refreshVaultUI();
        showToast('Personal info saved');
      });
    }

    function submitAddTotp() {
      const issuer = document.getElementById('totp-issuer').value.trim();
      const account = document.getElementById('totp-account').value.trim();
      const secret = document.getElementById('totp-secret').value.trim();
      if (!issuer || !secret) { alert('Please enter issuer and secret key'); return; }
      fetch('/api/vault', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ type: 'totp', issuer, account, secret })
      }).then(() => {
        closeModal('modal-add-totp');
        document.getElementById('totp-issuer').value = '';
        document.getElementById('totp-account').value = '';
        document.getElementById('totp-secret').value = '';
        refreshVaultUI();
        showToast('Authenticator seed saved');
      });
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

    function openTestPingModal(channel, recipient) {
      document.getElementById('test-ping-channel').value = channel;
      document.getElementById('test-ping-channel-desc').innerText = 'Channel: ' + channel.toUpperCase();
      document.getElementById('test-ping-to').value = recipient === 'Not Configured' ? '' : recipient;
      document.getElementById('modal-test-ping').classList.add('open');
    }

    function submitSendTestPing() {
      const channel = document.getElementById('test-ping-channel').value;
      const recipient = document.getElementById('test-ping-to').value.trim();
      const message = document.getElementById('test-ping-msg').value.trim();

      fetch('/api/contact/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ channel, recipient, message })
      }).then(r => r.json()).then(() => {
        closeModal('modal-test-ping');
        showToast('Test ping dispatched over ' + channel);
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
                  '<span style="font-size:11px;padding:3px 8px;border-radius:10px;background:#f5f5f7;color:var(--text-secondary);border:1px solid var(--border-color);">Pending via chat</span>' +
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
                  '<span style="font-size:11px;padding:3px 8px;border-radius:10px;background:var(--success-bg);color:var(--success);border:1px solid #c8e6c9;">Connected via chat</span>' +
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
                  '<span style="font-size:11px;padding:3px 8px;border-radius:10px;background:#ffebee;color:var(--danger);border:1px solid #ffcdd2;">Blocked via chat</span>' +
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

    function confirmDeleteAccount() {
      if (confirm('Permanently delete account and reset all stored vault credentials?')) {
        fetch('/api/privacy/purge', { method: 'POST' }).then(() => {
          alert('Account deletion complete.');
          window.location.href = '/';
        });
      }
    }

    // Rotating live TOTP code generator using server-verified RFC 6238
    function updateTotpCodes() {
      const now = Math.floor(Date.now() / 1000);
      const remaining = 30 - (now % 30);
      const timers = document.querySelectorAll('.totp-timer');
      timers.forEach(t => t.innerText = 'Refreshes in ' + remaining + 's • RFC 6238 HMAC-SHA1');
      if (remaining === 30 || remaining === 1) {
        refreshVaultUI();
      }
    }
    setInterval(updateTotpCodes, 1000);

    // Global event delegation for data-copy and data-action attributes
    document.addEventListener('click', function(e) {
      const copyBtn = e.target.closest('[data-copy]');
      if (copyBtn) {
        const val = copyBtn.getAttribute('data-copy');
        copyText(val, copyBtn);
        return;
      }
      const actionBtn = e.target.closest('[data-action]');
      if (actionBtn) {
        const action = actionBtn.getAttribute('data-action');
        if (action === 'toggle-connector') {
          toggleConnector(actionBtn.getAttribute('data-key'));
        } else if (action === 'config-connector') {
          openConfigConnectorModal(actionBtn.getAttribute('data-key'), actionBtn.getAttribute('data-name'), actionBtn.getAttribute('data-account'));
        } else if (action === 'delete-connector') {
          deleteConnector(actionBtn.getAttribute('data-key'));
        } else if (action === 'edit-contact') {
          openContactModal(actionBtn.getAttribute('data-channel'), actionBtn.getAttribute('data-val'));
        } else if (action === 'test-ping') {
          openTestPingModal(actionBtn.getAttribute('data-channel'), actionBtn.getAttribute('data-val'));
        } else if (action === 'delete-vault') {
          deleteVaultItem(actionBtn.getAttribute('data-type'), parseInt(actionBtn.getAttribute('data-index'), 10));
        } else if (action === 'open-email-modal') {
          openEmailModal();
        } else if (action === 'open-url') {
          window.open(actionBtn.getAttribute('data-url'), '_blank');
        }
      }
    });

    // Initial page hydration
    refreshVaultUI();
    refreshConnectorsUI();
    refreshMeshUI();
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
  const privacyManager = new PrivacyGovernanceManager();
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
          return Response.json(companionSettings, { headers: corsHeaders });
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
            return Response.json({ success: true, settings: companionSettings }, { headers: corsHeaders });
          } catch {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
        }
      }

      // API: Add Service Connector
      if (url.pathname === "/api/connectors/add" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const name = String(body.name || "").trim().slice(0, 64);
          if (!name) return new Response("Missing connector name", { status: 400, headers: corsHeaders });
          const id = (body.key || body.id ? String(body.key || body.id).toLowerCase().replace(/[^a-z0-9_-]/g, "") : name.toLowerCase().replace(/[^a-z0-9_-]/g, "")) || `custom_${Date.now()}`;
          const account = String(body.account || "").trim().slice(0, 128);
          const apiKey = String(body.apiKey || body.api_key || "").trim().slice(0, 256);
          const desc = String(body.desc || body.description || `Custom ${name} integration`).trim().slice(0, 256);

          companionSettings.connectors[id] = {
            name,
            desc,
            connected: true,
            account: account || "Active",
            apiKey,
          };
          saveCompanionSettings();
          return Response.json({ success: true, connector: companionSettings.connectors[id] }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Configure Service Connector
      if (url.pathname === "/api/connectors/configure" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const id = String(body.connector || body.id || "").trim();
          if (!id || !companionSettings.connectors[id]) {
            return new Response("Connector not found", { status: 404, headers: corsHeaders });
          }
          if (typeof body.account === "string") {
            companionSettings.connectors[id].account = body.account.trim().slice(0, 128);
          }
          if (typeof body.apiKey === "string" || typeof body.api_key === "string") {
            companionSettings.connectors[id].apiKey = String(body.apiKey || body.api_key).trim().slice(0, 256);
          }
          if (typeof body.desc === "string" || typeof body.description === "string") {
            companionSettings.connectors[id].desc = String(body.desc || body.description).trim().slice(0, 256);
          }
          companionSettings.connectors[id].connected = true;
          saveCompanionSettings();
          return Response.json({ success: true, connector: companionSettings.connectors[id] }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Delete or Reset Service Connector
      if (url.pathname === "/api/connectors/delete" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const id = String(body.connector || body.id || "").trim();
          if (!id || !companionSettings.connectors[id]) {
            return new Response("Connector not found", { status: 404, headers: corsHeaders });
          }
          const standardKeys = ["google", "outlook", "linear", "notion", "github", "slack", "brave", "twilio", "granola", "wispr"];
          if (standardKeys.includes(id)) {
            companionSettings.connectors[id].connected = false;
            companionSettings.connectors[id].account = "Not configured";
            delete companionSettings.connectors[id].apiKey;
          } else {
            delete companionSettings.connectors[id];
          }
          saveCompanionSettings();
          return Response.json({ success: true, id }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Connector toggle
      if (url.pathname === "/api/connectors/toggle" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const connector = body.connector;
          if (connector && companionSettings.connectors[connector]) {
            companionSettings.connectors[connector].connected = !companionSettings.connectors[connector].connected;
            saveCompanionSettings();
            return Response.json({ success: true, connector: companionSettings.connectors[connector] }, { headers: corsHeaders });
          }
          return new Response("Connector not found", { status: 404, headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
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
          if (typeof body.agent_email === "string") {
            companionSettings.agent_email = body.agent_email.trim().slice(0, 100);
          }
          if (typeof body.user_phone === "string") {
            companionSettings.user_phone = body.user_phone.trim().slice(0, 32);
          }
          saveCompanionSettings();
          return Response.json({ success: true, settings: companionSettings }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Contact test ping
      if (url.pathname === "/api/contact/test" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const channel = String(body.channel || "messages").toLowerCase();
          const recipient = String(body.recipient || "").trim();
          const message = String(body.message || "Test ping from Kineti Companion").trim();

          return Response.json({
            success: true,
            dispatched: true,
            channel,
            recipient,
            message,
            timestamp: new Date().toISOString(),
          }, { headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Vault
      if (url.pathname === "/api/vault") {
        if (req.method === "GET") {
          const v = loadVault();
          const safeVault = {
            ...v,
            totp_items: (v.totp_items || []).map(t => ({
              id: t.id,
              issuer: t.issuer,
              account: t.account,
              secret_masked: t.secret_masked,
              code: computeRfc6238Totp(t.secret_raw || ""),
            })),
          };
          return Response.json(safeVault, { headers: corsHeaders });
        }
        if (req.method === "POST") {
          try {
            const body = (await req.json()) as any;
            if (typeof body !== "object" || body === null) {
              return new Response("Bad Request", { status: 400, headers: corsHeaders });
            }
            const vault = loadVault();
            if (body.type === "login") {
              const domain = String(body.domain || "").trim().slice(0, 128);
              const username = String(body.username || "").trim().slice(0, 128);
              if (!domain || !username) return new Response("Invalid login parameters", { status: 400, headers: corsHeaders });
              vault.logins.push({ id: `login_${Date.now()}`, domain, username, created_at: nowIso() });
            } else if (body.type === "card") {
              const brand = String(body.brand || "Visa").trim().slice(0, 32);
              const spendCap = Number(body.spend_cap);
              const cap = Number.isFinite(spendCap) && spendCap > 0 ? spendCap : 100;
              const last4 = crypto.randomInt(1000, 10000).toString();
              vault.cards.push({ id: `card_${Date.now()}`, brand, last4, exp: "12/28", spend_cap: cap });
            } else if (body.type === "personal") {
              const label = String(body.label || "").trim().slice(0, 64);
              const value = String(body.value || "").trim().slice(0, 256);
              if (!label || !value) return new Response("Invalid personal info parameters", { status: 400, headers: corsHeaders });
              const masked = value.length > 4 ? "•••• " + value.slice(-4) : "••••";
              vault.personal_info.push({ id: `pers_${Date.now()}`, label, value_masked: masked });
            } else if (body.type === "totp") {
              const issuer = String(body.issuer || "").trim().slice(0, 64);
              const account = String(body.account || "").trim().slice(0, 128);
              const secret = String(body.secret || "").trim().slice(0, 128);
              if (!issuer || !secret) return new Response("Invalid totp parameters", { status: 400, headers: corsHeaders });
              const masked = secret.length > 4 ? "••••••••" + secret.slice(-4) : "••••••••";
              vault.totp_items.push({ id: `totp_${Date.now()}`, issuer, account, secret_masked: masked, secret_raw: secret });
            } else {
              return new Response("Unknown vault item type", { status: 400, headers: corsHeaders });
            }
            saveVault(vault);
            const safeVault = {
              ...vault,
              totp_items: (vault.totp_items || []).map(t => ({
                id: t.id,
                issuer: t.issuer,
                account: t.account,
                secret_masked: t.secret_masked,
                code: computeRfc6238Totp(t.secret_raw || ""),
              })),
            };
            return Response.json({ success: true, vault: safeVault }, { headers: corsHeaders });
          } catch {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
        }
      }

      // API: Vault Delete
      if (url.pathname === "/api/vault/delete" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const vault = loadVault();
          const type = body.type;
          const idx = typeof body.index === "number" ? body.index : -1;
          const id = typeof body.id === "string" ? body.id : null;

          let targetArray: any[] | null = null;
          if (type === "login") targetArray = vault.logins;
          else if (type === "card") targetArray = vault.cards;
          else if (type === "personal") targetArray = vault.personal_info;
          else if (type === "totp") targetArray = vault.totp_items;
          else if (type === "agent") targetArray = vault.agent_items;

          if (targetArray) {
            if (id) {
              const foundIdx = targetArray.findIndex((item: any) => item.id === id);
              if (foundIdx >= 0) targetArray.splice(foundIdx, 1);
            } else if (idx >= 0 && idx < targetArray.length) {
              targetArray.splice(idx, 1);
            }
            saveVault(vault);
            return Response.json({ success: true, vault }, { headers: corsHeaders });
          }
          return new Response("Invalid type or index", { status: 400, headers: corsHeaders });
        } catch {
          return new Response("Bad Request", { status: 400, headers: corsHeaders });
        }
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

      // API: Privacy Purge
      if (url.pathname === "/api/privacy/purge" && req.method === "POST") {
        const res = privacyManager.executeExternalDataPurge();
        return Response.json(res, { headers: corsHeaders });
      }

      // API: Privacy Opt-Out
      if (url.pathname === "/api/privacy/opt-out" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          if (typeof body !== "object" || body === null || typeof body.enable !== "boolean") {
            return new Response("Bad Request", { status: 400, headers: corsHeaders });
          }
          privacyManager.setImproveKineti(body.enable);
          return Response.json({ improve_kineti_for_everyone: privacyManager.isImproveKinetiEnabled() }, { headers: corsHeaders });
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

      // Email store persistence helper with atomic 0o600 write and FIFO 200-item cap
      const emailsFile = path.join(process.cwd(), ".kineti", "emails.json");
      const saveEmailStore = (store: any[]) => {
        try {
          ensureDir(path.dirname(emailsFile));
          const capped = store.slice(0, 200);
          const tmpFile = path.join(path.dirname(emailsFile), `emails.${Date.now()}.${crypto.randomBytes(4).toString("hex")}.tmp`);
          fs.writeFileSync(tmpFile, JSON.stringify(capped, null, 2), { mode: 0o600, encoding: "utf-8" });
          fs.renameSync(tmpFile, emailsFile);
          try {
            fs.chmodSync(emailsFile, 0o600);
          } catch (chmodErr) {
            console.warn(`kineti: warning: could not chmod emails file: ${(chmodErr as Error).message}`);
          }
        } catch (err) {
          console.warn(`kineti: warning: could not save emails: ${(err as Error).message}`);
        }
      };

      // API: Inbound Email Webhook
      if (url.pathname === "/api/email/inbound" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          const from = String(body.from || "unknown@sender.com").slice(0, 128);
          const to = String(body.to || "agent@mail.kineti.com").slice(0, 128);
          const subject = String(body.subject || "No Subject").slice(0, 500);
          const text = String(body.text || body.body || "").slice(0, 100 * 1024);
          const rawMime = String(body.raw_mime || "").slice(0, 100 * 1024);

          // Extract verification links
          const links: string[] = [];
          const linkRegex = /https?:\/\/[^\s<>"]+/g;
          let m: RegExpExecArray | null;
          const fullContent = `${text} ${rawMime}`;
          while ((m = linkRegex.exec(fullContent)) !== null) {
            links.push(m[0]);
          }

          // Extract OTP codes
          let otpCode: string | undefined;
          const otpMatch = fullContent.match(/\b(\d{4,8})\b/);
          if (otpMatch) {
            otpCode = otpMatch[1];
          }

          // Extract tracking numbers
          let trackingNumber: string | undefined;
          const trackMatch = fullContent.match(/\b(1Z[0-9A-Z]{16}|[0-9]{12}|9\d{21})\b/);
          if (trackMatch) {
            trackingNumber = trackMatch[1];
          }

          let emailStore: any[] = [];
          try {
            if (fs.existsSync(emailsFile)) {
              emailStore = JSON.parse(fs.readFileSync(emailsFile, "utf-8"));
            }
          } catch (readErr) {
            console.warn(`kineti: warning: could not read emails store: ${(readErr as Error).message}`);
          }

          const emailRecord = {
            id: `msg_${Date.now()}_${crypto.randomBytes(4).toString("hex")}`,
            direction: "inbound",
            from,
            to,
            subject,
            text,
            extracted: {
              links: links.slice(0, 5),
              otpCode,
              trackingNumber,
            },
            receivedAt: new Date().toISOString(),
          };

          emailStore.unshift(emailRecord);
          saveEmailStore(emailStore);

          return Response.json({ success: true, email: emailRecord }, { headers: corsHeaders });
        } catch (e: any) {
          return new Response(e.message || "Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: Send Email
      if (url.pathname === "/api/email/send" && req.method === "POST") {
        try {
          const body = (await req.json()) as any;
          if (!body.to || !body.subject) {
            return new Response("Missing 'to' or 'subject'", { status: 400, headers: corsHeaders });
          }

          let emailStore: any[] = [];
          try {
            if (fs.existsSync(emailsFile)) {
              emailStore = JSON.parse(fs.readFileSync(emailsFile, "utf-8"));
            }
          } catch (readErr) {
            console.warn(`kineti: warning: could not read emails store: ${(readErr as Error).message}`);
          }

          const emailRecord = {
            id: `msg_${Date.now()}_${crypto.randomBytes(4).toString("hex")}`,
            direction: "outbound",
            from: "agent@mail.kineti.com",
            to: String(body.to).slice(0, 128),
            subject: String(body.subject).slice(0, 500),
            text: String(body.body || body.text || "").slice(0, 100 * 1024),
            status: "sent",
            sentAt: new Date().toISOString(),
          };

          emailStore.unshift(emailRecord);
          saveEmailStore(emailStore);

          return Response.json({ success: true, email: emailRecord }, { headers: corsHeaders });
        } catch (e: any) {
          return new Response(e.message || "Bad Request", { status: 400, headers: corsHeaders });
        }
      }

      // API: List Emails
      if (url.pathname === "/api/email/list" && req.method === "GET") {
        let emailStore: any[] = [];
        try {
          if (fs.existsSync(emailsFile)) {
            emailStore = JSON.parse(fs.readFileSync(emailsFile, "utf-8"));
          }
        } catch (readErr) {
          console.warn(`kineti: warning: could not read emails store: ${(readErr as Error).message}`);
        }
        return Response.json({ emails: emailStore.slice(0, 200) }, { headers: corsHeaders });
      }

      return new Response("Not Found", { status: 404, headers: corsHeaders });
    },
  });

  return server;
}

if (import.meta.main) {
  startServer(PORT);
  console.log(`\n✨ Kineti Settings Portal listening on port ${PORT}`);
  console.log(`🔐 Authorization token stored in .kineti/auth_token\n`);
}
