import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import { BASE_COMMIT, EDITABLE, sha, type CheckResult } from "./types.ts";
const env = () => ({ ...process.env, DOCKER_CONFIG: process.env.DOCKER_CONFIG ?? path.join(process.cwd(), ".kineti/docker") });
export function prepareCandidate(repo: string, destination: string) {
    if (fs.existsSync(destination))
        throw new Error("refusing to replace candidate checkout");
    const archive = spawnSync("git", ["archive", "--format=tar", BASE_COMMIT], { cwd: repo, maxBuffer: 64 * 1024 * 1024 });
    if (archive.status !== 0)
        throw new Error("pinned baseline commit unavailable");
    fs.mkdirSync(destination, { recursive: true, mode: 0o755 });
    const extract = spawnSync("tar", ["-xf", "-", "-C", destination], { input: archive.stdout });
    if (extract.status !== 0)
        throw new Error("baseline extraction failed");
    const modes = (dir: string) => {
        fs.chmodSync(dir, 0o755);
        for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
            const full = path.join(dir, e.name);
            if (e.isSymbolicLink())
                throw new Error("symlink in baseline archive");
            if (e.isDirectory())
                modes(full);
            else if (e.isFile())
                fs.chmodSync(full, fs.statSync(full).mode & 0o111 ? 0o755 : 0o644);
        }
    };
    modes(destination);
    if (fs.existsSync(path.join(destination, ".git")) || fs.existsSync(path.join(destination, ".kineti")))
        throw new Error("unexpected private state in candidate archive");
    const manifest = snapshot(destination);
    return { base_commit: BASE_COMMIT, manifest, artifact_sha256: manifest[EDITABLE] };
}
export function snapshot(root: string) {
    const rows: Record<string, string> = {};
    const walk = (dir: string) => {
        for (const e of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
            const file = path.join(dir, e.name), rel = path.relative(root, file);
            if (e.isSymbolicLink())
                throw new Error("symlink in candidate snapshot");
            if (e.isDirectory())
                walk(file);
            else if (e.isFile())
                rows[rel] = sha(fs.readFileSync(file));
            else
                throw new Error("nonregular candidate input");
        }
    };
    walk(root);
    return rows;
}
export function verifyScope(root: string, baseline: Record<string, string>) {
    const current = snapshot(root);
    if (Object.keys(current).length !== Object.keys(baseline).length)
        throw new Error("candidate changed file inventory");
    for (const [name, digest] of Object.entries(baseline))
        if (name !== EDITABLE && current[name] !== digest)
            throw new Error("candidate changed immutable input");
    return current[EDITABLE];
}
export function workerImage() {
    const r = spawnSync("docker", ["image", "inspect", "kineti-harness-worker:acceptance", "--format", "{{.Id}}"], { env: env(), encoding: "utf8", timeout: 10000 });
    if (r.status !== 0 || !/^sha256:[a-f0-9]{64}$/.test(r.stdout.trim()))
        throw new Error("trusted worker image unavailable; build scripts/harness/build.sh");
    return r.stdout.trim();
}
export async function runCheck(candidate: string, signal?: AbortSignal, pinnedImage = workerImage()): Promise<CheckResult> {
    if (!/^sha256:[a-f0-9]{64}$/.test(pinnedImage))
        throw new Error("invalid pinned worker image");
    const expected = sha(fs.readFileSync(path.join(candidate, EDITABLE))), image = pinnedImage;
    const name = `kineti-task-${crypto.randomUUID()}`;
    const command = ["run", "--rm", "--init", "--name", name, "--network=none", "--read-only", "--memory=512m", "--pids-limit=64", "--cpus=2", "--cap-drop=ALL", "--cap-add=SETUID", "--cap-add=SETGID", "--cap-add=CHOWN", "--cap-add=DAC_OVERRIDE", "--security-opt=no-new-privileges", "--mount", `type=bind,src=${fs.realpathSync(candidate)},dst=/candidate,readonly`, "--tmpfs", "/scratch:mode=0755,nosuid,noexec", "--tmpfs", "/tmp:mode=1777,nosuid,noexec", image];
    return await new Promise((resolve, reject) => {
        const child = spawn("docker", command, { env: env(), stdio: ["ignore", "pipe", "pipe"] });
        let out = "", err = "", ended = false, stopped = false;
        const stop = () => {
            if (ended)
                return;
            stopped = true;
            child.kill("SIGKILL");
            spawnSync("docker", ["rm", "-f", name], { env: env(), stdio: "ignore", timeout: 5000 });
        };
        const timer = setTimeout(stop, 30000);
        signal?.addEventListener("abort", stop, { once: true });
        if (signal?.aborted)
            stop();
        child.stdout.on("data", b => {
            out += b;
            if (Buffer.byteLength(out) > 65536)
                stop();
        });
        child.stderr.on("data", b => {
            if (err.length < 4096)
                err += b;
        });
        child.on("error", reject);
        child.on("close", code => {
            ended = true;
            clearTimeout(timer);
            signal?.removeEventListener("abort", stop);
            if (code !== 0 || stopped)
                return reject(new Error("isolated check interrupted or failed: " + err.slice(0, 200)));
            try {
                const r = JSON.parse(out) as CheckResult;
                const expectedIds = ["unknown-log", "unknown-check", "unknown-status", "unknown-reset", "supported-log-and-check", "threshold-and-reset-preserved", "help-preserved", "corrupt-accounting-blocks", "unknown-option-cannot-reset-stop", "supported-management-commands", "supported-pricing-options"];
                if (r.version !== 1 || r.check_id !== "spend-cli-options-v1" || r.artifact_sha256 !== expected || r.cases?.length !== 11 || !expectedIds.every((id, i) => r.cases[i].id === id && typeof r.cases[i].passed === "boolean") || r.passed !== r.cases.every(c => c.passed) || sha(fs.readFileSync(path.join(candidate, EDITABLE))) !== expected)
                    throw new Error("trusted checker result or artifact substitution");
                resolve(r);
            }
            catch (e) {
                reject(e);
            }
        });
    });
}
