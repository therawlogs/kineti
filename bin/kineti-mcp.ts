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
import { checkedOptions } from "./strict-options.ts";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import * as readline from "node:readline";
import {createHash} from "node:crypto";
import { readJson, projectKdir, loadLimits, loadVerifyCommand, Limits, splitLegacyCommand, assertInsideProject, isAllowedEvidenceCommand } from "./lib.ts";
import { writeCursorConfig } from "./kineti-config.ts";
import { applyProtectedWrite, GateFailure } from "./kineti-protected-write.ts";
import { ContextMemoryStore, localContextScope } from "./kineti-memory-job.ts";
import { submitServiceRequest } from "./kineti-service-client.ts";
import { buildMemoryModelRequest } from "./kineti-context.ts";

// Works under bun (import.meta.dir) and plain node (file URL fallback)
const here: string =
  typeof (import.meta as any).dir === "string"
    ? (import.meta as any).dir
    : path.dirname(fileURLToPath(import.meta.url));

const PROTOCOL_VERSION = "2024-11-05";
const SERVER_NAME = "kineti-harness";
const SERVER_VERSION = resolvePackageVersion();

function resolvePackageVersion(): string {
  try {
    const pkg = JSON.parse(fs.readFileSync(path.resolve(here, "../package.json"), "utf8"));
    if (pkg && pkg.version) return String(pkg.version);
  } catch {}
  return "0.0.0-dev";
}

// Workspace root resolution
let workspaceRoot = process.cwd();
const args = checkedOptions(process.argv.slice(2), ["--workspace-root", "--host"], [], [], 2);

// Check if run as CLI helper (e.g. `kineti-mcp init`)
if (args[0] === "init") {
  handleInit(args.slice(1));
  process.exit(0);
}

// Parse --workspace-root if provided
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--workspace-root" && args[i + 1]) {
    workspaceRoot = assertInsideProject(path.resolve(args[i + 1]), process.env.KINETI_WORKSPACE_ROOT || process.cwd());
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
  const res = spawnSync(process.execPath, [scriptPath, ...subArgs], {
    cwd: safeRoot,
    env: { ...process.env, KINETI_MACHINE_DIR: process.env.KINETI_MACHINE_DIR || path.join(safeRoot, ".kineti", "machine") },
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
        required_tests: { type: "array", items: { type: "string" }, description: "Required test inventory paths; all must exist and be fingerprinted." },
        protected_write: { type: "object", description: "Optional exact destination, arguments, and content to bind to this test receipt." },
      },
      required: ["label", "command"],
    },
  },
  {
    name: "kineti_linux_service",
    description: "Submit an owner-signed artifact for independent Linux evaluation, export approved bytes, or inspect a durable request. Requires provisioned owner-only service-client configuration. Protects only the service-owned output; native editor writes remain outside this action.",
    inputSchema: {type:"object",properties:{operation:{type:"string",enum:["evaluate","export","inspect"]},arguments:{type:"object"}},required:["operation","arguments"]},
  },
  {
    name: "kineti_protected_write",
    description: "Write only the exact bytes, destination, and arguments bound to a fresh versioned Kineti receipt. Only writes routed through this MCP tool are gated; host-native file tools are outside its protection.",
    inputSchema: { type: "object", properties: { label: { type: "string" }, run_id: { type: "string" }, destination: { type: "string" }, arguments: {}, content: { type: "string" } }, required: ["label", "run_id", "destination", "arguments", "content"] },
  },
  {
    name: "kineti_context_memory",
    description: "Store, retrieve, prepare a bounded model context payload, correct, or delete encrypted memory scoped to the local OS user and current project. Memory is selected before it is returned to the host.",
    inputSchema: {
      type: "object", properties: {
        action: { type: "string", enum: ["store", "retrieve", "prepare", "correct", "delete"] },
        id: { type: "string", pattern: "^mem-[a-zA-Z0-9][a-zA-Z0-9-]{2,100}$", description: "Memory id beginning with mem-. For correction, the existing id; for delete, the id to remove." }, new_id: { type: "string", pattern: "^mem-[a-zA-Z0-9][a-zA-Z0-9-]{2,100}$", description: "New mem- id for a correction." }, content: { type: "string" },
        query: { type: "string" }, prompt: { type: "string", description: "Current user request for the prepared model payload." }, limit: { type: "integer", minimum: 1, maximum: 20 },
        tags: { type: "array", items: { type: "string" } },
        sources: { type: "array", items: { type: "object", properties: { source_id: { type: "string" }, path: { type: "string" }, sha256: { type: "string" } }, required: ["source_id", "path", "sha256"] } },
        confirm: { type: "string", description: "For deletion, repeat the exact memory id." },
      }, required: ["action"],
    },
  },
  {
    name: "kineti_workflow",
    description: "Inspect project workflow progress, record exact artifact versions, and register numbered scope conflicts. Human approvals require the Kineti terminal command.",
    inputSchema: { type: "object", properties: {
      action: { type: "string", enum: ["status", "record", "conflict-add", "conflict-list"] },
      step: { type: "string", enum: ["discovery", "design", "data-intake", "no-data", "architecture", "outcome"] },
      file: { type: "string", description: "Relative project path to the artifact" },
      id: { type: "string" }, request: { type: "string" }, affected: { type: "array", items: { type: "string" } },
      options: { type: "array", minItems: 2, items: { type: "string" } },
    }, required: ["action"] },
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
        usd_override: { type: "number", description: "Direct USD cost override" },
        estimate: { type: "boolean", description: "Allow fallback estimate pricing for unknown models" },
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
    description: "Rollback uncommitted actions on the LIFO saga stack. Human-only: needs a TTY y/N or explicit --yes; MCP calls fail closed with instructions.",
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
  {
    name: "kineti_talk",
    description: "Talk to Kineti in plain words. No skill or command names needed. Handles spending, undo, proof, status, yes/no, on/off, and routes tasks. Always returns plain words plus numbered choices.",
    inputSchema: {
      type: "object",
      properties: {
        message: { type: "string", description: "The user's plain words, e.g. 'how much have I spent?'" },
      },
      required: ["message"],
    },
  },
  {
    name: "kineti_epistemic_eval",
    description: "Evaluate a candidate action against the 360º Human Model (Rule-Exception hierarchies, Epistemic certainty, and Safety ceilings).",
    inputSchema: {
      type: "object",
      properties: {
        item: { type: "string", description: "Candidate item or action to evaluate (e.g. 'Chicken Biryani')" },
        tags: { type: "string", description: "Comma-separated tags (e.g. 'meat,chicken' or 'eggs')" },
        scope: { type: "string", description: "Context scope (health, work, finance, schedule, taste)" },
      },
      required: ["item"],
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

async function handleToolCall(name: string, args: Record<string, any>): Promise<{ content: { type: string; text: string }[]; isError?: boolean }> {
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
        const gate = isAllowedEvidenceCommand(cmdArgv, loadVerifyCommand(workspaceRoot));
        if (!gate.allowed) {
          return { content: [{ type: "text", text: `kineti: blocked: ${gate.reason}` }], isError: true };
        }
        const runArgs = ["run", "--label", String(args.label), "--", ...cmdArgv];
        if (Array.isArray(args.required_tests)) for (const testPath of args.required_tests) runArgs.splice(runArgs.indexOf("--"), 0, "--required-test", String(testPath));
        if (args.protected_write) runArgs.splice(runArgs.indexOf("--"), 0, "--protected-write", JSON.stringify(args.protected_write));
        // MCP has no TTY, so shell is never allowed here. Never forward allow_shell.
        const res = runBin("kineti-evidence.ts", runArgs);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_linux_service": {
        if (!["evaluate","export","inspect"].includes(args.operation) || !args.arguments || typeof args.arguments!=="object" || Array.isArray(args.arguments)) throw new Error("invalid Linux service request");
        const response=await submitServiceRequest(args.operation,args.arguments,workspaceRoot);
        return {content:[{type:"text",text:JSON.stringify(response)}],isError:response.http_status===null || response.http_status>=400};
      }
      case "kineti_protected_write": {
        try {
          const text = applyProtectedWrite({ label: args.label, runId: args.run_id, destination: args.destination, ...(Object.hasOwn(args, "arguments") ? { arguments: args.arguments } : {}), content: args.content }, workspaceRoot);
          return { content: [{ type: "text", text }] };
        } catch (error) {
          const code = error instanceof GateFailure ? error.exitCode : 3;
          return { content: [{ type: "text", text: `kineti: ${code === 1 ? "blocked" : code === 2 ? "invalid input" : "write failure"} (exit ${code}): ${(error as Error).message}` }], isError: true };
        }
      }

      case "kineti_context_memory": {
        const action = String(args.action);
        if (!["store", "retrieve", "prepare", "correct", "delete"].includes(action)) return { content: [{ type: "text", text: "kineti: invalid memory action" }], isError: true };
        try {
          const memory = new ContextMemoryStore(workspaceRoot, workspaceRoot), scope = localContextScope(workspaceRoot);
          if (action === "retrieve") return { content: [{ type: "text", text: JSON.stringify(memory.retrieve(scope, String(args.query ?? ""), args.limit === undefined ? 8 : Number(args.limit)), null, 2) }] };
          if (action === "prepare") {
            if (typeof args.prompt !== "string") throw new Error("memory prepare requires prompt text");
            const request = buildMemoryModelRequest(memory, scope, args.prompt, String(args.query ?? ""), args.limit === undefined ? 8 : Number(args.limit));
            return { content: [{ type: "text", text: JSON.stringify(request, null, 2) }] };
          }
          if (action === "store" || action === "correct") {
            const id = String(action === "store" ? args.id ?? "" : args.new_id ?? "");
            if (!id || typeof args.content !== "string") throw new Error("memory store requires id and content");
            const record = memory.append({ id, scope, content: args.content,
              tags: args.tags === undefined ? [] : args.tags as string[],
              sources: args.sources === undefined ? [] : args.sources as any[],
              ...(action === "correct" ? { correction_of: String(args.id ?? "") } : {}) });
            return { content: [{ type: "text", text: `STORED ${record.id}: ${record.data.content_sha256}` }] };
          }
          const id = String(args.id ?? "");
          if (!id || args.confirm !== id) throw new Error("memory delete requires confirm to repeat the exact memory id");
          memory.delete(id, scope);
          return { content: [{ type: "text", text: `DELETED ${id}: encrypted content and key removed; only journal audit metadata remains` }] };
        } catch (error) {
          return { content: [{ type: "text", text: `kineti: memory operation failed: ${(error as Error).message}` }], isError: true };
        }
      }

      case "kineti_workflow": {
        const action = String(args.action), cmdArgs: string[] = [];
        if (action === "status") cmdArgs.push("status");
        else if (action === "record") {
          if (!args.step || !args.file) return { content: [{ type: "text", text: "kineti: workflow record requires step and file" }], isError: true };
          cmdArgs.push("record", "--step", String(args.step), "--file", String(args.file));
        } else if (action === "conflict-add") {
          if (!args.id || !args.request || !Array.isArray(args.affected) || !Array.isArray(args.options)) return { content: [{ type: "text", text: "kineti: conflict-add requires id, request, affected, and options" }], isError: true };
          cmdArgs.push("conflict", "add", "--id", String(args.id), "--request", String(args.request), "--affected", args.affected.map(String).join(","));
          for (const option of args.options) cmdArgs.push("--option", String(option));
        } else if (action === "conflict-list") cmdArgs.push("conflict", "list");
        else return { content: [{ type: "text", text: "kineti: invalid workflow action" }], isError: true };
        const res = runBin("kineti-workflow.ts", cmdArgs);
        return { content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }], isError: res.exitCode !== 0 };
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
        if (args.usd_override !== undefined && args.usd_override !== null) {
          cmdArgs.push("--usd", String(args.usd_override));
        }
        if (args.estimate) {
          cmdArgs.push("--estimate");
        }
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
        const structured = Array.isArray(args.inverse);
        if (structured && (!args.inverse.length || !args.inverse[0] || !args.inverse.every((part: unknown) => typeof part === "string" && !part.includes("\0")))) throw new Error("inverse must be a nonempty argv array of strings");
        const command = structured ? ["push", "--label", String(args.step), "--inverse-argv", JSON.stringify(args.inverse)] : ["push", String(args.step), String(args.inverse)];
        const res = runBin("kineti-saga.ts", command);
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

      case "kineti_talk": {
        const res = runBin("kineti-router.ts", [String(args.message || "")]);
        return {
          content: [{ type: "text", text: (res.stdout + (res.stderr ? `\n${res.stderr}` : "")).trim() }],
          isError: res.exitCode !== 0,
        };
      }

      case "kineti_epistemic_eval": {
        const subArgs = ["eval", "--item", String(args.item || "")];
        if (args.tags) subArgs.push("--tags", String(args.tags));
        if (args.scope) subArgs.push("--scope", String(args.scope));
        const res = runBin("kineti-epistemic.ts", subArgs);
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

  rl.on("line", async (line: string) => {
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
      const result = await handleToolCall(toolName, toolArgs);
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
  const hostIndex=subArgs.indexOf("--host"), host=hostIndex<0?"cursor":subArgs[hostIndex+1];
  if(host!=="cursor"&&host!=="antigravity")throw new Error("MCP host must be cursor or antigravity");
  const rawTargetDir = subArgs[0] && !subArgs[0].startsWith("--") ? path.resolve(subArgs[0]) : process.cwd();
  const targetDir = assertInsideProject(rawTargetDir, process.env.KINETI_WORKSPACE_ROOT || process.cwd());
  const cursorDir = host==="cursor"?path.join(targetDir,".cursor"):path.join(targetDir,".agents/plugins/kineti");
  const cursorMcpFile = path.join(cursorDir,host==="cursor"?"mcp.json":"mcp_config.json");

  const kinetiServer = {
    command: process.execPath,
    args: [path.join(here, "kineti-protect.ts"), "--workspace-root", targetDir],
    env: { KINETI_WORKSPACE_ROOT: targetDir },
  };
  if(host==="antigravity") {
    const marker=path.join(cursorDir,"plugin.json");
    if(fs.existsSync(marker)) {if(JSON.parse(fs.readFileSync(marker,"utf8")).name!=="Kineti")throw new Error("existing plugin is not Kineti; configuration preserved");}
    else {
      const undo=spawnSync(process.execPath,[path.join(here,"kineti-saga.ts"),"push","antigravity-plugin-marker","--inverse-argv",JSON.stringify(["rm","-f",path.relative(targetDir,marker)])],{cwd:targetDir,encoding:"utf8"});
      if(undo.status!==0)throw new Error("could not register plugin undo");
      fs.mkdirSync(cursorDir,{recursive:true});fs.writeFileSync(marker,JSON.stringify({name:"Kineti"})+"\n",{flag:"wx",mode:0o600});
    }
  }
  const serverName=host==="antigravity"?`kineti_${createHash("sha256").update(fs.realpathSync(targetDir)).digest("hex").slice(0,12)}`:"kineti";
  const backup = writeCursorConfig(cursorMcpFile, kinetiServer,{serverName,migrateLegacy:host==="antigravity"});
  const mcpConfig = { mcpServers: { [serverName]: kinetiServer } };
  console.log(`[kineti-mcp] Merged ${host} MCP config at: ${cursorMcpFile}`);
  if (fs.existsSync(backup)) console.log(`[kineti-mcp] Backup: ${backup}`);
  console.log(`\nTo register with Claude Desktop, add the following to ~/Library/Application Support/Claude/claude_desktop_config.json:`);
  console.log(JSON.stringify(mcpConfig, null, 2));
}

startServer();
