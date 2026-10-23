/** Trusted driver. Candidate runs as a different UID, with captured output only. */
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
const source = "/candidate/bin/kineti-spend.ts";
const artifact = crypto.createHash("sha256").update(fs.readFileSync(source)).digest("hex");
const results: {
    id: string;
    passed: boolean;
    reason?: string;
}[] = [];
const observations: {
    arguments: string[];
    exit_code: number | null;
    stdout: string;
    stderr: string;
    output_sha256: string;
}[] = [];
let retained = 0;
const began = Date.now();
let sequence = 0;
function scratch() { const dir = `/scratch/case-${++sequence}`; fs.mkdirSync(dir, { mode: 0o700 }); fs.chownSync(dir, 10002, 10002); fs.writeFileSync(path.join(dir, "kineti.config.json"), JSON.stringify({ root_goal: "Isolated spending CLI acceptance", settings: { spend_limit_usd: { global: 50, per_stage_default: 10 } } }), { mode: 0o444 }); return dir; }
function snapshot(dir: string): string { const walk = (p: string): any => fs.readdirSync(p, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).map(e => { const q = path.join(p, e.name); return [e.name, e.isSymbolicLink() ? "symlink" : e.isDirectory() ? walk(q) : crypto.createHash("sha256").update(fs.readFileSync(q)).digest("hex")]; }); return JSON.stringify(walk(dir)); }
function run(dir: string, args: string[]) {
    const r = spawnSync("/usr/bin/setpriv", ["--reuid=10002", "--regid=10002", "--clear-groups", "--no-new-privs", "/usr/bin/timeout", "--signal=KILL", "5", "/usr/bin/prlimit", "--cpu=3", "--fsize=65536", "--", "/usr/local/bin/bun", source, ...args], { cwd: dir, env: { PATH: "/usr/local/bin:/usr/bin:/bin", KINETI_MACHINE_DIR: path.join(dir, "machine"), HOME: "/nonexistent", TZ: "UTC" }, encoding: "utf8", timeout: 6500, maxBuffer: 65536 });
    spawnSync("/usr/bin/setpriv", ["--reuid=10002", "--regid=10002", "--clear-groups", "--no-new-privs", "/bin/kill", "-KILL", "-1"], { stdio: "ignore" });
    const stdout = String(r.stdout ?? ""), stderr = String(r.stderr ?? ""), take = (v: string) => { const part = v.slice(0, Math.min(512, Math.max(0, 8192 - retained))); retained += part.length; return part; };
    observations.push({ arguments: args, exit_code: r.status, stdout: take(stdout), stderr: take(stderr), output_sha256: crypto.createHash("sha256").update(stdout).update("\0").update(stderr).digest("hex") });
    return r.status;
}
function record(id: string, body: () => boolean) {
    try {
        const passed = body();
        results.push({ id, passed, ...(!passed ? { reason: "trusted assertion failed" } : {}) });
    }
    catch (error) {
        results.push({ id, passed: false, reason: (error as Error).message.slice(0, 200) });
    }
}
for (const [id, args] of [
    ["unknown-log", ["log", "--stage", "build", "--tokens-in", "0", "--tokens-out", "0", "--usd", "0.01", "--unknown-spend-option"]],
    ["unknown-check", ["check", "--unknown-spend-option"]],
    ["unknown-status", ["status", "--unknown-spend-option"]],
    ["unknown-reset", ["reset", "--i-am-human", "--unknown-spend-option"]],
] as [
    string,
    string[]
][]) {
    record(id, () => { const dir = scratch(), before = snapshot(dir), code = run(dir, args); return code === 2 && snapshot(dir) === before; });
}
record("supported-log-and-check", () => { const dir = scratch(); return run(dir, ["log", "--stage", "build", "--tokens-in", "1", "--tokens-out", "1", "--usd", "0.01"]) === 0 && run(dir, ["check"]) === 0 && JSON.parse(fs.readFileSync(path.join(dir, ".kineti/spend.json"), "utf8")).entries === 1; });
record("threshold-and-reset-preserved", () => { const dir = scratch(); return run(dir, ["log", "--stage", "build", "--tokens-in", "0", "--tokens-out", "0", "--usd", "9.5"]) === 3 && run(dir, ["reset", "--i-am-human"]) === 0 && run(dir, ["check"]) === 3; });
record("help-preserved", () => run(scratch(), ["--help"]) === 0);
record("corrupt-accounting-blocks", () => {
    const dir = scratch();
    if (run(dir, ["log", "--stage", "build", "--tokens-in", "0", "--tokens-out", "0", "--usd", "0.01"]) !== 0)
        return false;
    const state = path.join(dir, ".kineti/spend.json"), log = path.join(dir, ".kineti/spend.log.jsonl"), history = fs.readFileSync(log);
    fs.writeFileSync(state, "{broken");
    return run(dir, ["check"]) === 3 && run(dir, ["log", "--stage", "build", "--tokens-in", "0", "--tokens-out", "0", "--usd", "0.01"]) === 3 && fs.readFileSync(log).equals(history);
});
record("unknown-option-cannot-reset-stop", () => {
    const dir = scratch();
    if (run(dir, ["log", "--stage", "build", "--tokens-in", "0", "--tokens-out", "0", "--usd", "9.5"]) !== 3)
        return false;
    const before = snapshot(dir);
    return run(dir, ["reset", "--i-am-human", "--unknown-spend-option"]) === 2 && snapshot(dir) === before;
});
record("supported-management-commands", () => {
    const dir = scratch();
    return run(dir, ["log", "--stage", "build", "--tokens-in", "0", "--tokens-out", "0", "--usd", "0.01"]) === 0 &&
        run(dir, ["status"]) === 0 && run(dir, ["economics"]) === 0 && run(dir, ["status", "--economics"]) === 0 &&
        run(dir, ["reset", "--i-am-human", "--extend-usd", "1"]) === 0 && run(dir, ["check"]) === 0 &&
        JSON.parse(fs.readFileSync(path.join(dir, ".kineti/mirror.json"), "utf8")).ceiling === 51;
});
record("supported-pricing-options", () => {
    const dir = scratch(), common = ["log", "--stage", "pricing", "--tokens-in", "100", "--tokens-out", "100"];
    if (run(dir, [...common, "--model", "gpt-4"]) !== 0 || run(dir, [...common, "--model", "custom", "--rate-in", "1", "--rate-out", "2"]) !== 0 || run(dir, [...common, "--model", "unlisted", "--estimate"]) !== 0)
        return false;
    const state = JSON.parse(fs.readFileSync(path.join(dir, ".kineti/spend.json"), "utf8"));
    return state.entries === 3 && state.total_microcents > 0 && run(dir, ["check"]) === 0;
});
console.log(JSON.stringify({ version: 1, check_id: "spend-cli-options-v1", artifact_sha256: artifact, passed: results.length === 11 && results.every(r => r.passed), cases: results, observations, duration_ms: Date.now() - began }));
