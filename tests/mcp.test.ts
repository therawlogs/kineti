import { describe, test, expect } from "bun:test";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const REPO = path.resolve(import.meta.dir, "..");
const MCP_SCRIPT = path.join(REPO, "bin", "kineti-mcp.ts");

function createMcpSession(cwd: string) {
  const proc = Bun.spawn({
    cmd: ["bun", MCP_SCRIPT, "--workspace-root", cwd],
    cwd,
    env: { ...process.env, KINETI_TRUST_CONFIRMED: "1" },
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  });

  let buffer = "";

  async function sendRpc(req: any): Promise<any> {
    const line = JSON.stringify(req) + "\n";
    proc.stdin.write(line);
    await proc.stdin.flush();

    // Read until a line is received
    const reader = proc.stdout.getReader();
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      buffer += new TextDecoder().decode(value);
      const nl = buffer.indexOf("\n");
      if (nl !== -1) {
        const fullLine = buffer.slice(0, nl).trim();
        buffer = buffer.slice(nl + 1);
        reader.releaseLock();
        return JSON.parse(fullLine);
      }
    }
    reader.releaseLock();
    throw new Error("Process stdout closed before response");
  }

  function close() {
    proc.kill();
  }

  return { sendRpc, close };
}

describe("Kineti Universal MCP Server (kineti-mcp.ts)", () => {
  test("initializes handshake and advertises tools capability", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "mcp-test-"));
    const session = createMcpSession(tmpDir);

    try {
      const initResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          protocolVersion: "2024-11-05",
          clientInfo: { name: "test-client", version: "1.0.0" },
        },
      });

      expect(initResp.result).toBeDefined();
      expect(initResp.result.protocolVersion).toBe("2024-11-05");
      expect(initResp.result.serverInfo.name).toBe("kineti-harness");
      expect(initResp.result.capabilities.tools).toBeDefined();
    } finally {
      session.close();
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  test("tools/list returns 12 core Kineti governance tools", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "mcp-test-"));
    const session = createMcpSession(tmpDir);

    try {
      await session.sendRpc({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { protocolVersion: "2024-11-05" },
      });

      const listResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 2,
        method: "tools/list",
        params: {},
      });

      expect(listResp.result.tools).toBeDefined();
      const toolNames = listResp.result.tools.map((t: any) => t.name);
      expect(toolNames.length).toBe(12);
      expect(toolNames).toContain("kineti_status");
      expect(toolNames).toContain("kineti_lock_goal");
      expect(toolNames).toContain("kineti_set_stage");
      expect(toolNames).toContain("kineti_set_gate");
      expect(toolNames).toContain("kineti_evidence_record");
      expect(toolNames).toContain("kineti_evidence_check");
      expect(toolNames).toContain("kineti_spend_record");
      expect(toolNames).toContain("kineti_spend_status");
      expect(toolNames).toContain("kineti_saga_push");
      expect(toolNames).toContain("kineti_saga_rollback");
      expect(toolNames).toContain("kineti_verify_gate_status");
      expect(toolNames).toContain("kineti_egress_record");
    } finally {
      session.close();
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  test("executes state management tools via tools/call", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "mcp-test-"));
    // Initialize kineti state in tmpDir
    fs.mkdirSync(path.join(tmpDir, ".kineti"), { recursive: true });
    Bun.spawnSync(["bun", path.join(REPO, "bin", "kineti-state.ts"), "init", "--project", "test-proj"], { cwd: tmpDir });

    const session = createMcpSession(tmpDir);

    try {
      await session.sendRpc({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { protocolVersion: "2024-11-05" },
      });

      // 1. Lock root goal
      const lockResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
          name: "kineti_lock_goal",
          arguments: { goal: "Deliver high-reliability MCP server" },
        },
      });
      expect(lockResp.result.content[0].text).toContain("Deliver high-reliability MCP server");

      // 2. Set stage
      const stageResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
          name: "kineti_set_stage",
          arguments: { stage: "build" },
        },
      });
      expect(stageResp.result.content[0].text).toContain("7 (build)");

      // 3. Set gate status
      const gateResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 4,
        method: "tools/call",
        params: {
          name: "kineti_set_gate",
          arguments: { gate: "spec", status: "pass" },
        },
      });
      expect(gateResp.result.content[0].text).toContain("Gate 'spec' set to: pass");

      // 3b. Set gate to pending
      const gatePendingResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 41,
        method: "tools/call",
        params: {
          name: "kineti_set_gate",
          arguments: { gate: "spec", status: "pending" },
        },
      });
      expect(gatePendingResp.result.content[0].text).toContain("Gate 'spec' set to: pending");

      // 4. Status check
      const statusResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 5,
        method: "tools/call",
        params: {
          name: "kineti_status",
          arguments: {},
        },
      });
      const parsedStatus = JSON.parse(statusResp.result.content[0].text);
      expect(parsedStatus.root_goal).toBe("Deliver high-reliability MCP server");
      expect(parsedStatus.stage).toBe(7);
      expect(parsedStatus.gates.spec).toBe("pending");
    } finally {
      session.close();
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  test("executes evidence and spend governance tools via tools/call", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "mcp-test-"));
    fs.mkdirSync(path.join(tmpDir, ".kineti"), { recursive: true });
    fs.writeFileSync(path.join(tmpDir, "index.ts"), "console.log('hello');");
    Bun.spawnSync(["bun", path.join(REPO, "bin", "kineti-state.ts"), "init", "--project", "test-proj"], { cwd: tmpDir });

    const session = createMcpSession(tmpDir);

    try {
      await session.sendRpc({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: { protocolVersion: "2024-11-05" },
      });

      // 1. Record evidence
      const evResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: {
          name: "kineti_evidence_record",
          arguments: { label: "lint", command: "echo lint-passed" },
        },
      });
      expect(evResp.result.isError).toBeFalsy();
      expect(evResp.result.content[0].text).toContain("proof recorded: lint");

      // 2. Check evidence freshness
      const checkResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 3,
        method: "tools/call",
        params: {
          name: "kineti_evidence_check",
          arguments: { label: "lint" },
        },
      });
      expect(checkResp.result.content[0].text).toContain("FRESH");

      // 3. Record spend
      const spendResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 4,
        method: "tools/call",
        params: {
          name: "kineti_spend_record",
          arguments: { model: "claude-3-5-sonnet", in_tokens: 1500, out_tokens: 400, stage: "build" },
        },
      });
      expect(spendResp.result.isError).toBeFalsy();

      // 4. Check spend status
      const spendStatusResp = await session.sendRpc({
        jsonrpc: "2.0",
        id: 5,
        method: "tools/call",
        params: {
          name: "kineti_spend_status",
          arguments: {},
        },
      });
      expect(spendStatusResp.result.content[0].text).toContain("tripped=false");
    } finally {
      session.close();
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });

  test("CLI init command creates .cursor/mcp.json", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "mcp-init-test-"));
    try {
      const res = Bun.spawnSync(["bun", MCP_SCRIPT, "init", tmpDir]);
      expect(res.exitCode).toBe(0);

      const cursorMcpPath = path.join(tmpDir, ".cursor", "mcp.json");
      expect(fs.existsSync(cursorMcpPath)).toBe(true);

      const parsed = JSON.parse(fs.readFileSync(cursorMcpPath, "utf8"));
      expect(parsed.mcpServers.kineti).toBeDefined();
      expect(parsed.mcpServers.kineti.args).toContain("--workspace-root");
    } finally {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
  });
});
