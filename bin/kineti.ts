#!/usr/bin/env bun
/**
 * Kineti OS - single command router.
 *
 * Use:
 *   kineti init [--host <name>]
 *   kineti companion [--port 8788]
 *   kineti mcp
 *   kineti test -- <cmd>
 *   kineti undo
 *   kineti spend [status|reset|add]
 *   kineti status
 *   kineti verify
 *   kineti swarm <goal>
 *   kineti ci
 */

import { spawnSync } from "node:child_process";
import * as path from "node:path";
import * as fs from "node:fs";

const [subcommand, ...subArgs] = process.argv.slice(2);

const COMMAND_MAP: Record<string, { script: string; isShell?: boolean }> = {
  init: { script: "setup.sh", isShell: true },
  companion: { script: "bin/kineti-companion.ts" },
  mcp: { script: "bin/kineti-mcp.ts" },
  test: { script: "bin/kineti-evidence.ts" },
  evidence: { script: "bin/kineti-evidence.ts" },
  undo: { script: "bin/kineti-saga.ts" },
  saga: { script: "bin/kineti-saga.ts" },
  spend: { script: "bin/kineti-spend.ts" },
  status: { script: "bin/kineti-state.ts" },
  state: { script: "bin/kineti-state.ts" },
  verify: { script: "bin/kineti-verify-gate.ts" },
  swarm: { script: "bin/kineti-swarm.ts" },
  ci: { script: "bin/kineti-ci.ts" },
  egress: { script: "bin/kineti-egress.ts" },
  memory: { script: "bin/kineti-memory-job.ts" },
};

if (!subcommand || subcommand === "--help" || subcommand === "-h") {
  printHelp();
  process.exit(0);
}

if (subcommand === "--version" || subcommand === "-v") {
  let version = "0.1.0";
  try {
    const pkg = JSON.parse(fs.readFileSync(path.resolve(import.meta.dir, "../package.json"), "utf8"));
    if (pkg && pkg.version) version = pkg.version;
  } catch {}
  console.log(`kineti v${version}`);
  process.exit(0);
}

const target = COMMAND_MAP[subcommand];
if (!target) {
  console.error(`kineti: unknown command '${subcommand}'`);
  printHelp();
  process.exit(1);
}

const rootDir = path.resolve(import.meta.dir, "..");
const fullPath = path.join(rootDir, target.script);

let execArgs = subArgs;
if (subcommand === "spend" && subArgs.length === 0) {
  execArgs = ["status"];
} else if (subcommand === "test" && subArgs.length > 0 && subArgs[0] !== "run" && subArgs[0] !== "check") {
  execArgs = ["run", ...subArgs];
} else if (subcommand === "undo" && subArgs.length === 0) {
  execArgs = ["rollback"];
} else if ((subcommand === "status" || subcommand === "state") && subArgs.length === 0) {
  execArgs = ["get"];
}

if (target.isShell) {
  const res = spawnSync("bash", [fullPath, ...execArgs], { stdio: "inherit", cwd: process.cwd() });
  process.exit(res.status ?? 0);
} else {
  const res = spawnSync("bun", [fullPath, ...execArgs], { stdio: "inherit", cwd: process.cwd() });
  process.exit(res.status ?? 0);
}

function printHelp() {
  console.log(`
Kineti OS - safe runtime for AI coding agents

Commands:
  kineti init          Set up project rules and Cursor/Claude MCP
  kineti companion     Open visual dashboard (http://127.0.0.1:8788)
  kineti mcp           Start MCP server
  kineti test -- <cmd> Run tests and save proof
  kineti undo          Undo last change
  kineti spend         Show spend vs $50 cap
  kineti status        Show task and gate state
  kineti verify        Check test proof before commit
  kineti swarm <goal>  Run multi-agent task
  kineti ci            Run PR check
`);
}
