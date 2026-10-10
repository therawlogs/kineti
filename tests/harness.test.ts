import { describe, test, expect, beforeAll } from "bun:test";
import fs from "node:fs";
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
    env: { ...process.env, KINETI_MACHINE_DIR: ctx.machine, KINETI_TRUST_CONFIRMED: "1" },
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
  const scratch = path.join(REPO, ".kineti", "test-harness");
  fs.mkdirSync(scratch, { recursive: true });
  const root = fs.mkdtempSync(path.join(scratch, "run-"));
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

  test("stage-agnostic execution supports arbitrary task types and stages", () => {
    const c = makeCtx();
    expect(run("kineti-state.ts", ["init", "--project", "acme", "--task", "bugfix", "--task-name", "Fix OAuth callback timeout"], c).status).toBe(0);
    expect(run("kineti-state.ts", ["get", "stage"], c).out.trim()).toBe("bugfix");
    expect(run("kineti-state.ts", ["get", "task.type"], c).out.trim()).toBe("bugfix");
    expect(run("kineti-state.ts", ["get", "task.name"], c).out.trim()).toBe("Fix OAuth callback timeout");

    // Setting a custom stage like refactor
    expect(run("kineti-state.ts", ["set", "stage", "refactor"], c).status).toBe(0);
    expect(run("kineti-state.ts", ["get", "stage"], c).out.trim()).toBe("refactor");

    // Setting standard named stage resolves to its number
    expect(run("kineti-state.ts", ["set", "stage", "build"], c).status).toBe(0);
    expect(run("kineti-state.ts", ["get", "stage"], c).out.trim()).toBe("7");

    // Validation passes
    expect(run("kineti-state.ts", ["validate"], c).status).toBe(0);
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
    const s = (a: string[]) => run("kineti-saga.ts", a, c);

    expect(s(["begin", "--run-id", "r1"]).status).toBe(0);
    // Safe argv-only inverses: no shell. true = ok, false = fail (exit 1).
    expect(s(["register", "--run-id", "r1", "--label", "step-a", "--inverse", "true"]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-b", "--inverse", "true"]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-bad", "--inverse", "false"]).status).toBe(0);
    expect(s(["register", "--run-id", "r1", "--label", "step-c", "--inverse", "true"]).status).toBe(0);

    const rb = s(["rollback", "--run-id", "r1", "--yes"]);
    expect(rb.status).toBe(1);
    // Prints each inverse with its hash before running
    expect(rb.out).toContain("hash:");
    // newest-first across all pending steps; the failing undo does not stop the rest
    const idxC = rb.out.indexOf("undone: step-c");
    const idxB = rb.out.indexOf("undone: step-b");
    const idxA = rb.out.indexOf("undone: step-a");
    expect(idxC).toBeGreaterThan(-1);
    expect(idxB).toBeGreaterThan(-1);
    expect(idxA).toBeGreaterThan(-1);
    expect(idxC).toBeLessThan(idxB);
    expect(idxB).toBeLessThan(idxA);
    // failing step is reported, later steps still run, overall result is incomplete
    expect(rb.err).toContain('CRITICAL undo failed for "step-bad"');
    expect(rb.err).toContain("rollback incomplete");
    expect(rb.out).not.toContain("rollback complete");
    expect(rb.out).toContain("newest-first");
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("failed undo stays pending and a retry does not skip it", () => {
    const c = makeCtx();
    const s = (a: string[]) => run("kineti-saga.ts", a, c);

    expect(s(["begin", "--run-id", "r-fail"]).status).toBe(0);
    expect(s(["register", "--run-id", "r-fail", "--label", "ok-step", "--inverse", "true"]).status).toBe(0);
    expect(s(["register", "--run-id", "r-fail", "--label", "bad-step", "--inverse", "false"]).status).toBe(0);

    const first = s(["rollback", "--run-id", "r-fail", "--yes"]);
    expect(first.status).toBe(1);
    expect(first.err).toContain("rollback incomplete");
    expect(first.out).toContain("undone: ok-step");
    expect(first.err).toContain('CRITICAL undo failed for "bad-step"');

    const retry = s(["rollback", "--run-id", "r-fail", "--yes"]);
    expect(retry.status).toBe(1);
    expect(retry.out).not.toContain("nothing to roll back");
    expect(retry.out).toContain("[bad-step]");
    expect(retry.out).not.toContain("[ok-step]");
    expect(retry.err).toContain('CRITICAL undo failed for "bad-step"');
    expect(retry.err).toContain("rollback incomplete");
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("rollback without --yes in non-TTY refuses and runs nothing", () => {
    const c = makeCtx();
    const s = (a: string[]) => run("kineti-saga.ts", a, c);
    const canary = path.join(c.cwd, "canary.txt");

    expect(s(["begin", "--run-id", "r2"]).status).toBe(0);
    expect(s(["register", "--run-id", "r2", "--label", "mk", "--inverse", `touch ${canary}`]).status).toBe(0);

    const rb = s(["rollback", "--run-id", "r2"]);
    expect(rb.status).toBe(2);
    expect(rb.err).toContain("needs a human");
    expect(fs.existsSync(canary)).toBe(false);

    // Nothing was undone: --yes still sees the pending step
    const rb2 = s(["rollback", "--run-id", "r2", "--yes"]);
    expect(rb2.status).toBe(0);
    expect(fs.existsSync(canary)).toBe(true);
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("planted rm inverse never runs: guard catches direct file edits", () => {
    const c = makeCtx();
    const s = (a: string[]) => run("kineti-saga.ts", a, c);
    const canary = path.join(c.cwd, "keep.txt");
    fs.writeFileSync(canary, "keep\n");

    expect(s(["begin", "--run-id", "evil"]).status).toBe(0);
    expect(s(["register", "--run-id", "evil", "--label", "good", "--inverse", "true"]).status).toBe(0);

    // Attacker plants an entry by editing saga.jsonl directly (bypasses push, guard goes stale).
    const sagaFile = path.join(c.cwd, ".kineti", "saga.jsonl");
    fs.appendFileSync(sagaFile, JSON.stringify({ at: new Date().toISOString(), kind: "register", run_id: "evil", label: "pwn", inverse: `rm -f ${canary}` }) + "\n");

    // Even WITH --yes, guard mismatch refuses before running anything.
    const rb = s(["rollback", "--run-id", "evil", "--yes"]);
    expect(rb.status).toBe(3);
    expect(rb.err).toContain("changed outside push");
    expect(fs.existsSync(canary)).toBe(true);

    // Without --yes also refuses (guard first), never silent.
    const rb2 = s(["rollback", "--run-id", "evil"]);
    expect(rb2.status).toBe(3);
    expect(fs.existsSync(canary)).toBe(true);
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
  test("untrusted blocks (9); trusted failing blocks (1); trusted passing opens (0)", async () => {
    const c = makeCtx();
    const { sha256 } = await import("../bin/lib.ts");
    const cfg = path.join(c.cwd, "kineti.config.json");
    const writeCfg = (cmd: string) =>
      fs.writeFileSync(cfg, JSON.stringify({ settings: { verify_command: cmd } }));
    const writeTrust = (cmd: string) => {
      const trustFile = path.join(c.machine, "trust.json");
      const h = sha256(cmd);
      const at = new Date().toISOString();
      // macOS /var vs /private/var: child process.cwd() may resolve symlinks
      // differently than the test's tmp path, so store both forms.
      const keys = new Set([c.cwd]);
      try { keys.add(fs.realpathSync(c.cwd)); } catch {}
      const trust: Record<string, { cmd_hash: string; at: string }> = {};
      for (const k of keys) trust[k] = { cmd_hash: h, at };
      fs.writeFileSync(trustFile, JSON.stringify(trust));
    };

    writeCfg("exit 1");
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(9);
    // Simulate prior human trust (TTY + typed hash) by writing trust file directly.
    writeTrust("exit 1");
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(1);

    writeCfg("true");
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(9);
    writeTrust("true");
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(0);
    expect(run("kineti-verify-gate.ts", ["--status"], c).out).toContain("trusted and current");
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("verify-gate --trust rejects non-TTY and ignores env bypass", () => {
    const c = makeCtx();
    const cfg = path.join(c.cwd, "kineti.config.json");
    fs.writeFileSync(cfg, JSON.stringify({ settings: { verify_command: "true" } }));
    for (const token of ["", "1"]) {
      const p = Bun.spawnSync({
        cmd: ["bun", path.join(REPO, "bin", "kineti-verify-gate.ts"), "--trust"],
        cwd: c.cwd,
        env: { ...process.env, KINETI_MACHINE_DIR: c.machine, KINETI_TRUST_CONFIRMED: token },
        stdout: "pipe",
        stderr: "pipe",
      });
      expect(p.exitCode).toBe(2);
      expect(p.stderr.toString()).toContain("human TTY");
    }
    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("verify-gate fails closed on empty verify_command", () => {
    const c = makeCtx();
    const cfg = path.join(c.cwd, "kineti.config.json");
    fs.writeFileSync(cfg, JSON.stringify({ settings: { verify_command: "" } }));
    expect(run("kineti-verify-gate.ts", [], c).status).toBe(9);
    expect(run("kineti-verify-gate.ts", ["--status"], c).status).toBe(9);
    expect(run("kineti-verify-gate.ts", ["--trust"], c).status).toBe(2);
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

describe("hardening-cryptography-and-units", () => {
  test("computeDelimitedHash prevents second-preimage delimiter collisions across variable-length fields", async () => {
    const { computeDelimitedHash } = await import("../bin/lib.ts");

    // Case A: entity_type = "ToolCall", entity_id = "1234abcd"
    const hashA = computeDelimitedHash(["genesis", "ToolCall", "1234abcd", "payload_hash"]);

    // Case B: entity_type = "Tool", entity_id = "Call1234abcd"
    const hashB = computeDelimitedHash(["genesis", "Tool", "Call1234abcd", "payload_hash"]);

    // With naive concatenation ("genesisToolCall1234abcdpayload_hash"), these would be equal.
    // With computeDelimitedHash, length-prefixing and null delimiters guarantee distinct digests!
    expect(hashA).not.toEqual(hashB);
  });

  test("usdToMicrocents and microcentsToUsd maintain exact integer arithmetic without float drift", async () => {
    const { usdToMicrocents, microcentsToUsd } = await import("../bin/lib.ts");

    expect(usdToMicrocents(1.42)).toBe(1420000);
    expect(usdToMicrocents(0.0001)).toBe(100);
    expect(usdToMicrocents(0.000001)).toBe(1);

    expect(microcentsToUsd(1420000)).toBe(1.42);
    expect(microcentsToUsd(100)).toBe(0.0001);
  });

  test("kineti-spend persists and calculates total_microcents correctly", () => {
    const c = makeCtx();
    const s = (a: string[]) => run("kineti-spend.ts", a, c);

    expect(s(["log", "--stage", "build", "--tokens-in", "1000", "--tokens-out", "500", "--usd", "1.42"]).status).toBe(0);

    const spendFile = path.join(c.cwd, ".kineti", "spend.json");
    expect(fs.existsSync(spendFile)).toBe(true);
    const state = JSON.parse(fs.readFileSync(spendFile, "utf8"));
    expect(state.total_microcents).toBe(1420000);
    expect(state.by_stage_microcents.build).toBe(1420000);
    expect(state.total_usd).toBe(1.42);

    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("scaffoldRootHooks generates zero-touch root files for Claude, Antigravity, Cursor, and Codex", async () => {
    const { scaffoldRootHooks } = await import("../bin/lib.ts");
    const c = makeCtx();
    const installed = scaffoldRootHooks(c.cwd);
    expect(installed).toContain("CLAUDE.md");
    expect(installed).toContain("AGENTS.md");
    expect(installed).toContain(".cursor/rules/kineti.mdc");
    expect(installed).toContain("CODEX.md");

    expect(fs.existsSync(path.join(c.cwd, "CLAUDE.md"))).toBe(true);
    expect(fs.existsSync(path.join(c.cwd, "AGENTS.md"))).toBe(true);
    expect(fs.existsSync(path.join(c.cwd, ".cursor", "rules", "kineti.mdc"))).toBe(true);
    expect(fs.existsSync(path.join(c.cwd, "CODEX.md"))).toBe(true);

    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

describe("governance state file permissions", () => {
  test("writeJson and appendJsonl create and re-assert owner-only mode", async () => {
    const { writeJson, appendJsonl } = await import("../bin/lib.ts");
    const dir = fs.mkdtempSync(path.join(REPO, ".kineti", "scratch", "perm-"));
    try {
      const jsonFile = path.join(dir, "state.json");
      const jsonlFile = path.join(dir, "journal.jsonl");

      writeJson(jsonFile, { hello: "world" });
      appendJsonl(jsonlFile, { line: 1 });
      expect(fs.statSync(jsonFile).mode & 0o777).toBe(0o600);
      expect(fs.statSync(jsonlFile).mode & 0o777).toBe(0o600);

      // A file that was created world-readable must be locked down on next write.
      fs.chmodSync(jsonFile, 0o644);
      fs.chmodSync(jsonlFile, 0o644);
      writeJson(jsonFile, { hello: "again" });
      appendJsonl(jsonlFile, { line: 2 });
      expect(fs.statSync(jsonFile).mode & 0o777).toBe(0o600);
      expect(fs.statSync(jsonlFile).mode & 0o777).toBe(0o600);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
});

describe("audit findings regressions (I-01 to I-07)", () => {
  test("I-01 & I-02: evidence expiry and workspace mutation rejection", () => {
    const c = makeCtx();
    const e = (a: string[]) => run("kineti-evidence.ts", a, c);

    // Initial clean test pass
    expect(e(["run", "--label", "test-pass", "--", "true"]).status).toBe(0);
    expect(e(["check", "--label", "test-pass"]).status).toBe(0);

    // I-01: Malformed --max-age
    const malformed = e(["check", "--label", "test-pass", "--max-age", "4h"]);
    expect(malformed.status).toBe(2);
    expect(malformed.err).toContain("--max-age must be a positive finite number of minutes");

    const negativeAge = e(["check", "--label", "test-pass", "--max-age", "-10"]);
    expect(negativeAge.status).toBe(2);

    // Future timestamp receipt fails
    const evPath = path.join(c.cwd, ".kineti", "evidence.jsonl");
    const futureReceipt = {
      at: new Date(Date.now() + 3600000).toISOString(),
      label: "future-test",
      cmd: "true",
      exit_code: 0,
      fingerprint: "placeholder",
    };
    fs.appendFileSync(evPath, JSON.stringify(futureReceipt) + "\n");
    const futureCheck = e(["check", "--label", "future-test"]);
    expect(futureCheck.status).toBe(4);
    expect(futureCheck.err).toContain("receipt timestamp is in the future");

    // I-02: Workspace modification during test execution
    const canary = path.join(c.cwd, "canary-mutation.txt");
    const mutateRun = e(["run", "--label", "test-mutate", "--", "touch", canary]);
    expect(mutateRun.status).toBe(1);
    expect(mutateRun.err).toContain("guard failed: workspace was modified during test execution");

    // Check rejects mutated receipt as STALE
    const mutateCheck = e(["check", "--label", "test-mutate"]);
    expect(mutateCheck.status).toBe(4);
    expect(mutateCheck.err).toContain("workspace changed during test execution");

    // Opt-in allow-workspace-mutation works
    const allowedMutate = e(["run", "--label", "test-allowed-mutate", "--allow-workspace-mutation", "--", "touch", canary]);
    expect(allowedMutate.status).toBe(0);
    expect(e(["check", "--label", "test-allowed-mutate", "--allow-workspace-mutation"]).status).toBe(0);

    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("I-04 & I-05: unknown-model pricing and spend reset headroom", () => {
    const c = makeCtx();
    const s = (a: string[]) => run("kineti-spend.ts", a, c);

    // I-04: Unknown model without override or estimate fails with exit 2
    const unknownRun = s(["log", "--stage", "build", "--model", "mystery-ai-v9", "--tokens-in", "1000", "--tokens-out", "1000"]);
    expect(unknownRun.status).toBe(2);
    expect(unknownRun.err).toContain("unknown model 'mystery-ai-v9'");

    // Succeeded with --usd override
    const overrideRun = s(["log", "--stage", "build", "--model", "mystery-ai-v9", "--tokens-in", "1000", "--tokens-out", "1000", "--usd", "0.05"]);
    expect(overrideRun.status).toBe(0);
    expect(overrideRun.out).toContain("override");

    // Succeeded with --estimate
    const estimateRun = s(["log", "--stage", "build", "--model", "mystery-ai-v9", "--tokens-in", "1000", "--tokens-out", "1000", "--estimate"]);
    expect(estimateRun.status).toBe(0);
    expect(estimateRun.out).toContain("estimate");

    // I-05: Headroom reporting and reset
    const statusRes = s(["status"]);
    expect(statusRes.status).toBe(0);
    expect(statusRes.out).toContain("headroom_to_trip");

    // Reset with extend
    expect(s(["reset", "--i-am-human", "--extend-usd", "10"]).status).toBe(0);

    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("I-06: second-task new-run lifecycle archives state and clears gates", () => {
    const c = makeCtx();
    const st = (a: string[]) => run("kineti-state.ts", a, c);
    const sg = (a: string[]) => run("kineti-saga.ts", a, c);

    // Run 1: init and pass gate
    expect(st(["init", "--project", "multi-task-proj", "--goal", "Task 1"]).status).toBe(0);
    expect(st(["set", "gate.spec", "pass"]).status).toBe(0);
    expect(st(["set", "gate.security", "pass"]).status).toBe(0);
    expect(st(["get", "gate.spec"]).out.trim()).toBe("pass");

    // Pending SAGA prevents new-run without force
    expect(sg(["begin", "--run-id", "r1"]).status).toBe(0);
    expect(sg(["register", "--run-id", "r1", "--label", "edit", "--inverse", "true"]).status).toBe(0);
    const blockedNewRun = st(["new-run", "--goal", "Task 2"]);
    expect(blockedNewRun.status).toBe(2);
    expect(blockedNewRun.err).toContain("pending undo inverses exist");

    // Commit SAGA and start new-run
    expect(sg(["commit", "--run-id", "r1"]).status).toBe(0);
    const newRunRes = st(["new-run", "--goal", "Task 2", "--stage", "build"]);
    expect(newRunRes.status).toBe(0);
    expect(newRunRes.out).toContain("new run started (goal locked)");

    // Task 2 does NOT inherit task 1 gates
    expect(st(["get", "root_goal"]).out.trim()).toBe("Task 2");
    expect(st(["get", "stage"]).out.trim()).toBe("7");
    const gatesRes = run("kineti-state.ts", ["get", "gates"], c);
    expect(gatesRes.out.trim()).toBe("{}");

    // Archive file exists in .kineti/runs/
    const runsDir = path.join(c.cwd, ".kineti", "runs");
    expect(fs.existsSync(runsDir)).toBe(true);
    expect(fs.readdirSync(runsDir).length).toBeGreaterThanOrEqual(1);

    fs.rmSync(c.root, { recursive: true, force: true });
  });

  test("I-07: subcommand --help exits with 0 for all tools", () => {
    const c = makeCtx();
    expect(run("kineti-spend.ts", ["--help"], c).status).toBe(0);
    expect(run("kineti-evidence.ts", ["--help"], c).status).toBe(0);
    expect(run("kineti-state.ts", ["--help"], c).status).toBe(0);
    expect(run("kineti-ci.ts", ["--help"], c).status).toBe(0);
    expect(run("kineti-saga.ts", ["--help"], c).status).toBe(0);
    fs.rmSync(c.root, { recursive: true, force: true });
  });
});

