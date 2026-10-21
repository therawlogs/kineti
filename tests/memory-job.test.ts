import { describe, test, expect } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { IncidentJournalWriter, incidentReport, JournalWriter, localContextScope } from "../bin/kineti-memory-job.ts";
import crypto from "node:crypto";

const REPO = path.resolve(import.meta.dir, "..");

function run(args: string[], cwd: string = REPO) {
  const p = Bun.spawnSync({
    cmd: ["bun", path.join(REPO, "bin", "kineti-memory-job.ts"), ...args],
    cwd,
    env: { ...process.env, KINETI_WORKSPACE_ROOT: REPO },
    stdout: "pipe",
    stderr: "pipe",
  });
  return { status: p.exitCode ?? -1, out: p.stdout?.toString() ?? "", err: p.stderr?.toString() ?? "" };
}

function makeProject(): string {
  const scratch = path.join(REPO, ".kineti", "scratch");
  fs.mkdirSync(scratch, { recursive: true });
  const dir = fs.mkdtempSync(path.join(scratch, "mem-test-"));
  fs.mkdirSync(path.join(dir, ".kineti"), { recursive: true });
  fs.writeFileSync(path.join(dir,".kineti/state.json"),JSON.stringify({project:"memory-maintenance"}),{mode:0o600});
  return dir;
}

function write(dir: string, recs: any[]) {
  let previous = "GENESIS";
  for (const record of recs) {
    if (!record.hash) { record.prev_hash = previous; record.hash = crypto.createHash("sha256").update(`${record.prev_hash}${record.at}${record.id}${JSON.stringify(record.data)}`).digest("hex"); }
    previous = record.hash;
  }
  fs.writeFileSync(
    path.join(dir, ".kineti", "journal.jsonl"),
    recs.map((r) => JSON.stringify(r)).join("\n") + "\n",
  );
}

const day = 86400000;
const iso = (daysAgo: number) => new Date(Date.now() - daysAgo * day).toISOString();

describe("kineti-memory-job", () => {
  test("incident journal resolves scoped evidence and separates observations from explanations", () => {
    const dir = makeProject();
    const logPath = path.join(dir, "actor-a.log");
    fs.writeFileSync(logPath, "actor A observed timeout\n");
    const digest = crypto.createHash("sha256").update(fs.readFileSync(logPath)).digest("hex");
    const writer = new IncidentJournalWriter(dir, REPO);
    const input = {
      id: "ir-sample-001", scope: localContextScope(dir), title: "Two-agent timeout",
      events: [
        { at: "2026-10-16T12:02:00.000Z", actor: "agent-b", task_id: "task-b", observation: "Retry timed out", evidence: [] },
        { at: "2026-10-16T12:00:00.000Z", actor: "agent-a", task_id: "task-a", observation: "Request started", evidence: [{ source_id: "src-log-a", path: "actor-a.log", sha256: digest }] },
      ],
      reported_causes: [{ claim: "The service was overloaded", reported_by: "operator-1", evidence_ids: ["src-log-a"] }],
      hypotheses: [{ claim: "The retry may have duplicated work", evidence_ids: [] }],
      dependencies: ["task-a requires task-b"],
      missing_evidence: ["No service-side trace was available"], conflicting_explanations: [],
      at: "2026-10-16T12:03:00.000Z",
    };
    try {
      const record = writer.append(input);
      expect(record.type).toBe("incident");
      expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
      const report = incidentReport(dir, input.id, input.scope);
      expect(report.record.data.events.map((event: any) => event.actor)).toEqual(["agent-b", "agent-a"]);
      expect(report.record.data.dependencies).toEqual(["task-a requires task-b"]);
      expect(report.source_status[0].status).toBe("verified");
      expect(report.notice).toContain("do not by themselves prove causation");
      expect(() => incidentReport(dir, input.id, { user: "other-user", project: "project-1" })).toThrow("scope");
      expect(() => incidentReport(dir, input.id, { user: "user-1", project: "other-project" })).toThrow("scope");
      fs.unlinkSync(logPath);
      expect(incidentReport(dir, input.id, input.scope).source_status[0].status).toBe("missing");
    } finally { fs.rmSync(dir, { recursive: true, force: true }); }
  });

  test("incident writer rejects fake sources and permits append-only same-scope corrections", () => {
    const dir = makeProject();
    const writer = new IncidentJournalWriter(dir, REPO);
    const sourcePath = path.join(dir, "evidence.txt");
    fs.writeFileSync(sourcePath, "observed value\n");
    const digest = crypto.createHash("sha256").update(fs.readFileSync(sourcePath)).digest("hex");
    const base = {
      scope: localContextScope(dir), title: "Incident", events: [
        { at: "2026-10-16T12:00:00.000Z", actor: "person-a", task_id: "task-1", observation: "An error appeared", evidence: [{ source_id: "src-error", path: "evidence.txt", sha256: digest }] },
      ], reported_causes: [], hypotheses: [], missing_evidence: [], conflicting_explanations: [],
    };
    try {
      expect(() => writer.append({ ...base, id: "ir-fake-001", events: [{ ...base.events[0], evidence: [{ source_id: "src-fake", path: "missing.log", sha256: digest }] }] })).toThrow("evidence must be inspectable");
      const first = writer.append({ ...base, id: "ir-correct-001" });
      const correction = writer.append({ ...base, id: "ir-correct-002", title: "Corrected incident", correction_of: first.id });
      expect(correction.data.correction_of).toBe(first.id);
      const restartedShow = run(["incident", "show", "--dir", dir, "--id", correction.id, "--user", base.scope.user, "--project", base.scope.project]);
      expect(restartedShow.status).toBe(0);
      expect(JSON.parse(restartedShow.out).record.data.title).toBe("Corrected incident");
      expect(() => writer.append({ ...base, id: "ir-correct-003", correction_of: "ir-does-not-exist" })).toThrow("existing incident");
      expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
    } finally { fs.rmSync(dir, { recursive: true, force: true }); }
  });

  test("incident command requires explicit user and project scope", () => {
    const dir = makeProject();
    try {
      const show = run(["incident", "show", "--dir", dir, "--id", "ir-missing-001", "--user", "user-a"], REPO);
      expect(show.status).toBe(2);
      expect(show.err).toContain("--id, --user, and --project scope");
    } finally { fs.rmSync(dir, { recursive: true, force: true }); }
  });

  test("JournalWriter appends scoped run records to the verified hash chain", () => {
    const dir = makeProject();
    const writer = new JournalWriter(dir, REPO);
    const base = {
      project: "kineti",
      data: {
        root_goal: "Build universal agent harness with cryptographic verification",
        stage_outcomes: ["unit checks passed"],
        costs_usd: null,
        failed_approaches: [],
      },
    };
    try {
      const first = writer.appendRunRecord({ ...base, id: "rr-writer-test-001", at: "2026-10-16T12:00:00.000Z" });
      const second = writer.appendRunRecord({ ...base, id: "rr-writer-test-002", at: "2026-10-16T12:01:00.000Z" });
      expect(first.prev_hash).toBe("GENESIS");
      expect(second.prev_hash).toBe(first.hash);
      expect(fs.statSync(path.join(dir, ".kineti", "journal.jsonl")).mode & 0o777).toBe(0o600);
      expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  test("JournalWriter refuses duplicate ids, locked writers, and damaged history without changing bytes", () => {
    const dir = makeProject();
    const writer = new JournalWriter(dir, REPO);
    const input = {
      project: "kineti",
      id: "rr-writer-test-003",
      data: { root_goal: "test goal", stage_outcomes: ["recorded"], costs_usd: 0, failed_approaches: [] },
      at: "2026-10-16T12:00:00.000Z",
    };
    try {
      writer.appendRunRecord(input);
      const journal = path.join(dir, ".kineti", "journal.jsonl");
      const intact = fs.readFileSync(journal, "utf8");
      expect(() => writer.appendRunRecord(input)).toThrow("id already exists");
      expect(fs.readFileSync(journal, "utf8")).toBe(intact);

      const lock = path.join(dir, ".kineti", "journal.lock");
      fs.writeFileSync(lock, "held", { mode: 0o600 });
      expect(() => writer.appendRunRecord({ ...input, id: "rr-writer-test-004" })).toThrow("locked");
      fs.rmSync(lock);

      fs.writeFileSync(journal, intact.replace("test goal", "changed goal"));
      const damaged = fs.readFileSync(journal, "utf8");
      expect(() => writer.appendRunRecord({ ...input, id: "rr-writer-test-005" })).toThrow("invalid journal");
      expect(fs.readFileSync(journal, "utf8")).toBe(damaged);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });

  test("sweep promotes expired actives; chain verifies then detects tamper (exit 3)", () => {
    const dir = makeProject();
    const r1: any = {
      at: iso(30), type: "run-record", state: "active", project: "p", id: "rr-001",
      data: { root_goal: "g" }, links: [], prev_hash: "GENESIS", hash: "",
    };
    const crypto = require("node:crypto");
    r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}${r1.at}${r1.id}${JSON.stringify(r1.data)}`).digest("hex");
    const r2: any = {
      at: iso(1), type: "run-record", state: "active", project: "p", id: "rr-002",
      data: { root_goal: "g2" }, links: [], prev_hash: r1.hash, hash: "",
    };
    r2.hash = crypto.createHash("sha256").update(`${r2.prev_hash}${r2.at}${r2.id}${JSON.stringify(r2.data)}`).digest("hex");
    const oldLearning: any = {
      at: iso(120), type: "learning", state: "active", project: "p", id: "lr-001",
      data: { lesson: "old", skill: "qa", trigger: "always" }, links: [],
      expires: iso(10),
      prev_hash: r2.hash,
      hash: "",
    };
    oldLearning.hash = crypto.createHash("sha256").update(`${oldLearning.prev_hash}${oldLearning.at}${oldLearning.id}${JSON.stringify(oldLearning.data)}`).digest("hex");
    write(dir, [r1, r2, oldLearning]);

    expect(run(["verify-chain", "--dir", dir]).status).toBe(0);
    expect(run(["sweep", "--dir", dir]).out).toContain("1 record(s) moved");

    const after = fs.readFileSync(path.join(dir, ".kineti", "journal.jsonl"), "utf8")
      .trim().split("\n").map((l) => JSON.parse(l));
    expect(after.find((r) => r.id === "lr-001").state).toBe("warm");
    expect(after.find((r) => r.id === "rr-002").state).toBe("active");

    // tamper with committed history
    after.find((r) => r.id === "rr-001").data.root_goal = "rewritten";
    write(dir, after);
    expect(run(["verify-chain", "--dir", dir]).status).toBe(3);

    fs.rmSync(dir, { recursive: true, force: true });
  });

  test("time-order flags effect-before-cause; promote surfaces frequent new words", () => {
    const dir = makeProject();
    write(dir, [
      { at: iso(5), type: "run-record", state: "active", project: "p", id: "late", data: {}, links: [] },
      { at: iso(9), type: "run-record", state: "active", project: "p", id: "early", data: {}, links: [] },
      {
        at: iso(1), type: "run-record", state: "active", project: "p", id: "with-links", data: {},
        links: [
          { word: "caused", from_id: "late", from_at: iso(5), to_id: "early", to_at: iso(9), status: "candidate" },
          { word: "degraded", from_id: "late", from_at: iso(5), to_id: "early", to_at: iso(9), status: "candidate" },
          { word: "degraded", from_id: "late", from_at: iso(5), to_id: "early", to_at: iso(9), status: "candidate" },
          { word: "degraded", from_id: "late", from_at: iso(5), to_id: "early", to_at: iso(9), status: "candidate" },
        ],
      },
    ]);
    expect(run(["time-order", "--dir", dir]).status).toBe(1);
    const promo = run(["promote", "--dir", dir]);
    expect(promo.status).toBe(0);
    expect(promo.out).toContain('"degraded"');
    fs.rmSync(dir, { recursive: true, force: true });
  });

  test("refuses --dir pointing outside project root (exit 2)", () => {
    const res = run(["sweep", "--dir", "/tmp/forbidden-outside-dir"]);
    expect(res.status).toBe(2);
    expect(res.err).toContain("Path traversal rejected");
  });
});
