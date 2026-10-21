import { afterEach, describe, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";

const REPO = path.resolve(import.meta.dir, "..");
const fixture = path.join(REPO, ".kineti", "package-install-tests");
afterEach(() => fs.rmSync(fixture, { recursive: true, force: true }));

describe("offline consumer package acceptance", () => {
  test("packages the workflow, memory, and supported governance commands and installs it from a local tarball", () => {
    fs.rmSync(fixture, { recursive: true, force: true });
    const home = path.join(fixture, "home"), packageDir = path.join(fixture, "package"), consumer = path.join(fixture, "consumer");
    fs.mkdirSync(home, { recursive: true }); fs.mkdirSync(packageDir, { recursive: true }); fs.mkdirSync(consumer, { recursive: true });
    const packed = Bun.spawnSync(["npm", "pack", "--json", "--ignore-scripts", "--pack-destination", packageDir], {
      cwd: REPO, env: { ...process.env, HOME: home }, stdout: "pipe", stderr: "pipe",
    });
    if (packed.exitCode !== 0) throw new Error(packed.stderr.toString() || "npm pack failed");
    const result = JSON.parse(packed.stdout.toString())[0];
    const files = new Set<string>((result.files as { path: string }[]).map((item) => item.path));
    for (const required of ["bin/kineti-workflow.ts", "bin/kineti-context.ts", "bin/kineti-memory-job.ts", "bin/kineti-conformance.ts", "src/security/evaluation-contract.ts", "hosts/capabilities.v1.json", "hosts/conformance.v2.json", "setup.sh"]) {
      expect(files.has(required), `package is missing ${required}`).toBe(true);
    }
    const contents = JSON.parse(fs.readFileSync(path.join(REPO, "package-contents.json"), "utf8")) as { allowed_root_files: string[]; allowed_prefixes: string[]; required_files: string[]; forbidden_files: string[] };
    for (const file of files) expect(contents.allowed_root_files.includes(file) || contents.allowed_prefixes.some(prefix => file.startsWith(prefix)), `undeclared package path ${file}`).toBe(true);
    for (const file of contents.required_files) expect(files.has(file)).toBe(true);
    for (const file of contents.forbidden_files) expect(files.has(file)).toBe(false);
    expect(result.size).toBeGreaterThan(0);
    expect(result.unpackedSize).toBeGreaterThan(result.size);

    const tarball = path.join(packageDir, result.filename);
    const installed = Bun.spawnSync(["npm", "install", "--offline", "--ignore-scripts", "--no-audit", "--no-fund", "--prefix", consumer, tarball], {
      cwd: REPO, env: { ...process.env, HOME: home }, stdout: "pipe", stderr: "pipe",
    });
    if (installed.exitCode !== 0) throw new Error(installed.stderr.toString() || "offline consumer install failed");
    const cli = path.join(consumer, "node_modules", "kineti", "bin", "kineti.js");
    const help = Bun.spawnSync(["node", cli, "--help"], { cwd: consumer, env: { ...process.env, HOME: home }, stdout: "pipe", stderr: "pipe" });
    expect(help.exitCode).toBe(0);
    const output = help.stdout.toString();
    expect(output).toContain("kineti workflow");
    expect(output).toContain("kineti incident");
    expect(output).toContain("kineti context");
    expect(output).toContain("kineti conformance");
  });
});
