import { afterEach, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { Broker, TOOLS } from "../src/harness-runtime/broker.ts";
import { RunJournal } from "../src/harness-runtime/journal.ts";
import { executeTask, preflight, inspectRun, cancelRun, accessError, type Preflight } from "../src/harness-runtime/controller.ts";
import { assertRunHeadroom } from "../src/harness-runtime/accounting.ts";
import { reserve, markUnknown, type AccessManifest } from "../src/routing/index.ts";
import { DEFAULT_LIMITS, BASE_COMMIT, EDITABLE, TASK_ID, sha, type CheckResult } from "../src/harness-runtime/types.ts";
import { snapshot } from "../src/harness-runtime/worker.ts";
import { readSpending, withAccountingLock } from "../bin/spending-state.ts";
import { readEvidence, verifyReceiptSignature } from "../bin/kineti-evidence.ts";
import type { HostClient, HostEvent } from "../src/harness-runtime/codex.ts";
const repo = process.cwd(), roots: string[] = [];
const oldSource = 'import fs from "node:fs";\nconst args=process.argv.slice(2);\nif(args[0]!=="--help")fs.writeFileSync("marker","changed");\n';
const insertion = 'const args=process.argv.slice(2);\nif(args.some(v=>v.startsWith("--unknown")))process.exit(2);';
function fixture() { const base = path.join(repo, ".kineti/acceptance/harness-runtime"); fs.mkdirSync(base, { recursive: true }); const root = fs.mkdtempSync(path.join(base, "run-")); roots.push(root); fs.mkdirSync(path.join(root, ".kineti/runs"), { recursive: true }); fs.writeFileSync(path.join(root, ".kineti/state.json"), JSON.stringify({ root_goal: "Test owned execution", mode: "harness", stage: 9 })); fs.writeFileSync(path.join(root, ".kineti/spend.json"), JSON.stringify({ total_microcents: 0, tripped: false })); return root; }
function policy(root: string): Preflight { return { ready: true, reasons: [], model: "gpt-6.1-sol", cap_usd: 3, estimate_usd: 0.1, policy: { schema_version: 1, revision: crypto.randomUUID(), host: "codex", host_version: "0.163.0", per_run_cap_usd: 3, max_wall_seconds: 180, class_models: { fix: ["gpt-6.1-sol"], code: [], plan: [], chat: [] }, cost_estimates: { "gpt-6.1-sol": { usd: 0.1, source: "fixture estimate only" } }, destinations: ["example.invalid"], required_check: { label: TASK_ID, command: ["fixture-check"] } } }; }
function prepare(_repo: string, candidate: string) { fs.mkdirSync(path.join(candidate, "bin"), { recursive: true }); fs.writeFileSync(path.join(candidate, EDITABLE), oldSource); return { base_commit: BASE_COMMIT, manifest: snapshot(candidate), artifact_sha256: sha(oldSource) }; }
async function check(candidate: string): Promise<CheckResult> { const root = path.dirname(candidate), dir = fs.mkdtempSync(path.join(root, "check-")); const artifact = sha(fs.readFileSync(path.join(candidate, EDITABLE))); const invalid = spawnSync(process.execPath, [path.join(candidate, EDITABLE), "log", "--unknown-option"], { cwd: dir }); const untouched = fs.readdirSync(dir).length === 0; fs.rmSync(path.join(dir, "marker"), { force: true }); const valid = spawnSync(process.execPath, [path.join(candidate, EDITABLE), "log"], { cwd: dir }); const cases = [{ id: "unknown-log", passed: invalid.status === 2 && untouched }, { id: "supported-control", passed: valid.status === 0 && fs.readFileSync(path.join(dir, "marker"), "utf8") === "changed" }]; return { version: 1, check_id: TASK_ID, artifact_sha256: artifact, passed: cases.every(c => c.passed), cases, duration_ms: 1 }; }
class FixtureClient implements HostClient {
    command = ["fixture-codex", "app-server"];
    version = "0.163.0";
    executable_sha256 = sha("fixture");
    live = false;
    closed = false;
    count = 0;
    onRequest: HostClient['onRequest'] = async () => { };
    onEvent: HostClient['onEvent'] = () => { };
    constructor(readonly mode = "fix") { }
    notify() { }
    close() { this.closed = true; }
    async request(method: string, params: any): Promise<any> {
        if (method === "initialize")
            return {};
        if (method === "thread/start")
            return { thread: { id: "fixture-thread" }, model: this.mode === "mismatch" ? "other-model" : "gpt-6.1-sol", modelProvider: "fixture", sandbox: { type: "readOnly" } };
        if (method !== "turn/start")
            throw new Error("unexpected fixture RPC");
        const turn = `fixture-turn-${++this.count}`;
        if (this.mode === "early")
            this.onEvent({ method: "turn/started", params: { threadId: "fixture-thread", turn: { id: turn } } });
        setTimeout(() => { void this.respond(turn).catch(error => this.onEvent({ method: "kineti/transportFailure", params: { reason: error.message } })); }, 1);
        return { turn: { id: turn } };
    }
    private async respond(turn: string) {
        if (this.closed)
            return;
        if (this.mode === "sleep")
            return;
        let n = 0;
        const call = (tool: string, args: any) => this.onRequest("item/tool/call", { callId: `${turn}-${++n}`, threadId: this.mode === "foreign" ? "other" : "fixture-thread", turnId: turn, tool, arguments: args });
        if (this.mode === "native") {
            await this.onRequest("item/commandExecution/requestApproval", {});
            return;
        }
        if (this.mode === "foreign-completion") {
            this.onEvent({ method: "turn/completed", params: { threadId: "fixture-thread", turn: { id: "other-turn", status: "completed" } } });
            return;
        }
        if (this.mode !== "noop" && this.mode !== "summary") {
            const r = await call("read_file", { path: EDITABLE });
            const row = JSON.parse(r.contentItems[0].text);
            await call("apply_patch", { path: EDITABLE, expected_sha256: row.sha256, old_text: "const args=process.argv.slice(2);", new_text: insertion });
            await call("run_check", { check_id: TASK_ID });
        }
        this.onEvent({ method: "item/completed", params: { threadId: "fixture-thread", turnId: turn, item: { type: "agentMessage", text: "all tests pass" } } });
        this.onEvent({ method: "turn/completed", params: { threadId: "fixture-thread", turn: { id: turn, status: "completed" } } });
    }
}
const task = { version: 1, task_id: TASK_ID, instruction: "Fix unknown CLI options before writes" };
afterEach(() => {
    for (const root of roots.splice(0))
        fs.rmSync(root, { recursive: true, force: true });
});
test("controller drives real fixture file reads, scoped patch, independent checks and a signed receipt", async () => {
    const root = fixture(), client = new FixtureClient();
    const result = await executeTask(root, task, { fixture: true, client: () => client, prepare, check, preflight: () => policy(root) });
    expect(result.status).toBe("passed");
    expect(result.functional_pass).toBe(true);
    expect(result.live_proof).toBe(false);
    expect(result.host_turn_ids).toHaveLength(1);
    expect(result.tool_calls).toBe(3);
    expect(result.checks[0].passed).toBe(false);
    expect(result.checks.at(-1)?.passed).toBe(true);
    const journal = new RunJournal(root, result.run_id);
    expect(journal.read().filter(r => r.type === "tool-intent")).toHaveLength(3);
    const receipt = readEvidence(root).at(-1)!;
    expect(receipt.parent_run_id).toBe(result.run_id);
    expect(receipt.session_id).toBe(result.session_id);
    expect(verifyReceiptSignature(receipt, fs.readFileSync(path.join(root, ".kineti/evidence-issuer.pem"), "utf8"))).toBe(true);
    expect(receipt.verification?.status).toBe("unsupported");
    const signed = (receipt.harness as any).journal_checkpoint;
    expect(journal.read()[signed.events - 1].hash).toBe(signed.head_sha256);
    expect(readSpending(root).routing_reservations?.[result.run_id].status).toBe("unknown");
    expect(result.model_identity).toBe("unverified");
});
test("a canned summary or unchanged artifact cannot pass coding acceptance", async () => {
    for (const mode of ["noop", "summary"]) {
        const root = fixture();
        const r = await executeTask(root, task, { fixture: true, client: () => new FixtureClient(mode), prepare, check, preflight: () => policy(root) });
        expect(r.functional_pass).toBe(false);
        expect(r.status).toBe("failed");
        expect(r.host_turn_ids).toHaveLength(2);
    }
});
test("native and foreign-session requests cannot become tool execution", async () => {
    for (const mode of ["native", "foreign"]) {
        const root = fixture();
        const r = await executeTask(root, task, { fixture: true, client: () => new FixtureClient(mode), prepare, check, preflight: () => policy(root) });
        expect(r.functional_pass).toBe(false);
        expect(r.reasons.join(" ")).toMatch(/native|foreign/);
        expect(new RunJournal(root, r.run_id).read().filter(e => e.type === "tool-intent")).toHaveLength(0);
    }
});
test("gate and pending modes fail preflight; invalid input causes no run writes", async () => {
    const root = fixture();
    for (const mode of ["gate", null]) {
        fs.writeFileSync(path.join(root, ".kineti/state.json"), JSON.stringify({ mode, root_goal: "same goal" }));
        expect(preflight(root).reasons.join(" ")).toMatch(/mode/);
    }
    await expect(executeTask(root, { ...task, unknown: true })).rejects.toThrow("unknown fields");
    expect(fs.readdirSync(path.join(root, ".kineti/runs"))).toEqual([]);
});
test("broker rejects escapes, symlinks, arbitrary commands, foreign IDs and duplicate effects", async () => {
    const root = fixture(), candidate = path.join(root, "candidate");
    prepare(root, candidate);
    const id = crypto.randomUUID(), journal = new RunJournal(root, id, true), abort = new AbortController(), broker = new Broker(candidate, journal, check, Date.now() + 10000, abort.signal, () => { });
    const invoke = (tool: string, args: any, callId = crypto.randomUUID()) => broker.call({ callId, threadId: "t", turnId: "v", tool, arguments: args }, "t", "v");
    for (const name of ["../outside", "/outside", ".kineti/signing.key"]) {
        await expect(invoke("read_file", { path: name })).rejects.toThrow("path");
    }
    fs.symlinkSync(EDITABLE, path.join(candidate, "link"));
    await expect(invoke("read_file", { path: "link" })).rejects.toThrow("symlink");
    fs.unlinkSync(path.join(candidate, "link"));
    await expect(invoke("shell", { command: "echo fake" })).rejects.toThrow("unknown tool");
    await expect(invoke("run_check", { check_id: "arbitrary" })).rejects.toThrow("unknown check");
    const args = { path: EDITABLE, expected_sha256: sha(oldSource), old_text: "const args=process.argv.slice(2);", new_text: insertion };
    const call = crypto.randomUUID();
    await invoke("apply_patch", args, call);
    const digest = sha(fs.readFileSync(path.join(candidate, EDITABLE)));
    await expect(invoke("apply_patch", args, call)).rejects.toThrow("duplicate");
    expect(sha(fs.readFileSync(path.join(candidate, EDITABLE)))).toBe(digest);
});
test("read-only reviewer cannot patch and call limits stop tools", async () => { const root = fixture(), candidate = path.join(root, "candidate"); prepare(root, candidate); const journal = new RunJournal(root, crypto.randomUUID(), true), broker = new Broker(candidate, journal, check, Date.now() + 10000, new AbortController().signal, () => { }, true); await expect(broker.call({ callId: "x", threadId: "t", turnId: "v", tool: "apply_patch", arguments: { path: EDITABLE, expected_sha256: sha(oldSource), old_text: "x", new_text: "y" } }, "t", "v")).rejects.toThrow("scope"); broker.calls = DEFAULT_LIMITS.tool_calls; await expect(broker.call({ callId: "y", threadId: "t", turnId: "v", tool: "read_file", arguments: { path: EDITABLE } }, "t", "v")).rejects.toThrow("limit"); });
test("durable journal detects truncation and leaves incomplete tool intent unreplayed", () => { const root = fixture(), j = new RunJournal(root, crypto.randomUUID(), true); j.append("tool-intent", { call_id: "uncertain", tool: "apply_patch" }); expect(j.read()).toHaveLength(1); fs.writeFileSync(j.events, ""); expect(() => j.read()).toThrow(); });
test("active cancellation terminates the model wait and retains uncertain cost", async () => {
    const root = fixture(), client = new FixtureClient("sleep");
    const pending = executeTask(root, task, { fixture: true, client: () => client, prepare, check, preflight: () => policy(root) });
    let id = "";
    for (let n = 0; n < 100; n++) {
        id = fs.readdirSync(path.join(root, ".kineti/runs"))[0] ?? "";
        if (id && inspectRun(root, id).status === "running")
            break;
        await Bun.sleep(10);
    }
    expect(cancelRun(root, id).cancel_requested).toBe(true);
    const r = await pending;
    expect(r.status).toBe("interrupted");
    expect(client.closed).toBe(true);
    expect(r.budget.status).toBe("unknown");
});
test("pre-inference model mismatch cancels only its definite unlaunched reservation", async () => { const root = fixture(), client = new FixtureClient("mismatch"); const r = await executeTask(root, task, { fixture: true, client: () => client, prepare, check, preflight: () => policy(root) }); expect(r.functional_pass).toBe(false); expect(r.model_identity).toBe("mismatch"); expect(r.budget.status).toBe("not-launched"); expect(client.count).toBe(0); expect(readSpending(root).routing_reservations?.[r.run_id]).toBeUndefined(); });
test("historical unknown costs prevent constructing a new model session", async () => { const root = fixture(), id = crypto.randomUUID(), session = crypto.randomUUID(); reserve(root, id, session, "9", 0.01, 3); markUnknown(root, id); let constructed = false; const r = await executeTask(root, task, { fixture: true, client: () => { constructed = true; return new FixtureClient(); }, prepare, check, preflight: () => policy(root) }); expect(constructed).toBe(false); expect(r.functional_pass).toBe(false); expect(readSpending(root).routing_reservations?.[id]).toMatchObject({ session_id: session, status: "unknown", microcents: 10000 }); });
test("continuation retains full reserved headroom after limits change", () => { const root = fixture(), id = crypto.randomUUID(), session = crypto.randomUUID(); reserve(root, id, session, "9", 3, 3); expect(() => assertRunHeadroom(root, id, session)).not.toThrow(); fs.writeFileSync(path.join(root, ".kineti/mirror.json"), JSON.stringify({ ceiling: 3 })); expect(() => assertRunHeadroom(root, id, session)).toThrow("headroom"); });
test("catalog discovery stays distinct from live access and stale or revoked access blocks", () => { const now = Date.now(), manifest: AccessManifest = { schema_version: 1, revision: crypto.randomUUID(), host: "codex", ttl_hours: 24, models: [{ id: "gpt-6.1-sol", provenance: ["user-declared", "host-listed"], listed_at: new Date(now).toISOString() }] }; expect(accessError(manifest, now)).toBeNull(); expect(manifest.models[0].provenance).not.toContain("live-call-confirmed"); expect(accessError(manifest, now + 86400000)).toMatch(/stale/); manifest.models[0].revoked = true; expect(accessError(manifest, now)).toMatch(/revoked/); });
test("early turn identity works while foreign completion cannot pass", async () => {
    for (const mode of ["early", "foreign-completion"]) {
        const root = fixture();
        const r = await executeTask(root, task, { fixture: true, client: () => new FixtureClient(mode), prepare, check, preflight: () => policy(root) });
        expect(r.functional_pass).toBe(mode === "early");
        if (mode !== "early")
            expect(r.reasons.join(" ")).toMatch(/foreign/);
    }
});
test("tests and expected results are outside the model readable source set", async () => { const root = fixture(), candidate = path.join(root, "candidate"); prepare(root, candidate); fs.mkdirSync(path.join(candidate, "tests")); fs.writeFileSync(path.join(candidate, "tests/secret.ts"), "owner-expected-answer"); const journal = new RunJournal(root, crypto.randomUUID(), true), broker = new Broker(candidate, journal, check, Date.now() + 10000, new AbortController().signal, () => { }); const invoke = (tool: string, args: any) => broker.call({ callId: crypto.randomUUID(), threadId: "t", turnId: "v", tool, arguments: args }, "t", "v"); await expect(invoke("read_file", { path: "tests/secret.ts" })).rejects.toThrow("not approved"); expect((await invoke("search_files", { query: "owner-expected-answer" })).hits).toEqual([]); });
test("expired deadlines and oversized patches stop before any tool intent", async () => {
    const root = fixture(), candidate = path.join(root, "candidate");
    prepare(root, candidate);
    const journal = new RunJournal(root, crypto.randomUUID(), true), abort = new AbortController();
    const invoke = (broker: Broker, args: any) => broker.call({ callId: crypto.randomUUID(), threadId: "t", turnId: "v", tool: "apply_patch", arguments: args }, "t", "v");
    const args = { path: EDITABLE, expected_sha256: sha(oldSource), old_text: oldSource, new_text: "x" };
    await expect(invoke(new Broker(candidate, journal, check, Date.now() - 1, abort.signal, () => { }), args)).rejects.toThrow("timed out");
    await expect(invoke(new Broker(candidate, journal, check, Date.now() + 10000, abort.signal, () => { }), { ...args, new_text: "x".repeat(65536) })).rejects.toThrow("limit");
    expect(journal.read()).toHaveLength(0);
    expect(fs.readFileSync(path.join(candidate, EDITABLE), "utf8")).toBe(oldSource);
});
test("status reports a dead controller as unknown without replaying its incomplete intent", async () => {
    const root = fixture(), r = await executeTask(root, task, { fixture: true, client: () => new FixtureClient(), prepare, check, preflight: () => policy(root) });
    const journal = new RunJournal(root, r.run_id);
    journal.append("tool-intent", { call_id: "interrupted-patch", tool: "apply_patch" });
    journal.save({ ...r, status: "running", controller_pid: 99999999 });
    const before = fs.readFileSync(journal.events);
    expect(inspectRun(root, r.run_id).status).toBe("unknown");
    expect(fs.readFileSync(journal.events).equals(before)).toBe(true);
});
