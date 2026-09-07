#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { die, ok, projectKdir, readJson, readJsonl, ensureDir, nowIso } from "./lib.ts";

const PORT = Number(process.env.KINETI_COMPANION_PORT || 8788);
const REPO_ROOT = process.cwd();

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

  const evidence = readJsonl<any>(evidencePath).slice(-10).reverse();

  const currentStageInfo = STAGES.find((s) => s.id === state.stage) ?? STAGES[0];

  return {
    project: state.project,
    root_goal: state.root_goal,
    root_goal_locked_at: state.root_goal_locked_at,
    stage: state.stage,
    stage_name: currentStageInfo.name,
    stage_label: currentStageInfo.label,
    stages: STAGES.map((s) => ({
      ...s,
      is_current: s.id === state.stage,
      is_past: s.id < state.stage,
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
    auth_required: true,
  };
}

function renderHtmlDashboard(): string {
  return `<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Kineti OS — Visual Companion Canvas</title>
  <style>
    :root {
      --bg: #09090b;
      --card: #18181b;
      --card-border: #27272a;
      --text: #f4f4f5;
      --text-muted: #a1a1aa;
      --accent: #8b5cf6;
      --accent-hover: #7c3aed;
      --green: #10b981;
      --red: #ef4444;
      --amber: #f59e0b;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background-color: var(--bg);
      color: var(--text);
      line-height: 1.5;
      padding: 24px;
      -webkit-font-smoothing: antialiased;
    }
    .container { max-width: 1280px; margin: 0 auto; }
    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 20px;
      border-bottom: 1px solid var(--card-border);
      margin-bottom: 24px;
    }
    .logo-group { display: flex; align-items: center; gap: 12px; }
    .logo-badge {
      background: linear-gradient(135deg, #8b5cf6, #ec4899);
      color: #fff;
      font-weight: 800;
      font-size: 14px;
      padding: 4px 10px;
      border-radius: 6px;
      letter-spacing: 0.5px;
    }
    .title { font-size: 20px; font-weight: 700; color: #fff; }
    .status-pill {
      font-size: 12px;
      padding: 4px 10px;
      border-radius: 9999px;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      font-weight: 600;
      border: 1px solid transparent;
    }
    .status-pill.online { background: rgba(16, 185, 129, 0.15); color: #34d399; border-color: rgba(16, 185, 129, 0.3); }
    .status-pill.tripped { background: rgba(239, 68, 68, 0.15); color: #f87171; border-color: rgba(239, 68, 68, 0.3); }
    .status-dot { width: 8px; height: 8px; border-radius: 50%; background: currentColor; }

    .grid { display: grid; grid-template-columns: 2fr 1fr; gap: 24px; margin-bottom: 24px; }
    @media (max-width: 900px) { .grid { grid-template-columns: 1fr; } }

    .card {
      background: var(--card);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 20px;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
    }
    .card-title {
      font-size: 14px;
      font-weight: 600;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      margin-bottom: 16px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    /* Spend Meter */
    .spend-meter-box { text-align: center; padding: 12px 0; }
    .spend-val { font-size: 38px; font-weight: 800; color: #fff; }
    .spend-sub { font-size: 13px; color: var(--text-muted); margin-top: 4px; }
    .progress-bar {
      height: 8px;
      background: #27272a;
      border-radius: 4px;
      overflow: hidden;
      margin: 16px 0 8px;
    }
    .progress-fill {
      height: 100%;
      background: linear-gradient(90deg, #10b981, #f59e0b, #ef4444);
      border-radius: 4px;
      transition: width 0.3s ease;
    }

    /* 13 Stage Flow */
    .stages-list { display: flex; flex-direction: column; gap: 10px; }
    .stage-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 10px 14px;
      border-radius: 8px;
      background: rgba(255, 255, 255, 0.02);
      border: 1px solid rgba(255, 255, 255, 0.05);
      font-size: 14px;
      transition: all 0.2s ease;
    }
    .stage-item.current {
      background: rgba(139, 92, 246, 0.12);
      border-color: rgba(139, 92, 246, 0.4);
      box-shadow: 0 0 12px rgba(139, 92, 246, 0.2);
    }
    .stage-item.past { opacity: 0.6; }
    .stage-left { display: flex; align-items: center; gap: 12px; }
    .stage-num {
      width: 24px;
      height: 24px;
      border-radius: 6px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 12px;
      font-weight: 700;
      background: #27272a;
      color: #a1a1aa;
    }
    .stage-item.current .stage-num { background: var(--accent); color: #fff; }
    .stage-name { font-weight: 600; color: #fff; }
    .stage-desc { font-size: 12px; color: var(--text-muted); margin-left: 8px; }

    /* Gate Action Buttons */
    .btn-group { display: flex; gap: 8px; }
    .btn {
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
      border: 1px solid transparent;
      transition: all 0.15s ease;
    }
    .btn-primary { background: var(--accent); color: #fff; }
    .btn-primary:hover { background: var(--accent-hover); }
    .btn-pass { background: rgba(16, 185, 129, 0.2); color: #34d399; border-color: rgba(16, 185, 129, 0.3); }
    .btn-pass:hover { background: rgba(16, 185, 129, 0.35); }
    .btn-fail { background: rgba(239, 68, 68, 0.2); color: #f87171; border-color: rgba(239, 68, 68, 0.3); }
    .btn-fail:hover { background: rgba(239, 68, 68, 0.35); }
    .btn-reset { background: rgba(245, 158, 11, 0.2); color: #fbbf24; border-color: rgba(245, 158, 11, 0.3); }
    .btn-reset:hover { background: rgba(245, 158, 11, 0.35); }

    /* Evidence table */
    .table { width: 100%; border-collapse: collapse; font-size: 13px; }
    .table th { text-align: left; padding: 8px 12px; color: var(--text-muted); border-bottom: 1px solid var(--card-border); font-size: 11px; text-transform: uppercase; }
    .table td { padding: 10px 12px; border-bottom: 1px solid rgba(255, 255, 255, 0.04); font-family: monospace; }
    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <div class="logo-group">
        <span class="logo-badge">KINETI OS</span>
        <h1 class="title" id="proj-name">Local Runtime Companion</h1>
      </div>
      <div id="status-pill-container">
        <span class="status-pill online"><span class="status-dot"></span> <span id="runtime-status">ONLINE</span></span>
      </div>
    </div>

    <div class="grid">
      <!-- Left Column: Pipeline Stages & Gates -->
      <div class="card">
        <div class="card-title">
          <span>13-Stage Pipeline & Governance Gates</span>
          <span class="mono" style="color: var(--accent);" id="active-stage-label">Stage 1</span>
        </div>
        <div class="stages-list" id="stages-container">
          <!-- Populated dynamically via JS -->
        </div>
      </div>

      <!-- Right Column: Spend & Quick Actions -->
      <div style="display: flex; flex-direction: column; gap: 24px;">
        <div class="card">
          <div class="card-title">Real-Time Spend Circuit Breaker</div>
          <div class="spend-meter-box">
            <div class="spend-val" id="spend-total">$0.00</div>
            <div class="spend-sub" id="spend-sub">Limit: $50.00 (Safety: $47.50)</div>
            <div class="progress-bar">
              <div class="progress-fill" id="spend-progress" style="width: 0%;"></div>
            </div>
          </div>
          <div id="breaker-alert" style="display: none; margin-top: 12px; padding: 10px; background: rgba(239, 68, 68, 0.15); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 8px;">
            <div style="font-weight: 700; color: #f87171; font-size: 13px;">CIRCUIT BREAKER TRIPPED</div>
            <div id="breaker-reason" style="font-size: 12px; color: #fca5a5; margin-top: 4px;"></div>
            <button class="btn btn-reset" style="margin-top: 10px; width: 100%;" onclick="resetBreaker()">I AM HUMAN: Reset Breaker</button>
          </div>
        </div>

        <div class="card">
          <div class="card-title">Root Goal Lock</div>
          <div style="font-size: 13px; color: var(--text-muted); margin-bottom: 8px;">Immutable objective boundary:</div>
          <div id="root-goal-text" class="mono" style="padding: 10px; background: rgba(0,0,0,0.3); border-radius: 6px; font-size: 13px; color: #e4e4e7;">
            (No root goal locked yet)
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom: Evidence Proofs -->
    <div class="card">
      <div class="card-title">Latest Cryptographic Evidence Proofs</div>
      <table class="table">
        <thead>
          <tr>
            <th>Time</th>
            <th>Label</th>
            <th>Command</th>
            <th>Exit</th>
            <th>Fingerprint</th>
          </tr>
        </thead>
        <tbody id="evidence-tbody">
          <tr><td colspan="5" style="text-align: center; color: var(--text-muted);">No evidence logs recorded yet.</td></tr>
        </tbody>
      </table>
    </div>
  </div>

  <script>
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

    function render(data) {
      document.getElementById("proj-name").textContent = data.project + " — Companion";
      document.getElementById("active-stage-label").textContent = "Stage " + data.stage + ": " + data.stage_label;
      document.getElementById("root-goal-text").textContent = data.root_goal || "(No root goal locked yet)";

      // Spend
      const total = data.spend.total_usd || 0;
      const ceiling = data.spend.ceiling_usd || 50;
      document.getElementById("spend-total").textContent = "$" + total.toFixed(2);
      const pct = Math.min(100, Math.round((total / ceiling) * 100));
      document.getElementById("spend-progress").style.width = pct + "%";

      if (data.spend.tripped) {
        document.getElementById("breaker-alert").style.display = "block";
        document.getElementById("breaker-reason").textContent = data.spend.reason || "Ceiling exceeded";
        document.getElementById("status-pill-container").innerHTML = '<span class="status-pill tripped"><span class="status-dot"></span> TRIPPED</span>';
      } else {
        document.getElementById("breaker-alert").style.display = "none";
        document.getElementById("status-pill-container").innerHTML = '<span class="status-pill online"><span class="status-dot"></span> ONLINE</span>';
      }

      // Stages
      const stagesBox = document.getElementById("stages-container");
      stagesBox.innerHTML = "";
      data.stages.forEach(stg => {
        const div = document.createElement("div");
        div.className = "stage-item" + (stg.is_current ? " current" : "") + (stg.is_past ? " past" : "");

        let gateUi = "";
        if (stg.gate) {
          const gStatus = stg.gate_status || "pending";
          let pillColor = gStatus === "pass" ? "btn-pass" : gStatus === "fail" ? "btn-fail" : "btn-reset";
          gateUi = \`
            <div class="btn-group">
              <span class="status-pill \${pillColor}" style="margin-right: 4px;">\${stg.gate.toUpperCase()}: \${gStatus.toUpperCase()}</span>
              <button class="btn btn-pass" onclick="setGate('\${stg.gate}', 'pass')">Pass</button>
              <button class="btn btn-fail" onclick="setGate('\${stg.gate}', 'fail')">Fail</button>
            </div>
          \`;
        }

        div.innerHTML = \`
          <div class="stage-left">
            <div class="stage-num">\${stg.id}</div>
            <div>
              <span class="stage-name">\${stg.label}</span>
              <span class="stage-desc">\${stg.desc}</span>
            </div>
          </div>
          <div>\${gateUi}</div>
        \`;
        stagesBox.appendChild(div);
      });

      // Evidence
      const evTbody = document.getElementById("evidence-tbody");
      if (data.evidence && data.evidence.length > 0) {
        evTbody.innerHTML = "";
        data.evidence.forEach(ev => {
          const tr = document.createElement("tr");
          const fp = ev.fingerprint ? ev.fingerprint.slice(0, 16) + "..." : "none";
          tr.innerHTML = \`
            <td style="color: var(--text-muted);">\${new Date(ev.at).toLocaleTimeString()}</td>
            <td><strong>\${ev.label}</strong></td>
            <td><code class="mono">\${ev.cmd}</code></td>
            <td><span style="color: \${ev.exit_code === 0 ? 'var(--green)' : 'var(--red)'};">\${ev.exit_code}</span></td>
            <td class="mono" style="color: var(--accent);">\${fp}</td>
          \`;
          evTbody.appendChild(tr);
        });
      }
    }

    async function setGate(gate, status) {
      await fetch("/api/gate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ gate, status }),
      });
      fetchStatus();
    }

    async function resetBreaker() {
      await fetch("/api/spend/reset", { method: "POST" });
      fetchStatus();
    }

    setInterval(fetchStatus, 2000);
    fetchStatus();
  </script>
</body>
</html>`;
}

function startServer(port: number) {
  const server = Bun.serve({
    port,
    fetch(req) {
      const url = new URL(req.url);

      // CORS & Host check (Security hardening)
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
