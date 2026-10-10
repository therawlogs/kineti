#!/usr/bin/env bun
/**
 * Kineti OS - single command router.
 * Single source of truth: bin/kineti.js is GENERATED from this file.
 * Never hand-edit bin/kineti.js. Regenerate with: bun run build:router
 *
 * Use:
 *   kineti init [--host <name>]
 *   kineti state init --project <name> --goal <text>
 *   kineti companion [--port 8788]
 *   kineti mcp
 *   kineti test -- <cmd>
 *   kineti undo
 *   kineti spend [status|reset|add]
 *   kineti status
 *   kineti verify
 *   kineti swarm <goal>
 *   kineti ci [--require-evidence <label> ...]
 *   kineti seed [--stage <1-13>]
 */

import { spawnSync } from "node:child_process";
import * as path from "node:path";
import * as fs from "node:fs";
import { fileURLToPath } from "node:url";

// Works under bun (import.meta.dir) and plain node (file URL fallback)
// so this file stays the single router source for both runtimes.
const here: string =
  typeof (import.meta as any).dir === "string"
    ? (import.meta as any).dir
    : path.dirname(fileURLToPath(import.meta.url));

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
  seed: { script: "bin/kineti-seed.ts" },
  egress: { script: "bin/kineti-egress.ts" },
  memory: { script: "bin/kineti-memory-job.ts" },
};

if (!subcommand || subcommand === "--help" || subcommand === "-h") {
  printHelp();
  process.exit(0);
}

if (subcommand === "--version" || subcommand === "-v") {
  let version = "0.0.0-dev";
  try {
    const pkg = JSON.parse(fs.readFileSync(path.resolve(here, "../package.json"), "utf8"));
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

const rootDir = path.resolve(here, "..");
const fullPath = path.join(rootDir, target.script);

let execArgs = subArgs;
const isHelp = subArgs.includes("--help") || subArgs.includes("-h");

if (subcommand === "spend" && subArgs.length === 0) {
  execArgs = ["status"];
} else if (subcommand === "test") {
  if (isHelp && (subArgs.length === 1 || subArgs[0] === "--help" || subArgs[0] === "-h")) {
    execArgs = ["--help"];
  } else if (subArgs[0] === "check") {
    execArgs = subArgs;
  } else if (subArgs[0] === "run") {
    const hasLabel = subArgs.includes("--label");
    execArgs = hasLabel ? subArgs : ["run", "--label", "test", ...subArgs.slice(1)];
  } else if (subArgs.length > 0) {
    const hasLabel = subArgs.includes("--label");
    execArgs = hasLabel ? ["run", ...subArgs] : ["run", "--label", "test", ...subArgs];
  }
} else if (subcommand === "undo" && subArgs.length === 0) {
  execArgs = ["rollback"];
} else if ((subcommand === "status" || subcommand === "state") && subArgs.length === 0) {
  execArgs = ["get"];
}

if (target.isShell) {
  const res = spawnSync("bash", [fullPath, ...execArgs], { stdio: "inherit", cwd: process.cwd() });
  if (res.error) {
    console.error(`kineti: failed to execute ${fullPath}: ${res.error.message}`);
    process.exit(1);
  }
  process.exit(res.status ?? 0);
} else {
  // npm bin runs under node: sub-scripts still need the bun runtime.
  const probe = spawnSync("bun", ["--version"], { stdio: "ignore" });
  if (probe.error || probe.status !== 0) {
    console.error("kineti: Bun runtime is required. Install with: curl -fsSL https://bun.sh/install | bash");
    process.exit(1);
  }
  const res = spawnSync("bun", [fullPath, ...execArgs], { stdio: "inherit", cwd: process.cwd() });
  if (res.error) {
    console.error(`kineti: failed to execute bun ${fullPath}: ${res.error.message}`);
    process.exit(1);
  }
  process.exit(res.status ?? 1);
}

function printHelp() {
  console.log(`
Kineti OS - safe runtime for AI coding agents

Commands:
  kineti init          Install agent instructions and skills
  kineti state init    Create local project state and lock a goal
  kineti seed          Create missing CI state from kineti.config.json
  kineti companion     Open visual dashboard (http://127.0.0.1:8788)
  kineti mcp           Start MCP server
  kineti test -- <cmd> Run tests and save proof
  kineti undo          Undo last change
  kineti spend         Show spend vs $50 cap
  kineti status        Show task and gate state
  kineti verify        Check test proof before commit
  kineti swarm <goal>  Run multi-agent task
  kineti ci            Check required test evidence and project gates
`);
}
