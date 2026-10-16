import { expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dir, "..");
const version = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")).version;
const candidateTgz = path.join(root, ".kineti", "acceptance", "candidate", `kineti-${version}.tgz`);
const npmCli = path.join(root, ".kineti", "acceptance", "npm-candidate", "node_modules", ".bin", "kineti");
const crateCli = path.join(root, ".kineti", "acceptance", "crate-candidate", "bin", "kineti-cli");

test("locally installed npm tarball and crate expose candidate CLIs", () => {
  expect(fs.existsSync(candidateTgz)).toBe(true);
  expect(fs.existsSync(npmCli)).toBe(true);
  expect(fs.existsSync(crateCli)).toBe(true);
  const npmHelp = Bun.spawnSync([npmCli, "--help"], { cwd: root, stdout: "pipe", stderr: "pipe" });
  expect(npmHelp.exitCode).toBe(0);
  const npmText = new TextDecoder().decode(npmHelp.stdout);
  expect(npmText).toContain("evidence-verify");
  expect(npmText).toContain("protected-write");
  const crateVersion = Bun.spawnSync([crateCli, "--version"], { cwd: root, stdout: "pipe", stderr: "pipe" });
  expect(crateVersion.exitCode).toBe(0);
  expect(new TextDecoder().decode(crateVersion.stdout)).toContain(`kineti-cli ${version}`);
});
