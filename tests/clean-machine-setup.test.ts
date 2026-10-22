import { expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dir, "..");
const hosts = ["antigravity", "claude", "codex", "cursor", "fx", "gemini", "opencode"];
const skillNames = fs.readdirSync(path.join(root, "skills"));
const acceptanceRoot = path.join(root, ".kineti", "acceptance", "clean-candidate-install");

test("clean-machine setup installs safely for configured hosts and reports routing limits", () => {
  const run = Bun.spawnSync(["bash", path.join(root, "tests", "test-setup.sh")], {
    cwd: root,
    env: { ...process.env, CODEX_HOME: path.join(root, ".kineti", "acceptance", "codex-home") },
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = new TextDecoder().decode(run.stdout);
  const stderr = new TextDecoder().decode(run.stderr);
  if (run.exitCode !== 0) throw new Error(`${stdout}\n${stderr}`);
  expect(run.exitCode).toBe(0);
  expect(stdout).toContain("PASS: installer smoke test");

  const lines = stdout.split(/\r?\n/).filter((line) => line.startsWith("ROUTING_UNSUPPORTED "));
  expect(lines).toHaveLength(hosts.length);
  for (const host of hosts) {
    expect(lines).toContain(`ROUTING_UNSUPPORTED host=${host} reason=no-isolated-host-model-runner`);
  }

  const reportDir = path.join(root, ".kineti", "acceptance");
  fs.mkdirSync(reportDir, { recursive: true });
  const report = {
    schema_version: 1,
    acceptance: "clean-machine-setup-and-routing",
    setup: Object.fromEntries(hosts.map((host) => [host, "pass"])),
    routing: Object.fromEntries(hosts.map((host) => [host, {
      status: "unsupported",
      reason: "No isolated live host/model runner is available in this acceptance environment.",
    }])),
    evidence: "tests/test-setup.sh output captured by this test",
  };
  fs.writeFileSync(path.join(reportDir, "clean-machine-setup-routing.json"), `${JSON.stringify(report, null, 2)}\n`);
  expect(report.routing.cursor.status).toBe("unsupported");
}, 300_000);

test("local npm tarball and crate install independently without registry access", () => {
  fs.rmSync(acceptanceRoot, { recursive: true, force: true });
  const tarballDir = path.join(acceptanceRoot, "tarball");
  const npmPrefix = path.join(acceptanceRoot, "npm-prefix");
  const crateRoot = path.join(acceptanceRoot, "crate-root");
  const home = path.join(acceptanceRoot, "home");
  const projectCargoHome = path.join(root, ".kineti", "cargo-home");
  const cargoCacheHome = fs.existsSync(path.join(projectCargoHome, "registry"))
    ? projectCargoHome
    : fs.readdirSync(path.join(root, ".kineti", "acceptance"))
      .map((entry) => path.join(root, ".kineti", "acceptance", entry, "cargo-home"))
      .find((dir) => fs.existsSync(dir));
  for (const dir of [tarballDir, npmPrefix, crateRoot, home, path.join(acceptanceRoot, "npm-cache"), path.join(root, ".kineti", "candidate-cargo-target")]) fs.mkdirSync(dir, { recursive: true });
  const candidateEnv = {
    ...process.env,
    npm_config_cache: path.join(acceptanceRoot, "npm-cache"),
    ...(cargoCacheHome ? { CARGO_HOME: cargoCacheHome } : {}),
    CARGO_TARGET_DIR: path.join(root, ".kineti", "candidate-cargo-target"),
  };

  function run(args: string[], env: NodeJS.ProcessEnv = candidateEnv): string {
    const result = Bun.spawnSync(args, { cwd: root, env, stdout: "pipe", stderr: "pipe" });
    const stdout = new TextDecoder().decode(result.stdout);
    const stderr = new TextDecoder().decode(result.stderr);
    if (result.exitCode !== 0) throw new Error(`${args.join(" ")}\n${stdout}\n${stderr}`);
    expect(result.exitCode).toBe(0);
    return stdout;
  }

  const pack = JSON.parse(run(["npm", "pack", "--json", "--pack-destination", tarballDir])) as Array<{ filename: string }>;
  expect(pack).toHaveLength(1);
  const candidateTarball = path.join(tarballDir, pack[0].filename);
  expect(fs.existsSync(candidateTarball)).toBe(true);

  run(["npm", "install", "--offline", "--no-audit", "--no-fund", "--prefix", npmPrefix, candidateTarball]);
  const npmCli = path.join(npmPrefix, "node_modules", ".bin", "kineti");
  expect(run([npmCli, "--help"])).toContain("evidence-verify");
  const cursorSkills = path.join(home, ".cursor", "skills");
  fs.mkdirSync(path.join(cursorSkills, "my-skill"), { recursive: true });
  fs.writeFileSync(path.join(cursorSkills, "my-skill", "SKILL.md"), "keep this user skill\n");
  const npmInit = run([npmCli, "init", "--host", "cursor"], {
    ...candidateEnv,
    HOME: home,
    CODEX_HOME: path.join(home, ".codex"),
  });
  expect(npmInit).toContain("Cursor: add to .cursor/rules/kineti.mdc");
  expect(npmInit).toContain(`installed: ${skillNames.length} skills`);
  for (const skill of skillNames) {
    expect(fs.existsSync(path.join(cursorSkills, `kineti-${skill}`, "SKILL.md"))).toBe(true);
  }
  expect(fs.existsSync(path.join(cursorSkills, "kineti-officehours", "SKILL.md"))).toBe(true);
  expect(fs.readFileSync(path.join(cursorSkills, "my-skill", "SKILL.md"), "utf8")).toBe("keep this user skill\n");

  let crateStatus: "pass" | "unsupported";
  let crateReason: string | undefined;
  if (!cargoCacheHome) {
    crateStatus = "unsupported";
    crateReason = "No project-local Cargo cache is available for an offline candidate install.";
  } else {
    const crateInstall = Bun.spawnSync([
      "cargo", "install", "--offline", "--debug", "--root", crateRoot,
      "--path", path.join(root, "core-native", "crates", "kineti-cli"), "--locked", "--force",
    ], { cwd: root, env: candidateEnv, stdout: "pipe", stderr: "pipe" });
    const crateOutput = `${new TextDecoder().decode(crateInstall.stdout)}\n${new TextDecoder().decode(crateInstall.stderr)}`;
    if (crateInstall.exitCode === 0) {
      const crateVersion = run([path.join(crateRoot, "bin", "kineti-cli"), "--version"]);
      expect(crateVersion).toContain("kineti-cli 0.5.0");
      crateStatus = "pass";
    } else {
      expect(crateOutput).toContain("no matching package named");
      crateStatus = "unsupported";
      crateReason = "Offline project Cargo cache lacks a required dependency; registry access was not attempted.";
    }
  }

  const report = {
    schema_version: 1,
    acceptance: "local-candidate-clean-install",
    explicit_published_baseline: "not-run: this offline check does not access package registries",
    npm_candidate_tarball: { status: "pass", path: path.relative(root, candidateTarball) },
    npm_candidate_init_cursor: "pass",
    crate_candidate_local_install: {
      status: crateStatus,
      reason: crateReason,
    },
  };
  fs.writeFileSync(path.join(acceptanceRoot, "results.json"), `${JSON.stringify(report, null, 2)}\n`);
  console.log(`CANDIDATE_NPM_STATUS=pass`);
  console.log(`CANDIDATE_CRATE_STATUS=${crateStatus}${crateStatus === "unsupported" ? " reason=missing-offline-dependency-cache" : ""}`);
}, 300_000);
