const result = Bun.spawnSync({
  cmd: ["bash", "demo-no-key.sh"],
  cwd: process.cwd(),
  stdout: "inherit",
  stderr: "inherit",
});

if (result.exitCode !== 0) process.exit(result.exitCode ?? 1);
