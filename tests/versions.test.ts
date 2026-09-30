import { describe, test, expect } from "bun:test";
import fs from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

function read(file: string): string {
  return fs.readFileSync(path.join(ROOT, file), "utf8");
}

describe("repo version and layout drift guards", () => {
  test("release version agrees across packages, Rust crates, and current docs", () => {
    const pkg = JSON.parse(read("package.json"));
    const version = pkg.version as string;
    const cfg = JSON.parse(read("kineti.config.json"));
    const server = JSON.parse(read("server.json"));
    const mismatches: string[] = [];
    const check = (label: string, actual: string | undefined) => {
      if (actual !== version) mismatches.push(`${label}: ${actual ?? "missing"}`);
    };
    const workspaceVersion = (file: string) =>
      read(file).match(/\[workspace\.package\][^\[]*?version\s*=\s*"([^"]+)"/s)?.[1];

    check("kineti.config.json", cfg.version);
    check("server.json", server.version);
    for (const [index, pkgEntry] of server.packages.entries()) {
      check(`server.json package ${index}`, pkgEntry.version);
    }
    check("Cargo.toml workspace", workspaceVersion("Cargo.toml"));
    check("core-native/Cargo.toml workspace", workspaceVersion("core-native/Cargo.toml"));

    const cratesDir = path.join(ROOT, "core-native/crates");
    const crateNames = fs.readdirSync(cratesDir)
      .filter((name) => fs.statSync(path.join(cratesDir, name)).isDirectory())
      .sort();
    for (const crateName of crateNames) {
      const file = `core-native/crates/${crateName}/Cargo.toml`;
      const manifest = read(file);
      const dependencies = [...manifest.matchAll(/^\s*(kineti-[\w-]+)\s*=\s*\{[^}\n]*version\s*=\s*"([^"]+)"/gm)];
      for (const [, dependency, dependencyVersion] of dependencies) {
        check(`${file} dependency ${dependency}`, dependencyVersion);
      }
    }

    for (const lockFile of ["Cargo.lock", "core-native/Cargo.lock"]) {
      const packages = [...read(lockFile).matchAll(/\[\[package\]\]\s*\nname = "(kineti-[^"]+)"\s*\nversion = "([^"]+)"/g)];
      const lockedNames = packages.map(([, name]) => name).sort();
      if (lockedNames.join(",") !== crateNames.join(",")) {
        mismatches.push(`${lockFile} Kineti crate set: ${lockedNames.join(",")}`);
      }
      for (const [, name, lockedVersion] of packages) check(`${lockFile} ${name}`, lockedVersion);
    }

    const currentDocs: Array<[string, string[]]> = [
      ["README.md", [`v${version}`, `kineti@${version}`, `kineti-cli@${version}`]],
      ["CONTRIBUTING.md", [`crates.io at \`${version}\``]],
      ["SECURITY.md", [`${version} or newer`]],
      [".github/workflows/release.yml", [`default: 'v${version}'`]],
      ["scripts/demo-spend-cap.sh", [`\"version\": \"${version}\"`]],
      ["core-native/crates/kineti-cli/src/main.rs", [`kineti-cli ${version}`, `Kineti v${version}`, `Print version (${version})`]],
      ["tests/ui.test.ts", [`badgeText: \"v${version}\"`, `toContain(\"v${version}\")`]],
    ];
    for (const [file, expected] of currentDocs) {
      const content = read(file);
      for (const text of expected) {
        if (!content.includes(text)) mismatches.push(`${file} missing ${text}`);
      }
    }

    for (const skillName of fs.readdirSync(path.join(ROOT, "skills"))) {
      const file = `skills/${skillName}/SKILL.md`;
      if (!fs.existsSync(path.join(ROOT, file))) continue;
      const skillVersion = read(file).match(/^version:\s*(\S+)/m)?.[1];
      check(file, skillVersion);
    }

    expect(mismatches).toEqual([]);
  });

  test("Codex setup docs use the current table and include the list check", () => {
    for (const file of ["README.md", "docs/TUTORIAL-first-run.md"]) {
      const content = read(file);
      expect(content).toContain("[mcp_servers.kineti]");
      expect(content).not.toContain("[mcp.servers.kineti]");
      expect(content).toContain("codex mcp list");
    }
  });

  test("README counts match the code they describe", () => {
    const tsTests = fs
      .readdirSync(path.join(ROOT, "tests"))
      .filter((f) => f.endsWith(".ts"))
      .reduce((n, f) => n + (read(`tests/${f}`).match(/\b(?:it|test)\s*\(/g) || []).length, 0);

    let rustTests = 0;
    const walk = (dir: string): void => {
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        if (entry.isDirectory()) {
          if (entry.name === "target" || entry.name === "node_modules") continue;
          walk(path.join(dir, entry.name));
        } else if (entry.name.endsWith(".rs")) {
          const src = fs.readFileSync(path.join(dir, entry.name), "utf8");
          rustTests += (src.match(/#\[(?:test|tokio::test|rstest)\]/g) || []).length;
        }
      }
    };
    walk(path.join(ROOT, "core-native"));

    const skills = fs.readdirSync(path.join(ROOT, "skills")).length;
    const mcpTools = (read("bin/kineti-mcp.ts").match(/^\s+name: "kineti_[a-z_]+"/gm) || []).length;
    const readme = read("README.md");

    expect(readme).toContain(`${tsTests} TypeScript governance tests`);
    expect(readme).toContain(`${rustTests} native Rust tests`);
    expect(readme).toContain(`${tsTests + rustTests} total passed tests`);
    expect(readme).toContain(`${tsTests} TypeScript governance & causal test suites`);
    expect(readme).toContain(`${skills} agent workflow skills`);
    expect(readme).toContain(`${mcpTools} native governance tools`);
  });

  test("host installer help lists every host config that exists", () => {
    const hosts = fs
      .readdirSync(path.join(ROOT, "hosts"))
      .filter((f) => f.endsWith(".conf"))
      .map((f) => f.replace(/\.conf$/, ""));
    const setup = read("setup.sh");
    // Help text must be derived from hosts/*.conf, not a hardcoded list.
    expect(setup).toContain("host_list()");
    expect(setup).not.toContain("--host opencode|claude|gemini|codex|cursor]");
    const config = JSON.parse(read("kineti.config.json"));
    expect((config.hosts || []).length).toBe(hosts.length);
  });

  test("MCP install commands match each host's documented CLI syntax", () => {
    const readme = read("README.md");
    const tutorial = read("docs/TUTORIAL-first-run.md");
    // OpenCode takes the command after `--`; a positional form does not exist.
    expect(readme).toContain("opencode mcp add kineti -- npx -y kineti mcp");
    expect(tutorial).toContain("opencode mcp add kineti -- npx -y kineti mcp");
    for (const content of [readme, tutorial]) {
      expect(content).not.toMatch(/opencode mcp add kineti npx/);
      // init does not write MCP config; docs must not claim otherwise.
      expect(content).not.toContain("links project rules and MCP configurations");
      expect(content).not.toContain("configures your detected AI editors");
    }
  });

  test("README does not repeat stale capability claims", () => {
    const readme = read("README.md");
    const stale = [
      "<12 KB payload",
      "168 TypeScript",
      "419 total",
      "16 agent workflow",
      "Single static page (index.html only)",
      "(PGP/disclosure policy",
      "Cursor & Windsurf",
    ];
    for (const claim of stale) expect(readme).not.toContain(claim);
  });

  test("SECURITY.md claims match what the code actually does", () => {
    const policy = read("SECURITY.md");
    // The companion prints its token to the terminal by design.
    expect(policy).not.toContain("never print secrets, passwords, or raw auth tokens");
    expect(policy).toContain("0600");
    // The shared writers are what make the permission claim true.
    const lib = read("bin/lib.ts");
    expect(lib).toContain("mode: 0o600");
    expect(lib).toContain("lockFileMode");
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
