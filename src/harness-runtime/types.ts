/** Owned task execution. Host claims, test acceptance and billing remain separate. */
export const BASE_COMMIT = "88b518d951a30614450c6fec22b9ed5195776a17";
export const TASK_ID = "spend-cli-options-v1";
export const EDITABLE = "bin/kineti-spend.ts";
export const READABLE = [EDITABLE, "bin/lib.ts", "bin/ledger-integrity.ts"] as const;
export const DEFAULT_LIMITS = { wall_ms: 180000, tool_calls: 12, tool_bytes: 65536, corrections: 1 } as const;
export type RunStatus = "preflight" | "running" | "checking" | "passed" | "failed" | "blocked" | "interrupted" | "unknown";
export interface Task {
    version: 1;
    task_id: typeof TASK_ID;
    instruction: string;
}
export interface CheckResult {
    version: 1;
    check_id: string;
    artifact_sha256: string;
    passed: boolean;
    cases: {
        id: string;
        passed: boolean;
        reason?: string;
    }[];
    duration_ms: number;
    observations?: {
        arguments: string[];
        exit_code: number | null;
        stdout: string;
        stderr: string;
        output_sha256: string;
    }[];
}
export interface RunRecord {
    version: 1;
    run_id: string;
    session_id: string;
    parent_run_id?: string;
    at: string;
    updated_at: string;
    host: "codex";
    requested_model: string;
    host_reported_model: string | null;
    provider_resolved_model: string | null;
    model_identity: "unverified" | "mismatch";
    required_tests?: "not-run" | "passed" | "failed";
    status: RunStatus;
    functional_pass: boolean;
    live_proof: boolean;
    task: Task;
    base_commit: string;
    artifact_sha256: string | null;
    patch_sha256: string | null;
    host_thread_id: string | null;
    host_turn_ids: string[];
    tool_calls: number;
    checks: CheckResult[];
    budget: {
        cap_usd: number;
        estimate_usd: number | null;
        status: "not-launched" | "reserved" | "unknown" | "reported";
        usage_estimate_usd: number | null;
        provider_invoice_verified: false;
    };
    limits: typeof DEFAULT_LIMITS;
    reasons: string[];
    receipt_run_id?: string;
    controller_pid?: number;
    controller_identity?: string;
    command?: string[];
    host_version?: string;
    executable_sha256?: string;
    journal_checkpoint?: {
        events: number;
        head_sha256: string;
    };
    bindings?: {
        task_sha256: string;
        policy_sha256: string;
        baseline_manifest_sha256: string;
        checker_sha256: string;
        worker_image: string;
        readable: readonly string[];
    };
}
export const sha = (value: Uint8Array | string) => new Bun.CryptoHasher("sha256").update(value).digest("hex");
export function uuid(value: unknown): value is string { return typeof value === "string" && /^[a-f0-9]{8}(-[a-f0-9]{4}){3}-[a-f0-9]{12}$/.test(value); }
export function exactObject(v: unknown, keys: string[]): asserts v is Record<string, any> {
    if (!v || typeof v !== "object" || Array.isArray(v) || Object.keys(v).some(k => !keys.includes(k)))
        throw new Error("unknown fields or malformed object");
}
export function validateTask(v: unknown): Task {
    exactObject(v, ["version", "task_id", "instruction"]);
    if (v.version !== 1 || v.task_id !== TASK_ID || typeof v.instruction !== "string" || !v.instruction.trim() || Buffer.byteLength(v.instruction) > 65536)
        throw new Error("unsupported task or invalid instruction");
    return v as Task;
}
