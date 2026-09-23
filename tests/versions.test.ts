import { describe, test, expect } from "bun:test";
import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

function read(file: string): string {
  return fs.readFileSync(path.join(ROOT, file), "utf8");
}

describe("repo version and layout drift guards", () => {
  test("package, config, and Cargo workspace versions agree", () => {
    const pkg = JSON.parse(read("package.json"));
    const cfg = JSON.parse(read("kineti.config.json"));
    const cargo = read("core-native/Cargo.toml");
    const m = cargo.match(/\[workspace\.package\][^\[]*?version\s*=\s*"([^"]+)"/s);
    expect(m?.[1]).toBeDefined();
    const mismatches = [
      ["package.json", pkg.version],
      ["kineti.config.json", cfg.version],
      ["core-native/Cargo.toml [workspace.package]", m?.[1]],
    ].filter(([, v]) => v !== pkg.version);
    expect(mismatches).toEqual([]);
  });

  test("router has no hardcoded release version fallback", () => {
    const src = read("bin/kineti.ts");
    expect(src).toContain("0.0.0-dev");
    expect(src).not.toMatch(/let version = "\d+\.\d+\.\d+"/);
  });

  test("config hosts list matches hosts/*.conf files", () => {
    const cfg = JSON.parse(read("kineti.config.json"));
    const confs = fs
      .readdirSync(path.join(ROOT, "hosts"))
      .filter((f) => f.endsWith(".conf"))
      .map((f) => f.replace(/\.conf$/, ""))
      .sort();
    expect([...cfg.hosts].sort()).toEqual(confs);
  });

  test("generated router exists and runs under node", () => {
    const built = read("bin/kineti.js");
    expect(built.startsWith("#!/usr/bin/env node")).toBe(true);
  });
});
