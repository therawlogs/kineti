#!/usr/bin/env bun
/**
 * Kineti Universal MCP Server (Model Context Protocol)
 *
 * Exposes Kineti OS governance, evidence, spend caps, saga rollbacks,
 * and state gates as standard MCP tools over stdio JSON-RPC.
 *
 * Compatible with: Claude Code, Cursor, Antigravity, OpenAI Codex / Operator.
 */

import * as fs from "node:fs";
import * as path from "node:path";
import { spawnSync } from "node:child_process";
import * as readline from "node:readline";
import { readJson, projectKdir, loadLimits, Limits, splitLegacyCommand } from "./lib.ts";

const PROTOCOL_VERSION = "2024-11-05";
const SERVER_NAME = "kineti-harness";
const SERVER_VERSION = "3.0.0";

// Workspace root resolution
let workspaceRoot = process.cwd();
const args = process.argv.slice(2);

// Check if run as CLI helper (e.g. `kineti-mcp init`)
if (args[0] === "init") {
  handleInit(args.slice(1));
  process.exit(0);
}

// Parse --workspace-root if provided
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--workspace-root" && args[i + 1]) {
    workspaceRoot = path.resolve(args[i + 1]);
    i++;
  }
}

function runBin(scriptName: string, subArgs: string[]): { exitCode: number; stdout: string; stderr: string } {
  // Allow only kineti-*.ts scripts, no shell, cwd jailed to workspaceRoot, 60s timeout.
  if (!/^kineti-[a-z-]+\.ts$/.test(scriptName)) {
    return { exitCode: 2, stdout: "", stderr: `kineti: blocked: script not in allowlist: ${scriptName}` };
  }
  const scriptPath = path.resolve(__dirname, scriptName);
  const binDir = path.resolve(__dirname);
  if (scriptPath !== path.join(binDir, scriptName)) {
    return { exitCode: 2, stdout: "", stderr: "kineti: blocked: script path escapes bin dir" };
  }
  let safeRoot = path.resolve(workspaceRoot);
  try {
    const st = require("node:fs").statSync(safeRoot);
    if (!st.isDirectory()) safeRoot = process.cwd();
  } catch {
    safeRoot = process.cwd();
  }
  const res = spawnSync("bun", [scriptPath, ...subArgs], {
    cwd: safeRoot,
    env: { ...process.env, KINETI_MACHINE_DIR: path.join(safeRoot, ".kineti", "machine") },
    encoding: "utf8",
    timeout: 60000,
    killSignal: "SIGKILL",
  } as any);
  let stderr = (res as any).stderr?.toString() || (res.stderr as any) || "";
  if ((res as any).error && String((res as any).error).includes("ETIMEDOUT")) {
    stderr = (stderr ? stderr + "\n" : "") + "kineti: timeout after 60000ms";
  }
  return {
    exitCode: res.status ?? 1,
    stdout: (res.stdout as any)?.toString?.() || (res.stdout as any) || "",
    stderr: stderr?.toString?.() || String(stderr),
  };
}

// Tools definitions according to MCP specification
const TOOLS = [
  {
    name: "kineti_status",
    description: "Retrieve current Kineti harness status: active project, locked root goal, current stage, and verification gates.",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "kineti_lock_goal",
    description: "Lock the immutable root goal for the current task run. Fails if goal is already locked.",
    inputSchema: {
      type: "object",
      properties: {
        goal: { type: "string", description: "The single, testable root goal for this run" },
      },
      required: ["goal"],
    },
  },
  {
    name: "kineti_set_stage",
    description: "Update the active stage or task in the Kineti OS runtime (e.g. 'build', 'spec', 'bugfix', 'refactor', or numbers 1-13).",
    inputSchema: {
      type: "object",
      properties: {
        stage: { type: "string", description: "Target stage name, number (1-13), or flexible task type (e.g. 'bugfix', 'refactor', 'feature')" },
        task_name: { type: "string", description: "Optional description of the task or objective" },
      },
      required: ["stage"],
    },
  },
  {
    name: "kineti_set_gate",
    description: "Set status for a pipeline quality gate ('spec', 'feasibility', 'security', 'audit').",
    inputSchema: {
      type: "object",
      properties: {
        gate: { type: "string", description: "Gate identifier (e.g. 'spec', 'security')" },
        status: { type: "string", enum: ["pass", "fail", "pending"], description: "Gate outcome" },
      },
      required: ["gate", "status"],
    },
  },
  {
    name: "kineti_evidence_record",
    description: "Execute a verification test command and record cryptographic SHA-256 fingerprint proof. Takes a command array against an allowlist (bun test, pytest, npm test, project verify). Shell needs human --allow-shell plus TTY.",
    inputSchema: {
      type: "object",
      properties: {
        label: { type: "string", description: "Unique proof label (e.g. 'unit-tests', 'lint')" },
        command: { description: "Command array or plain string without shell metachars (e.g. ['bun','test'] or 'bun test')", anyOf: [{ type: "string" }, { type: "array", items: { type: "string" } }] },
        allow_shell: { type: "boolean", description: "Human-only: allow shell with TTY confirm. MCP has no TTY so this always fails here." },
      },
      required: ["label", "command"],
    },
  },
  {
    name: "kineti_evidence_check",
    description: "Check if an existing verification proof is FRESH or STALE against current workspace code.",
    inputSchema: {
      type: "object",
      properties: {
        label: { type: "string", description: "Proof label to inspect" },
        expect_cmd: { type: "string", description: "Optional verification command substring expected" },
      },
      required: ["label"],
    },
  },
  {
    name: "kineti_spend_record",
    description: "Record LLM token consumption and enforce stage and task financial circuit breakers.",
    inputSchema: {
      type: "object",
      properties: {
        model: { type: "string", description: "Model identifier (e.g. 'claude-3-5-sonnet', 'gpt-4o')" },
        in_tokens: { type: "number", description: "Input prompt tokens consumed" },
        out_tokens: { type: "number", description: "Output completion tokens generated" },
        stage: { type: "string", description: "Active development stage" },
      },
      required: ["model", "in_tokens", "out_tokens"],
    },
  },
  {
    name: "kineti_spend_status",
    description: "Check current spend across stages, total task spend, and circuit breaker trip status.",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "kineti_saga_push",
    description: "Register an undo action on the LIFO rollback stack before executing a state mutation. Takes a command array against an allowlist. Shell needs human --allow-shell plus TTY.",
    inputSchema: {
      type: "object",
      properties: {
        step: { type: "string", description: "Human-readable label describing the mutation" },
        inverse: { description: "Undo command array or plain string without shell metachars", anyOf: [{ type: "string" }, { type: "array", items: { type: "string" } }] },
      },
      required: ["step", "inverse"],
    },
  },
  {
    name: "kineti_saga_rollback",
    description: "Rollback uncommitted actions on the LIFO saga stack in reverse order of creation.",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "kineti_verify_gate_status",
    description: "Inspect pre-flight verify gate status and current repository trust.",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "kineti_egress_record",
    description: "Record an outbound network request in the tamper-evident cryptographic egress ledger.",
    inputSchema: {
      type: "object",
      properties: {
        host: { type: "string", description: "Remote hostname accessed (e.g. 'api.github.com')" },
        desc: { type: "string", description: "Purpose of external network request" },
      },
      required: ["host", "desc"],
    },
  },
];

const STAGE_NAMES: Record<string, number> = {
  officehours: 1,
  diagnose: 2,
  design: 3,
  architecture: 4,
  feasibility: 5,
  spec: 6,
  build: 7,
  review: 8,
  qa: 9,
  security: 10,
  ship: 11,
  watch: 12,
  retro: 13,
};

function resolveStageNumber(val: any): number | null {
  const n = Number(val);
  if (Number.isInteger(n) && n >= 1 && n <= 13) return n;
  const s = String(val).toLowerCase().trim();
  return STAGE_NAMES[s] ?? null;
}

function handleToolCall(name: string, args: Record<string, any>): { content: { type: string; text: string }[]; isError?: boolean } {
  try {
    switch (name) {
      case "kineti_status": {
        const res = runBin("kineti-state.ts", ["get"]);
        if (res.exitCode !== 0) {
          return { content: [{ type: "text", text: res.stderr || "Failed to get status" }], isError: true };
        }
        return { content: [{ type: "text", text: res.stdout }] };
      }

      case "kineti_lock_goal": {
        const res = runBin("kineti-state.ts", ["set", "root_goal", String(args.goal)]);
        if (res.exitCode !== 0) {
          return { content: [{ type: "text", text: res.stderr || "Failed to lock root goal" }], isError: true };
        }
        return { content: [{ type: "text", text: `Root goal successfully locked: ${args.goal}` }] };
      }

      case "kineti_set_stage": {
        const num = resolveStageNumber(args.stage);
        const stageArg = num ? String(num) : String(args.stage);
        const res = runBin("kineti-state.ts", ["set", "stage", stageArg]);
        if (res.exitCode !== 0) {
          return { content: [{ type: "text", text: res.stderr || "Failed to set stage" }], isError: true };
        }
        if (args.task_name) {
          runBin("kineti-state.ts", ["set", "task.name", String(args.task_name)]);
        }
        return { content: [{ type: "text", text: num ? `Stage updated to: ${num} (${args.stage})` : `Stage updated to: ${args.stage}` }] };
      }

      case "kineti_set_gate": {
        const res = runBin("kineti-state.ts", ["set", `gate.${args.gate}`, String(args.status)]);
        if (res.exitCode !== 0) {
          return { content: [{ type: "text", text: res.stderr || `Failed to set gate.${args.gate}` }], isError: true };
        }
        return { content: [{ type: "text", text: `Gate '${args.gate}' set to: ${args.status}` }] };
      }

      case "kineti_evidence_record": {
        const cmdArgv = Array.isArray(args.command) ? (args.command as string[]).map(String) : splitLegacyCommand(String(args.command || ""));
        const runArgs = ["run", "--label", String(args.label), "--", ...cmdArgv];
        // MCP has no TTY, so shell is never allowed here. Never forward allow_shell.
        const res = runBin("kineti-evidence.ts", runArgs);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_evidence_check": {
        const cmdArgs = ["check", "--label", String(args.label)];
        if (args.expect_cmd) {
          cmdArgs.push("--expect-cmd", String(args.expect_cmd));
        }
        const res = runBin("kineti-evidence.ts", cmdArgs);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_spend_record": {
        const stage = args.stage || "build";
        const cmdArgs = [
          "log",
          "--stage", String(stage),
          "--model", String(args.model || "default"),
          "--tokens-in", String(args.in_tokens ?? 0),
          "--tokens-out", String(args.out_tokens ?? 0),
        ];
        const res = runBin("kineti-spend.ts", cmdArgs);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_spend_status": {
        const res = runBin("kineti-spend.ts", ["status"]);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_saga_push": {
        const inverseStr = Array.isArray(args.inverse) ? (args.inverse as string[]).map(String).join(" ") : String(args.inverse);
        const res = runBin("kineti-saga.ts", ["push", String(args.step), inverseStr]);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_saga_rollback": {
        const res = runBin("kineti-saga.ts", ["rollback"]);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_verify_gate_status": {
        const res = runBin("kineti-verify-gate.ts", ["--status"]);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_egress_record": {
        const res = runBin("kineti-egress.ts", ["record", "--host", String(args.host), "--desc", String(args.desc)]);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      default:
        return {
          content: [{ type: "text", text: `Unknown tool: ${name}` }],
          isError: true,
        };
    }
  } catch (err: any) {
    return {
      content: [{ type: "text", text: `Tool execution error: ${err.message}` }],
      isError: true,
    };
  }
}

// Stdio JSON-RPC dispatch loop
function startServer() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false,
  });

  rl.on("line", (line: string) => {
    const trimmed = line.trim();
    if (!trimmed) return;

    let req: any;
    try {
      req = JSON.parse(trimmed);
    } catch {
      sendResponse(null, { code: -32700, message: "Parse error" }, null);
      return;
    }

    const { id, method, params } = req;

    // Handshake
    if (method === "initialize") {
      sendResponse(id, null, {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: {
          tools: {},
        },
        serverInfo: {
          name: SERVER_NAME,
          version: SERVER_VERSION,
        },
      });
      return;
    }

    if (method === "notifications/initialized") {
      // Client handshake acknowledged
      return;
    }

    if (method === "ping") {
      sendResponse(id, null, {});
      return;
    }

    if (method === "tools/list") {
      sendResponse(id, null, { tools: TOOLS });
      return;
    }

    if (method === "tools/call") {
      const toolName = params?.name;
      const toolArgs = params?.arguments ?? {};
      const result = handleToolCall(toolName, toolArgs);
      sendResponse(id, null, result);
      return;
    }

    // Default: method not found
    if (id !== undefined && id !== null) {
      sendResponse(id, { code: -32601, message: `Method not found: ${method}` }, null);
    }
  });
}

function sendResponse(id: string | number | null, error: any, result: any) {
  const resp: any = { jsonrpc: "2.0", id };
  if (error) resp.error = error;
  else resp.result = result;
  process.stdout.write(JSON.stringify(resp) + "\n");
}

function handleInit(subArgs: string[]) {
  const targetDir = subArgs[0] ? path.resolve(subArgs[0]) : process.cwd();
  const cursorDir = path.join(targetDir, ".cursor");
  const cursorMcpFile = path.join(cursorDir, "mcp.json");

  fs.mkdirSync(cursorDir, { recursive: true });

  const mcpConfig = {
    mcpServers: {
      kineti: {
        command: "bun",
        args: [path.resolve(__filename), "--workspace-root", targetDir],
      },
    },
  };

  fs.writeFileSync(cursorMcpFile, JSON.stringify(mcpConfig, null, 2) + "\n");
  console.log(`[kineti-mcp] Created Cursor MCP config at: ${cursorMcpFile}`);
  console.log(`\nTo register with Claude Desktop, add the following to ~/Library/Application Support/Claude/claude_desktop_config.json:`);
  console.log(JSON.stringify(mcpConfig, null, 2));
}

startServer();
