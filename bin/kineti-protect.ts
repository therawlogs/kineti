#!/usr/bin/env bun
/** Cursor MCP launcher for Kineti's opt-in protected-write tool. */
import { spawn } from "node:child_process";
import path from "node:path";

const server = path.join(import.meta.dir, "kineti-mcp.ts");
const child = spawn(process.execPath, [server, ...process.argv.slice(2)], { stdio: "inherit" });
child.on("error", (error) => { console.error(`kineti protect: could not start MCP server: ${error.message}`); process.exit(2); });
child.on("exit", (code, signal) => process.exit(code ?? (signal ? 1 : 0)));
