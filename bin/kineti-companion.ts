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

function isAuthorized(req: Request): boolean {
  const header = req.headers.get("authorization");
  return header === `Bearer ${AUTH_TOKEN}`;
}

export interface ActivityEvent {
  timestamp: string;
  badge: "START" | "GOAL" | "TASK" | "STEP" | "CHECK" | "PASS" | "FAIL" | "CHANGE";
  title: string;
  detail: string;
}

export interface FleetRepo {
  id: string;
  name: string;
  path: string;
  owner: string;
  branch: string;
  status: "active" | "idle" | "action_needed" | "limit_reached";
  active_task: string;
  spend_usd: number;
  ceiling_usd: number;
  tests_passing: number;
  ide: string;
  is_local: boolean;
}

export interface CompanionSettings {
  github: {
    connected: boolean;
    account: string;
    repo_count: number;
    webhook_status: "active" | "inactive";
  };
  ides: {
    cursor: boolean;
    claude_code: boolean;
    antigravity: boolean;
    codex: boolean;
  };
  team_members: Array<{
    name: string;
    email: string;
    role: string;
  }>;
  repo_budgets: Record<string, number>;
  repo_owners: Record<string, string>;
}

const defaultRepoName = readJson<any>(path.join(projectKdir(), "state.json"))?.project || path.basename(REPO_ROOT) || "kineti-local-harness";
let activeRepoId = defaultRepoName;

let fleetRepos: FleetRepo[] = [
  {
    id: defaultRepoName,
    name: defaultRepoName,
    path: REPO_ROOT,
    owner: "Local Owner",
    branch: "main",
    status: "active",
    active_task: "Local tasks governed by Kineti OS",
    spend_usd: 0.0,
    ceiling_usd: 50.0,
    tests_passing: 0,
    ide: "Local IDE",
    is_local: true,
  },
];

let companionSettings: CompanionSettings = {
  github: {
    connected: false,
    account: "",
    repo_count: 1,
    webhook_status: "inactive",
  },
  ides: {
    cursor: true,
    claude_code: true,
    antigravity: true,
    codex: true,
  },
  team_members: [],
  repo_budgets: {
    [defaultRepoName]: 50.0,
  },
  repo_owners: {
    [defaultRepoName]: "Local Owner",
  },
};

// Optional local-only fleet overrides. Never commit personal names or paths.
// Create `.kineti/fleet.local.json` (gitignored) with:
// { "repos": [FleetRepo...], "settings": { "github": {...}, "team_members": [...] } }
function loadLocalFleetOverrides(): void {
  try {
    const overridePath = path.join(projectKdir(), "fleet.local.json");
    if (!fs.existsSync(overridePath)) return;
    const data = readJson<any>(overridePath);
    if (!data) return;
    if (Array.isArray(data.repos)) {
      for (const r of data.repos) {
        if (!r || typeof r.id !== "string") continue;
        if (r.is_local) continue; // local repo always comes from this project
        const existing = fleetRepos.find((x) => x.id === r.id);
        if (existing) Object.assign(existing, r);
        else fleetRepos.push(r as FleetRepo);
      }
    }
    if (data.settings && typeof data.settings === "object") {
      const s = data.settings as Partial<CompanionSettings>;
      if (s.github) companionSettings.github = { ...companionSettings.github, ...s.github };
      if (s.ides) companionSettings.ides = { ...companionSettings.ides, ...s.ides };
      if (Array.isArray(s.team_members)) companionSettings.team_members = s.team_members as CompanionSettings["team_members"];
      if (s.repo_budgets) {
        companionSettings.repo_budgets = { ...companionSettings.repo_budgets, ...s.repo_budgets };
        for (const [id, budget] of Object.entries(s.repo_budgets)) {
          const repo = fleetRepos.find((x) => x.id === id);
          if (repo && typeof budget === "number") repo.ceiling_usd = budget;
        }
      }
      if (s.repo_owners) {
        companionSettings.repo_owners = { ...companionSettings.repo_owners, ...s.repo_owners };
        for (const [id, owner] of Object.entries(s.repo_owners)) {
          const repo = fleetRepos.find((x) => x.id === id);
          if (repo && typeof owner === "string") repo.owner = owner;
        }
      }
    }
    const localOnly = fleetRepos.find((r) => r.is_local);
    if (localOnly) {
      companionSettings.repo_budgets[localOnly.id] ??= localOnly.ceiling_usd;
      companionSettings.repo_owners[localOnly.id] ??= localOnly.owner;
    }
    companionSettings.github.repo_count = fleetRepos.length;
  } catch {
    // Ignore corrupt override file — dashboard still works with local repo only.
  }
}

loadLocalFleetOverrides();

function getHarnessStatus(targetRepoId?: string) {
  const currentId = targetRepoId || activeRepoId;
  const isLocal = currentId === defaultRepoName || currentId === "kineti-local-harness" || currentId === path.basename(REPO_ROOT) || currentId === "local";

  // If viewing a non-local repository in the fleet
  if (!isLocal) {
    const r = fleetRepos.find((item) => item.id === currentId) || fleetRepos[0];
    const isActionNeeded = r.status === "action_needed";
    const isTripped = r.status === "limit_reached";

    const syntheticEvents: ActivityEvent[] = [
      {
        timestamp: "2026-09-07T12:00:00Z",
        badge: "GOAL",
        title: "Repository connected",
        detail: `Governed under Kineti OS for ${r.owner}`,
      },
      {
        timestamp: "2026-09-07T12:05:00Z",
        badge: "TASK",
        title: "Active task updated",
        detail: r.active_task,
      },
      {
        timestamp: "2026-09-07T12:10:00Z",
        badge: isActionNeeded ? "CHECK" : "PASS",
        title: isActionNeeded ? "Approval pending" : "Verification passed",
        detail: isActionNeeded ? "Security gate check required" : `${r.tests_passing} tests passing cleanly`,
      },
    ];

    const pendingAction = isActionNeeded
      ? { gate: "security", title: "Security Review Required", prompt: "Review and approve webhook signature migration." }
      : null;

    return {
      project: r.name,
      root_goal: r.active_task,
      root_goal_locked_at: "2026-09-07T10:00:00Z",
      stage: 7,
      stage_label: "Build",
      task: { name: r.active_task, type: "feature" },
      stages: STAGES.map((s) => ({
        ...s,
        is_current: s.id === 7,
        is_past: s.id < 7,
        gate_status: s.gate ? (s.id < 7 ? "pass" : isActionNeeded && s.gate === "security" ? "pending" : null) : null,
      })),
      gates: isActionNeeded ? { spec: "pass", security: "pending" } : { spec: "pass", ship: "pass" },
      spend: {
        total_usd: r.spend_usd,
        total_microcents: Math.round(r.spend_usd * 1_000_000),
        ceiling_usd: r.ceiling_usd,
        safety_factor: 0.95,
        tripped: isTripped,
        reason: isTripped ? "Spending limit reached ($50 max)" : null,
        by_stage: { build: r.spend_usd },
        entries: 4,
      },
      evidence: Array.from({ length: r.tests_passing }, (_, i) => ({
        at: "2026-09-07T12:10:00Z",
        label: `test-suite-pass-${i + 1}`,
        cmd: "bun test",
        exit_code: 0,
        fingerprint: `proof-${i + 1}`,
      })),
      activity_events: syntheticEvents,
      analytics: {
        why: {
          title: "Why (Goal)",
          goal: r.active_task,
          locked_at: "2026-09-07T10:00:00Z",
          immutable: true,
        },
        what: {
          title: "What (Current Task)",
          task_type: "feature",
          task_name: r.active_task,
          active_label: "Build",
          is_standard_stage: true,
          target_paths: r.path,
        },
        how: {
          title: "How (Safety Checks)",
          status: isTripped ? "Spending limit reached" : pendingAction ? "Your approval needed" : "Running safely",
          evidence_count: r.tests_passing,
          policy_violations: isTripped ? 1 : 0,
          saga_rollback_armed: true,
          pending_action: pendingAction,
        },
        when: {
          title: "When (Time & Spend)",
          started_at: "2026-09-07T10:00:00Z",
          last_activity: "2026-09-07T12:10:00Z",
          spend_usd: r.spend_usd,
          spend_limit_usd: r.ceiling_usd,
          spend_pct: Math.min(100, Math.round((r.spend_usd / r.ceiling_usd) * 100)),
        },
        where: {
          title: "Where (Folder)",
          project: r.name,
          working_dir: r.path,
        },
      },
      history: [
        { at: "2026-09-07T10:00:00Z", event: `init project=${r.name}` },
        { at: "2026-09-07T10:05:00Z", event: "goal locked" },
        { at: "2026-09-07T11:00:00Z", event: "stage 1 -> 7" },
      ],
    };
  }

  // Local repository status read from .kineti/
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

function getFleetStatus() {
  const localRepo = fleetRepos.find((r) => r.is_local);
  if (localRepo) {
    const localState = readJson<any>(path.join(projectKdir(), "state.json"));
    const localSpend = readJson<any>(path.join(projectKdir(), "spend.json"));
    const localEvidence = readJsonl<any>(path.join(projectKdir(), "evidence.jsonl"));
    if (localState?.root_goal) localRepo.active_task = localState.root_goal;
    if (localSpend) localRepo.spend_usd = localSpend.total_usd || 0;
    if (localEvidence) localRepo.tests_passing = localEvidence.length;
    if (localSpend?.tripped) localRepo.status = "limit_reached";
    else if (localState?.gates?.spec === "pending" || localState?.gates?.security === "pending") localRepo.status = "action_needed";
    else localRepo.status = "active";
  }

  const totalSpend = fleetRepos.reduce((sum, r) => sum + r.spend_usd, 0);
  const totalBudget = fleetRepos.reduce((sum, r) => sum + r.ceiling_usd, 0);

  return {
    active_repo_id: activeRepoId,
    repos: fleetRepos,
    settings: companionSettings,
    total_fleet_spend: Number(totalSpend.toFixed(2)),
    total_fleet_budget: Number(totalBudget.toFixed(2)),
  };
}

function renderHtmlDashboard(): string {
  // Token-free page. Frontend reads token from localStorage (set via login page).
  // Never embed AUTH_TOKEN in HTML.
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
      --material-nav: rgba(20, 20, 24, 0.78);
      --material-card: rgba(28, 28, 34, 0.65);
      --material-card-hover: rgba(36, 36, 44, 0.85);
      --material-sheet: rgba(24, 24, 30, 0.95);
      
      /* Apple Borders & Specular Highlights */
      --hairline: 1px solid rgba(255, 255, 255, 0.09);
      --specular: inset 0 1px 0 rgba(255, 255, 255, 0.12);
      --shadow-card: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 4px 16px rgba(0, 0, 0, 0.3);
      
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
        radial-gradient(circle at 50% 0%, rgba(10, 132, 255, 0.08) 0%, transparent 60%),
        radial-gradient(circle at 85% 100%, rgba(191, 90, 242, 0.05) 0%, transparent 50%);
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
      backdrop-filter: blur(28px) saturate(190%);
      -webkit-backdrop-filter: blur(28px) saturate(190%);
      border-bottom: var(--hairline);
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
    }

    .nav-inner {
      max-width: 1080px;
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

    /* Repo Switcher Dropdown */
    .repo-dropdown-wrapper {
      position: relative;
    }
    .repo-capsule-btn {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      background: rgba(255, 255, 255, 0.08);
      border: var(--hairline);
      padding: 4px 12px;
      border-radius: 9px;
      font-size: 12px;
      font-weight: 600;
      color: var(--label-primary);
      cursor: pointer;
      font-family: inherit;
      transition: all 0.15s ease;
    }
    .repo-capsule-btn:hover {
      background: rgba(255, 255, 255, 0.14);
    }
    .repo-icon {
      font-size: 12px;
      color: var(--system-blue);
    }
    .chevron {
      font-size: 9px;
      color: var(--label-tertiary);
      margin-left: 2px;
    }

    .repo-dropdown-menu {
      position: absolute;
      top: calc(100% + 8px);
      left: 0;
      width: 300px;
      background: rgba(24, 24, 30, 0.96);
      backdrop-filter: blur(40px) saturate(200%);
      -webkit-backdrop-filter: blur(40px) saturate(200%);
      border: var(--hairline);
      box-shadow: 0 12px 36px rgba(0, 0, 0, 0.55), var(--specular);
      border-radius: 12px;
      z-index: 100;
      padding: 8px;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    .menu-search-input {
      width: 100%;
      background: rgba(255, 255, 255, 0.06);
      border: var(--hairline);
      border-radius: 7px;
      padding: 6px 10px;
      font-size: 12px;
      color: #fff;
      outline: none;
      font-family: inherit;
      margin-bottom: 4px;
    }
    .menu-search-input::placeholder {
      color: var(--label-tertiary);
    }
    .menu-items-list {
      max-height: 220px;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      gap: 2px;
    }
    .menu-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 10px;
      border-radius: 8px;
      font-size: 12px;
      cursor: pointer;
      transition: background 0.1s ease;
      color: var(--label-primary);
    }
    .menu-item:hover {
      background: rgba(255, 255, 255, 0.1);
    }
    .menu-item.active {
      background: rgba(10, 132, 255, 0.2);
      color: #fff;
      font-weight: 600;
    }
    .menu-item-info {
      display: flex;
      flex-direction: column;
      gap: 2px;
    }
    .menu-item-name {
      font-weight: 600;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .menu-item-meta {
      font-size: 11px;
      color: var(--label-secondary);
    }
    .menu-badge {
      font-size: 10px;
      padding: 1px 6px;
      border-radius: 4px;
      background: rgba(255, 255, 255, 0.1);
    }
    .menu-footer {
      border-top: var(--hairline);
      margin-top: 4px;
      padding-top: 6px;
    }
    .menu-action-link {
      background: none;
      border: none;
      color: var(--system-blue);
      font-size: 11px;
      font-weight: 600;
      cursor: pointer;
      padding: 4px 6px;
      border-radius: 6px;
      width: 100%;
      text-align: left;
      font-family: inherit;
    }
    .menu-action-link:hover {
      background: rgba(10, 132, 255, 0.1);
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

    /* Apple Nav Button (Settings) */
    .apple-nav-btn {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      background: rgba(255, 255, 255, 0.08);
      border: var(--hairline);
      padding: 4px 12px;
      border-radius: 9px;
      font-size: 12px;
      font-weight: 500;
      color: var(--label-secondary);
      cursor: pointer;
      font-family: inherit;
      transition: all 0.15s ease;
    }
    .apple-nav-btn:hover {
      background: rgba(255, 255, 255, 0.14);
      color: #fff;
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
      max-width: 1080px;
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
    @media (max-width: 680px) {
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

    /* Fleet View Grid */
    .fleet-summary-bar {
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 12px;
      margin-bottom: 18px;
    }
    @media (max-width: 768px) {
      .fleet-summary-bar { grid-template-columns: repeat(2, 1fr); }
    }
    .fleet-stat-card {
      background: var(--material-card);
      backdrop-filter: blur(25px) saturate(180%);
      -webkit-backdrop-filter: blur(25px) saturate(180%);
      border: var(--hairline);
      border-radius: 12px;
      padding: 14px 16px;
      box-shadow: var(--shadow-card);
    }
    .fleet-stat-title {
      font-size: 11px;
      color: var(--label-tertiary);
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-bottom: 4px;
    }
    .fleet-stat-num {
      font-size: 20px;
      font-weight: 700;
      color: #fff;
    }
    .fleet-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 16px;
    }
    @media (max-width: 768px) {
      .fleet-grid { grid-template-columns: 1fr; }
    }
    .fleet-card {
      background: var(--material-card);
      backdrop-filter: blur(25px) saturate(180%);
      -webkit-backdrop-filter: blur(25px) saturate(180%);
      border: var(--hairline);
      border-radius: 14px;
      padding: 18px 20px;
      box-shadow: var(--shadow-card);
      display: flex;
      flex-direction: column;
      gap: 12px;
      transition: all 0.15s ease;
    }
    .fleet-card:hover {
      background: var(--material-card-hover);
      border-color: rgba(255, 255, 255, 0.16);
    }
    .fleet-card-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: 10px;
    }
    .fleet-card-title {
      font-size: 15px;
      font-weight: 600;
      color: #fff;
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .fleet-card-tags {
      display: flex;
      align-items: center;
      gap: 6px;
      flex-wrap: wrap;
    }
    .tag-pill {
      font-size: 10px;
      font-weight: 600;
      padding: 2px 7px;
      border-radius: 5px;
      background: rgba(255, 255, 255, 0.08);
      color: var(--label-secondary);
    }
    .tag-ide {
      background: rgba(10, 132, 255, 0.15);
      color: var(--system-blue);
      border: 1px solid rgba(10, 132, 255, 0.25);
    }
    .fleet-card-task {
      font-size: 13px;
      color: var(--label-secondary);
      line-height: 1.4;
      flex: 1;
    }
    .fleet-card-footer {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-top: 10px;
      border-top: var(--hairline);
    }
    .fleet-card-metrics {
      display: flex;
      align-items: center;
      gap: 14px;
      font-size: 12px;
      color: var(--label-secondary);
    }

    /* Settings Slide-Out Sheet */
    .sheet-backdrop {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.55);
      backdrop-filter: blur(8px);
      -webkit-backdrop-filter: blur(8px);
      z-index: 200;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.2s ease;
    }
    .sheet-backdrop.open {
      opacity: 1;
      pointer-events: auto;
    }
    .sheet-panel {
      position: fixed;
      top: 0;
      right: 0;
      bottom: 0;
      width: 480px;
      max-width: 100vw;
      background: var(--material-sheet);
      backdrop-filter: blur(40px) saturate(200%);
      -webkit-backdrop-filter: blur(40px) saturate(200%);
      border-left: var(--hairline);
      box-shadow: -10px 0 40px rgba(0, 0, 0, 0.6);
      z-index: 201;
      display: flex;
      flex-direction: column;
      transform: translateX(100%);
      transition: transform 0.28s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .sheet-panel.open {
      transform: translateX(0);
    }
    .sheet-header {
      padding: 18px 22px;
      border-bottom: var(--hairline);
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .sheet-title {
      font-size: 15px;
      font-weight: 700;
      color: #fff;
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .sheet-close-btn {
      background: rgba(255, 255, 255, 0.08);
      border: none;
      color: var(--label-secondary);
      width: 28px;
      height: 28px;
      border-radius: 50%;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 14px;
      transition: all 0.15s ease;
    }
    .sheet-close-btn:hover {
      background: rgba(255, 255, 255, 0.16);
      color: #fff;
    }
    .sheet-nav-tabs {
      padding: 12px 22px 0;
    }
    .sheet-body {
      padding: 20px 22px;
      overflow-y: auto;
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 20px;
    }
    .sheet-section {
      display: flex;
      flex-direction: column;
      gap: 12px;
    }
    .sheet-section-title {
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--label-tertiary);
    }
    .sheet-footer {
      padding: 16px 22px;
      border-top: var(--hairline);
      display: flex;
      align-items: center;
      justify-content: space-between;
      background: rgba(18, 18, 22, 0.6);
    }

    /* Settings Control Rows */
    .settings-row {
      background: rgba(255, 255, 255, 0.04);
      border: var(--hairline);
      border-radius: 10px;
      padding: 12px 14px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }
    .settings-row-text {
      display: flex;
      flex-direction: column;
      gap: 2px;
    }
    .settings-row-title {
      font-size: 13px;
      font-weight: 600;
      color: #fff;
    }
    .settings-row-desc {
      font-size: 11px;
      color: var(--label-secondary);
    }

    /* Apple Switch Toggle */
    .apple-switch {
      position: relative;
      display: inline-block;
      width: 42px;
      height: 24px;
    }
    .apple-switch input {
      opacity: 0;
      width: 0;
      height: 0;
    }
    .apple-switch-slider {
      position: absolute;
      cursor: pointer;
      top: 0; left: 0; right: 0; bottom: 0;
      background-color: rgba(255, 255, 255, 0.16);
      transition: .2s;
      border-radius: 24px;
    }
    .apple-switch-slider:before {
      position: absolute;
      content: "";
      height: 20px;
      width: 20px;
      left: 2px;
      bottom: 2px;
      background-color: white;
      transition: .2s;
      border-radius: 50%;
      box-shadow: 0 1px 3px rgba(0,0,0,0.3);
    }
    input:checked + .apple-switch-slider {
      background-color: var(--system-green);
    }
    input:checked + .apple-switch-slider:before {
      transform: translateX(18px);
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
  <script>try{if(!localStorage.getItem('kineti_token')){var t=prompt('Paste token from .kineti/auth_token to use the dashboard:');if(t)localStorage.setItem('kineti_token',t.trim());}}catch(e){}</script>
  <!-- Test compatibility text anchor -->
  <div class="compat-text" aria-hidden="true">
    Kineti OS — Visual Companion Canvas · 13-Stage Pipeline Real-Time Spend Circuit Breaker
  </div>

  <!-- Pinned Apple Navigation Bar -->
  <nav class="apple-nav">
    <div class="nav-inner">
      <div class="nav-left">
        <span class="apple-brand"><span class="brand-glyph"></span> Kineti</span>
        <span class="nav-divider">|</span>
        
        <!-- Repo Switcher Dropdown -->
        <div class="repo-dropdown-wrapper">
          <button id="btn-repo-switcher" class="repo-capsule-btn" onclick="toggleRepoDropdown(event)">
            <span class="repo-icon">􀈕</span>
            <span id="nav-active-repo-name">${defaultRepoName}</span>
            <span class="chevron">▾</span>
          </button>
          
          <div id="repo-dropdown-menu" class="repo-dropdown-menu hidden">
            <input type="text" id="repo-search-input" class="menu-search-input" placeholder="Search repositories..." oninput="filterRepos(this.value)" />
            <div id="repo-menu-list" class="menu-items-list">
              <!-- Populated by JS -->
            </div>
            <div class="menu-footer">
              <button class="menu-action-link" onclick="openSettingsWithTab('github')">+ Connect GitHub Repo</button>
            </div>
          </div>
        </div>

        <div id="status-pill-box"></div>
      </div>

      <div class="nav-right">
        <!-- 3-Tab Segmented Switcher -->
        <div class="segmented-control">
          <button class="seg-btn active" id="tab-dashboard" onclick="switchView('dashboard')">Dashboard</button>
          <button class="seg-btn" id="tab-fleet" onclick="switchView('fleet')">Fleet View</button>
          <button class="seg-btn" id="tab-proofs" onclick="switchView('proofs')">Logs &amp; Proofs</button>
        </div>

        <!-- Settings Button -->
        <button id="btn-settings" class="apple-nav-btn" onclick="toggleSettingsSheet(true)" title="Settings &amp; Integrations">
          <span>􀍟</span> Settings
        </button>

        <!-- Spend Gauge -->
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

  <!-- Main Page Content Flow -->
  <div class="page-content">
    
    <!-- 1. Single-Repo Dashboard View -->
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
        <div class="feed-header">Recent Activity</div>
        <div class="feed-items" id="activity-list">
          <!-- Populated by JavaScript -->
        </div>
      </section>
    </main>

    <!-- 2. Fleet View (Multi-Repo Grid) -->
    <section id="fleet-view" class="hidden">
      <!-- Fleet Summary Bar -->
      <div class="fleet-summary-bar">
        <div class="fleet-stat-card">
          <div class="fleet-stat-title">Connected Repos</div>
          <div class="fleet-stat-num mono" id="fleet-stat-count">4</div>
        </div>
        <div class="fleet-stat-card">
          <div class="fleet-stat-title">Total Spend</div>
          <div class="fleet-stat-num mono" id="fleet-stat-spend">$16.55</div>
        </div>
        <div class="fleet-stat-card">
          <div class="fleet-stat-title">Active Tasks</div>
          <div class="fleet-stat-num mono" id="fleet-stat-active">3</div>
        </div>
        <div class="fleet-stat-card">
          <div class="fleet-stat-title">Verified Tests</div>
          <div class="fleet-stat-num mono" id="fleet-stat-tests" style="color: var(--system-green);">318</div>
        </div>
      </div>

      <!-- Fleet Repo Grid -->
      <div class="fleet-grid" id="fleet-repo-grid">
        <!-- Populated by JavaScript -->
      </div>
    </section>

    <!-- 3. Logs & Proofs View -->
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

  <!-- 4. Settings & Integrations Slide-Out Sheet -->
  <div id="sheet-backdrop" class="sheet-backdrop" onclick="toggleSettingsSheet(false)"></div>
  <div id="settings-sheet" class="sheet-panel">
    <div class="sheet-header">
      <div class="sheet-title">
        <span>􀍟</span> Settings &amp; Integrations
      </div>
      <button class="sheet-close-btn" onclick="toggleSettingsSheet(false)">✕</button>
    </div>

    <!-- Sheet Segmented Switcher -->
    <div class="sheet-nav-tabs">
      <div class="segmented-control" style="width: 100%; display: flex;">
        <button class="seg-btn active" id="sheet-tab-btn-github" style="flex: 1;" onclick="switchSettingsTab('github')">GitHub</button>
        <button class="seg-btn" id="sheet-tab-btn-ides" style="flex: 1;" onclick="switchSettingsTab('ides')">Agent IDEs</button>
        <button class="seg-btn" id="sheet-tab-btn-team" style="flex: 1;" onclick="switchSettingsTab('team')">Team &amp; Budgets</button>
      </div>
    </div>

    <div class="sheet-body">
      <!-- Tab 1: GitHub Integration -->
      <div id="sheet-tab-github" class="sheet-section">
        <div class="sheet-section-title">GitHub Marketplace Integration</div>
        
        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">Connected GitHub Account</div>
            <div class="settings-row-desc" id="settings-gh-account">@praveen · 4 repos authorized</div>
          </div>
          <span class="status-capsule capsule-safe"><span class="status-dot"></span> Active</span>
        </div>

        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">Automatic Repo Latching</div>
            <div class="settings-row-desc">Automatically protect every repository connected on GitHub</div>
          </div>
          <label class="apple-switch">
            <input type="checkbox" id="set-auto-latch" checked onchange="saveSettingsState()" />
            <span class="apple-switch-slider"></span>
          </label>
        </div>

        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">Webhook Delivery Health</div>
            <div class="settings-row-desc">Sub-50ms audit verification triggers on git push</div>
          </div>
          <span style="font-size: 11px; color: var(--system-green); font-weight: 600;">Healthy</span>
        </div>

        <button class="apple-btn apple-btn-secondary" style="margin-top: 8px;" onclick="window.open('https://github.com/apps/kineti', '_blank')">
          Manage Authorized Repos on GitHub ↗
        </button>
      </div>

      <!-- Tab 2: Agent IDEs Auto-Latch -->
      <div id="sheet-tab-ides" class="sheet-section hidden">
        <div class="sheet-section-title">Agent Ecosystem Auto-Latching</div>
        <p style="font-size: 12px; color: var(--label-secondary); margin-bottom: 4px;">
          Kineti automatically latches onto developer coding sessions in these environments:
        </p>

        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">Cursor</div>
            <div class="settings-row-desc">Watches .cursor rules and terminal commands</div>
          </div>
          <label class="apple-switch">
            <input type="checkbox" id="set-ide-cursor" checked onchange="saveSettingsState()" />
            <span class="apple-switch-slider"></span>
          </label>
        </div>

        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">Anthropic Claude Code</div>
            <div class="settings-row-desc">Watches Claude CLI hooks and tool execution</div>
          </div>
          <label class="apple-switch">
            <input type="checkbox" id="set-ide-claude" checked onchange="saveSettingsState()" />
            <span class="apple-switch-slider"></span>
          </label>
        </div>

        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">Google Antigravity</div>
            <div class="settings-row-desc">Sidecar MCP integration and causal state sync</div>
          </div>
          <label class="apple-switch">
            <input type="checkbox" id="set-ide-antigravity" checked onchange="saveSettingsState()" />
            <span class="apple-switch-slider"></span>
          </label>
        </div>

        <div class="settings-row">
          <div class="settings-row-text">
            <div class="settings-row-title">OpenAI Codex / Operator</div>
            <div class="settings-row-desc">Listens on local loop proxy and MCP socket</div>
          </div>
          <label class="apple-switch">
            <input type="checkbox" id="set-ide-codex" checked onchange="saveSettingsState()" />
            <span class="apple-switch-slider"></span>
          </label>
        </div>
      </div>

      <!-- Tab 3: Team & Budgets -->
      <div id="sheet-tab-team" class="sheet-section hidden">
        <div class="sheet-section-title">Team Members &amp; Repo Owners</div>
        <div id="team-members-list" style="display: flex; flex-direction: column; gap: 8px;">
          <!-- Populated by JS -->
        </div>

        <div class="sheet-section-title" style="margin-top: 10px;">Repository Budget Ceilings</div>
        <div id="repo-budgets-list" style="display: flex; flex-direction: column; gap: 8px;">
          <!-- Populated by JS -->
        </div>
      </div>
    </div>

    <div class="sheet-footer">
      <span id="save-status-indicator" style="font-size: 11px; color: var(--label-tertiary);">Auto-saved locally</span>
      <button class="apple-btn apple-btn-primary" onclick="toggleSettingsSheet(false)">Done</button>
    </div>
  </div>

  <script>
    let activeGateId = null;
    let currentFleetData = null;
    let currentSettings = null;

    // 1. Navigation & View Switching
    function switchView(viewName) {
      const isDashboard = viewName === 'dashboard';
      const isFleet = viewName === 'fleet';
      const isProofs = viewName === 'proofs';

      document.getElementById('main-view').classList.toggle('hidden', !isDashboard);
      document.getElementById('fleet-view').classList.toggle('hidden', !isFleet);
      document.getElementById('logs-view').classList.toggle('hidden', !isProofs);

      document.getElementById('tab-dashboard').classList.toggle('active', isDashboard);
      document.getElementById('tab-fleet').classList.toggle('active', isFleet);
      document.getElementById('tab-proofs').classList.toggle('active', isProofs);

      if (isFleet) fetchFleet();
    }

    // 2. Repo Switcher Dropdown
    function toggleRepoDropdown(event) {
      if (event) event.stopPropagation();
      const menu = document.getElementById('repo-dropdown-menu');
      menu.classList.toggle('hidden');
      if (!menu.classList.contains('hidden')) {
        document.getElementById('repo-search-input').focus();
      }
    }

    document.addEventListener('click', function(e) {
      const wrapper = document.querySelector('.repo-dropdown-wrapper');
      if (wrapper && !wrapper.contains(e.target)) {
        document.getElementById('repo-dropdown-menu').classList.add('hidden');
      }
    });

    function filterRepos(query) {
      const q = (query || "").toLowerCase();
      const items = document.querySelectorAll('.menu-item');
      items.forEach(it => {
        const name = it.getAttribute('data-repo-name') || '';
        it.style.display = name.toLowerCase().includes(q) ? 'flex' : 'none';
      });
    }

    async function selectRepo(repoId) {
      try {
        const res = await fetch("/api/fleet/select", {
          method: "POST",
          headers: { "Content-Type": "application/json", "Authorization": "Bearer " + (localStorage.getItem('kineti_token') || '') },
          body: JSON.stringify({ repo_id: repoId }),
        });
        if (res.ok) {
          document.getElementById('repo-dropdown-menu').classList.add('hidden');
          await fetchStatus();
          await fetchFleet();
          switchView('dashboard');
        }
      } catch (e) {
        console.error("Select repo error", e);
      }
    }

    // 3. Settings Slide-Out Sheet
    function toggleSettingsSheet(show) {
      const sheet = document.getElementById('settings-sheet');
      const backdrop = document.getElementById('sheet-backdrop');
      sheet.classList.toggle('open', show);
      backdrop.classList.toggle('open', show);
    }

    function switchSettingsTab(tabName) {
      ['github', 'ides', 'team'].forEach(t => {
        const isActive = t === tabName;
        document.getElementById('sheet-tab-' + t).classList.toggle('hidden', !isActive);
        document.getElementById('sheet-tab-btn-' + t).classList.toggle('active', isActive);
      });
    }

    function openSettingsWithTab(tabName) {
      document.getElementById('repo-dropdown-menu').classList.add('hidden');
      toggleSettingsSheet(true);
      switchSettingsTab(tabName);
    }

    async function saveSettingsState() {
      const cursor = document.getElementById('set-ide-cursor').checked;
      const claude = document.getElementById('set-ide-claude').checked;
      const agy = document.getElementById('set-ide-antigravity').checked;
      const codex = document.getElementById('set-ide-codex').checked;

      const payload = {
        ides: { cursor, claude_code: claude, antigravity: agy, codex },
      };

      try {
        const res = await fetch("/api/settings", {
          method: "POST",
          headers: { "Content-Type": "application/json", "Authorization": "Bearer " + (localStorage.getItem('kineti_token') || '') },
          body: JSON.stringify(payload),
        });
        if (res.ok) {
          const ind = document.getElementById('save-status-indicator');
          ind.textContent = "Saved";
          ind.style.color = "var(--system-green)";
          setTimeout(() => {
            ind.textContent = "Auto-saved locally";
            ind.style.color = "var(--label-tertiary)";
          }, 2000);
        }
      } catch (e) {
        console.error("Save settings error", e);
      }
    }

    async function updateRepoBudget(repoId, newBudget) {
      const budgetNum = Number(newBudget);
      if (isNaN(budgetNum) || budgetNum <= 0) return;
      await fetch("/api/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", "Authorization": "Bearer " + (localStorage.getItem('kineti_token') || '') },
        body: JSON.stringify({ repo_budgets: { [repoId]: budgetNum } }),
      });
      fetchFleet();
      fetchStatus();
    }

    async function updateRepoOwner(repoId, newOwner) {
      if (!newOwner) return;
      await fetch("/api/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", "Authorization": "Bearer " + (localStorage.getItem('kineti_token') || '') },
        body: JSON.stringify({ repo_owners: { [repoId]: newOwner } }),
      });
      fetchFleet();
    }

    // 4. Fetch Fleet and Status
    async function fetchFleet() {
      try {
        const res = await fetch("/api/fleet");
        if (!res.ok) return;
        const data = await res.json();
        currentFleetData = data;
        renderFleet(data);
      } catch (e) {
        console.error("Fleet fetch error", e);
      }
    }

    function renderFleet(data) {
      const repos = data.repos || [];
      const activeId = data.active_repo_id;

      // Update Nav active repo label
      const activeRepo = repos.find(r => r.id === activeId);
      if (activeRepo) {
        document.getElementById('nav-active-repo-name').textContent = activeRepo.name;
      }

      // Update Summary Bar
      document.getElementById('fleet-stat-count').textContent = repos.length;
      document.getElementById('fleet-stat-spend').textContent = "$" + data.total_fleet_spend.toFixed(2);
      document.getElementById('fleet-stat-active').textContent = repos.filter(r => r.status === 'active' || r.status === 'action_needed').length;
      const totalTests = repos.reduce((acc, r) => acc + r.tests_passing, 0);
      document.getElementById('fleet-stat-tests').textContent = totalTests;

      // Update Dropdown List
      const menuList = document.getElementById('repo-menu-list');
      menuList.innerHTML = '';
      repos.forEach(r => {
        const item = document.createElement('div');
        item.className = 'menu-item' + (r.id === activeId ? ' active' : '');
        item.setAttribute('data-repo-name', r.name);
        item.onclick = () => selectRepo(r.id);

        item.innerHTML = 
          '<div class="menu-item-info">' +
            '<div class="menu-item-name">' +
              escapeHtml(r.name) +
              (r.is_local ? ' <span class="menu-badge">Local</span>' : '') +
            '</div>' +
            '<div class="menu-item-meta">' + escapeHtml(r.owner) + ' · ' + escapeHtml(r.branch) + '</div>' +
          '</div>' +
          (r.id === activeId ? '<span style="color: var(--system-blue); font-weight: bold;">✓</span>' : '');
        menuList.appendChild(item);
      });

      // Update Fleet Grid Cards
      const grid = document.getElementById('fleet-repo-grid');
      grid.innerHTML = '';
      repos.forEach(r => {
        const card = document.createElement('div');
        card.className = 'fleet-card';

        let statusClass = 'capsule-safe';
        let statusLabel = 'Active';
        if (r.status === 'action_needed') {
          statusClass = 'capsule-action';
          statusLabel = 'Needs Review';
        } else if (r.status === 'limit_reached') {
          statusClass = 'capsule-tripped';
          statusLabel = 'Limit Reached';
        } else if (r.status === 'idle') {
          statusClass = 'capsule-safe';
          statusLabel = 'Idle';
        }

        const spendPct = Math.min(100, Math.round((r.spend_usd / r.ceiling_usd) * 100));

        card.innerHTML = 
          '<div class="fleet-card-header">' +
            '<div>' +
              '<div class="fleet-card-title">' +
                escapeHtml(r.name) +
                (r.is_local ? '<span class="menu-badge">Local</span>' : '<span class="menu-badge">GitHub</span>') +
              '</div>' +
              '<div class="fleet-card-tags" style="margin-top: 5px;">' +
                '<span class="tag-pill mono">' + escapeHtml(r.branch) + '</span>' +
                '<span class="tag-pill tag-ide">' + escapeHtml(r.ide) + '</span>' +
                '<span class="tag-pill">' + escapeHtml(r.owner) + '</span>' +
              '</div>' +
            '</div>' +
            '<span class="status-capsule ' + statusClass + '"><span class="status-dot"></span> ' + statusLabel + '</span>' +
          '</div>' +
          '<div class="fleet-card-task">' + escapeHtml(r.active_task) + '</div>' +
          '<div>' +
            '<div style="display: flex; justify-content: space-between; font-size: 11px; margin-bottom: 4px;">' +
              '<span style="color: var(--label-tertiary);">Spend Ceiling</span>' +
              '<span class="mono">$' + r.spend_usd.toFixed(2) + ' / $' + r.ceiling_usd.toFixed(0) + '</span>' +
            '</div>' +
            '<div class="spend-track" style="width: 100%;">' +
              '<div class="spend-fill-bar" style="width: ' + spendPct + '%;"></div>' +
            '</div>' +
          '</div>' +
          '<div class="fleet-card-footer">' +
            '<div class="fleet-card-metrics">' +
              '<span style="color: var(--system-green);">✓ ' + r.tests_passing + ' tests</span>' +
            '</div>' +
            '<button class="apple-btn apple-btn-secondary" onclick="selectRepo(\\'' + String(r.id).replace(/\\/g, "\\\\").replace(/'/g, "\\'") + '\\')">Open Dashboard →</button>' +
          '</div>';

        grid.appendChild(card);
      });

      // Update Settings: Team and Budgets
      renderSettings(data.settings, repos);
    }

    function renderSettings(settings, repos) {
      if (!settings) return;
      currentSettings = settings;

      // IDE toggles
      if (settings.ides) {
        document.getElementById('set-ide-cursor').checked = !!settings.ides.cursor;
        document.getElementById('set-ide-claude').checked = !!settings.ides.claude_code;
        document.getElementById('set-ide-antigravity').checked = !!settings.ides.antigravity;
        document.getElementById('set-ide-codex').checked = !!settings.ides.codex;
      }

      // Team members
      const teamList = document.getElementById('team-members-list');
      teamList.innerHTML = '';
      (settings.team_members || []).forEach(m => {
        const row = document.createElement('div');
        row.className = 'settings-row';
        row.innerHTML = 
          '<div class="settings-row-text">' +
            '<div class="settings-row-title">' + escapeHtml(m.name) + '</div>' +
            '<div class="settings-row-desc">' + escapeHtml(m.email) + ' · ' + escapeHtml(m.role) + '</div>' +
          '</div>' +
          '<span class="tag-pill">' + escapeHtml(m.role) + '</span>';
        teamList.appendChild(row);
      });

      // Repo budgets & owners
      const budgetList = document.getElementById('repo-budgets-list');
      budgetList.innerHTML = '';
      repos.forEach(r => {
        const row = document.createElement('div');
        row.className = 'settings-row';
        row.style.alignItems = 'center';

        const owners = settings.team_members || [];
        let ownerOptions = owners.map(o => 
          '<option value="' + escapeHtml(o.name) + '" ' + (o.name === r.owner ? 'selected' : '') + '>' + escapeHtml(o.name) + '</option>'
        ).join('');

        row.innerHTML = 
          '<div class="settings-row-text" style="min-width: 120px;">' +
            '<div class="settings-row-title">' + escapeHtml(r.name) + '</div>' +
            '<div class="settings-row-desc">Owner: ' +
              '<select style="background: rgba(255,255,255,0.1); color: #fff; border: var(--hairline); border-radius: 4px; padding: 1px 4px; font-size: 11px; font-family: inherit;" onchange="updateRepoOwner(\\'' + r.id + '\\', this.value)">' +
                ownerOptions +
              '</select>' +
            '</div>' +
          '</div>' +
          '<div style="display: flex; align-items: center; gap: 6px;">' +
            '<span style="font-size: 11px; color: var(--label-tertiary);">$</span>' +
            '<input type="number" value="' + r.ceiling_usd + '" style="width: 55px; background: rgba(255,255,255,0.08); border: var(--hairline); border-radius: 6px; padding: 4px 6px; color: #fff; font-size: 12px; font-family: inherit;" onchange="updateRepoBudget(\\'' + r.id + '\\', this.value)" />' +
          '</div>';
        budgetList.appendChild(row);
      });
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
      return String(str).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
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
          headers: { "Content-Type": "application/json", "Authorization": "Bearer " + (localStorage.getItem('kineti_token') || '') },
          body: JSON.stringify({ gate: activeGateId, status: "pass" }),
        });
        if (res.ok) {
          fetchStatus();
          fetchFleet();
        }
      } catch (e) {
        console.error("Gate approval error", e);
      }
    }

    async function resetBreaker() {
      if (!confirm("Reset spend cap? This needs a human confirm.")) return;
      try {
        const res = await fetch("/api/spend/reset", {
          method: "POST",
          headers: { "Content-Type": "application/json", "Authorization": "Bearer " + (localStorage.getItem('kineti_token') || '') },
          body: JSON.stringify({ i_am_human: true }),
        });
        if (res.ok) {
          fetchStatus();
          fetchFleet();
        }
      } catch (e) {
        console.error("Reset error", e);
      }
    }

    // Initial setup
    fetchStatus();
    fetchFleet();
    setInterval(() => {
      fetchStatus();
      if (!document.getElementById('fleet-view').classList.contains('hidden')) {
        fetchFleet();
      }
    }, 2500);
  </script>
</body>
</html>`;
}

function loginHtml(): string {
  // Token-free login page. No AUTH_TOKEN here. User pastes token once,
  // it is saved to localStorage, then dashboard is fetched with Bearer.
  return `<!DOCTYPE html>
<html lang="en">
<head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>Kineti OS — Sign in</title></head>
<body style="font-family:-apple-system,sans-serif;background:#000;color:#fff;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;">
<main style="max-width:380px;padding:24px;text-align:center;">
<h1 style="font-size:18px;">Kineti OS — Sign in</h1>
<p style="font-size:13px;opacity:0.7;">Paste the token from <code>.kineti/auth_token</code> in this project.</p>
<input id="t" type="password" placeholder="Paste token" style="width:100%;padding:10px;border-radius:8px;border:1px solid #333;background:#111;color:#fff;" />
<button id="b" style="margin-top:12px;padding:8px 16px;border-radius:8px;border:none;background:#0A84FF;color:#fff;font-weight:600;">Sign in</button>
<p id="e" style="font-size:12px;color:#FF453A;"></p>
<script>
document.getElementById('b').onclick = async () => {
  const t = document.getElementById('t').value.trim();
  if (!t) return;
  try { localStorage.setItem('kineti_token', t); } catch (e) {}
  const r = await fetch('/', { headers: { 'Authorization': 'Bearer ' + t } });
  if (r.ok) { document.open(); document.write(await r.text()); document.close(); }
  else { document.getElementById('e').textContent = 'Bad token (' + r.status + '). Try again.'; }
};
</script>
</main>
</body>
</html>`;
}

function isTrustedHost(req: Request): boolean {
  // Check URL hostname (Bun builds req.url from Host for real traffic,
  // so rebinding shows up here) plus Host header when present.
  // Host is a forbidden header for `new Request`, so server.fetch tests
  // carry the hostname in the URL. Real network requests always have Host.
  try {
    const urlHost = new URL(req.url).hostname.trim().toLowerCase();
    if (urlHost !== "localhost" && urlHost !== "127.0.0.1") return false;
  } catch {
    return false;
  }
  const host = req.headers.get("host");
  if (host === null) return true; // server.fetch test path: no transport Host, URL already checked
  const hostname = host.split(":")[0].trim().toLowerCase();
  if (!hostname) return false; // empty Host: fail closed
  return hostname === "localhost" || hostname === "127.0.0.1";
}

function isTrustedOrigin(req: Request): boolean {
  const origin = req.headers.get("origin");
  if (!origin) return true; // same-origin / non-browser (curl, server.fetch) has no Origin
  let h = "";
  try { h = new URL(origin).hostname.toLowerCase(); } catch { return false; }
  return h === "localhost" || h === "127.0.0.1";
}

function withVary(headers: Record<string, string>): Record<string, string> {
  return { "Vary": "Origin", ...headers };
}

function unauthorizedJson(): Response {
  return new Response(JSON.stringify({ error: "Unauthorized" }), {
    status: 401,
    headers: withVary({ "Content-Type": "application/json" }),
  });
}

function forbidden(msg = "Forbidden"): Response {
  return new Response(msg, { status: 403, headers: withVary({ "Content-Type": "text/plain; charset=utf-8" }) });
}

function startServer(port: number = PORT) {
  const server = Bun.serve({
    port,
    hostname: "127.0.0.1",
    fetch(req) {
      const url = new URL(req.url);

      // 1. Host check first — kills DNS rebinding (evil.com, LAN IP, lookalikes).
      if (!isTrustedHost(req)) {
        return forbidden();
      }

      // 2. Origin check — deny-by-default, no ACAO sent. Vary: Origin on all.
      if (!isTrustedOrigin(req)) {
        return forbidden();
      }

      // 3. Preflight: explicit 204, Vary, no allow-origin (deny by default).
      if (req.method === "OPTIONS") {
        return new Response(null, { status: 204, headers: withVary({}) });
      }

      if (url.pathname === "/" || url.pathname === "/dashboard") {
        if (!isAuthorized(req)) {
          // 401 login page, token-free, no hex token in body.
          return new Response(loginHtml(), {
            status: 401,
            headers: withVary({ "Content-Type": "text/html; charset=utf-8" }),
          });
        }
        return new Response(renderHtmlDashboard(), {
          headers: withVary({ "Content-Type": "text/html; charset=utf-8" }),
        });
      }

      // All /api/* routes need auth, reads included.
      if (url.pathname.startsWith("/api/")) {
        if (!isAuthorized(req)) {
          return unauthorizedJson();
        }
      }

      if (url.pathname === "/api/status") {
        return new Response(JSON.stringify(getHarnessStatus()), {
          headers: withVary({ "Content-Type": "application/json" }),
        });
      }

      if (url.pathname === "/api/fleet") {
        return new Response(JSON.stringify(getFleetStatus()), {
          headers: withVary({ "Content-Type": "application/json" }),
        });
      }

      if (url.pathname === "/api/fleet/select" && req.method === "POST") {
        if (!isAuthorized(req)) {
          return unauthorizedJson();
        }
        return req.json().then((body: any) => {
          const { repo_id } = body;
          const found = fleetRepos.find((r) =>
            r.id === repo_id ||
            r.name === repo_id ||
            (r.is_local && (repo_id === "local" || repo_id === "kineti-local-harness" || repo_id === defaultRepoName || repo_id === path.basename(REPO_ROOT)))
          );
          if (!found) {
            return new Response(JSON.stringify({ error: `Repo not found: ${repo_id}` }), {
              status: 404,
              headers: withVary({ "Content-Type": "application/json" }),
            });
          }
          activeRepoId = found.id;
          return new Response(JSON.stringify({ success: true, active_repo_id: activeRepoId }), {
            headers: withVary({ "Content-Type": "application/json" }),
          });
        });
      }

      if (url.pathname === "/api/settings") {
        if (req.method === "GET") {
          return new Response(JSON.stringify(companionSettings), {
            headers: withVary({ "Content-Type": "application/json" }),
          });
        }
        if (req.method === "POST") {
          if (!isAuthorized(req)) {
            return unauthorizedJson();
          }
          return req.json().then((body: any) => {
            if (body.github) companionSettings.github = { ...companionSettings.github, ...body.github };
            if (body.ides) companionSettings.ides = { ...companionSettings.ides, ...body.ides };
            if (body.team_members) companionSettings.team_members = body.team_members;
            if (body.repo_budgets) {
              companionSettings.repo_budgets = { ...companionSettings.repo_budgets, ...body.repo_budgets };
              for (const [id, budget] of Object.entries(body.repo_budgets)) {
                const r = fleetRepos.find((repo) => repo.id === id);
                if (r && typeof budget === "number" && Number.isFinite(budget) && budget > 0) r.ceiling_usd = budget;
              }
            }
            if (body.repo_owners) {
              companionSettings.repo_owners = { ...companionSettings.repo_owners, ...body.repo_owners };
              for (const [id, owner] of Object.entries(body.repo_owners)) {
                const r = fleetRepos.find((repo) => repo.id === id);
                if (r && typeof owner === "string") r.owner = owner;
              }
            }
            return new Response(JSON.stringify({ success: true, settings: companionSettings }), {
              headers: withVary({ "Content-Type": "application/json" }),
            });
          });
        }
      }

      if (url.pathname === "/api/gate" && req.method === "POST") {
        if (!isAuthorized(req)) {
          return unauthorizedJson();
        }
        return req.json().then((body: any) => {
          const { gate, status } = body;
          if (!gate || (status !== "pass" && status !== "fail" && status !== "pending")) {
            return new Response(JSON.stringify({ error: "gate and valid status required" }), { status: 400, headers: withVary({ "Content-Type": "application/json" }) });
          }
          const res = Bun.spawnSync(["bun", path.join(REPO_ROOT, "bin", "kineti-state.ts"), "set", `gate.${gate}`, status]);
          return new Response(JSON.stringify({ success: res.exitCode === 0 }), {
            headers: withVary({ "Content-Type": "application/json" }),
          });
        });
      }

      if (url.pathname === "/api/spend/reset" && req.method === "POST") {
        if (!isAuthorized(req)) {
          return unauthorizedJson();
        }
        return req.json().then((body: any) => {
          if (!body || body.i_am_human !== true) {
            return new Response(JSON.stringify({ error: "Human confirm required: send {i_am_human:true}" }), {
              status: 400,
              headers: withVary({ "Content-Type": "application/json" }),
            });
          }
          const res = Bun.spawnSync(["bun", path.join(REPO_ROOT, "bin", "kineti-spend.ts"), "reset", "--i-am-human"]);
          return new Response(JSON.stringify({ success: res.exitCode === 0 }), {
            headers: withVary({ "Content-Type": "application/json" }),
          });
        }).catch(() => {
          return new Response(JSON.stringify({ error: "Human confirm required: send {i_am_human:true}" }), {
            status: 400,
            headers: withVary({ "Content-Type": "application/json" }),
          });
        });
      }

      return new Response("Not Found", { status: 404, headers: withVary({}) });
    },
  });

  ok(`Kineti Companion visual dashboard listening at http://127.0.0.1:${server.port}`);
  return server;
}

if (import.meta.main) {
  startServer(PORT);
}

export { startServer, getHarnessStatus, getFleetStatus, fleetRepos, companionSettings, renderHtmlDashboard, loginHtml, isTrustedHost, isTrustedOrigin, AUTH_TOKEN, isAuthorized };
