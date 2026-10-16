import path from "node:path";
import fs from "node:fs";

const root = path.resolve(import.meta.dir, "..");
const work = path.join(root, ".kineti", "acceptance", `${Date.now()}-${process.pid}`);
const version = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")).version;
const candidate = path.join(root, ".kineti", "acceptance", "candidate", `kineti-${version}.tgz`);
const env = { ...process.env, npm_config_cache: path.join(work, "npm-cache"), CARGO_HOME: path.join(work, "cargo-home") };
for (const dir of ["npm-baseline", "npm-candidate", "crate-baseline", "crate-candidate", "npm-cache", "cargo-home"]) await Bun.$`mkdir -p ${path.join(work, dir)}`;
function run(cmd: string[]) {
  const p = Bun.spawnSync(cmd, { cwd: root, env, stdout: "pipe", stderr: "pipe" });
  const out = new TextDecoder().decode(p.stdout), err = new TextDecoder().decode(p.stderr);
  if (out) process.stdout.write(out);
  if (err) process.stderr.write(err);
  if (p.exitCode !== 0) throw new Error(`command failed (${p.exitCode}): ${cmd.join(" ")}`);
}

console.log("Baseline: installing published npm kineti@0.4.0");
run(["npm", "install", "--prefix", path.join(work, "npm-baseline"), "--no-audit", "--no-fund", "kineti@0.4.0"]);
run([path.join(work, "npm-baseline", "node_modules", ".bin", "kineti"), "--help"]);

console.log(`Candidate: installing local npm tarball ${candidate}`);
run(["npm", "install", "--prefix", path.join(work, "npm-candidate"), "--no-audit", "--no-fund", candidate]);
run([path.join(work, "npm-candidate", "node_modules", ".bin", "kineti"), "--help"]);

console.log("Baseline: installing published kineti-cli@0.4.0");
run(["cargo", "install", "--root", path.join(work, "crate-baseline"), "--locked", "kineti-cli@0.4.0"]);
run([path.join(work, "crate-baseline", "bin", "kineti-cli"), "--version"]);

console.log("Candidate: installing local crate kineti-cli");
run(["cargo", "install", "--root", path.join(work, "crate-candidate"), "--path", path.join(root, "core-native", "crates", "kineti-cli"), "--locked", "--force"]);
run([path.join(work, "crate-candidate", "bin", "kineti-cli"), "--version"]);
console.log("1 pass\n0 fail\nCLEAN INSTALL ACCEPTANCE PASSED");
