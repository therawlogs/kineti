import { afterEach, describe, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { seedCIState } from "../bin/kineti-seed.ts";

describe("public CI seed", () => {
  let root: string | undefined;
  const scratch = path.join(process.cwd(), ".kineti", "test-ci-seed");

  function makeProject(config: unknown = {
    name: "seed-project",
    root_goal: "Test the project safely",
    settings: { ci_stage: 9, spend_limit_usd: { global: 40 } },
  }): string {
    fs.mkdirSync(scratch, { recursive: true });
    root = fs.mkdtempSync(path.join(scratch, "seed-"));
    fs.writeFileSync(path.join(root, "kineti.config.json"), JSON.stringify(config));
    fs.writeFileSync(path.join(root, "package.json"), JSON.stringify({ name: "seed-package" }));
    return root;
  }

  afterEach(() => {
    if (root) fs.rmSync(root, { recursive: true, force: true });
    root = undefined;
  });

  test("creates missing state and spend from project configuration", () => {
    const dir = makeProject();
    const result = seedCIState(dir);
    const state = JSON.parse(fs.readFileSync(path.join(dir, ".kineti/state.json"), "utf8"));
    const spend = JSON.parse(fs.readFileSync(path.join(dir, ".kineti/spend.json"), "utf8"));

    expect(result).toMatchObject({ project: "seed-package", stage: 9, stateCreated: true, spendCreated: true, spendLimitUsd: 40 });
    expect(state.root_goal).toBe("Test the project safely");
    expect(state.root_goal_locked_at).toBeString();
    expect(spend.limit_microcents).toBe(40_000_000);
    if (process.platform !== "win32") {
      expect(fs.statSync(path.join(dir, ".kineti/state.json")).mode & 0o777).toBe(0o600);
    }
  });

  test("uses a valid stage override", () => {
    const dir = makeProject();
    expect(seedCIState(dir, 10).stage).toBe(10);
  });

  test("rejects invalid stage and missing root goal without creating state", () => {
    const dir = makeProject({ name: "x", root_goal: "", settings: {} });
    expect(() => seedCIState(dir, 14)).toThrow("--stage");
    expect(() => seedCIState(dir)).toThrow("root_goal");
    expect(fs.existsSync(path.join(dir, ".kineti"))).toBe(false);
  });

  test("does not overwrite existing state or spend", () => {
    const dir = makeProject();
    const kdir = path.join(dir, ".kineti");
    fs.mkdirSync(kdir);
    const state = { version: 1, project: "kept", root_goal: "Existing goal", stage: 4, gates: {}, history: [] };
    const spend = { total_microcents: 123, tripped: false };
    fs.writeFileSync(path.join(kdir, "state.json"), JSON.stringify(state));
    fs.writeFileSync(path.join(kdir, "spend.json"), JSON.stringify(spend));

    expect(() => seedCIState(dir, 8)).toThrow("kept unchanged");
    const result = seedCIState(dir);

    expect(result.project).toBe("kept");
    expect(result.stage).toBe(4);
    expect(result.stateCreated).toBe(false);
    expect(result.spendCreated).toBe(false);
    expect(JSON.parse(fs.readFileSync(path.join(kdir, "state.json"), "utf8"))).toEqual(state);
    expect(JSON.parse(fs.readFileSync(path.join(kdir, "spend.json"), "utf8"))).toEqual(spend);
  });

  test("does not backfill a goal into an existing unlocked state", () => {
    const dir = makeProject();
    const kdir = path.join(dir, ".kineti");
    fs.mkdirSync(kdir);
    fs.writeFileSync(path.join(kdir, "state.json"), JSON.stringify({ root_goal: null }));

    expect(() => seedCIState(dir)).toThrow("set the goal explicitly");
    expect(fs.existsSync(path.join(kdir, "spend.json"))).toBe(false);
  });
});
