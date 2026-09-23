import { describe, test, expect, beforeEach, afterEach } from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const REPO = process.cwd();
const SYNC = path.join(REPO, "bin", "kineti-sync.ts");
const SWITCH_FILE = path.join(REPO, ".kineti", "kineti.json");
const JOURNAL_FILE = path.join(REPO, ".kineti", "journal.jsonl");
const STATE_FILE = path.join(REPO, ".kineti", "state.json");
let backupSwitch: string | null = null;
let backupJournal: string | null = null;
let backupState: string | null = null;
let tmpDir = "";

function run(args: string[], env: Record<string, string> = {}) {
  return spawnSync("bun", [SYNC, ...args], {
    cwd: REPO,
    env: { ...process.env, ...env },
    encoding: "utf8",
  } as any);
}

beforeEach(() => {
  try {
    backupSwitch = fs.existsSync(SWITCH_FILE) ? fs.readFileSync(SWITCH_FILE, "utf8") : null;
  } catch { backupSwitch = null; }
  try {
    backupJournal = fs.existsSync(JOURNAL_FILE) ? fs.readFileSync(JOURNAL_FILE, "utf8") : null;
  } catch { backupJournal = null; }
  try {
    backupState = fs.existsSync(STATE_FILE) ? fs.readFileSync(STATE_FILE, "utf8") : null;
  } catch { backupState = null; }
  // CI checkouts have no .kineti/: seed a minimal locked goal for the roundtrip test.
  if (backupState === null) {
    fs.mkdirSync(path.dirname(STATE_FILE), { recursive: true });
    fs.writeFileSync(STATE_FILE, JSON.stringify({
      version: 1,
      project: "kineti-test",
      root_goal: "Test goal for sync",
      root_goal_locked_at: new Date().toISOString(),
      stage: 7,
      gates: { spec: "pass", ship: "pass", security: "pass" },
    }));
  }
  tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "kineti-sync-"));
});

afterEach(() => {
  try {
    if (backupSwitch === null) fs.rmSync(SWITCH_FILE, { force: true });
    else fs.writeFileSync(SWITCH_FILE, backupSwitch);
    if (backupJournal === null) fs.rmSync(JOURNAL_FILE, { force: true });
    else fs.writeFileSync(JOURNAL_FILE, backupJournal);
    if (backupState === null) fs.rmSync(STATE_FILE, { force: true });
    else fs.writeFileSync(STATE_FILE, backupState);
    fs.rmSync(tmpDir, { recursive: true, force: true });
  } catch { /* ignore */ }
});

describe("kineti-sync encrypted device sync", () => {
  test("off by default blocks export", () => {
    const out = path.join(tmpDir, "s.json");
    const r = run(["export", "--out", out, "--passphrase-env", "T_PW"], { T_PW: "long-passphrase-123" });
    expect(r.status).not.toBe(0);
    expect((r.stderr as string) + (r.stdout as string)).toContain("sync is off");
  });

  test("export then import roundtrips notes without touching root goal", () => {
    expect(run(["on"]).status).toBe(0);
    const goalBefore = JSON.parse(fs.readFileSync(path.join(REPO, ".kineti", "state.json"), "utf8")).root_goal;
    const out = path.join(tmpDir, "s.json");
    const exp = run(["export", "--out", out, "--passphrase-env", "T_PW"], { T_PW: "long-passphrase-123" });
    expect(exp.status).toBe(0);
    const payload = JSON.parse(fs.readFileSync(out, "utf8"));
    expect(payload.algo).toBe("aes-256-gcm/scrypt");
    expect(fs.statSync(out).mode & 0o777 & 0o077).toBe(0);
    // Wrong passphrase fails.
    const bad = run(["import", "--file", out, "--passphrase-env", "T_BAD"], { T_BAD: "wrong-passphrase-000" });
    expect(bad.status).not.toBe(0);
    // Right passphrase imports.
    const imp = run(["import", "--file", out, "--passphrase-env", "T_PW"], { T_PW: "long-passphrase-123" });
    expect(imp.status).toBe(0);
    expect(imp.stdout as string).toContain("Root goal untouched");
    const goalAfter = JSON.parse(fs.readFileSync(path.join(REPO, ".kineti", "state.json"), "utf8")).root_goal;
    expect(goalAfter).toBe(goalBefore);
  });

  test("short passphrase refused", () => {
    expect(run(["on"]).status).toBe(0);
    const out = path.join(tmpDir, "s.json");
    const r = run(["export", "--out", out, "--passphrase-env", "T_PW"], { T_PW: "short" });
    expect(r.status).not.toBe(0);
  });
});
