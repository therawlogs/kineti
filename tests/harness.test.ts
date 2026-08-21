import { describe, test, expect, beforeAll } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const REPO = path.resolve(import.meta.dir, "..");

interface Res { status: number; out: string; err: string }

function run(
  prog: string,
  args: string[],
  ctx: { cwd: string; machine: string },
): Res {
  const p = Bun.spawnSync({
    cmd: ["bun", path.join(REPO, "bin", prog), ...args],
    cwd: ctx.cwd,
    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine },
    stdout: "pipe",
    stderr: "pipe",
  });
  return {
    status: p.exitCode ?? -1,
    out: p.stdout?.toString() ?? "",
    err: p.stderr?.toString() ?? "",
  };
}

function makeCtx(): { root: string; cwd: string; machine: string } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-test-"));
  const cwd = path.join(root, "proj");
  const machine = path.join(root, "machine");
  fs.mkdirSync(cwd, { recursive: true });
  fs.mkdirSync(machine, { recursive: true });
  return { root, cwd, machine };
}

describe("kineti-state", () => {
  test("goal locks forever; mutation refused with exit 3", () => {
    const c = makeCtx();
    expect(run("kineti-state.ts", ["init", "--project", "acme"], c).status).toBe(0);
    expect(run("kineti-state.ts", ["set", "root_goal", "ship search v1"], c).status).toBe(0);
    const blocked = run("kineti-state.ts", ["set", "root_goal", "different goal"], c);
    expect(blocked.status).toBe(3);
    expect(blocked.err).toContain("locked");
    expect(run("kineti-state.ts", ["get", "root_goal"], c).out.trim()).toBe("ship search v1");
    expect(run("kineti-state.ts", ["set", "stage", "14"], c).status).toBe(2);
    expect(run("kineti-state.ts", ["set", "gate.feasibility", "pass"], c).status).toBe(0);
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

describe("kineti-spend", () => {
  test("synthetic loop trips breaker; human-only reset", () => {
    const c = makeCtx();
    fs.writeFileSync(path.join(c.cwd, "kineti.config.json"), JSON.stringify({
      settings: { spend_limit_usd: { global: 1, per_stage_default: 0.4, safety_factor: 0.95 } },
    }));
    let trips = 0;
    let successes = 0;
    for (let i = 0; i < 100; i++) {
      const r = run("kineti-spend.ts", ["log", "--stage", "build", "--tokens-in", "1", "--tokens-out", "1", "--usd", "0.02"], c);
      if (r.status === 3) { trips++; break; }
      expect(r.status).toBe(0);
      successes++;
    }
    expect(trips).toBe(1);
    expect(successes).toBeLessThan(100);
    expect(run("kineti-spend.ts", ["check"], c).status).toBe(3);
    expect(run("kineti-spend.ts", ["reset"], c).status).toBe(2);
    expect(run("kineti-spend.ts", ["reset", "--i-am-human"], c).status).toBe(0);
    expect(run("kineti-spend.ts", ["check"], c).status).toBe(0);
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

describe("kineti-saga", () => {
  test("rollback unwinds newest-first and continues past a failing undo", () => {
    const c = makeCtx();
    const orderFile = path.join(c.root, "order.txt");
    const inv = (mark: string) => `echo ${mark} >> ${orderFile}`;
    const s = (a: string[]) => run("kineti-saga.ts", a, c);

    expect(s(["begin", "--run-id", "r1"]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-a", "--inverse", inv("A")]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-b", "--inverse", inv("B")]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-bad", "--inverse", "exit 7"]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-c", "--inverse", inv("C")]).status).toBe(0);

    const rb = s(["rollback", "--run-id", "r1"]);
    expect(rb.status).toBe(0);
    // newest-first across all pending steps; the failing undo does not stop the rest
    expect(fs.readFileSync(orderFile, "utf8").split("\n").filter(Boolean)).toEqual(["C", "B", "A"]);
    expect(rb.out).toContain("newest-first");

    // idempotent: second rollback has nothing pending
    expect(s(["rollback", "--run-id", "r1"]).out).toContain("nothing to roll back");
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

describe("kineti-evidence", () => {
  test("FRESH flips to STALE when code changes; MISSING when absent", () => {
    const c = makeCtx();
    fs.mkdirSync(path.join(c.cwd, "src"));
    fs.writeFileSync(path.join(c.cwd, "src", "a.ts"), "export const x = 1;\n");
    const e = (a: string[]) => run("kineti-evidence.ts", a, c);

    expect(e(["run", "--label", "unit", "--", "true"]).status).toBe(0);
    expect(e(["check", "--label", "unit"]).status).toBe(0);
    expect(e(["check", "--label", "unit"]).out).toContain("FRESH");

    fs.writeFileSync(path.join(c.cwd, "src", "b.ts"), "export const y = 2;\n");
    const stale = e(["check", "--label", "unit"]);
    expect(stale.status).toBe(4);
    expect(stale.err).toContain("STALE");

    expect(e(["check", "--label", "never-ran"]).status).toBe(5);
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

describe("kineti-verify-gate", () => {
  test("untrusted blocks (9); trusted failing blocks (1); trusted passing opens (0)", () => {
    const c = makeCtx();
    const cfg = path.join(c.cwd, "kineti.config.json");
    const writeCfg = (cmd: string) =>
      fs.writeFileSync(cfg, JSON.stringify({ settings: { verify_command: cmd } }));

    writeCfg("exit 1");
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(9);
    expect(run("kineti-verify-gate.ts", ["--trust"], c).status).toBe(0);
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(1);

    writeCfg("true");
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(9);
    run("kineti-verify-gate.ts", ["--trust"], c);
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(0);
    expect(run("kineti-verify-gate.ts", ["--status"], c).out).toContain("trusted and current");
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

describe("kineti-egress", () => {
  test("chain verifies; any edit breaks detection with exit 3", () => {
    const c = makeCtx();
    const e = (a: string[]) => run("kineti-egress.ts", a, c);

    expect(e(["record", "--host", "docs.example.com", "--description", "fetch framework docs"]).status).toBe(0);
    expect(e(["record", "--host", "api.vendor.io", "--description", "price table lookup"]).status).toBe(0);
    expect(e(["verify"]).status).toBe(0);
    expect(e(["verify"]).out).toContain("2 receipts");

    const ledger = path.join(c.machine, "egress.jsonl");
    const lines = fs.readFileSync(ledger, "utf8").trim().split("\n");
    const mid = JSON.parse(lines[0]);
    mid.description = "edited after the fact";
    lines[0] = JSON.stringify(mid);
    fs.writeFileSync(ledger, lines.join("\n") + "\n");

    expect(e(["verify"]).status).toBe(3);
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});
