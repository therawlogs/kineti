import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { EventEmitter } from "node:events";
import { loadConfig, reserve, markUnknown } from "../routing/index.ts";
import { readSpending, spendingStop, withAccountingLock } from "../../bin/spending-state.ts";
import { loadLimits } from "../../bin/lib.ts";
import { fingerprint, issueEvidence, toolchainVersions, type EvidenceRecord } from "../../bin/kineti-evidence.ts";
import { cancelUnlaunched, assertRunHeadroom } from "./accounting.ts";
import { verifyControls } from "./capabilities.ts";
import { Broker, TOOLS, type CheckRunner } from "./broker.ts";
import { CodexClient, type HostClient } from "./codex.ts";
import { RunJournal } from "./journal.ts";
import { prepareCandidate, runCheck, verifyScope, workerImage } from "./worker.ts";
import { BASE_COMMIT, DEFAULT_LIMITS, EDITABLE, READABLE, TASK_ID, sha, validateTask, type RunRecord, type Task } from "./types.ts";
export interface Preflight {
    ready: boolean;
    reasons: string[];
    model: string;
    cap_usd: number;
    estimate_usd: number | null;
    policy?: ReturnType<typeof loadConfig>["policy"];
}
export function accessError(manifest: ReturnType<typeof loadConfig>["manifest"], now = Date.now()): string | null {
    const row = manifest.models.find(m => m.id === "gpt-6.1-sol");
    if (!row || row.revoked || !row.provenance.includes("user-declared"))
        return "owner permission for gpt-6.1-sol missing or revoked";
    const live = row.provenance.includes("live-call-confirmed"), stamp = Date.parse((live ? row.confirmed_at : row.listed_at) ?? "");
    if (!Number.isFinite(stamp) || stamp > now || now - stamp >= manifest.ttl_hours * 3600000 || (live && Date.parse(row.expires_at ?? "") <= now) || (!live && !row.provenance.includes("host-listed")))
        return "model discovery/access record missing or stale; a fresh listing is not successful live access";
    return null;
}
export function preflight(root: string): Preflight {
    const reasons: string[] = [];
    let policy: Preflight["policy"], estimate: number | null = null;
    try {
        const state = JSON.parse(fs.readFileSync(path.join(root, ".kineti/state.json"), "utf8"));
        if (!["harness", "both"].includes(state.mode))
            reasons.push(state.mode === "gate" ? "gate mode does not execute tasks" : "product mode pending: select harness or both explicitly");
        if (state.mode === "both" && Object.values(state.gates ?? {}).some(g => g === "fail"))
            reasons.push("a required project gate has failed");
    }
    catch {
        reasons.push("project state missing or malformed");
    }
    try {
        const config = loadConfig(root, "codex");
        policy = config.policy;
        const denied = accessError(config.manifest);
        if (denied)
            reasons.push(denied);
        if (!policy.class_models.fix.includes("gpt-6.1-sol"))
            reasons.push("model is not permitted for the fix task");
        if (policy.host_version !== "0.163.0")
            reasons.push("Codex policy must pin version 0.163.0");
        if (policy.per_run_cap_usd > 3)
            reasons.push("task authorization is capped at $3");
        const cost = policy.cost_estimates["gpt-6.1-sol"];
        if (!cost || !cost.source.trim() || cost.usd > policy.per_run_cap_usd)
            reasons.push("supported model-specific estimate missing or exceeds cap");
        else
            estimate = cost.usd;
    }
    catch {
        reasons.push("Codex manifest/policy or supported price estimate missing");
    }
    try {
        const stop = withAccountingLock(root, () => spendingStop(readSpending(root, true), loadLimits(root)));
        if (stop)
            reasons.push(stop);
    }
    catch {
        reasons.push("accounting damaged or incomplete");
    }
    try {
        workerImage();
    }
    catch (e) {
        reasons.push((e as Error).message);
    }
    return { ready: reasons.length === 0, reasons, model: "gpt-6.1-sol", cap_usd: policy?.per_run_cap_usd ?? 3, estimate_usd: estimate, ...(policy ? { policy } : {}) };
}
function processIdentity(pid: number) {
    try {
        return fs.readFileSync(`/proc/${pid}/stat`, "utf8").split(") ")[1].split(" ")[19];
    }
    catch {
        return "unavailable";
    }
}
export function inspectRun(root: string, id: string) {
    const journal = new RunJournal(root, id), record = journal.status();
    if (["running", "checking"].includes(record.status) && (!record.controller_pid || processIdentity(record.controller_pid) !== record.controller_identity))
        return { ...record, status: "unknown", reasons: [...record.reasons, "controller interrupted; inspect intents and retained bytes; do not replay"] };
    return record;
}
export function cancelRun(root: string, id: string) {
    const j = new RunJournal(root, id);
    const r = j.status();
    if (!["running", "checking", "preflight"].includes(r.status))
        return { run_id: id, status: r.status, cancel_requested: false };
    j.cancel();
    return { run_id: id, status: r.status, cancel_requested: true };
}
export interface ControllerDependencies {
    client?: (work: string, signal: AbortSignal) => HostClient;
    check?: CheckRunner;
    prepare?: typeof prepareCandidate;
    preflight?: (root: string) => Preflight;
    fixture?: boolean;
}
export async function executeTask(root: string, taskInput: unknown, dependencies: ControllerDependencies = {}) {
    const task = validateTask(taskInput), runId = crypto.randomUUID(), sessionId = crypto.randomUUID(), at = new Date().toISOString();
    fs.mkdirSync(path.join(root, ".kineti/runs"), { recursive: true, mode: 0o700 });
    const journal = new RunJournal(root, runId, true), ready = (dependencies.preflight ?? preflight)(root);
    const record: RunRecord = { version: 1, run_id: runId, session_id: sessionId, at, updated_at: at, host: "codex", requested_model: ready.model, host_reported_model: null, provider_resolved_model: null, model_identity: "unverified", status: "preflight", functional_pass: false, live_proof: false, task, base_commit: BASE_COMMIT, artifact_sha256: null, patch_sha256: null, host_thread_id: null, host_turn_ids: [], tool_calls: 0, checks: [], budget: { cap_usd: ready.cap_usd, estimate_usd: ready.estimate_usd, status: "not-launched", usage_estimate_usd: null, provider_invoice_verified: false }, limits: DEFAULT_LIMITS, reasons: [], controller_pid: process.pid, controller_identity: processIdentity(process.pid) };
    journal.save(record);
    if (!ready.ready) {
        record.status = "blocked";
        record.reasons = ready.reasons;
        journal.save(record);
        return record;
    }
    const startFingerprint = fingerprint(root), deadline = Date.parse(at) + DEFAULT_LIMITS.wall_ms, abort = new AbortController();
    const originalConfig = Object.keys(dependencies).length ? null : loadConfig(root, "codex");
    let client: HostClient | undefined, broker: Broker | undefined, reserved = false, inference = false, turn = "", thread = "", finished = false;
    let protocolFailure: Error | undefined;
    let startingTurn = false, acceptingTools = false;
    let toolQueue: Promise<unknown> = Promise.resolve();
    let settleTurn: ((value: any) => void) | undefined, rejectTurn: ((error: Error) => void) | undefined;
    const guard = () => {
        if (protocolFailure)
            throw protocolFailure;
        if (abort.signal.aborted || journal.cancelled() || Date.now() >= deadline)
            throw new Error("run cancelled or timed out");
        if (originalConfig) {
            const current = loadConfig(root, "codex"), denied = accessError(current.manifest);
            if (denied)
                throw new Error(denied);
            if (JSON.stringify(current) !== JSON.stringify(originalConfig))
                throw new Error("routing access or policy changed during execution");
        }
        assertRunHeadroom(root, runId, sessionId);
    };
    const timer = setTimeout(() => abort.abort(), Math.max(0, deadline - Date.now())), watch = setInterval(() => {
        if (journal.cancelled())
            abort.abort();
    }, 100);
    const onAbort = () => { rejectTurn?.(new Error("run cancelled or timed out")); client?.close(); };
    abort.signal.addEventListener("abort", onAbort, { once: true });
    const onSignal = () => abort.abort();
    process.on("SIGINT", onSignal);
    process.on("SIGTERM", onSignal);
    record.required_tests = "not-run";
    try {
        const candidate = path.join(journal.dir, "candidate"), prepared = (dependencies.prepare ?? prepareCandidate)(root, candidate);
        journal.append("baseline", prepared);
        const image = dependencies.check ? "fixture-not-live" : workerImage();
        record.bindings = { task_sha256: sha(JSON.stringify(task)), policy_sha256: sha(JSON.stringify(ready.policy)), baseline_manifest_sha256: sha(JSON.stringify(prepared.manifest)), checker_sha256: sha(fs.readFileSync(path.resolve(import.meta.dir, "../../scripts/harness/check-spend.ts"))), worker_image: image, readable: READABLE };
        const check = dependencies.check ?? ((candidate, signal) => runCheck(candidate, signal, image));
        const baseline = await check(candidate, abort.signal);
        record.checks.push(baseline);
        journal.append("baseline-check", baseline);
        if (baseline.passed || !baseline.cases.some(c => c.id.startsWith("unknown-") && !c.passed) || !baseline.cases.filter(c => !c.id.startsWith("unknown-")).every(c => c.passed))
            throw new Error("baseline did not reproduce the expected bug with passing controls");
        const original = fs.readFileSync(path.join(candidate, EDITABLE)), policy = ready.policy!;
        fs.writeFileSync(path.join(journal.dir, "baseline-artifact.ts"), original, { mode: 0o400, flag: "wx" });
        const stage = String(JSON.parse(fs.readFileSync(path.join(root, ".kineti/state.json"), "utf8")).stage ?? "harness");
        // Reserve the full owner cap rather than treating the estimate as an invoice bound.
        reserve(root, runId, sessionId, stage, ready.cap_usd, ready.cap_usd);
        reserved = true;
        record.budget.status = "reserved";
        client = dependencies.client ? dependencies.client(path.join(journal.dir, "host"), abort.signal) : new CodexClient(policy.executable ?? "codex", path.join(journal.dir, "host"), abort.signal);
        record.command = client.command;
        record.host_version = client.version;
        record.executable_sha256 = client.executable_sha256;
        broker = new Broker(candidate, journal, check, deadline, abort.signal, guard);
        client.onRequest = (method, params) => {
            if (!acceptingTools)
                return Promise.reject(new Error("tool request outside active turn"));
            const work = toolQueue.then(async () => {
                if (method !== "item/tool/call") {
                    journal.append("native-request-denied", { method });
                    throw new Error("native execution or permission request denied");
                }
                const value = await broker!.call(params, thread, turn);
                record.tool_calls = broker!.calls;
                return { contentItems: [{ type: "inputText", text: JSON.stringify(value) }], success: true };
            });
            toolQueue = work.catch(() => { });
            return work;
        };
        client.onEvent = ({ method, params }) => {
            try {
                if (/commandExecution|fileChange|mcpToolCall|webSearch|collabAgent|computerUse/.test(method))
                    throw new Error("unexpected native tool event");
                if (method === "kineti/transportFailure")
                    throw new Error(params.reason);
                if (params?.threadId && thread && params.threadId !== thread)
                    throw new Error("foreign host thread event");
                if (params?.turnId && turn && params.turnId !== turn)
                    throw new Error("foreign host turn event");
                if (method === "turn/started" && startingTurn) {
                    const id = params?.turn?.id;
                    if (typeof id !== "string" || !id || (turn && turn !== id))
                        throw new Error("invalid started turn");
                    turn = id;
                }
                if (method === "item/started" || method === "item/completed") {
                    const item = params?.item;
                    if (!item || typeof item.type !== "string")
                        throw new Error("malformed host item");
                    if (!["agentMessage", "reasoning", "plan", "dynamicToolCall"].includes(item.type))
                        throw new Error("unexpected native tool activity");
                    journal.append("host-item", { method, params });
                }
                if (method === "thread/tokenUsage/updated")
                    journal.append("usage", params);
                if (method === "item/agentMessage/delta" || method === "error")
                    journal.append("host-partial", { method, params });
                if (method === "turn/completed") {
                    if (params?.threadId !== thread || params.turn?.id !== turn)
                        throw new Error("foreign host turn completion");
                    if (params.turn.status !== "completed")
                        throw new Error("host turn failed or interrupted");
                    settleTurn?.(params.turn);
                    settleTurn = undefined;
                    rejectTurn = undefined;
                    acceptingTools = false;
                }
            }
            catch (e) {
                protocolFailure = e as Error;
                rejectTurn?.(e as Error);
                abort.abort();
                client?.close();
            }
        };
        await client.request("initialize", { clientInfo: { name: "kineti-harness", version: "0.5.0" }, capabilities: { experimentalApi: true } });
        client.notify("initialized");
        const controls = dependencies.fixture ? { ready: true, live: false, tests: [{ id: "fixture-only", passed: true }] } : await verifyControls(client, path.join(journal.dir, "host"));
        journal.append("host-controls", controls);
        if (!controls.ready)
            throw new Error("Codex capability checks failed; native controls or provider retries remain unsupported");
        const started = await client.request("thread/start", { model: ready.model, allowProviderModelFallback: false, cwd: path.join(journal.dir, "host"), permissions: "kineti_harness", approvalPolicy: "never", environments: [], runtimeWorkspaceRoots: [], ephemeral: true, dynamicTools: TOOLS, multiAgentMode: { custom: "No native delegation. Use only Kineti tools." }, developerInstructions: "Use only the four Kineti tools. Change only bin/kineti-spend.ts. Read the file first. Fix unknown CLI flags before writes. Do not change tests, policies, dependencies or expected output. Run spend-cli-options-v1. A textual test claim is insufficient." });
        thread = started.thread?.id;
        if (typeof thread !== "string" || !thread)
            throw new Error("missing host session");
        record.host_thread_id = thread;
        record.host_reported_model = started.model ?? null;
        if (started.model !== ready.model) {
            record.model_identity = "mismatch";
            throw new Error("host selected a different model");
        }
        journal.append("host-session", { thread_id: thread, host_reported_model: started.model, provider: started.modelProvider, permission: started.sandbox });
        const doTurn = async (prompt: string) => {
            guard();
            turn = "";
            startingTurn = true;
            acceptingTools = true;
            inference = true;
            journal.append("inference-intent", { thread_id: thread, model: ready.model, prompt, prompt_sha256: sha(prompt), continuation: record.host_turn_ids.length > 0 });
            markUnknown(root, runId);
            record.budget.status = "unknown";
            record.status = "running";
            journal.save(record);
            const done = new Promise<any>((resolve, reject) => { settleTurn = resolve; rejectTurn = reject; });
            // Attach the rejection handler before starting the request to avoid an orphan rejection.
            void done.catch(() => { });
            const response = await client!.request("turn/start", { threadId: thread, model: ready.model, environments: [], runtimeWorkspaceRoots: [], permissions: "kineti_harness", approvalPolicy: "never", input: [{ type: "text", text: prompt }] });
            const returned = response.turn?.id;
            if (typeof returned !== "string" || !returned || (turn && turn !== returned))
                throw new Error("missing or inconsistent host turn ID");
            turn = returned;
            startingTurn = false;
            record.host_turn_ids.push(turn);
            await done;
            await toolQueue;
            guard();
        };
        await doTurn(task.instruction + "\nThe required trusted check is spend-cli-options-v1. You must read the source, apply a patch, and run that check.");
        record.status = "checking";
        journal.save(record);
        let final = await check(candidate, abort.signal);
        record.checks.push(final);
        record.required_tests = final.passed ? "passed" : "failed";
        if (!final.passed) {
            await doTurn("The trusted controller checks failed. Make one bounded correction using the tools, then finish. Results: " + JSON.stringify(final));
            record.status = "checking";
            journal.save(record);
            final = await check(candidate, abort.signal);
            record.checks.push(final);
            record.required_tests = final.passed ? "passed" : "failed";
        }
        guard();
        record.artifact_sha256 = verifyScope(candidate, prepared.manifest);
        record.patch_sha256 = sha(Buffer.concat([original, Buffer.from("\0"), fs.readFileSync(path.join(candidate, EDITABLE))]));
        const events = journal.read(), read = events.some(e => e.type === "tool-complete" && events.some(i => i.type === "tool-intent" && i.data.call_id === e.data.call_id && i.data.tool === "read_file")), patched = events.some(e => e.type === "tool-intent" && e.data.tool === "apply_patch");
        if (!final.passed || !broker.changed || !read || !patched || record.artifact_sha256 === prepared.artifact_sha256)
            throw new Error("coding acceptance requires a live read, substantive broker patch and trusted passing checks");
        if (startFingerprint !== fingerprint(root))
            throw new Error("controller source changed during execution");
        record.functional_pass = true;
        record.live_proof = client.live && !Object.keys(dependencies).length;
        record.status = "passed";
        finished = true;
    }
    catch (error) {
        record.status = abort.signal.aborted ? "interrupted" : reserved ? "failed" : "blocked";
        record.reasons.push((error as Error).message);
        journal.append("run-error", { reason: (error as Error).message });
    }
    finally {
        clearTimeout(timer);
        clearInterval(watch);
        EventEmitter.prototype.removeListener.call(process, "SIGINT", onSignal);
        EventEmitter.prototype.removeListener.call(process, "SIGTERM", onSignal);
        client?.close();
        record.tool_calls = broker?.calls ?? 0;
        if (protocolFailure && !record.reasons.includes(protocolFailure.message))
            record.reasons.push(protocolFailure.message);
        if (reserved) {
            try {
                if (inference) {
                    record.budget.status = "unknown";
                    markUnknown(root, runId);
                }
                else {
                    cancelUnlaunched(root, runId, sessionId);
                    record.budget.status = "not-launched";
                    journal.append("no-inference", { reason: "setup ended before any turn/start attempt" });
                }
            }
            catch (e) {
                record.status = "unknown";
                record.functional_pass = false;
                record.live_proof = false;
                record.reasons.push((e as Error).message);
            }
        }
        if (!finished)
            record.live_proof = false;
        journal.save(record);
        const history = journal.read();
        record.journal_checkpoint = { events: history.length, head_sha256: history.at(-1).hash };
        const receipt: EvidenceRecord = { receipt_version: 3, run_id: crypto.randomUUID(), session_id: sessionId, parent_run_id: runId, at: new Date().toISOString(), label: `harness:${runId}`, cmd: (record.command ?? ["kineti", "run"]).join(" "), result_kind: "check", outcome: record.functional_pass ? "passed" : "failed", exit_code: record.functional_pass ? 0 : 1, fingerprint: fingerprint(root), verification: { status: "unsupported", parser: "owned-harness-v1", passed: record.functional_pass ? 1 : 0, failed: record.functional_pass ? 0 : 1, skipped: 0, inventory: [] }, harness: { ...record }, manifest: { command: record.command ?? ["kineti", "run"], dependencies: [], toolchain: toolchainVersions(), environment: {}, input_digest: startFingerprint, artifact_digests: [], parser_sha256: sha(fs.readFileSync(path.resolve(import.meta.dir, "../../bin/kineti-evidence.ts"))) } };
        issueEvidence(receipt, root);
        record.receipt_run_id = receipt.run_id;
        journal.save(record);
    }
    return record;
}
