import { describe, test, expect, beforeEach, afterEach } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import {
  livePairing, makeCode, makePairing, markUsed, minutesLeft, revokePairing, PAIR_TTL_MS,
} from "../bin/kineti-pairing.ts";

const PAIR_FILE = path.join(process.cwd(), ".kineti", "pairing.json");
let backup: string | null = null;
let machineBackup: string | undefined;
let tmpMachine = "";

beforeEach(() => {
  try {
    backup = fs.existsSync(PAIR_FILE) ? fs.readFileSync(PAIR_FILE, "utf8") : null;
  } catch { backup = null; }
  machineBackup = process.env.KINETI_MACHINE_DIR;
  tmpMachine = fs.mkdtempSync(path.join(fs.realpathSync("/tmp"), "kineti-pair-"));
  process.env.KINETI_MACHINE_DIR = tmpMachine;
  try { fs.rmSync(PAIR_FILE, { force: true }); } catch {}
});

afterEach(() => {
  try {
    if (backup === null) fs.rmSync(PAIR_FILE, { force: true });
    else fs.writeFileSync(PAIR_FILE, backup);
    if (machineBackup === undefined) delete process.env.KINETI_MACHINE_DIR;
    else process.env.KINETI_MACHINE_DIR = machineBackup;
    fs.rmSync(tmpMachine, { recursive: true, force: true });
  } catch { /* ignore */ }
});

describe("kineti-pairing cloud link codes", () => {
  test("codes look like KIN-XXXX-XXXX with plain alphabet", () => {
    for (let i = 0; i < 20; i++) {
      expect(makeCode()).toMatch(/^KIN-[A-Z2-9]{4}-[A-Z2-9]{4}$/);
    }
  });

  test("make then status shows a live code with minutes left", () => {
    const p = makePairing("pair-test");
    expect(p.code).toMatch(/^KIN-/);
    expect(p.used).toBe(false);
    const live = livePairing();
    expect(live?.code).toBe(p.code);
    expect(minutesLeft(p)).toBeGreaterThan(0);
    expect(minutesLeft(p)).toBeLessThanOrEqual(10);
  });

  test("expired codes are not live", () => {
    makePairing("pair-test");
    const raw = JSON.parse(fs.readFileSync(PAIR_FILE, "utf8"));
    raw.expires_at = new Date(Date.now() - PAIR_TTL_MS).toISOString();
    fs.writeFileSync(PAIR_FILE, JSON.stringify(raw));
    expect(livePairing()).toBeNull();
  });

  test("used codes are single use only", () => {
    makePairing("pair-test");
    expect(markUsed("cloud")).toBe(true);
    expect(livePairing()).toBeNull();
    expect(markUsed("cloud")).toBe(false);
  });

  test("revoke drops the link, safe when none exists", () => {
    expect(revokePairing("pair-test")).toBe(false);
    makePairing("pair-test");
    expect(revokePairing("pair-test")).toBe(true);
    expect(livePairing()).toBeNull();
    expect(fs.existsSync(PAIR_FILE)).toBe(false);
  });
});
