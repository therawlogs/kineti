import { describe, test, expect, beforeEach, afterEach } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { checkInput, proveRepeat, estimateTokens, signatureOf } from "../bin/kineti-schema.ts";

const SCHEMA_FILE = path.join(process.cwd(), ".kineti", "schema.json");
let backup: string | null = null;
let machineBackup: string | undefined;
let tmpMachine = "";

beforeEach(() => {
  try {
    backup = fs.existsSync(SCHEMA_FILE) ? fs.readFileSync(SCHEMA_FILE, "utf8") : null;
  } catch { backup = null; }
  machineBackup = process.env.KINETI_MACHINE_DIR;
  tmpMachine = fs.mkdtempSync(path.join(fs.realpathSync("/tmp"), "kineti-schema-"));
  process.env.KINETI_MACHINE_DIR = tmpMachine;
  try { fs.rmSync(SCHEMA_FILE, { force: true }); } catch {}
});

afterEach(() => {
  try {
    if (backup === null) fs.rmSync(SCHEMA_FILE, { force: true });
    else fs.writeFileSync(SCHEMA_FILE, backup);
    if (machineBackup === undefined) delete process.env.KINETI_MACHINE_DIR;
    else process.env.KINETI_MACHINE_DIR = machineBackup;
    fs.rmSync(tmpMachine, { recursive: true, force: true });
  } catch { /* ignore */ }
});

describe("kineti-schema two-pass standardizer", () => {
  test("checker fixes fields and drops waste", () => {
    const noisy = "  fix   the   login   bug\n\nin `src/auth/login.ts`   \n\nok thanks bye bye";
    const c = checkInput({ text: noisy }, "schema-test");
    expect(c.kind).toBe("fix");
    expect(c.task.length).toBeLessThanOrEqual(2000);
    expect(c.task).not.toContain("  ");
    expect(c.entities).toContain("src/auth/login.ts");
    expect(c.droppedChars).toBeGreaterThan(0);
    expect(c.tokensAfter).toBeLessThanOrEqual(c.tokensBefore);
    expect(c.memoryHit).toBe(false);
  });

  test("image, audio, and tool refs become slots, not content", () => {
    const c = checkInput(
      { text: "what is in this photo", imageRef: "IMG_01.jpg", audioRef: "note.m4a", toolOutput: "x".repeat(5000) },
      "schema-test",
    );
    expect(c.images).toEqual(["IMG_01.jpg"]);
    expect(c.audio).toEqual(["note.m4a"]);
    expect(c.task.length).toBeLessThanOrEqual(2000);
    expect(c.droppedChars).toBeGreaterThan(3000);
  });

  test("repeat task hits memory and costs less", () => {
    const raw = { text: "plan the v5 ship release checklist" };
    const { first, second, saved } = proveRepeat(raw, "schema-test");
    expect(first.memoryHit).toBe(false);
    expect(second.memoryHit).toBe(true);
    expect(second.task).toBe(first.task);
    expect(second.tokensAfter).toBeLessThanOrEqual(second.tokensBefore);
    expect(saved).toBeGreaterThanOrEqual(0);
  });

  test("signatures are stable and distinct", () => {
    const a = signatureOf({ text: "  Fix THE login bug " });
    const b = signatureOf({ text: "fix the login bug" });
    const c = signatureOf({ text: "fix the login bug", imageRef: "a.png" });
    expect(a).toBe(b);
    expect(a).not.toBe(c);
    expect(estimateTokens("abcd")).toBe(1);
  });

  test("memory file caps at 100 entries", () => {
    for (let i = 0; i < 105; i++) checkInput({ text: `unique task number ${i} fix bug` }, "schema-test");
    const disk = JSON.parse(fs.readFileSync(SCHEMA_FILE, "utf8"));
    expect(disk.entries.length).toBe(100);
  });
});
