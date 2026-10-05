import { describe, test, expect, beforeEach, afterEach } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import {
  livePairing, makeCode, makePairing, markUsed, minutesLeft, revokePairing, PAIR_TTL_MS,
  cloudStatus, dropLink, pollPairing, getMirror, setMirror,
} from "../bin/kineti-pairing.ts";
import { loadLimits } from "../bin/lib.ts";

const PAIR_FILE = path.join(process.cwd(), ".kineti", "pairing.json");
const MIRROR_FILE = path.join(process.cwd(), ".kineti", "mirror.json");
let backup: string | null = null;
let mirrorBackup: string | null = null;
let machineBackup: string | undefined;
let tmpMachine = "";

beforeEach(() => {
  try {
    backup = fs.existsSync(PAIR_FILE) ? fs.readFileSync(PAIR_FILE, "utf8") : null;
  } catch { backup = null; }
  try {
    mirrorBackup = fs.existsSync(MIRROR_FILE) ? fs.readFileSync(MIRROR_FILE, "utf8") : null;
  } catch { mirrorBackup = null; }
  machineBackup = process.env.KINETI_MACHINE_DIR;
  const machineScratch = path.join(process.cwd(), ".kineti", "test-pairing-machine");
  fs.mkdirSync(machineScratch, { recursive: true });
  tmpMachine = fs.mkdtempSync(path.join(machineScratch, "run-"));
  process.env.KINETI_MACHINE_DIR = tmpMachine;
  try { fs.rmSync(PAIR_FILE, { force: true }); } catch {}
});

afterEach(() => {
  try {
    if (backup === null) fs.rmSync(PAIR_FILE, { force: true });
    else fs.writeFileSync(PAIR_FILE, backup);
    if (mirrorBackup === null) fs.rmSync(MIRROR_FILE, { force: true });
    else fs.writeFileSync(MIRROR_FILE, mirrorBackup, { mode: 0o600 });
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

  test("cloud state never reports a local pairing code as a live link", () => {
    makePairing("pair-test");
    expect(livePairing()).not.toBeNull();
    expect(cloudStatus()).toEqual({ linked: false, available: false });
    expect(fs.existsSync(path.join(tmpMachine, "cloud.json"))).toBe(false);
  });

  test("drop removes only local pairing data and makes no cloud claim", () => {
    makePairing("pair-test");
    const d = dropLink("pair-test");
    expect(d.hadPairing).toBe(true);
    expect(d.hadCloud).toBe(false);
    expect(cloudStatus()).toEqual({ linked: false, available: false });
    expect(fs.existsSync(PAIR_FILE)).toBe(false);
    const again = dropLink("pair-test");
    expect(again.hadPairing).toBe(false);
    expect(again.hadCloud).toBe(false);
  });

  test("poll ends used, expired, or timeout", async () => {
    expect(await pollPairing(1000, 250)).toEqual({ outcome: "expired" });
    makePairing("pair-test");
    expect(await pollPairing(1000, 250)).toEqual({ outcome: "timeout" });
    markUsed("cloud");
    expect(await pollPairing(5000, 250)).toEqual({ outcome: "used" });
  });

  test("mirror is off with notes excluded by default", () => {
    try { fs.rmSync(MIRROR_FILE, { force: true }); } catch {}
    const m = getMirror();
    expect(m.enabled).toBe(false);
    expect(m.note_sync).toBe(false);
    expect(m.ceiling).toBe(50);
    const on = setMirror(true, false, "pair-test");
    expect(on.enabled).toBe(true);
    expect(on.note_sync).toBe(false);
    expect(on.ceiling).toBe(50);
    const notes = setMirror(true, true, "pair-test");
    expect(notes.note_sync).toBe(true);
    const capped = setMirror(true, false, "pair-test", 40);
    expect(capped.ceiling).toBe(40);
    const bad = setMirror(true, false, "pair-test", 5000);
    expect(bad.ceiling).toBe(40);
    setMirror(false, false, "pair-test");
    try { fs.rmSync(MIRROR_FILE, { force: true }); } catch {}
  });

  test("mirror ceiling drives the spend breaker, invalid falls back to 50", () => {
    const scratch = path.join(process.cwd(), ".kineti", "test-pairing-limits");
    fs.mkdirSync(scratch, { recursive: true });
    const dir = fs.mkdtempSync(path.join(scratch, "run-"));
    try {
      fs.mkdirSync(path.join(dir, ".kineti"), { recursive: true });
      expect(loadLimits(dir).globalUsd).toBe(50);
      fs.writeFileSync(path.join(dir, ".kineti", "mirror.json"), JSON.stringify({ ceiling: 40 }));
      expect(loadLimits(dir).globalUsd).toBe(40);
      fs.writeFileSync(path.join(dir, ".kineti", "mirror.json"), JSON.stringify({ ceiling: 5000 }));
      expect(loadLimits(dir).globalUsd).toBe(50);
      fs.writeFileSync(path.join(dir, ".kineti", "mirror.json"), JSON.stringify({ ceiling: "lots" }));
      expect(loadLimits(dir).globalUsd).toBe(50);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  });
});
