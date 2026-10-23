/** Real host tests, with no inference and no credentials in the candidate worker. */
import fs from "node:fs";
import path from "node:path";
import net from "node:net";
import type { HostClient } from "./codex.ts";
import { DISABLED_FEATURES } from "./codex.ts";
export interface CapabilityResult {
    version: 1;
    host_version: string;
    executable_sha256: string;
    live: boolean;
    ready: boolean;
    tests: {
        id: string;
        passed: boolean;
        reason?: string;
    }[];
    provider: string;
}
export async function verifyControls(client: HostClient, work: string): Promise<CapabilityResult> {
    const tests: CapabilityResult["tests"] = [];
    const config = (await client.request("config/read", { cwd: work, includeLayers: false })).config;
    if (!config || typeof config !== "object")
        throw new Error("host effective configuration unavailable");
    const provider = config.model_provider ?? "openai", settings = config.model_providers?.[provider];
    tests.push({ id: "native-tool-features-disabled", passed: DISABLED_FEATURES.every(k => config.features?.[k] === false) });
    tests.push({ id: "external-tools-disabled", passed: config.web_search === "disabled" && Object.values(config.mcp_servers ?? {}).every((v: any) => v.enabled === false) && Object.keys(config.hooks ?? {}).length === 0 });
    // Reserved built-in provider settings cannot be overridden in 0.163.0. Do not
    // silently rename/swap providers or claim their SDK retries are disabled.
    tests.push({ id: "provider-retries-disabled", passed: settings?.request_max_retries === 0 && settings?.stream_max_retries === 0, reason: "existing provider must expose verified zero request/stream retries; built-in defaults are not proof" });
    const profiles = await client.request("permissionProfile/list", { cwd: work });
    const profile = profiles.data?.find((p: any) => p.id === "kineti_harness");
    tests.push({ id: "restricted-profile-available", passed: !!profile && profile.allowed === true });
    if (!profile || profile.allowed !== true)
        return { version: 1, host_version: client.version, executable_sha256: client.executable_sha256, live: client.live, ready: false, tests, provider };
    fs.mkdirSync(work, { recursive: true, mode: 0o700 });
    const inside = path.join(work, "allowed-sentinel"), outside = path.join(path.dirname(work), "protected-sentinel");
    fs.writeFileSync(inside, "allowed", { mode: 0o600, flag: "wx" });
    fs.writeFileSync(outside, "unchanged", { mode: 0o600, flag: "wx" });
    const command = (argv: string[]) => client.request("command/exec", { command: argv, cwd: work, permissionProfile: "kineti_harness", timeoutMs: 3000, outputBytesCap: 4096 });
    try {
        const allowed = await command(["/usr/bin/cat", inside]);
        tests.push({ id: "allowed-native-read-control", passed: allowed.exitCode === 0 && allowed.stdout === "allowed", ...(allowed.exitCode !== 0 ? { reason: JSON.stringify({ exit_code: allowed.exitCode, stderr: allowed.stderr?.slice(0, 512) }) } : {}) });
        const denied = await command(["/usr/bin/cat", outside]);
        tests.push({ id: "outside-native-read-denied", passed: denied.exitCode !== 0 && !denied.stdout?.includes("unchanged") });
        const write = await command(["/usr/bin/dash", "-c", 'printf changed > "$1"', "sh", outside]);
        tests.push({ id: "outside-native-write-contained", passed: [0, 1, 2].includes(write.exitCode) && fs.readFileSync(outside, "utf8") === "unchanged", reason: JSON.stringify({ exit_code: write.exitCode, unchanged: fs.readFileSync(outside, "utf8") === "unchanged", stderr: write.stderr?.slice(0, 300) }) });
        const server = net.createServer(socket => { socket.destroy(); });
        let accepts = 0;
        server.on("connection", () => accepts++);
        await new Promise<void>((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", () => resolve()); });
        try {
            const address = server.address() as net.AddressInfo;
            const network = await command([path.join(work, "network-probe"), String(address.port)]);
            tests.push({ id: "native-network-denied", passed: network.exitCode === 0 && accepts === 0, reason: JSON.stringify({ exit_code: network.exitCode, accepts, stderr: network.stderr?.slice(0, 300) }) });
        }
        finally {
            server.close();
        }
    }
    catch (error) {
        tests.push({ id: "native-controls-executed", passed: false, reason: (error as Error).message });
    }
    return { version: 1, host_version: client.version, executable_sha256: client.executable_sha256, live: client.live, ready: tests.every(t => t.passed), tests, provider };
}
export async function inspectCapabilities(root: string) {
    const { CodexClient } = await import("./codex.ts"), crypto = await import("node:crypto");
    const id = crypto.randomUUID(), base = path.join(root, ".kineti/harness-capabilities", id), work = path.join(base, "host");
    fs.mkdirSync(base, { recursive: true, mode: 0o700 });
    const abort = new AbortController(), timer = setTimeout(() => abort.abort(), 30000);
    let client: HostClient | undefined;
    try {
        client = new CodexClient("codex", work, abort.signal);
        await client.request("initialize", { clientInfo: { name: "kineti-controls", version: "0.5.0" }, capabilities: { experimentalApi: true } });
        client.notify("initialized");
        const result = await verifyControls(client, work);
        const report = { id, at: new Date().toISOString(), inference_started: false, ...result };
        fs.writeFileSync(path.join(base, "report.json"), JSON.stringify(report, null, 2) + "\n", { mode: 0o600 });
        return report;
    }
    finally {
        clearTimeout(timer);
        client?.close();
    }
}
