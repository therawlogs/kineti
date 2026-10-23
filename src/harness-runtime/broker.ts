import fs from "node:fs";
import path from "node:path";
import { RunJournal } from "./journal.ts";
import { DEFAULT_LIMITS, EDITABLE, READABLE, exactObject, sha, type CheckResult } from "./types.ts";
export type CheckRunner = (candidate: string, signal?: AbortSignal) => Promise<CheckResult>;
export class Broker {
    calls = 0;
    changed = false;
    constructor(readonly candidate: string, readonly journal: RunJournal, readonly check: CheckRunner, readonly deadline: number, readonly signal: AbortSignal, readonly beforeContinue: () => void, readonly readOnly = false) { }
    private safe(rel: unknown) {
        if (typeof rel !== "string" || !rel || rel.includes("\0") || path.isAbsolute(rel) || rel.split(/[\\/]/).some(p => p === ".." || p === "" || p.startsWith(".")))
            throw new Error("path denied");
        const full = path.resolve(this.candidate, rel);
        if (!full.startsWith(path.resolve(this.candidate) + path.sep))
            throw new Error("path escape");
        let walk = this.candidate;
        for (const part of rel.split("/")) {
            walk = path.join(walk, part);
            if (fs.lstatSync(walk).isSymbolicLink())
                throw new Error("symlink denied");
        }
        if (!(READABLE as readonly string[]).includes(rel))
            throw new Error("source path not approved");
        return full;
    }
    async call(call: {
        callId: string;
        threadId: string;
        turnId: string;
        tool: string;
        arguments: unknown;
    }, thread: string, turn: string) {
        if (call.threadId !== thread || call.turnId !== turn || !call.callId || call.callId.length > 256)
            throw new Error("foreign tool execution/session reference");
        if (this.journal.read().some(r => r.type === "tool-intent" && r.data.call_id === call.callId))
            throw new Error("duplicate tool call; effects are not replayed");
        if (this.signal.aborted || this.journal.cancelled() || Date.now() >= this.deadline)
            throw new Error("run interrupted or timed out");
        if (++this.calls > DEFAULT_LIMITS.tool_calls)
            throw new Error("tool call limit reached");
        if (Buffer.byteLength(JSON.stringify(call.arguments)) > DEFAULT_LIMITS.tool_bytes)
            throw new Error("tool input exceeds limit");
        this.beforeContinue();
        const args = call.arguments;
        let operation: () => Promise<any>;
        if (call.tool === "read_file") {
            exactObject(args, ["path"]);
            const file = this.safe(args.path);
            if (!fs.statSync(file).isFile() || fs.statSync(file).size > DEFAULT_LIMITS.tool_bytes)
                throw new Error("read size limit");
            operation = async () => ({ path: args.path, content: fs.readFileSync(file, "utf8"), sha256: sha(fs.readFileSync(file)) });
        }
        else if (call.tool === "search_files") {
            exactObject(args, ["query"]);
            if (typeof args.query !== "string" || !args.query || args.query.length > 256)
                throw new Error("invalid literal query");
            operation = async () => {
                const hits: {
                    path: string;
                    line: number;
                    text: string;
                }[] = [];
                for (const rel of READABLE) {
                    if (!fs.existsSync(path.join(this.candidate, rel)))
                        continue;
                    const full = this.safe(rel);
                    if (!fs.statSync(full).isFile() || fs.statSync(full).size > 65536)
                        continue;
                    fs.readFileSync(full, "utf8").split("\n").forEach((line, i) => {
                        if (hits.length < 50 && line.includes(args.query))
                            hits.push({ path: rel, line: i + 1, text: line.slice(0, 512) });
                    });
                }
                return { hits, approved_paths: READABLE };
            };
        }
        else if (call.tool === "apply_patch") {
            exactObject(args, ["path", "expected_sha256", "old_text", "new_text"]);
            if (this.readOnly || args.path !== EDITABLE || typeof args.old_text !== "string" || !args.old_text || typeof args.new_text !== "string")
                throw new Error("patch scope denied");
            const file = this.safe(args.path), bytes = fs.readFileSync(file), text = bytes.toString("utf8");
            if (args.expected_sha256 !== sha(bytes))
                throw new Error("stale patch");
            const first = text.indexOf(args.old_text);
            if (first < 0 || text.indexOf(args.old_text, first + args.old_text.length) >= 0)
                throw new Error("patch must match exactly one occurrence");
            const next = text.slice(0, first) + args.new_text + text.slice(first + args.old_text.length);
            if (Buffer.byteLength(next) > 256 * 1024)
                throw new Error("artifact exceeds limit");
            operation = async () => {
                const stage = file + ".kineti-stage";
                const fd = fs.openSync(stage, "wx", 0o644);
                try {
                    fs.fchmodSync(fd, 0o644);
                    fs.writeSync(fd, next);
                    fs.fsyncSync(fd);
                }
                finally {
                    fs.closeSync(fd);
                }
                fs.renameSync(stage, file);
                const dir = fs.openSync(path.dirname(file), "r");
                try {
                    fs.fsyncSync(dir);
                }
                finally {
                    fs.closeSync(dir);
                }
                this.changed = true;
                return { path: args.path, sha256: sha(next) };
            };
        }
        else if (call.tool === "run_check") {
            exactObject(args, ["check_id"]);
            if (args.check_id !== "spend-cli-options-v1")
                throw new Error("unknown check; arbitrary commands denied");
            operation = () => this.check(this.candidate, this.signal);
        }
        else
            throw new Error("unknown tool; native tools are not permitted");
        this.journal.append("tool-intent", { call_id: call.callId, thread_id: thread, turn_id: turn, tool: call.tool, arguments: args });
        try {
            const value = await operation();
            if (Buffer.byteLength(JSON.stringify(value)) > DEFAULT_LIMITS.tool_bytes)
                throw new Error("tool output exceeds limit");
            this.journal.append("tool-complete", { call_id: call.callId, result: value });
            return value;
        }
        catch (error) {
            this.journal.append("tool-unknown", { call_id: call.callId, reason: (error as Error).message });
            throw error;
        }
    }
}
const shape = (properties: Record<string, any>) => ({ type: "object", properties, required: Object.keys(properties), additionalProperties: false });
export const TOOLS = [
    { name: "search_files", description: "Search approved source files for a literal string", inputSchema: shape({ query: { type: "string" } }) },
    { name: "read_file", description: "Read a bounded approved source file and its SHA-256", inputSchema: shape({ path: { type: "string" } }) },
    { name: "apply_patch", description: "Replace one exact occurrence in the permitted source file; provide the current SHA-256", inputSchema: shape({ path: { type: "string" }, expected_sha256: { type: "string" }, old_text: { type: "string" }, new_text: { type: "string" } }) },
    { name: "run_check", description: "Run the trusted immutable spend-cli-options-v1 check", inputSchema: shape({ check_id: { type: "string", enum: ["spend-cli-options-v1"] } }) },
].map(t => ({ ...t, type: "function", deferLoading: false }));
