#!/usr/bin/env bun
/** Kineti owns this lifecycle; host output alone never passes coding acceptance. */
import fs from "node:fs";
import { strictOptions } from "./strict-options.ts";
import { executeTask, inspectRun, cancelRun, preflight } from "../src/harness-runtime/controller.ts";
import { inspectCapabilities } from "../src/harness-runtime/capabilities.ts";
import { validateTask } from "../src/harness-runtime/types.ts";
async function main(raw: string[]) {
    if (!raw.length || raw[0] === "--help" || raw[0] === "-h") {
        console.log(JSON.stringify({ commands: ["kineti run --host codex --task-file FILE", "kineti run preflight --host codex", "kineti run capabilities --host codex", "kineti run status RUN_ID", "kineti run cancel RUN_ID"], task: { version: 1, task_id: "spend-cli-options-v1", instruction: "Fix unknown spending options before writes; preserve supported commands." }, limits: { wall_seconds: 180, tool_calls: 12, tool_bytes: 65536, correction_turns: 1 }, notice: "$3 owner cap controls admission and continuation, not provider invoice or in-flight overrun. No transport retries. A functional proof does not establish provider-resolved model identity." }));
        return;
    }
    const root = process.cwd(), out = (v: unknown) => console.log(JSON.stringify(v));
    if (["status", "cancel"].includes(raw[0])) {
        const args = strictOptions(raw.slice(1), [], [], [], 1);
        if (args.length !== 1)
            throw new Error("one run ID required");
        out(raw[0] === "status" ? inspectRun(root, args[0]) : cancelRun(root, args[0]));
        return;
    }
    const isPreflight = raw[0] === "preflight", isCapabilities = raw[0] === "capabilities", args = strictOptions(isPreflight || isCapabilities ? raw.slice(1) : raw, isPreflight || isCapabilities ? ["--host"] : ["--host", "--task-file"], [], [], 0);
    const value = (name: string) => { const i = args.indexOf(name); return i < 0 ? undefined : args[i + 1]; };
    if (value("--host") !== "codex")
        throw new Error("this milestone supports only the explicitly selected Codex host");
    if (isCapabilities) {
        const report = await inspectCapabilities(root);
        out(report);
        if (!report.ready)
            process.exitCode = 1;
        return;
    }
    if (isPreflight) {
        const report = preflight(root);
        out(report);
        if (!report.ready)
            process.exitCode = 1;
        return;
    }
    const file = value("--task-file");
    if (!file || !fs.statSync(file).isFile() || fs.statSync(file).size > 65536)
        throw new Error("task file required, at most 64 KiB");
    const task = validateTask(JSON.parse(fs.readFileSync(file, "utf8")));
    console.error("$3 cap limits admission and continuation; provider invoice and in-flight overrun are not guaranteed. No transport retries.");
    const r = await executeTask(root, task);
    out(r);
    if (!r.functional_pass)
        process.exitCode = 1;
}
if (import.meta.main)
    try {
        await main(process.argv.slice(2));
    }
    catch (e) {
        console.log(JSON.stringify({ status: "blocked", reason: (e as Error).message }));
        process.exitCode = 2;
    }
