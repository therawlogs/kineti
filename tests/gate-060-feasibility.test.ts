import { expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";

test("pinned Codex loads isolated native MCP configuration, discovers and calls without a model turn", () => {
  const root = path.resolve(import.meta.dir, "..");
  const result = Bun.spawnSync(["python3", "scripts/check-gate-060-codex.py"], {
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
    timeout: 120_000,
  });
  if (result.exitCode !== 0) {
    throw new Error(new TextDecoder().decode(result.stderr));
  }
  const state = path.join(root, ".kineti/gate-060-feasibility");
  const proof = JSON.parse(fs.readFileSync(path.join(state, "codex-probe.result.json"), "utf8"));
  const transcript = JSON.parse(fs.readFileSync(path.join(state, "codex-probe.transcript.json"), "utf8"));
  expect(proof.version).toBe("codex-cli 0.163.0-alpha.11.2");
  expect(proof.passed).toBe(true);
  expect(proof.fixture_only).toBe(true);
  expect(proof.model_turn_started).toBe(false);
  expect(proof.call.isError).toBe(false);
  const sent = transcript.filter((item: { direction: string }) => item.direction === "send");
  expect(sent.map((item: { message: { method: string } }) => item.message.method)).toEqual([
    "initialize", "initialized", "config/read", "thread/start", "mcpServerStatus/list", "mcpServer/tool/call",
  ]);
  expect(transcript.some((item: { message: { method?: string } }) => item.message.method === "turn/started")).toBe(false);
}, 130_000);
