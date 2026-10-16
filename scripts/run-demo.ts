const result = Bun.spawnSync(["bash", "demo-no-key.sh"], { cwd: process.cwd(), stdout: "pipe", stderr: "pipe" });
if (result.stdout.length) process.stdout.write(new TextDecoder().decode(result.stdout));
if (result.stderr.length) process.stderr.write(new TextDecoder().decode(result.stderr));
process.exit(result.exitCode ?? 1);
