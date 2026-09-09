import { describe, test, expect } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const REPO = path.resolve(import.meta.dir, "..");

function makeCtx(): { root: string; cwd: string; machine: string } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-exec-safe-"));
  const cwd = path.join(root, "proj");
  const machine = path.join(root, "machine");
  fs.mkdirSync(cwd, { recursive: true });
  fs.mkdirSync(machine, { recursive: true });
  return { root, cwd, machine };
}

function run(prog: string, args: string[], ctx: { cwd: string; machine: string }) {
  const p = Bun.spawnSync({
    cmd: ["bun", path.join(REPO, "bin", prog), ...args],
    cwd: ctx.cwd,
    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine },
    stdout: "pipe",
    stderr: "pipe",
  });
  return { status: p.exitCode ?? -1, out: p.stdout?.toString() ?? "", err: p.stderr?.toString() ?? "" };
}

describe("0.1 shell removal", () => {
  test("; rm payload runs nothing and exits non-zero", () => {
    const c = makeCtx();
    fs.mkdirSync(path.join(c.cwd, "src"), { recursive: true });
    fs.writeFileSync(path.join(c.cwd, "src", "a.ts"), "export const x = 1;\n");
    const canary = path.join(c.cwd, "pwned.txt");
    // Try to break out via ; — must not create the file, must exit non-zero.
    const r = run("kineti-evidence.ts", ["run", "--label", "inject-semi", "--", "echo", "hi; touch", canary], c);
    expect(r.status).not.toBe(0);
    expect(r.err).toContain("blocked");
    expect(fs.existsSync(canary)).toBe(false);
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("$() payload runs nothing and exits non-zero", () => {
    const c = makeCtx();
    fs.mkdirSync(path.join(c.cwd, "src"), { recursive: true });
    fs.writeFileSync(path.join(c.cwd, "src", "a.ts"), "export const x = 1;\n");
    const canary = path.join(c.cwd, "pwned2.txt");
    const r = run("kineti-evidence.ts", ["run", "--label", "inject-dollar", "--", "echo", `$(touch ${canary})`], c);
    expect(r.status).not.toBe(0);
    expect(r.err).toContain("blocked");
    expect(fs.existsSync(canary)).toBe(false);
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("pipe and redirect payloads are blocked", () => {
    const c = makeCtx();
    const r1 = run("kineti-evidence.ts", ["run", "--label", "inject-pipe", "--", "echo", "hi", "|", "cat"], c);
    expect(r1.status).not.toBe(0);
    expect(r1.err).toContain("blocked");
    const r2 = run("kineti-evidence.ts", ["run", "--label", "inject-redir", "--", "echo", "hi", ">", "out.txt"], c);
    expect(r2.status).not.toBe(0);
    expect(r2.err).toContain("blocked");
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("non-allowlisted binary is blocked without --allow-shell", () => {
    const c = makeCtx();
    const r = run("kineti-evidence.ts", ["run", "--label", "inject-bin", "--", "curl", "https://example.com"], c);
    expect(r.status).not.toBe(0);
    expect(r.err).toContain("allowlist");
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("allowlisted commands still run: true, echo, bun", () => {
    const c = makeCtx();
    fs.mkdirSync(path.join(c.cwd, "src"), { recursive: true });
    fs.writeFileSync(path.join(c.cwd, "src", "a.ts"), "export const x = 1;\n");
    expect(run("kineti-evidence.ts", ["run", "--label", "ok-true", "--", "true"], c).status).toBe(0);
    expect(run("kineti-evidence.ts", ["run", "--label", "ok-echo", "--", "echo", "hello"], c).status).toBe(0);
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("saga rollback blocks shell inverses without --allow-shell", () => {
    const c = makeCtx();
    const s = (a: string[]) => run("kineti-saga.ts", a, c);
    expect(s(["begin", "--run-id", "evil"]).status).toBe(0);
    expect(s(["register", "--run-id", "evil", "--label", "bad", "--inverse", "echo hi; rm -rf /"]).status).toBe(0);
    const rb = s(["rollback", "--run-id", "evil"]);
    expect(rb.status).toBe(0);
    // Blocked step exits non-zero, reported as failed, continues.
    expect(rb.err).toContain("CRITICAL undo failed");
    expect(rb.err).toContain("blocked");
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("safe exec unit: timeout kills long runs", async () => {
    const { runSafeCommand } = await import("../bin/lib.ts");
    const res = runSafeCommand(["sleep", "2"], { cwd: process.cwd(), workspaceRoot: process.cwd(), timeoutMs: 400 });
    expect(res.exitCode).not.toBe(0);
    expect(res.stderr).toContain("timeout");
  });
});
