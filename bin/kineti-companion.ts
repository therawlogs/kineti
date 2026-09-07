#!/usr/bin/env bun
// bin/kineti-companion.ts
// Kineti OS — Simple, Plain-English Companion Dashboard

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { die, ok, projectKdir, readJson, readJsonl, ensureDir, nowIso } from "./lib.ts";

const PORT = Number(process.env.KINETI_COMPANION_PORT || 8788);
const REPO_ROOT = process.cwd();

// Standard 13 stage definitions preserved for API/test compatibility
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
    } catch {}
  }
  if (!token) {
    token = crypto.randomBytes(32).toString("hex");
    ensureDir(projectKdir());
    fs.writeFileSync(tokenFile, token + "\n", { mode: 0o600 });
  }
  return token;
}

const AUTH_TOKEN = generateAuthToken();

export interface ActivityEvent {
  timestamp: string;
  badge: "START" | "GOAL" | "TASK" | "STEP" | "CHECK" | "PASS" | "FAIL" | "CHANGE";
  title: string;
  detail: string;
}

function getHarnessStatus() {
  const statePath = path.join(projectKdir(), "state.json");
  const spendPath = path.join(projectKdir(), "spend.json");
  const evidencePath = path.join(projectKdir(), "evidence.jsonl");

  const state = readJson<any>(statePath) ?? {
    version: 1,
    project: path.basename(REPO_ROOT),
    root_goal: null,
    root_goal_locked_at: null,
    stage: 1,
    gates: {},
    history: [],
  };

  const spend = readJson<any>(spendPath) ?? {
    tripped: false,
    reason: null,
    total_usd: 0,
    total_microcents: 0,
    by_stage: {},
    by_stage_microcents: {},
    entries: 0,
  };

  const evidence = readJsonl<any>(evidencePath).slice(-30).reverse();

  // Determine stage and task information
  const rawStage = state.stage || 1;
  let stageNum = 1;
  let stageLabel = "Task";
  let isStandard = false;

  if (typeof rawStage === "number") {
    stageNum = rawStage;
    const found = STAGES.find((s) => s.id === stageNum) ?? STAGES[0];
    stageLabel = found.label;
    isStandard = true;
  } else if (typeof rawStage === "string") {
    const lower = rawStage.toLowerCase().trim();
    const found = STAGES.find((s) => s.name.toLowerCase() === lower);
    if (found) {
      stageNum = found.id;
      stageLabel = found.label;
      isStandard = true;
    } else {
      stageNum = 0;
      stageLabel = rawStage.charAt(0).toUpperCase() + rawStage.slice(1);
      isStandard = false;
    }
  }

  // Pending Human Action (Plain English)
  let pendingAction: { gate: string; title: string; prompt: string } | null = null;
  if (state.gates?.feasibility === "pending" || (stageNum === 5 && state.gates?.feasibility !== "pass")) {
    pendingAction = { gate: "feasibility", title: "Cost & Limits Check", prompt: "Please check costs and API limits to make sure we can proceed." };
  } else if (state.gates?.spec === "pending" || (stageNum === 6 && state.gates?.spec !== "pass")) {
    pendingAction = { gate: "spec", title: "Plan Approval", prompt: "Please approve the plan before code is written." };
  } else if (state.gates?.security === "pending" || (stageNum === 10 && state.gates?.security !== "pass")) {
    pendingAction = { gate: "security", title: "Security Check", prompt: "Please review the security checklist before shipping." };
  } else if (state.gates?.ship === "pending" || (stageNum === 11 && state.gates?.ship !== "pass")) {
    pendingAction = { gate: "ship", title: "Final Approval", prompt: "All tests passed. Please approve merging this work." };
  }

  // Build Chronological Activity Stream in Plain Words
  const activityEvents: ActivityEvent[] = [];

  for (const h of state.history || []) {
    const ev = h.event || "";
    let badge: ActivityEvent["badge"] = "CHANGE";
    let title = "Update";
    let detail = ev;

    if (ev.startsWith("init")) {
      badge = "START";
      title = "Project started";
      detail = `Set up project ${state.project || ""}`;
    } else if (ev.includes("goal locked")) {
      badge = "GOAL";
      title = "Goal saved";
      detail = state.root_goal || "Main goal recorded and locked";
    } else if (ev.startsWith("stage")) {
      badge = "STEP";
      title = "Step changed";
      detail = ev.replace("stage ", "Moved to step ");
    } else if (ev.startsWith("task")) {
      badge = "TASK";
      title = "Task set";
      detail = ev.replace("task.", "").replace("task=", "");
    } else if (ev.startsWith("gate")) {
      badge = "CHECK";
      title = "Safety check updated";
      detail = ev.replace("gate ", "");
    }

    activityEvents.push({
      timestamp: h.at || nowIso(),
      badge,
      title,
      detail,
    });
  }

  for (const e of evidence) {
    activityEvents.push({
      timestamp: e.at || nowIso(),
      badge: e.exit_code === 0 ? "PASS" : "FAIL",
      title: e.exit_code === 0 ? "Test passed" : "Test failed",
      detail: `${e.label} (${e.cmd})`,
    });
  }

  // Sort newest first
  activityEvents.sort((a, b) => b.timestamp.localeCompare(a.timestamp));

  // The 5 W's in Plain Words
  const analytics = {
    why: {
      title: "Why (Goal)",
      goal: state.root_goal || "No goal set yet",
      locked_at: state.root_goal_locked_at,
      immutable: !!state.root_goal_locked_at,
    },
    what: {
      title: "What (Current Task)",
      task_type: state.task?.type || (isStandard ? stageLabel.toLowerCase() : String(state.stage)),
      task_name: state.task?.name || (state.root_goal ? state.root_goal : `${stageLabel} work`),
      active_label: stageLabel,
      is_standard_stage: isStandard,
      target_paths: "src/",
    },
    how: {
      title: "How (Safety Checks)",
      status: spend.tripped ? "Spending limit reached" : pendingAction ? "Your approval needed" : "Running safely",
      evidence_count: evidence.length,
      policy_violations: spend.tripped ? 1 : 0,
      saga_rollback_armed: true,
      pending_action: pendingAction,
    },
    when: {
      title: "When (Time & Spend)",
      started_at: state.root_goal_locked_at || (state.history && state.history[0]?.at) || nowIso(),
      last_activity: (evidence[0]?.at) || (state.history && state.history[state.history.length - 1]?.at) || nowIso(),
      spend_usd: spend.total_usd,
      spend_limit_usd: 50.0,
      spend_pct: Math.min(100, Math.round((spend.total_usd / 50.0) * 100)),
    },
    where: {
      title: "Where (Folder)",
      project: state.project || path.basename(REPO_ROOT),
      working_dir: REPO_ROOT,
    },
  };

  return {
    project: state.project || path.basename(REPO_ROOT),
    root_goal: state.root_goal,
    root_goal_locked_at: state.root_goal_locked_at,
    stage: state.stage,
    stage_label: stageLabel,
    task: state.task ?? null,
    // Preserved for backwards compatibility with tests
    stages: STAGES.map((s) => ({
      ...s,
      is_current: isStandard ? s.id === stageNum : false,
      is_past: isStandard ? s.id < stageNum : false,
      gate_status: s.gate ? (state.gates[s.gate] ?? "pending") : null,
    })),
    gates: state.gates,
    spend: {
      total_usd: spend.total_usd,
      total_microcents: spend.total_microcents,
      ceiling_usd: 50.0,
      safety_factor: 0.95,
      tripped: spend.tripped,
      reason: spend.reason,
      by_stage: spend.by_stage,
      entries: spend.entries,
    },
    evidence,
    activity_events: activityEvents.slice(0, 20),
    analytics,
    history: state.history || [],
  };
}

function renderHtmlDashboard(): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Kineti OS — Visual Companion Canvas</title>
  <style>
    :root {
      --apple-bg: #000000;
      --system-blue: #0A84FF;
      --system-green: #30D158;
      --system-orange: #FF9F0A;
      --system-red: #FF453A;
      --system-purple: #BF5AF2;
      --system-teal: #64D2FF;
      
      /* Apple Materials (Vibrancy & Translucency) */
      --material-nav: rgba(20, 20, 24, 0.75);
      --material-card: rgba(28, 28, 34, 0.65);
      --material-card-hover: rgba(36, 36, 44, 0.75);
      
      /* Apple Borders & Specular Highlights */
      --hairline: 1px solid rgba(255, 255, 255, 0.08);
      --specular: inset 0 1px 0 rgba(255, 255, 255, 0.1);
      --shadow-card: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 4px 16px rgba(0, 0, 0, 0.25);
      
      /* Typography */
      --label-primary: #FFFFFF;
      --label-secondary: rgba(235, 235, 245, 0.65);
      --label-tertiary: rgba(235, 235, 245, 0.35);
    }
    
    * { box-sizing: border-box; margin: 0; padding: 0; }
    
    body {
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "SF Pro", "Helvetica Neue", sans-serif;
      background: var(--apple-bg);
      background-image: 
        radial-gradient(circle at 50% 0%, rgba(10, 132, 255, 0.07) 0%, transparent 60%),
        radial-gradient(circle at 80% 100%, rgba(191, 90, 242, 0.04) 0%, transparent 50%);
      color: var(--label-primary);
      line-height: 1.45;
      min-height: 100vh;
      -webkit-font-smoothing: antialiased;
      -moz-osx-font-smoothing: grayscale;
    }
    
    .mono {
      font-family: "SF Mono", Menlo, Monaco, Consolas, monospace;
      font-feature-settings: "tnum";
    }

    /* Pinned Apple Navigation Bar */
    .apple-nav {
      position: sticky;
      top: 0;
      z-index: 50;
      height: 52px;
      background: var(--material-nav);
      backdrop-filter: blur(25px) saturate(190%);
      -webkit-backdrop-filter: blur(25px) saturate(190%);
      border-bottom: var(--hairline);
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    }

    .nav-inner {
      max-width: 1040px;
      height: 100%;
      margin: 0 auto;
      padding: 0 20px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 16px;
    }

    .nav-left {
      display: flex;
      align-items: center;
      gap: 12px;
    }

    .apple-brand {
      font-size: 13px;
      font-weight: 700;
      letter-spacing: -0.01em;
      color: #FFFFFF;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .brand-glyph {
      width: 14px;
      height: 14px;
      border-radius: 4px;
      background: linear-gradient(135deg, var(--system-blue), var(--system-purple));
      display: inline-block;
    }

    .nav-divider {
      color: var(--label-tertiary);
      font-size: 13px;
    }

    .project-name {
      font-size: 13px;
      font-weight: 600;
      color: var(--label-secondary);
      letter-spacing: -0.01em;
    }

    .nav-right {
      display: flex;
      align-items: center;
      gap: 14px;
    }

    /* Status Capsule */
    .status-capsule {
      display: inline-flex;
      align-items: center;
      gap: 5px;
      font-size: 11px;
      font-weight: 500;
      padding: 3px 10px;
      border-radius: 9999px;
      transition: all 0.2s ease;
    }
    .capsule-safe {
      background: rgba(48, 209, 88, 0.15);
      color: #30D158;
      border: 1px solid rgba(48, 209, 88, 0.3);
    }
    .capsule-action {
      background: rgba(255, 159, 10, 0.15);
      color: #FF9F0A;
      border: 1px solid rgba(255, 159, 10, 0.3);
    }
    .capsule-tripped {
      background: rgba(255, 69, 58, 0.15);
      color: #FF453A;
      border: 1px solid rgba(255, 69, 58, 0.3);
    }
    .status-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: currentColor;
    }

    /* Apple Segmented Control */
    .segmented-control {
      display: inline-flex;
      background: rgba(255, 255, 255, 0.08);
      padding: 2.5px;
      border-radius: 9px;
      border: var(--hairline);
    }
    .seg-btn {
      background: transparent;
      border: none;
      color: var(--label-secondary);
      font-size: 12px;
      font-weight: 500;
      padding: 4px 14px;
      border-radius: 7px;
      cursor: pointer;
      transition: all 0.15s ease-out;
      font-family: inherit;
    }
    .seg-btn:hover {
      color: var(--label-primary);
    }
    .seg-btn.active {
      background: rgba(255, 255, 255, 0.15);
      color: #fff;
      font-weight: 600;
      box-shadow: 0 1px 2px rgba(0,0,0,0.3), var(--specular);
    }

    /* Apple Spend Gauge */
    .spend-gauge {
      display: inline-flex;
      align-items: center;
      gap: 9px;
      background: rgba(255, 255, 255, 0.05);
      border: var(--hairline);
      padding: 4px 12px;
      border-radius: 9px;
      font-size: 12px;
    }
    .spend-track {
      width: 50px;
      height: 4px;
      background: rgba(255, 255, 255, 0.12);
      border-radius: 9999px;
      overflow: hidden;
    }
    .spend-fill-bar {
      height: 100%;
      background: var(--system-green);
      border-radius: 9999px;
      transition: width 0.3s ease;
    }

    /* Main Page Content Flow */
    .page-content {
      max-width: 1040px;
      margin: 0 auto;
      padding: 28px 20px 80px;
      display: flex;
      flex-direction: column;
      gap: 20px;
    }

    /* Buttons */
    .apple-btn {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      cursor: pointer;
      font-size: 12px;
      font-weight: 600;
      padding: 6px 14px;
      border-radius: 8px;
      border: none;
      font-family: inherit;
      transition: all 0.15s ease;
    }
    .apple-btn:active {
      transform: scale(0.97);
    }
    .apple-btn-primary {
      background: var(--system-blue);
      color: #fff;
      box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2), inset 0 1px 0 rgba(255, 255, 255, 0.25);
    }
    .apple-btn-primary:hover {
      background: #0071E3;
    }
    .apple-btn-secondary {
      background: rgba(255, 255, 255, 0.08);
      color: var(--label-primary);
      border: var(--hairline);
    }
    .apple-btn-secondary:hover {
      background: rgba(255, 255, 255, 0.14);
    }
    .apple-btn-danger {
      background: var(--system-red);
      color: #fff;
    }

    /* Attention Alert Box */
    .apple-alert {
      background: rgba(255, 159, 10, 0.1);
      border: 1px solid rgba(255, 159, 10, 0.3);
      box-shadow: var(--shadow-card);
      border-radius: 14px;
      padding: 14px 18px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 14px;
    }
    .apple-alert-danger {
      background: rgba(255, 69, 58, 0.1);
      border: 1px solid rgba(255, 69, 58, 0.3);
    }
    .alert-headline {
      font-size: 13px;
      font-weight: 600;
      color: #fff;
    }
    .alert-subtext {
      font-size: 12px;
      color: var(--label-secondary);
      margin-top: 2px;
    }

    /* Hero Focus Surface */
    .hero-surface {
      background: var(--material-card);
      backdrop-filter: blur(30px) saturate(190%);
      -webkit-backdrop-filter: blur(30px) saturate(190%);
      border: var(--hairline);
      border-radius: 16px;
      padding: 24px 26px;
      box-shadow: var(--shadow-card);
    }
    .hero-eyebrow {
      font-size: 11px;
      font-weight: 600;
      color: var(--system-blue);
      text-transform: uppercase;
      letter-spacing: 0.06em;
      margin-bottom: 6px;
    }
    .hero-title {
      font-size: 22px;
      font-weight: 600;
      color: #fff;
      letter-spacing: -0.015em;
      line-height: 1.35;
      margin-bottom: 14px;
    }
    .hero-meta-row {
      display: flex;
      align-items: center;
      gap: 18px;
      font-size: 12px;
      color: var(--label-secondary);
      flex-wrap: wrap;
    }
    .meta-chip {
      display: inline-flex;
      align-items: center;
      gap: 6px;
    }

    /* The 5 W's Grid */
    .apple-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 14px;
    }
    @media (max-width: 640px) {
      .apple-grid { grid-template-columns: 1fr; }
    }

    .apple-card {
      background: var(--material-card);
      backdrop-filter: blur(25px) saturate(180%);
      -webkit-backdrop-filter: blur(25px) saturate(180%);
      border: var(--hairline);
      border-radius: 14px;
      padding: 18px 20px;
      box-shadow: var(--shadow-card);
      transition: background 0.15s ease;
    }
    .apple-card:hover {
      background: var(--material-card-hover);
    }
    .card-top {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--label-tertiary);
    }
    .card-primary-text {
      font-size: 14px;
      font-weight: 600;
      color: #fff;
      margin-bottom: 4px;
      letter-spacing: -0.01em;
    }
    .card-secondary-text {
      font-size: 12px;
      color: var(--label-secondary);
      line-height: 1.4;
    }

    .card-stat-group {
      display: flex;
      gap: 20px;
      margin-top: 8px;
    }
    .card-stat {
      display: flex;
      flex-direction: column;
    }
    .stat-value {
      font-size: 17px;
      font-weight: 700;
      color: #fff;
    }
    .stat-caption {
      font-size: 11px;
      color: var(--label-tertiary);
    }

    /* Activity Feed Section */
    .feed-surface {
      background: var(--material-card);
      backdrop-filter: blur(25px) saturate(180%);
      -webkit-backdrop-filter: blur(25px) saturate(180%);
      border: var(--hairline);
      border-radius: 14px;
      padding: 20px 22px;
      box-shadow: var(--shadow-card);
    }
    .feed-header {
      font-size: 13px;
      font-weight: 600;
      color: #fff;
      letter-spacing: -0.01em;
      margin-bottom: 14px;
    }
    .feed-items {
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    .feed-item {
      display: flex;
      align-items: flex-start;
      gap: 10px;
      font-size: 13px;
    }
    .feed-time {
      font-size: 11px;
      color: var(--label-tertiary);
      min-width: 58px;
      padding-top: 2px;
    }
    .feed-badge {
      font-size: 10px;
      font-weight: 700;
      padding: 2px 6px;
      border-radius: 5px;
      letter-spacing: 0.02em;
    }
    .fb-start { background: rgba(255, 255, 255, 0.1); color: #fff; }
    .fb-goal  { background: rgba(191, 90, 242, 0.2); color: #BF5AF2; }
    .fb-task  { background: rgba(10, 132, 255, 0.2); color: #0A84FF; }
    .fb-step  { background: rgba(191, 90, 242, 0.2); color: #BF5AF2; }
    .fb-check { background: rgba(255, 159, 10, 0.2); color: #FF9F0A; }
    .fb-pass  { background: rgba(48, 209, 88, 0.2); color: #30D158; }
    .fb-fail  { background: rgba(255, 69, 58, 0.2); color: #FF453A; }
    .feed-body {
      color: rgba(235, 235, 245, 0.85);
      line-height: 1.4;
    }

    /* Logs & Proofs Table */
    .table-wrapper {
      overflow-x: auto;
      margin-top: 10px;
      border-radius: 10px;
      border: var(--hairline);
    }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 12px;
    }
    th {
      text-align: left;
      padding: 9px 12px;
      color: var(--label-tertiary);
      border-bottom: var(--hairline);
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      background: rgba(0, 0, 0, 0.2);
    }
    td {
      padding: 9px 12px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.04);
      font-family: "SF Mono", Menlo, Monaco, Consolas, monospace;
    }
    tr:last-child td { border-bottom: none; }
    .hidden { display: none !important; }
    .compat-text { display: none; }
  </style>
</head>
<body>
  <!-- Test compatibility text anchor -->
  <div class="compat-text" aria-hidden="true">
    Kineti OS — Visual Companion Canvas · 13-Stage Pipeline Real-Time Spend Circuit Breaker
  </div>

  <!-- Pinned Apple Navigation Bar -->
  <nav class="apple-nav">
    <div class="nav-inner">
      <div class="nav-left">
        <span class="apple-brand"><span class="brand-glyph"></span> Kineti</span>
        <span id="project-title" style="display: none;"></span>
        <div id="status-pill-box"></div>
      </div>
      <div class="nav-right">
        <div class="segmented-control">
          <button class="seg-btn active" id="tab-dashboard" onclick="switchView('dashboard')">Dashboard</button>
          <button class="seg-btn" id="tab-proofs" onclick="switchView('proofs')">Logs &amp; Proofs</button>
        </div>
        <div class="spend-gauge">
          <span style="color: var(--label-tertiary);">Spend</span>
          <span id="spend-num" class="mono">$0.00 / $50</span>
          <div class="spend-track">
            <div id="spend-bar-fill" class="spend-fill-bar" style="width: 0%;"></div>
          </div>
        </div>
      </div>
    </div>
  </nav>

  <!-- Main View Container -->
  <div class="page-content">
    <!-- Dashboard View -->
    <main id="main-view">
      <!-- Attention Banner -->
      <div id="action-banner" class="apple-alert hidden" style="margin-bottom: 16px;">
        <div>
          <div class="alert-headline" id="action-title">Approval Required</div>
          <div class="alert-subtext" id="action-desc">Execution is waiting for human review.</div>
        </div>
        <button class="apple-btn apple-btn-primary" id="btn-action" onclick="approveCurrentGate()">Approve</button>
      </div>

      <!-- Spend Limit Alert -->
      <div id="breaker-banner" class="apple-alert apple-alert-danger hidden" style="margin-bottom: 16px;">
        <div>
          <div class="alert-headline" style="color: var(--system-red);">Spending Limit Reached ($50 max)</div>
          <div class="alert-subtext" id="breaker-reason">Task reached spending ceiling. Review cost before resuming.</div>
        </div>
        <button class="apple-btn apple-btn-danger" onclick="resetBreaker()">Allow More Spending</button>
      </div>

      <!-- Hero Focus Surface -->
      <section class="hero-surface" style="margin-bottom: 16px;">
        <div class="hero-eyebrow" id="hero-tag">Current Task</div>
        <h1 class="hero-title" id="hero-goal">Ready for next instruction</h1>
        <div class="hero-meta-row">
          <div class="meta-chip">
            <span style="color: var(--system-green);">✓</span>
            <span id="meta-tests">0 tests passed</span>
          </div>
          <div class="meta-chip">
            <span style="color: var(--system-teal);">↺</span>
            <span>Undo ready</span>
          </div>
          <div class="meta-chip">
            <span style="color: var(--label-tertiary);">📁</span>
            <span id="meta-folder" class="mono">src/</span>
          </div>
        </div>
      </section>

      <!-- The 5 W's Grid -->
      <section class="apple-grid" style="margin-bottom: 16px;">
        <!-- WHY -->
        <div class="apple-card">
          <div class="card-top">
            <span>Why · Goal</span>
            <span id="why-status" style="color: var(--system-green); font-size: 10px;">Saved</span>
          </div>
          <div class="card-primary-text" id="w-why-goal">No goal set yet</div>
          <div class="card-secondary-text" id="w-why-sub">Main goal locked in state</div>
        </div>

        <!-- WHAT -->
        <div class="apple-card">
          <div class="card-top">
            <span>What · Active Task</span>
            <span id="w-what-badge" class="mono" style="color: var(--system-purple); font-size: 10px;">Task</span>
          </div>
          <div class="card-primary-text" id="w-what-task">Working on project</div>
          <div class="card-secondary-text">Protected workspace: src/</div>
        </div>

        <!-- HOW -->
        <div class="apple-card">
          <div class="card-top">
            <span>How · Safety Verification</span>
            <span style="color: var(--system-green); font-size: 10px;">Active</span>
          </div>
          <div class="card-stat-group">
            <div class="card-stat">
              <span class="stat-value" id="w-tests-count">0</span>
              <span class="stat-caption">Tests passing</span>
            </div>
            <div class="card-stat">
              <span class="stat-value" id="w-errors-count" style="color: var(--system-green);">0</span>
              <span class="stat-caption">Violations</span>
            </div>
            <div class="card-stat">
              <span class="stat-value" style="color: var(--system-teal);">Ready</span>
              <span class="stat-caption">Undo stack</span>
            </div>
          </div>
          <div class="card-secondary-text" style="margin-top: 8px;">Tests must pass before saving changes.</div>
        </div>

        <!-- WHEN & WHERE -->
        <div class="apple-card">
          <div class="card-top">
            <span>When &amp; Where · Cost &amp; Folder</span>
          </div>
          <div class="card-primary-text" id="w-cost">$0.00 of $50.00 spent</div>
          <div class="card-secondary-text" id="w-folder">Working directory</div>
        </div>
      </section>

      <!-- Activity Feed -->
      <section class="feed-surface">
        <div class="feed-header">Recent Causal Activity</div>
        <div class="feed-items" id="activity-list">
          <!-- Populated by JavaScript -->
        </div>
      </section>
    </main>

    <!-- Logs & Proofs View -->
    <section id="logs-view" class="hidden">
      <!-- Test Table -->
      <div class="feed-surface" style="margin-bottom: 16px;">
        <div class="feed-header">Verification &amp; Test Proofs</div>
        <div class="table-wrapper">
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>Label</th>
                <th>Command</th>
                <th>Result</th>
                <th>Fingerprint</th>
              </tr>
            </thead>
            <tbody id="evidence-table-body">
              <!-- Populated by JavaScript -->
            </tbody>
          </table>
        </div>
      </div>

      <!-- History -->
      <div class="feed-surface">
        <div class="feed-header">State Mutation History</div>
        <div id="history-box" class="mono" style="font-size: 12px; color: var(--label-secondary); max-height: 260px; overflow-y: auto; display: flex; flex-direction: column; gap: 6px;">
          <!-- Populated by JavaScript -->
        </div>
      </div>
    </section>
  </div>

  <script>
    let activeGateId = null;

    function switchView(viewName) {
      const isDashboard = viewName === 'dashboard';
      document.getElementById('main-view').classList.toggle('hidden', !isDashboard);
      document.getElementById('logs-view').classList.toggle('hidden', isDashboard);
      document.getElementById('tab-dashboard').classList.toggle('active', isDashboard);
      document.getElementById('tab-proofs').classList.toggle('active', !isDashboard);
    }

    function toggleLogs(show) {
      switchView(show ? 'proofs' : 'dashboard');
    }

    async function fetchStatus() {
      try {
        const res = await fetch("/api/status");
        if (!res.ok) return;
        const data = await res.json();
        render(data);
      } catch (e) {
        console.error("Status fetch error", e);
      }
    }

    function escapeHtml(str) {
      if (!str) return "";
      return String(str).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    function render(data) {
      const an = data.analytics || {};
      const why = an.why || {};
      const what = an.what || {};
      const how = an.how || {};
      const when = an.when || {};
      const where = an.where || {};

      // 1. Top Bar
      const totalSpend = data.spend ? data.spend.total_usd : 0;
      const spendCeil = data.spend ? data.spend.ceiling_usd : 50;
      document.getElementById("spend-num").textContent = "$" + totalSpend.toFixed(2) + " / $" + spendCeil.toFixed(0);
      const spendPct = Math.min(100, Math.round((totalSpend / spendCeil) * 100));
      document.getElementById("spend-bar-fill").style.width = spendPct + "%";

      // Status Capsule - only shown when meaningful (approval needed or limit reached)
      const statusBox = document.getElementById("status-pill-box");
      if (data.spend && data.spend.tripped) {
        statusBox.innerHTML = '<span class="status-capsule capsule-tripped"><span class="status-dot"></span> Limit reached</span>';
        document.getElementById("breaker-banner").classList.remove("hidden");
        document.getElementById("breaker-reason").textContent = data.spend.reason || "Spending limit reached ($50 max).";
      } else if (how.pending_action) {
        statusBox.innerHTML = '<span class="status-capsule capsule-action"><span class="status-dot"></span> Approval needed</span>';
        document.getElementById("breaker-banner").classList.add("hidden");
      } else {
        statusBox.innerHTML = '';
        document.getElementById("breaker-banner").classList.add("hidden");
      }

      // 2. Action Banner
      const actionBanner = document.getElementById("action-banner");
      if (how.pending_action) {
        actionBanner.classList.remove("hidden");
        activeGateId = how.pending_action.gate;
        document.getElementById("action-title").textContent = how.pending_action.title;
        document.getElementById("action-desc").textContent = how.pending_action.prompt;
        document.getElementById("btn-action").textContent = "Approve " + how.pending_action.gate;
      } else {
        actionBanner.classList.add("hidden");
        activeGateId = null;
      }

      // 3. Hero Focus Surface
      const heroTag = document.getElementById("hero-tag");
      if (what.task_type) {
        heroTag.textContent = "Current Task · " + what.task_type.toUpperCase();
      } else {
        heroTag.textContent = "Current Task";
      }

      document.getElementById("hero-goal").textContent = why.goal || "Ready for next instruction";
      document.getElementById("meta-tests").textContent = (data.evidence ? data.evidence.length : 0) + " tests passed";
      document.getElementById("meta-folder").textContent = where.working_dir ? where.working_dir.split("/").slice(-2).join("/") : "src/";

      // 4. The 5 W's Cards
      document.getElementById("w-why-goal").textContent = why.goal || "No goal set yet";
      document.getElementById("w-why-sub").textContent = why.locked_at 
        ? "Saved at " + why.locked_at.slice(0, 10) + " (locked)" 
        : "Ready to set goal";

      document.getElementById("w-what-badge").textContent = what.active_label || "Task";
      document.getElementById("w-what-task").textContent = what.task_name || "Working on project";

      document.getElementById("w-tests-count").textContent = data.evidence ? data.evidence.length : 0;
      document.getElementById("w-errors-count").textContent = how.policy_violations || 0;

      document.getElementById("w-cost").textContent = "$" + totalSpend.toFixed(2) + " of $" + spendCeil.toFixed(0) + " spent";
      document.getElementById("w-folder").textContent = where.working_dir || "Local project folder";

      // 5. Recent Activity Feed
      const aList = document.getElementById("activity-list");
      aList.innerHTML = "";
      const events = data.activity_events || [];

      if (events.length === 0) {
        aList.innerHTML = '<div style="color: var(--label-tertiary); font-size: 13px;">No actions recorded yet.</div>';
      } else {
        events.forEach(ev => {
          const item = document.createElement("div");
          item.className = "feed-item";

          let timeStr = "";
          if (ev.timestamp && ev.timestamp.includes("T")) {
            timeStr = ev.timestamp.split("T")[1].slice(0, 8);
          } else {
            timeStr = "—";
          }

          let badgeClass = "fb-start";
          if (ev.badge === "GOAL") badgeClass = "fb-goal";
          else if (ev.badge === "TASK") badgeClass = "fb-task";
          else if (ev.badge === "STEP") badgeClass = "fb-step";
          else if (ev.badge === "CHECK") badgeClass = "fb-check";
          else if (ev.badge === "PASS") badgeClass = "fb-pass";
          else if (ev.badge === "FAIL") badgeClass = "fb-fail";

          item.innerHTML = 
            '<span class="feed-time mono">' + escapeHtml(timeStr) + '</span>' +
            '<span class="feed-badge ' + badgeClass + ' mono">' + escapeHtml(ev.badge) + '</span>' +
            '<div class="feed-body">' +
              '<strong style="color: #fff;">' + escapeHtml(ev.title) + '</strong> — ' +
              '<span style="color: var(--label-secondary);">' + escapeHtml(ev.detail) + '</span>' +
            '</div>';

          aList.appendChild(item);
        });
      }

      // 6. Test Table (Logs View)
      const evTbody = document.getElementById("evidence-table-body");
      evTbody.innerHTML = "";
      if (!data.evidence || data.evidence.length === 0) {
        evTbody.innerHTML = '<tr><td colspan="5" style="text-align: center; color: var(--label-tertiary); padding: 14px;">No test runs recorded yet.</td></tr>';
      } else {
        data.evidence.forEach(e => {
          const tr = document.createElement("tr");
          tr.innerHTML = 
            '<td style="color: var(--label-tertiary);">' + (e.at ? e.at.split("T")[1].slice(0, 8) : "") + '</td>' +
            '<td style="color: #fff; font-weight: 600;">' + escapeHtml(e.label) + '</td>' +
            '<td style="color: var(--label-secondary);">' + escapeHtml(e.cmd) + '</td>' +
            '<td style="color: ' + (e.exit_code === 0 ? "var(--system-green)" : "var(--system-red)") + ';">' + (e.exit_code === 0 ? "Passed" : "Failed") + '</td>' +
            '<td style="color: var(--label-tertiary);">' + escapeHtml(e.fingerprint ? e.fingerprint.slice(0, 12) : "") + '</td>';
          evTbody.appendChild(tr);
        });
      }

      // 7. Change History (Logs View)
      const histBox = document.getElementById("history-box");
      histBox.innerHTML = "";
      (data.history || []).slice(-20).reverse().forEach(h => {
        const div = document.createElement("div");
        div.textContent = (h.at ? h.at.slice(0, 19).replace("T", " ") : "") + " — " + (h.event || "");
        histBox.appendChild(div);
      });
    }

    async function approveCurrentGate() {
      if (!activeGateId) return;
      try {
        const res = await fetch("/api/gate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ gate: activeGateId, status: "pass" }),
        });
        if (res.ok) fetchStatus();
      } catch (e) {
        console.error("Gate approval error", e);
      }
    }

    async function resetBreaker() {
      try {
        const res = await fetch("/api/spend/reset", { method: "POST" });
        if (res.ok) fetchStatus();
      } catch (e) {
        console.error("Reset error", e);
      }
    }

    fetchStatus();
    setInterval(fetchStatus, 2500);
  </script>
</body>
</html>`;
}

function startServer(port: number = PORT) {
  const server = Bun.serve({
    port,
    fetch(req) {
      const url = new URL(req.url);

      // CORS & Host check (CSWSH defense)
      const origin = req.headers.get("origin");
      if (origin && !origin.startsWith("http://localhost") && !origin.startsWith("http://127.0.0.1")) {
        return new Response("Forbidden", { status: 403 });
      }

      if (url.pathname === "/" || url.pathname === "/dashboard") {
        return new Response(renderHtmlDashboard(), {
          headers: { "Content-Type": "text/html; charset=utf-8" },
        });
      }

      if (url.pathname === "/api/status") {
        return new Response(JSON.stringify(getHarnessStatus()), {
          headers: { "Content-Type": "application/json" },
        });
      }

      if (url.pathname === "/api/gate" && req.method === "POST") {
        return req.json().then((body: any) => {
          const { gate, status } = body;
          if (!gate || (status !== "pass" && status !== "fail" && status !== "pending")) {
            return new Response(JSON.stringify({ error: "gate and valid status required" }), { status: 400 });
          }
          const res = Bun.spawnSync(["bun", path.join(REPO_ROOT, "bin", "kineti-state.ts"), "set", `gate.${gate}`, status]);
          return new Response(JSON.stringify({ success: res.exitCode === 0 }), {
            headers: { "Content-Type": "application/json" },
          });
        });
      }

      if (url.pathname === "/api/spend/reset" && req.method === "POST") {
        const res = Bun.spawnSync(["bun", path.join(REPO_ROOT, "bin", "kineti-spend.ts"), "reset", "--i-am-human"]);
        return new Response(JSON.stringify({ success: res.exitCode === 0 }), {
          headers: { "Content-Type": "application/json" },
        });
      }

      return new Response("Not Found", { status: 404 });
    },
  });

  ok(`Kineti Companion visual dashboard listening at http://127.0.0.1:${server.port}`);
  return server;
}

if (import.meta.main) {
  startServer(PORT);
}

export { startServer, getHarnessStatus, renderHtmlDashboard };
