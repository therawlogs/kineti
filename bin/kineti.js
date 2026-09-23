#!/usr/bin/env node
// @bun

// bin/kineti.ts
import { spawnSync } from "child_process";
import * as path from "path";
import * as fs from "fs";
import { fileURLToPath } from "url";
var here = typeof import.meta.dir === "string" ? import.meta.dir : path.dirname(fileURLToPath(import.meta.url));
var [subcommand, ...subArgs] = process.argv.slice(2);
var COMMAND_MAP = {
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
  stripe: { script: "bin/kineti-stripe.ts" },
  privacy: { script: "bin/kineti-privacy.ts" },
  invite: { script: "bin/kineti-invite.ts" }
};
if (!subcommand || subcommand === "--help" || subcommand === "-h") {
  printHelp();
  process.exit(0);
}
if (subcommand === "--version" || subcommand === "-v") {
  let version = "0.0.0-dev";
  try {
    const pkg = JSON.parse(fs.readFileSync(path.resolve(here, "../package.json"), "utf8"));
    if (pkg && pkg.version)
      version = pkg.version;
  } catch {}
  console.log(`kineti v${version}`);
  process.exit(0);
}
var target = COMMAND_MAP[subcommand];
if (!target) {
  console.error(`kineti: unknown command '${subcommand}'`);
  printHelp();
  process.exit(1);
}
var rootDir = path.resolve(here, "..");
var fullPath = path.join(rootDir, target.script);
var execArgs = subArgs;
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
  const probe = spawnSync("bun", ["--version"], { stdio: "ignore" });
  if (probe.error || probe.status !== 0) {
    console.error("kineti: Bun runtime is required. Install with: curl -fsSL https://bun.sh/install | bash");
    process.exit(1);
  }
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
  kineti stripe        Manage virtual cards with spend caps and SAGA undo
  kineti privacy       Self-serve external data purge & model training opt-out
  kineti invite        Viral vanity referrals and tier quotas
`);
}
