/** Pinned app-server client. It dispatches tool requests; it never executes native tools. */
import { spawn, spawnSync, type ChildProcessWithoutNullStreams } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { prepareNetworkProbe } from "./network-probe.ts";
import { sha } from "./types.ts";
export interface HostEvent {
    method: string;
    params: any;
}
export interface HostClient {
    readonly command: string[];
    readonly version: string;
    readonly executable_sha256: string;
    readonly live: boolean;
    request(method: string, params: any): Promise<any>;
    notify(method: string, params?: any): void;
    onRequest: (method: string, params: any) => Promise<any>;
    onEvent: (event: HostEvent) => void;
    close(): void;
}
export const DISABLED_FEATURES = ["shell_tool", "unified_exec", "multi_agent", "multi_agent_v2", "apps", "plugins", "remote_plugin", "skill_search", "shell_snapshot", "code_mode_host", "code_mode", "sleep_tool", "tool_suggest"];
export function hostCommand(executable: string, work: string) {
    const runtime = ["/usr/bin/cat", "/usr/bin/dash", "/usr/bin/python3", "/lib64/ld-linux-x86-64.so.2", "/lib/x86_64-linux-gnu/libc.so.6", "/usr/lib/x86_64-linux-gnu/libc.so.6"];
    const paths = [work, executable, path.join(work, "allowed-sentinel"), path.join(work, "network-probe"), ...runtime].filter(p => fs.existsSync(p) || p.startsWith(work));
    const canonical = paths.filter(p => fs.existsSync(p)).map(p => fs.realpathSync(p));
    const readRoots = [...new Set([...paths, ...canonical])].map(p => `${JSON.stringify(p)} = "read"`).join(", ");
    const profile = `{ filesystem = { ":root" = "none", ${readRoots} }, network = { enabled = false } }`;
    const args = ["-c", `sqlite_home=${JSON.stringify(path.join(work, "sqlite"))}`, "-c", "web_search=\"disabled\"", "-c", "model_auto_compact_token_limit=1000000000", "-c", `permissions.kineti_harness=${profile}`, "-c", 'default_permissions="kineti_harness"'];
    for (const feature of DISABLED_FEATURES)
        args.push("--disable", feature);
    // Kineti never retries JSON-RPC. Provider retry controls are verified separately.
    args.push("app-server", "--listen", "stdio://");
    return [executable, ...args];
}
export class CodexClient implements HostClient {
    readonly command: string[];
    readonly version = "0.163.0";
    readonly executable_sha256: string;
    readonly live = true;
    onRequest: HostClient['onRequest'] = async () => { throw new Error("unexpected server request"); };
    onEvent: HostClient['onEvent'] = () => { };
    private child: ChildProcessWithoutNullStreams;
    private sequence = 0;
    private closed = false;
    private buffer = "";
    private bytes = 0;
    private pending = new Map<number, {
        resolve: (v: any) => void;
        reject: (e: Error) => void;
        timer: ReturnType<typeof setTimeout>;
    }>();
    private decoder = new TextDecoder("utf-8", { fatal: true });
    private serverIds = new Set<string | number>();
    constructor(executable: string, work: string, signal: AbortSignal) {
        const exe = Bun.which(executable) ?? (path.isAbsolute(executable) && fs.existsSync(executable) ? executable : null);
        if (!exe)
            throw new Error("Codex not installed");
        const check = spawnSync(exe, ["--version"], { encoding: "utf8", timeout: 10000 });
        if (check.status !== 0 || check.stdout.match(/\d+\.\d+\.\d+/)?.[0] !== this.version)
            throw new Error("Codex must be pinned to 0.163.0");
        prepareNetworkProbe(work);
        this.executable_sha256 = sha(fs.readFileSync(exe));
        this.command = hostCommand(fs.realpathSync(exe), work);
        fs.mkdirSync(work, { recursive: true, mode: 0o700 });
        this.child = spawn(this.command[0], this.command.slice(1), { cwd: work, stdio: ["pipe", "pipe", "pipe"], detached: process.platform !== "win32" });
        let stderrBytes = 0;
        const stderrFile = path.join(work, "host.stderr");
        this.child.stderr.on("data", b => {
            if (stderrBytes < 65536) {
                const part = b.subarray(0, 65536 - stderrBytes);
                fs.appendFileSync(stderrFile, part, { mode: 0o600 });
                stderrBytes += part.length;
            }
        });
        this.child.stdin.on("error", () => this.fail(new Error("host input closed")));
        this.child.on("error", () => this.fail(new Error("host spawn failed")));
        this.child.on("close", () => this.fail(new Error("host closed before completion")));
        this.child.stdout.on("data", b => {
            try {
                this.bytes += b.length;
                if (this.bytes > 8 * 1024 * 1024)
                    throw new Error("host output limit");
                this.buffer += this.decoder.decode(b, { stream: true });
                if (this.buffer.length > 1024 * 1024)
                    throw new Error("oversize protocol message");
                let i;
                while ((i = this.buffer.indexOf("\n")) >= 0) {
                    const line = this.buffer.slice(0, i);
                    this.buffer = this.buffer.slice(i + 1);
                    if (!line.trim())
                        throw new Error("empty protocol record");
                    this.dispatch(JSON.parse(line));
                }
            }
            catch (e) {
                this.fail(e as Error);
            }
        });
        signal.addEventListener("abort", () => this.close(), { once: true });
        if (signal.aborted)
            this.close();
    }
    private send(value: any) {
        if (this.closed)
            throw new Error("host connection closed");
        this.child.stdin.write(JSON.stringify({ jsonrpc: "2.0", ...value }) + "\n");
    }
    request(method: string, params: any) {
        return new Promise<any>((resolve, reject) => {
            const id = ++this.sequence, timer = setTimeout(() => { this.pending.delete(id); reject(new Error(`host RPC timeout: ${method}`)); this.close(); }, 15000);
            this.pending.set(id, { resolve, reject, timer });
            try {
                this.send({ id, method, params });
            }
            catch (e) {
                clearTimeout(timer);
                this.pending.delete(id);
                reject(e);
            }
        });
    }
    notify(method: string, params?: any) { this.send({ method, ...(params === undefined ? {} : { params }) }); }
    private dispatch(message: any) {
        if (!message || typeof message !== "object" || Array.isArray(message))
            throw new Error("malformed host protocol");
        if (typeof message.method === "string") {
            if (message.id !== undefined) {
                const id = message.id;
                if ((typeof id !== "string" && !Number.isSafeInteger(id)) || this.serverIds.has(id))
                    throw new Error("invalid or duplicate server RPC identity");
                this.serverIds.add(id);
                void this.onRequest(message.method, message.params).then(result => {
                    if (!this.closed)
                        this.send({ id, result });
                }).catch(error => {
                    if (!this.closed)
                        this.send({ id, error: { code: -32602, message: "Kineti denied this tool request" } });
                    this.fail(error);
                });
            }
            else
                this.onEvent({ method: message.method, params: message.params });
            return;
        }
        if (typeof message.id !== "number" || !this.pending.has(message.id))
            throw new Error("unexpected RPC response identity");
        if (Object.hasOwn(message, "result") === Object.hasOwn(message, "error"))
            throw new Error("malformed RPC response");
        const p = this.pending.get(message.id)!;
        this.pending.delete(message.id);
        clearTimeout(p.timer);
        if (message.error)
            p.reject(new Error(`host RPC rejected: ${JSON.stringify(message.error).slice(0, 512)}`));
        else if (Object.hasOwn(message, "result"))
            p.resolve(message.result);
        else
            p.reject(new Error("RPC response has no result"));
    }
    private fail(error: Error) {
        for (const p of this.pending.values()) {
            clearTimeout(p.timer);
            p.reject(error);
        }
        this.pending.clear();
        if (!this.closed) {
            try {
                this.onEvent({ method: "kineti/transportFailure", params: { reason: error.message } });
            }
            finally {
                this.close();
            }
        }
    }
    close() {
        if (this.closed)
            return;
        this.closed = true;
        for (const p of this.pending.values()) {
            clearTimeout(p.timer);
            p.reject(new Error("host interrupted"));
        }
        this.pending.clear();
        try {
            if (this.child.pid) {
                if (process.platform !== "win32")
                    process.kill(-this.child.pid, "SIGKILL");
                else
                    this.child.kill("SIGKILL");
            }
        }
        catch { }
        ;
    }
}
