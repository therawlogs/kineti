import { test, expect } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import crypto from "node:crypto";
import { SchedulerEngine } from "../src/scheduler/engine.ts";
import { generateCIReport } from "../bin/kineti-ci.ts";
import { fingerprint, unsignedRecord, type EvidenceRecord } from "../bin/kineti-evidence.ts";
import { readJsonl } from "../bin/lib.ts";

const repo = path.resolve(import.meta.dir, "..");
const scratch = path.join(repo, ".kineti", "completion-regressions");
fs.mkdirSync(scratch, { recursive: true });
function fixture() { return fs.mkdtempSync(path.join(scratch, "run-")); }
function spend(root: string, args: string[]) {
  return Bun.spawnSync([process.execPath, path.join(repo, "bin/kineti-spend.ts"), ...args], {
    cwd: root, env: { ...process.env, KINETI_MACHINE_DIR: path.join(root, ".kineti", "machine") },
  });
}

test("repeated local hours always advance and visit both 01:30 instants", () => {
  const previous = process.env.TZ;
  process.env.TZ = "America/New_York";
  const root = fixture();
  try {
    const engine = new SchedulerEngine(path.join(root, "jobs.json"));
    for (const anchor of ["2026-11-01T05:59:30Z", "2026-11-01T06:15:30Z"]) {
      const time = Date.parse(anchor);
      expect(engine.calculateNextCronRun("* * * * *", new Date(time)).getTime()).toBe(Math.floor(time / 60000) * 60000 + 60000);
    }
    const first = engine.calculateNextCronRun("30 1 * * *", new Date("2026-11-01T05:29:30Z"));
    expect(first.toISOString()).toBe("2026-11-01T05:30:00.000Z");
    expect(engine.calculateNextCronRun("30 1 * * *", first).toISOString()).toBe("2026-11-01T06:30:00.000Z");
  } finally {
    if (previous === undefined) delete process.env.TZ; else process.env.TZ = previous;
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("simultaneous spend writers retain every entry and exact total", async () => {
  const root = fixture();
  try {
    const runs = Array.from({ length: 64 }, () => Bun.spawn([
      process.execPath, path.join(repo, "bin/kineti-spend.ts"), "log", "--stage", "build",
      "--tokens-in", "1", "--tokens-out", "1", "--usd", "0.001001",
    ], { cwd: root, env: { ...process.env, KINETI_MACHINE_DIR: path.join(root, ".kineti", "machine") }, stdout: "ignore", stderr: "ignore" }));
    expect(await Promise.all(runs.map(r => r.exited))).toEqual(Array(64).fill(0));
    const state = JSON.parse(fs.readFileSync(path.join(root, ".kineti/spend.json"), "utf8"));
    expect(state.entries).toBe(64);
    expect(state.total_microcents).toBe(64064);
    expect(spend(root, ["check"]).exitCode).toBe(0);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("damaged or missing initialized spend state blocks work without changing history", () => {
  for (const damage of ["{broken", "null", JSON.stringify({ total_usd: -1 }), "missing", "truncated-log"]) {
    const root = fixture();
    try {
      expect(spend(root, ["log", "--stage", "build", "--tokens-in", "1", "--tokens-out", "1", "--usd", "0.01"]).exitCode).toBe(0);
      const state = path.join(root, ".kineti/spend.json"), log = path.join(root, ".kineti/spend.log.jsonl");
      if (damage === "missing") fs.unlinkSync(state);
      else if (damage === "truncated-log") fs.writeFileSync(log, "");
      else fs.writeFileSync(state, damage);
      const bytes = fs.readFileSync(log);
      expect(spend(root, ["check"]).exitCode).toBe(3);
      expect(spend(root, ["log", "--stage", "build", "--tokens-in", "1", "--tokens-out", "1", "--usd", "0"]).exitCode).toBe(3);
      expect(fs.readFileSync(log)).toEqual(bytes);
    } finally { fs.rmSync(root, { recursive: true, force: true }); }
  }
});

test("CI rejects fabricated unsigned evidence even with current fingerprint", () => {
  const root = fixture();
  try {
    fs.mkdirSync(path.join(root, ".kineti"));
    fs.writeFileSync(path.join(root, ".kineti/state.json"), JSON.stringify({ stage: 7, root_goal: "synthetic task" }));
    fs.writeFileSync(path.join(root, ".kineti/evidence.jsonl"), JSON.stringify({ at: new Date().toISOString(), label: "forged", cmd: "bun test", exit_code: 0, fingerprint: fingerprint(root) }) + "\n");
    expect(generateCIReport(root, { requiredEvidenceLabels: ["forged"] }).checksPassed).toBe(false);
    expect(generateCIReport(root).checksPassed).toBe(false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("CI validates authentic receipts and rejects altered, skipped, zero-case and unsupported results", () => {
  const root = fixture();
  try {
    fs.mkdirSync(path.join(root, ".kineti"));
    fs.writeFileSync(path.join(root, ".kineti/state.json"), JSON.stringify({ stage: 7, root_goal: "synthetic task" }));
    fs.writeFileSync(path.join(root, "control.test.ts"), 'import {test,expect} from "bun:test"; test("control",()=>expect(2+2).toBe(4));\n');
    const result = Bun.spawnSync([process.execPath, path.join(repo, "bin/kineti-evidence.ts"), "run", "--label", "control", "--required-test", "control.test.ts", "--", "bun", "test", "control.test.ts"], {
      cwd: root, env: { ...process.env, KINETI_MACHINE_DIR: path.join(root, ".kineti", "machine") },
    });
    expect(result.exitCode).toBe(0);
    const ledger = path.join(root, ".kineti/evidence.jsonl");
    const receipt = JSON.parse(fs.readFileSync(ledger, "utf8")) as EvidenceRecord;
    expect(generateCIReport(root, { requiredEvidenceLabels: ["control"] }).checksPassed).toBe(true);
    const privateKey = fs.readFileSync(path.join(root, ".kineti/evidence-issuer.pem.private"));
    for (const change of ["unsigned", "forged", "skipped", "zero", "unsupported", "unrun", "empty-inventory", "future-version"]) {
      const altered = structuredClone(receipt);
      if (change === "skipped") altered.verification!.skipped = 1;
      if (change === "zero") altered.verification!.passed = 0;
      if (change === "unsupported") altered.verification!.status = "unsupported";
      if (change === "unrun") altered.exit_code = null;
      if (change === "empty-inventory") altered.verification!.inventory = [];
      if (change === "future-version") (altered as any).receipt_version = 99;
      altered.signature = crypto.sign(null, Buffer.from(unsignedRecord(altered)), privateKey).toString("base64");
      if (change === "unsigned") delete altered.signature;
      if (change === "forged") altered.cmd = "forged";
      fs.writeFileSync(ledger, JSON.stringify(altered) + "\n");
      expect(generateCIReport(root, { requiredEvidenceLabels: ["control"] }).checksPassed).toBe(false);
    }
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("integrity readers reject malformed lines rather than skipping history", () => {
  const root = fixture();
  try {
    const file = path.join(root, "records.jsonl");
    fs.writeFileSync(file, '{"id":1}\n{broken\n{"id":2}\n');
    expect(() => readJsonl(file)).toThrow();
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("corrupt initialized goal files cannot be replaced by init", () => {
  const root = fixture();
  try {
    fs.mkdirSync(path.join(root, ".kineti"));
    const file = path.join(root, ".kineti/state.json");
    for (const bytes of ["{broken", "null", "[]", '{"version":1}']) {
      fs.writeFileSync(file, bytes);
      const result = Bun.spawnSync([process.execPath, path.join(repo, "bin/kineti-state.ts"), "init", "--project", "replacement", "--mode", "both"], { cwd: root });
      expect(result.exitCode).not.toBe(0);
      expect(fs.readFileSync(file, "utf8")).toBe(bytes);
    }
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("generated Companion script parses", async () => {
  const { startServer, AUTH_TOKEN } = await import("../bin/kineti-companion.ts");
  const server = startServer(0);
  try {
    const response = await server.fetch(new Request("http://localhost/", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }));
    expect(response.status).toBe(200);
    const scripts = [...(await response.text()).matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/gi)];
    expect(scripts.length).toBeGreaterThan(0);
    for (const [, script] of scripts) expect(() => new vm.Script(script)).not.toThrow();
  } finally { server.stop(true); }
});
