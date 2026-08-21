import { describe, test, expect } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const REPO = path.resolve(import.meta.dir, "..");

function run(args: string[], cwd: string = REPO) {
  const p = Bun.spawnSync({
    cmd: ["bun", path.join(REPO, "bin", "kineti-memory-job.ts"), ...args],
    cwd,
    stdout: "pipe",
    stderr: "pipe",
  });
  return { status: p.exitCode ?? -1, out: p.stdout?.toString() ?? "", err: p.stderr?.toString() ?? "" };
}

function makeProject(): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-mem-"));
  fs.mkdirSync(path.join(dir, ".kineti"), { recursive: true });
  return dir;
}

function write(dir: string, recs: any[]) {
  fs.writeFileSync(
    path.join(dir, ".kineti", "journal.jsonl"),
    recs.map((r) => JSON.stringify(r)).join("\n") + "\n",
  );
}

const day = 86400000;
const iso = (daysAgo: number) => new Date(Date.now() - daysAgo * day).toISOString();

describe("kineti-memory-job", () => {
  test("sweep promotes expired actives; chain verifies then detects tamper (exit 3)", () => {
    const dir = makeProject();
    const r1: any = {
      at: iso(30), type: "run-record", state: "active", project: "p", id: "rr-001",
      data: { root_goal: "g" }, links: [], prev_hash: "GENESIS", hash: "",
    };
    // hash computed the same way the program does
    const crypto = require("node:crypto");
    r1.hash = crypto.createHash("sha256").update(`${r1.prev_hash}|${r1.at}|${r1.id}|${JSON.stringify(r1.data)}`).digest("hex");
    const r2: any = {
      at: iso(1), type: "run-record", state: "active", project: "p", id: "rr-002",
      data: { root_goal: "g2" }, links: [], prev_hash: r1.hash, hash: "",
    };
    r2.hash = crypto.createHash("sha256").update(`${r2.prev_hash}|${r2.at}|${r2.id}|${JSON.stringify(r2.data)}`).digest("hex");
    const oldLearning = {
      at: iso(120), type: "learning", state: "active", project: "p", id: "lr-001",
      data: { skill: "qa", trigger: "always", lesson: "old" }, links: [],
      expires: iso(10),
    };
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
});
