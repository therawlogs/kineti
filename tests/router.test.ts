import { describe, test, expect, beforeEach, afterEach } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { classifyIntent, route, setEnabled, isEnabled } from "../bin/kineti-router.ts";

const SWITCH_FILE = path.join(process.cwd(), ".kineti", "kineti.json");
const SWARM_FILE = path.join(process.cwd(), ".kineti", "swarm.json");
const PAIR_FILE = path.join(process.cwd(), ".kineti", "pairing.json");
const STATE_FILE = path.join(process.cwd(), ".kineti", "state.json");
let backup: string | null = null;
let swarmBackup: string | null = null;
let pairBackup: string | null = null;
let stateBackup: string | null = null;
let machineBackup: string | undefined;
let tmpMachine: string = "";

beforeEach(() => {
  try {
    if (fs.existsSync(SWITCH_FILE)) backup = fs.readFileSync(SWITCH_FILE, "utf8");
    else backup = null;
  } catch { backup = null; }
  try {
    if (fs.existsSync(SWARM_FILE)) swarmBackup = fs.readFileSync(SWARM_FILE, "utf8");
    else swarmBackup = null;
  } catch { swarmBackup = null; }
  try {
    if (fs.existsSync(PAIR_FILE)) pairBackup = fs.readFileSync(PAIR_FILE, "utf8");
    else pairBackup = null;
  } catch { pairBackup = null; }
  try {
    if (fs.existsSync(STATE_FILE)) stateBackup = fs.readFileSync(STATE_FILE, "utf8");
    else stateBackup = null;
  } catch { stateBackup = null; }
  // Router replies branch on a locked goal. CI checkouts have no .kineti/,
  // so seed a minimal goal here and restore afterwards.
  if (stateBackup === null) {
    fs.mkdirSync(path.dirname(STATE_FILE), { recursive: true });
    fs.writeFileSync(STATE_FILE, JSON.stringify({
      version: 1,
      project: "kineti-test",
      root_goal: "Test goal for router",
      root_goal_locked_at: new Date().toISOString(),
      stage: 7,
      gates: { spec: "pass", ship: "pass", security: "pass" },
    }));
  }
  // Keep test audit entries out of the real ~/.kineti log.
  machineBackup = process.env.KINETI_MACHINE_DIR;
  const machineScratch = path.join(process.cwd(), ".kineti", "test-router-machine");
  fs.mkdirSync(machineScratch, { recursive: true });
  tmpMachine = fs.mkdtempSync(path.join(machineScratch, "run-"));
  process.env.KINETI_MACHINE_DIR = tmpMachine;
  setEnabled(true, "router-test");
});

afterEach(() => {
  try {
    if (backup === null) fs.rmSync(SWITCH_FILE, { force: true });
    else fs.writeFileSync(SWITCH_FILE, backup);
    if (swarmBackup === null) fs.rmSync(SWARM_FILE, { force: true });
    else fs.writeFileSync(SWARM_FILE, swarmBackup);
    if (pairBackup === null) fs.rmSync(PAIR_FILE, { force: true });
    else fs.writeFileSync(PAIR_FILE, pairBackup);
    if (stateBackup === null) fs.rmSync(STATE_FILE, { force: true });
    else fs.writeFileSync(STATE_FILE, stateBackup);
    if (machineBackup === undefined) delete process.env.KINETI_MACHINE_DIR;
    else process.env.KINETI_MACHINE_DIR = machineBackup;
    fs.rmSync(tmpMachine, { recursive: true, force: true });
  } catch { /* ignore */ }
});

describe("kineti-router intent classification", () => {
  test("spend words route to spend", () => {
    expect(classifyIntent("how much have I spent?")).toBe("spend");
    expect(classifyIntent("what is my budget left")).toBe("spend");
  });
  test("undo words route to undo", () => {
    expect(classifyIntent("undo that")).toBe("undo");
    expect(classifyIntent("roll back please")).toBe("undo");
  });
  test("proof words route to proof", () => {
    expect(classifyIntent("did tests pass?")).toBe("proof");
  });
  test("natural yes routes to approve_yes", () => {
    for (const w of ["yes", "ok", "go ahead", "do it", "looks good"]) {
      expect(classifyIntent(w)).toBe("approve_yes");
    }
  });
  test("natural fix routes to approve_fix", () => {
    expect(classifyIntent("no, change the color")).toBe("approve_fix");
    expect(classifyIntent("fix the title")).toBe("approve_fix");
  });
  test("on and off switch intents", () => {
    expect(classifyIntent("Kineti off")).toBe("kineti_off");
    expect(classifyIntent("Kineti on")).toBe("kineti_on");
  });
  test("swarm budget ask routes correctly", () => {
    expect(classifyIntent("we have a team, separate budgets?")).toBe("swarm_budget");
  });
  test("forget words route to a clear unsupported response without claiming deletion", () => {
    expect(classifyIntent("forget my diet")).toBe("forget");
    const r = route("forget my diet");
    expect(r.reply).toContain("cannot delete or verify deletion");
    expect(r.reply).toContain("Nothing was changed");
    expect(r.reply).not.toContain("proof receipt");
  });
  test("risky tasks warn and ask first", () => {
    const r = route("please pay the invoice now");
    expect(r.reply).toContain("hard to undo");
    expect(r.reply).toContain("does not execute or block the action");
  });
  test("model words route to model with ask-first suggestion", () => {
    expect(classifyIntent("which model is best for this bug?")).toBe("model");
    const r = route("which model is best for this bug?");
    expect(r.intent).toBe("model");
    expect(r.reply).toContain("Static suggestion");
    expect(r.reply).toContain("does not switch your active tool or model");
    expect(r.reply).toContain("Choices:");
  });
  test("auto-switch requests are only static suggestions", () => {
    const on = route("turn auto switch on");
    expect(on.reply).toContain("does not switch your active tool or model");
    const off = route("turn auto switch off");
    expect(off.reply).toContain("does not switch your active tool or model");
  });
  test("sync words explain that transfer is manual", () => {
    expect(classifyIntent("sync my devices")).toBe("sync");
    const r = route("sync my devices");
    expect(r.reply).toContain("manual encrypted export and import only");
    expect(r.reply).toContain("Choices:");
  });
  test("dashboard words do not claim that cloud pairing is available", () => {
    expect(classifyIntent("kineti-dashboard")).toBe("dashboard");
    const ask = route("kineti-dashboard");
    expect(ask.intent).toBe("dashboard");
    expect(ask.reply).toContain("Cloud dashboard linking is not available");
    expect(ask.reply).toContain("Choices:");
    const made = route("yes, make a cloud link");
    expect(made.intent).toBe("dashboard");
    expect(made.reply).toContain("no pairing code was made");
    expect(made.reply).not.toContain("KIN-");
    const dropped = route("revoke cloud link");
    expect(dropped.reply).toContain("No cloud service is connected");
  });
  test("swarm budgets save from plain words with audit-safe store", () => {
    const r = route("separate budgets: coder 15, reviewer 10");
    expect(r.intent).toBe("swarm_save");
    expect(r.reply).toContain("Saved separate budgets");
    const stored = JSON.parse(fs.readFileSync(SWARM_FILE, "utf8"));
    expect(stored.mode).toBe("separate");
    expect(stored.budgets.coder).toBe(15);
    expect(stored.budgets.reviewer).toBe(10);
    const shared = route("share one budget for the team");
    expect(shared.reply).toContain("one shared budget");
  });
});

describe("kineti-router replies stay plain", () => {
  test("spend reply has numbers and choices, no skill names", () => {
    const r = route("how much have I spent?");
    expect(r.intent).toBe("spend");
    expect(r.reply).toContain("Choices:");
    expect(r.reply).not.toContain("kineti-");
    expect(r.reply).not.toContain("bin/");
  });
  test("off pauses this helper without claiming to stop the agent", () => {
    const off = route("Kineti off");
    expect(off.reply).toContain("plain-talk helper is off");
    expect(isEnabled()).toBe(false);
    const blocked = route("how much have I spent?");
    expect(blocked.reply).toContain("This does not stop your coding agent");
    const on = route("Kineti on");
    expect(on.reply).toContain("plain-talk helper is on");
    expect(isEnabled()).toBe(true);
  });
  test("every reply offers numbered choices", () => {
    for (const msg of ["where are we?", "did tests pass?", "undo that", "yes", "my idea: fix login", "which model fits?", "forget my diet", "kineti-dashboard"]) {
      const r = route(msg);
      expect(r.reply).toContain("1.");
    }
  });
});
