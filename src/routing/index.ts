/** Single-agent routing. Local identity and host claims are not independent evaluator authority. */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { EventEmitter } from "node:events";
import { spawn, spawnSync } from "node:child_process";
import { loadLimits, sha256, writeJson, appendJsonl } from "../../bin/lib.ts";
import {
  fingerprint,
  toolchainVersions,
  issueEvidence,
  readEvidence,
  type EvidenceRecord,
} from "../../bin/kineti-evidence.ts";
import {
  readSpending,
  spendingStop,
  withAccountingLock,
} from "../../bin/spending-state.ts";

export type Host = "opencode" | "codex";
export type TaskClass = "code" | "plan" | "chat" | "fix";
const CLASSES: TaskClass[] = ["code", "plan", "chat", "fix"];
const UUID = /^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/;
const id = (value: unknown): value is string =>
  typeof value === "string" && UUID.test(value);
const object = (v: unknown): v is { [key: string]: any } =>
  !!v && typeof v === "object" && !Array.isArray(v);
const positive = (v: unknown): v is number =>
  typeof v === "number" && Number.isFinite(v) && v > 0;
export function hostName(v: unknown): Host {
  if (v !== "opencode" && v !== "codex")
    throw new Error("host must be opencode or codex");
  return v;
}
export function classify(text: string, defaultClass?: TaskClass) {
  if (!text.trim()) throw new Error("empty task");
  const rules: [TaskClass, RegExp][] = [
    ["fix", /\b(fix|repair|bug|broken|error|fail(?:ure|ing)?|crash|debug)\b/i],
    [
      "plan",
      /\b(plan|design|architect(?:ure)?|roadmap|spec(?:ification)?|strategy)\b/i,
    ],
    [
      "code",
      /\b(code|implement|build|write|refactor|function|class|api|endpoint)\b/i,
    ],
  ];
  const matches = rules
    .filter(([, rule]) => rule.test(text))
    .map(([kind]) => kind);
  const ambiguous = matches.length > 1;
  if (ambiguous && !defaultClass)
    return {
      rule: "keywords",
      version: 1,
      task_class: null,
      matches,
      basis:
        "multiple task classes; explicit default or clarification required",
      ambiguous,
    };
  return {
    rule: "keywords",
    version: 1,
    task_class: ambiguous
      ? defaultClass!
      : (matches[0] ?? ("chat" as TaskClass)),
    matches,
    basis: ambiguous
      ? "owner-configured ambiguity default"
      : matches.length
        ? "matched deterministic action words"
        : "nonempty conversation without action words",
    ambiguous,
  };
}
export interface AccessModel {
  id: string;
  provenance: ("user-declared" | "host-listed" | "live-call-confirmed")[];
  listed_at?: string;
  confirmed_at?: string;
  expires_at?: string;
  confirmation_run_id?: string;
  revoked?: boolean;
}
export interface AccessManifest {
  schema_version: 1;
  revision: string;
  host: Host;
  ttl_hours: number;
  models: AccessModel[];
}
export interface Policy {
  schema_version: 1;
  revision: string;
  host: Host;
  host_version: string;
  executable?: string;
  default_class?: TaskClass;
  per_run_cap_usd: number;
  max_wall_seconds: number;
  class_models: Record<TaskClass, string[]>;
  cost_estimates: Record<string, { usd: number; source: string }>;
  required_check: { label: string; command: string[] };
  destinations: string[];
}
export function validateManifest(value: unknown): AccessManifest {
  if (
    !object(value) ||
    Object.keys(value).some(
      (k) =>
        !["schema_version", "revision", "host", "ttl_hours", "models"].includes(
          k,
        ),
    ) ||
    value.schema_version !== 1 ||
    !id(value.revision)
  )
    throw new Error("invalid manifest version/revision or unknown fields");
  hostName(value.host);
  if (
    !positive(value.ttl_hours) ||
    value.ttl_hours > 720 ||
    !Array.isArray(value.models)
  )
    throw new Error("invalid manifest TTL/models");
  const seen = new Set<string>();
  for (const m of value.models) {
    if (
      !object(m) ||
      Object.keys(m).some(
        (k) =>
          ![
            "id",
            "provenance",
            "listed_at",
            "confirmed_at",
            "expires_at",
            "confirmation_run_id",
            "revoked",
          ].includes(k),
      ) ||
      typeof m.id !== "string" ||
      !m.id.trim() ||
      m.id.startsWith("-") ||
      m.id.includes("\0") ||
      seen.has(m.id) ||
      !Array.isArray(m.provenance) ||
      !m.provenance.length ||
      new Set(m.provenance).size !== m.provenance.length ||
      !m.provenance.every((p: unknown) =>
        ["user-declared", "host-listed", "live-call-confirmed"].includes(
          String(p),
        ),
      )
    )
      throw new Error("invalid or duplicate access model");
    seen.add(m.id);
    if (m.revoked !== undefined && typeof m.revoked !== "boolean")
      throw new Error("invalid revocation status");
    for (const key of ["listed_at", "confirmed_at", "expires_at"]) {
      if (
        m[key] !== undefined &&
        (!Number.isFinite(Date.parse(m[key])) ||
          new Date(m[key]).toISOString() !== m[key])
      )
        throw new Error("invalid access timestamp");
    }
    if (
      m.provenance.includes("live-call-confirmed") &&
      (!id(m.confirmation_run_id) ||
        !m.confirmed_at ||
        !m.expires_at ||
        Date.parse(m.expires_at) <= Date.parse(m.confirmed_at))
    )
      throw new Error(
        "live confirmation requires execution identity and valid dates",
      );
  }
  return value as AccessManifest;
}
export function validatePolicy(value: unknown): Policy {
  if (
    !object(value) ||
    Object.keys(value).some(
      (k) =>
        ![
          "schema_version",
          "revision",
          "host",
          "host_version",
          "executable",
          "default_class",
          "per_run_cap_usd",
          "max_wall_seconds",
          "class_models",
          "cost_estimates",
          "required_check",
          "destinations",
        ].includes(k),
    ) ||
    value.schema_version !== 1 ||
    !id(value.revision)
  )
    throw new Error("invalid policy version/revision or unknown fields");
  hostName(value.host);
  if (
    typeof value.host_version !== "string" ||
    !/^\d+\.\d+\.\d+/.test(value.host_version)
  )
    throw new Error("policy must pin an installed host version");
  if (
    !positive(value.per_run_cap_usd) ||
    !positive(value.max_wall_seconds) ||
    value.max_wall_seconds > 600
  )
    throw new Error("explicit positive cap and bounded wall time required");
  if (
    value.default_class !== undefined &&
    !CLASSES.includes(value.default_class)
  )
    throw new Error("invalid ambiguity default");
  if (!object(value.class_models) || !object(value.cost_estimates))
    throw new Error("model mapping and cost estimates required");
  for (const kind of CLASSES) {
    const models = value.class_models[kind];
    if (
      !Array.isArray(models) ||
      !models.every(
        (m: unknown) =>
          typeof m === "string" &&
          m.trim() &&
          !m.startsWith("-") &&
          !m.includes("\0"),
      ) ||
      new Set(models).size !== models.length
    )
      throw new Error(`invalid ${kind} preference order`);
  }
  for (const cost of Object.values(value.cost_estimates)) {
    if (
      !object(cost) ||
      !positive(cost.usd) ||
      Math.round(cost.usd * 1e6) < 1 ||
      typeof cost.source !== "string" ||
      !cost.source.trim()
    )
      throw new Error("unknown/zero cost cannot be an admission estimate");
  }
  if (
    !object(value.required_check) ||
    typeof value.required_check.label !== "string" ||
    !value.required_check.label.trim() ||
    !Array.isArray(value.required_check.command) ||
    !value.required_check.command.length ||
    !value.required_check.command.every(
      (part: unknown) => typeof part === "string" && !part.includes("\0"),
    )
  )
    throw new Error("one explicit required test label and command required");
  if (
    !Array.isArray(value.destinations) ||
    !value.destinations.length ||
    !value.destinations.every(
      (d: unknown) =>
        typeof d === "string" &&
        /^(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$/i.test(d),
    )
  )
    throw new Error("explicit network destination hostnames required");
  if (
    value.executable !== undefined &&
    (typeof value.executable !== "string" ||
      !value.executable.trim() ||
      value.executable.includes("\0"))
  )
    throw new Error("invalid host executable");
  return value as Policy;
}
export function routingPath(
  root: string,
  host: Host,
  kind: "manifest" | "policy",
) {
  return path.join(root, ".kineti/routing", `${host}.${kind}.json`);
}
export function loadConfig(root: string, host: Host) {
  return {
    manifest: validateManifest(
      JSON.parse(fs.readFileSync(routingPath(root, host, "manifest"), "utf8")),
    ),
    policy: validatePolicy(
      JSON.parse(fs.readFileSync(routingPath(root, host, "policy"), "utf8")),
    ),
  };
}
function validAccess(
  root: string,
  manifest: AccessManifest,
  model: AccessModel,
  now: number,
): string | null {
  if (model.revoked) return "revoked";
  if (!model.provenance.includes("live-call-confirmed"))
    return "access not live-confirmed";
  const at = Date.parse(model.confirmed_at!),
    until = Date.parse(model.expires_at!);
  if (
    at > now + 60_000 ||
    now >= Math.min(until, at + manifest.ttl_hours * 3_600_000)
  )
    return "access stale or clock invalid";
  const records = readEvidence(root).filter(
    (r) => r.run_id === model.confirmation_run_id,
  );
  if (records.length !== 1) return "missing or ambiguous access execution";
  const r = records[0],
    route = r.routing as any;
  if (
    r.receipt_version !== 3 ||
    !r.session_id ||
    r.exit_code !== 0 ||
    !route ||
    route.host !== manifest.host ||
    route.requested_model !== model.id ||
    route.execution?.status !== "completed" ||
    route.model_identity?.status === "mismatch"
  )
    return "access execution does not confirm this host/model";
  // Manifest claims alone cannot manufacture live confirmation, including locally edited records.
  const key = fs.readFileSync(
    path.join(root, ".kineti/evidence-issuer.pem"),
    "utf8",
  );
  if (!verifyExecutionSignature(r, key))
    return "invalid access execution signature";
  return null;
}
import { verifyReceiptSignature as verifyExecutionSignature } from "../../bin/kineti-evidence.ts";
export function planRoute(
  root: string,
  host: Host,
  text: string,
  manifest: AccessManifest,
  policy: Policy,
  probeModel?: string,
  now = Date.now(),
) {
  validateManifest(manifest);
  validatePolicy(policy);
  if (manifest.host !== host || policy.host !== host)
    throw new Error("foreign host manifest/policy");
  const classification = classify(text, policy.default_class);
  if (!classification.task_class)
    throw new Error(
      "ambiguous task; configure a default or clarify before execution",
    );
  const candidates = policy.class_models[classification.task_class];
  const rejected: { model: string; reason: string }[] = [];
  for (const model of candidates) {
    if (probeModel && model !== probeModel) continue;
    const access = manifest.models.find((m) => m.id === model);
    const reason = !access
      ? "not declared/listed"
      : access.revoked
        ? "revoked"
        : probeModel
          ? null
          : validAccess(root, manifest, access, now);
    if (reason) {
      rejected.push({ model, reason });
      continue;
    }
    const estimate = Object.hasOwn(policy.cost_estimates, model)
      ? policy.cost_estimates[model]
      : undefined;
    if (!estimate || estimate.usd > policy.per_run_cap_usd) {
      rejected.push({
        model,
        reason: "missing estimate or estimate exceeds run cap",
      });
      continue;
    }
    return {
      host,
      classification,
      requested_model: model,
      manifest_revision: manifest.revision,
      policy_revision: policy.revision,
      candidates,
      rejected,
      selection_source: "owner-preference-order",
      estimate,
      cap_usd: policy.per_run_cap_usd,
      probe: !!probeModel,
    };
  }
  throw new Error(
    `no permitted accessible model: ${JSON.stringify(rejected)}${probeModel ? " (probe must also be in the permitted order)" : ""}`,
  );
}
export function hostArguments(host: Host, model: string): string[] {
  return host === "codex"
    ? [
        "exec",
        "--model",
        model,
        "--json",
        "--sandbox",
        "read-only",
        "--ephemeral",
        "-",
      ]
    : ["run", "--model", model, "--format", "json"];
}
export interface HostObservation {
  session_id: string | null;
  completed: boolean;
  malformed: boolean;
  failed: boolean;
  model_identity: {
    status: "unverified" | "mismatch";
    requested: string;
    observed: string[];
    source: string;
    reason: string;
  };
  cost: { usd: number | null; source: string };
  usage: { input_tokens: number; output_tokens: number } | null;
}
export function observeHost(
  host: Host,
  requested: string,
  stdout: string,
): HostObservation {
  let malformed = false,
    completed = false,
    failed = false,
    hasResponse = false,
    cost: number | null = null,
    usage: HostObservation["usage"] = null;
  const sessions = new Set<string>(),
    models = new Set<string>();
  const lines = stdout.split("\n").filter((l) => l.trim());
  if (!stdout.endsWith("\n") || !lines.length) malformed = true;
  for (const line of lines) {
    let e: any;
    try {
      e = JSON.parse(line);
    } catch {
      malformed = true;
      continue;
    }
    if (!object(e) || typeof e.type !== "string") {
      malformed = true;
      continue;
    }
    if (host === "codex") {
      if (
        e.type === "thread.started" &&
        typeof e.thread_id === "string" &&
        e.thread_id.trim()
      )
        sessions.add(e.thread_id);
      if (e.type === "turn.completed") {
        completed = true;
        const u = e.usage;
        if (
          object(u) &&
          Number.isSafeInteger(u.input_tokens) &&
          u.input_tokens >= 0 &&
          Number.isSafeInteger(u.output_tokens) &&
          u.output_tokens >= 0
        )
          usage = {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
          };
      }
      if (
        e.type === "item.completed" &&
        e.item?.type === "agent_message" &&
        typeof e.item.text === "string" &&
        e.item.text.trim()
      )
        hasResponse = true;
      if (e.type === "turn.failed" || e.type === "error") failed = true;
    } else {
      if (typeof e.sessionID === "string" && e.sessionID.trim())
        sessions.add(e.sessionID);
      if (e.type === "step_finish" && e.part?.reason === "stop")
        completed = true;
      if (
        e.type === "text" &&
        typeof e.part?.text === "string" &&
        e.part.text.trim()
      )
        hasResponse = true;
      if (e.type === "error") failed = true;
      if (e.type === "step_finish" && e.part?.cost !== undefined) {
        if (
          typeof e.part.cost !== "number" ||
          !Number.isFinite(e.part.cost) ||
          e.part.cost < 0
        )
          malformed = true;
        else cost = (cost ?? 0) + e.part.cost;
      }
    }
    // A requested-model echo or candidate text is never resolved-model evidence.
    // Preserve response-level observations, but these CLI versions have no verified model channel.
    if (
      e.type === "response.completed" &&
      typeof e.response?.model === "string"
    )
      models.add(e.response.model);
  }
  if (sessions.size !== 1 || !hasResponse) malformed = true;
  const mismatch = [...models].some((model) => model !== requested);
  return {
    session_id: sessions.size === 1 ? [...sessions][0] : null,
    completed,
    malformed,
    failed,
    model_identity: {
      status: mismatch ? "mismatch" : "unverified",
      requested,
      observed: [...models],
      source: "CLI JSON events",
      reason: mismatch
        ? "conflicting response model observation"
        : "supported CLI events do not establish a verified resolved-model channel",
    },
    cost: {
      usd: cost,
      source:
        cost === null
          ? "unknown"
          : "host-calculated; provider invoice unverified",
    },
    usage,
  };
}
interface Reservation {
  session_id: string;
  stage: string;
  microcents: number;
  status: "reserved" | "unknown";
}
function reservations(s: any): Record<string, Reservation> {
  const values = s.routing_reservations ?? {};
  if (
    !object(values) ||
    !Object.entries(values).every(
      ([key, r]) =>
        id(key) &&
        object(r) &&
        id(r.session_id) &&
        typeof r.stage === "string" &&
        Number.isSafeInteger(r.microcents) &&
        r.microcents > 0 &&
        ["reserved", "unknown"].includes(r.status),
    )
  )
    throw new Error("corrupt routing reservations");
  return values;
}
function reserve(
  root: string,
  runId: string,
  sessionId: string,
  stage: string,
  estimate: number,
  cap: number,
) {
  withAccountingLock(root, () => {
    const s = readSpending(root),
      limits = loadLimits(root),
      pending = reservations(s),
      stop = spendingStop(
        { ...s, routing_reservations: undefined } as any,
        limits,
      );
    if (stop) throw new Error(stop);
    if (Object.values(pending).some((r) => r.status === "unknown"))
      throw new Error("unresolved cost blocks further launches");
    const micro = Math.round(estimate * 1e6);
    if (!Number.isSafeInteger(micro) || micro <= 0 || estimate > cap)
      throw new Error("invalid estimate or missing headroom");
    const outstanding = Object.values(pending).reduce(
        (n, r) => n + r.microcents,
        0,
      ),
      stageOutstanding = Object.values(pending)
        .filter((r) => r.stage === stage)
        .reduce((n, r) => n + r.microcents, 0);
    if (
      s.total_microcents + outstanding + micro >=
        Math.round(limits.globalUsd * limits.safetyFactor * 1e6) ||
      (s.by_stage_microcents[stage] ?? 0) + stageOutstanding + micro >=
        Math.round(
          (limits.perStage[stage] ?? limits.perStageDefaultUsd) *
            limits.safetyFactor *
            1e6,
        )
    )
      throw new Error("reserved work would exhaust total/stage headroom");
    pending[runId] = {
      session_id: sessionId,
      stage,
      microcents: micro,
      status: "reserved",
    };
    writeJson(path.join(root, ".kineti/spend.json"), {
      ...s,
      routing_reservations: pending,
    });
    appendJsonl(path.join(root, ".kineti/spend.log.jsonl"), {
      at: new Date().toISOString(),
      kind: "reservation",
      run_id: runId,
      session_id: sessionId,
      stage,
      microcents: micro,
      pricing_mode: "estimate",
    });
  });
}
export function settle(
  root: string,
  runId: string,
  usd: number,
  source: string,
) {
  if (
    !id(runId) ||
    typeof usd !== "number" ||
    !Number.isFinite(usd) ||
    usd < 0 ||
    !source.trim()
  )
    throw new Error(
      "settlement requires run UUID, nonnegative reported cost and source",
    );
  withAccountingLock(root, () => {
    const s = readSpending(root),
      pending = reservations(s),
      r = pending[runId];
    if (!r)
      throw new Error(
        "missing reservation; refusing double or foreign-run settlement",
      );
    const micro = Math.round(usd * 1e6);
    if (
      !Number.isSafeInteger(micro) ||
      !Number.isSafeInteger(s.total_microcents + micro)
    )
      throw new Error("unsafe cost range");
    s.total_microcents += micro;
    s.total_usd = s.total_microcents / 1e6;
    s.by_stage_microcents[r.stage] =
      (s.by_stage_microcents[r.stage] ?? 0) + micro;
    s.by_stage[r.stage] = s.by_stage_microcents[r.stage] / 1e6;
    s.entries++;
    delete pending[runId];
    const stop = spendingStop(
      { ...s, routing_reservations: undefined } as any,
      loadLimits(root),
    );
    if (stop) {
      s.tripped = true;
      s.reason = stop;
    }
    writeJson(path.join(root, ".kineti/spend.json"), {
      ...s,
      routing_reservations: pending,
    });
    appendJsonl(path.join(root, ".kineti/spend.log.jsonl"), {
      at: new Date().toISOString(),
      kind: "settlement",
      run_id: runId,
      session_id: r.session_id,
      stage: r.stage,
      microcents: micro,
      usd,
      pricing_mode: "reported",
      source,
      provider_invoice_verified: false,
    });
  });
}
function markUnknown(root: string, runId: string) {
  withAccountingLock(root, () => {
    const s = readSpending(root),
      pending = reservations(s);
    if (!pending[runId]) throw new Error("lost routing reservation");
    pending[runId].status = "unknown";
    writeJson(path.join(root, ".kineti/spend.json"), {
      ...s,
      routing_reservations: pending,
    });
  });
}
export async function runHost(
  executable: string,
  args: string[],
  input: string,
  root: string,
  seconds: number,
) {
  return await new Promise<{
    stdout: string;
    stdout_bytes: Buffer;
    stderr: string;
    exit_code: number | null;
    termination: string;
  }>((resolve) => {
    let stdout = "",
      stderr = "",
      byteLength = 0;
    const chunks: Buffer[] = [];
    let termination = "exited",
      settled = false;
    const child = spawn(executable, args, {
      cwd: root,
      env: process.env,
      stdio: ["pipe", "pipe", "pipe"],
      detached: process.platform !== "win32",
    });
    const kill = (reason: string) => {
      if (settled) return;
      termination = reason;
      try {
        if (process.platform !== "win32" && child.pid)
          process.kill(-child.pid, "SIGKILL");
        else child.kill("SIGKILL");
      } catch {}
    };
    const timer = setTimeout(() => kill("timeout"), seconds * 1000),
      interrupt = () => kill("interrupted");
    const signals: EventEmitter = process;
    signals.once("SIGINT", interrupt);
    signals.once("SIGTERM", interrupt);
    child.stdout.on("data", (chunk) => {
      chunks.push(Buffer.from(chunk));
      byteLength += chunk.length;
      if (byteLength > 8 * 1024 * 1024) kill("output-limit");
    });
    child.stderr.on("data", (chunk) => {
      if (stderr.length < 65536) stderr += chunk.toString();
    });
    child.on("error", () => {
      termination = "spawn-error";
    });
    child.stdin.on("error", () => {});
    child.stdin.end(input);
    child.on("close", (code, signal) => {
      settled = true;
      clearTimeout(timer);
      signals.off("SIGINT", interrupt);
      signals.off("SIGTERM", interrupt);
      const stdoutBytes = Buffer.concat(chunks);
      try {
        stdout = new TextDecoder("utf-8", { fatal: true }).decode(stdoutBytes);
      } catch {
        termination = "invalid-utf8";
      }
      resolve({
        stdout,
        stdout_bytes: stdoutBytes,
        stderr,
        exit_code: code,
        termination:
          signal && termination === "exited" ? `signal:${signal}` : termination,
      });
    });
  });
}
export async function execute(
  root: string,
  host: Host,
  text: string,
  manifest: AccessManifest,
  policy: Policy,
  options: { session_id?: string; probe_model?: string; consent: boolean },
) {
  if (!options.consent)
    throw new Error("explicit consent required for a live host call");
  const sessionId = options.session_id ?? crypto.randomUUID();
  if (!id(sessionId)) throw new Error("invalid session identity");
  const plan = planRoute(
    root,
    host,
    text,
    manifest,
    policy,
    options.probe_model,
  );
  const requested = policy.executable ?? host,
    exe =
      Bun.which(requested) ??
      (path.isAbsolute(requested) && fs.existsSync(requested)
        ? requested
        : null);
  if (!exe) throw new Error(`host not installed: ${host}`);
  const version = spawnSync(exe, ["--version"], {
    cwd: root,
    encoding: "utf8",
    timeout: 10_000,
  });
  if (
    version.status !== 0 ||
    version.stdout.match(/\d+\.\d+\.\d+/)?.[0] !== policy.host_version
  )
    throw new Error(
      "installed host version missing or differs from pinned policy",
    );
  const args = hostArguments(host, plan.requested_model),
    command = [fs.realpathSync(exe), ...args],
    exeDigest = sha256(fs.readFileSync(exe));
  const runId = crypto.randomUUID(),
    stage = String(
      JSON.parse(fs.readFileSync(path.join(root, ".kineti/state.json"), "utf8"))
        .stage ?? "routing",
    );
  if (
    !stage.trim() ||
    Object.hasOwn(Object.prototype, stage) ||
    stage === "prototype" ||
    stage.includes("\0")
  )
    throw new Error("invalid current stage");
  const before = fingerprint(root),
    manifestDigest = sha256(JSON.stringify(manifest)),
    policyDigest = sha256(JSON.stringify(policy));
  // Write the declared destinations before the host can send a request. Host-internal egress is not mediated here.
  for (const domain of policy.destinations) {
    const r = spawnSync(
      "bun",
      [
        path.resolve(import.meta.dir, "../../bin/kineti-egress.ts"),
        "record",
        "--host",
        domain,
        "--description",
        `Bounded ${host} execution ${runId}; task content omitted`,
      ],
      { cwd: root, encoding: "utf8" },
    );
    if (r.status !== 0) throw new Error("egress recording unavailable");
  }
  reserve(root, runId, sessionId, stage, plan.estimate.usd, plan.cap_usd);
  const result = await runHost(exe, args, text, root, policy.max_wall_seconds),
    observation = observeHost(host, plan.requested_model, result.stdout);
  const after = fingerprint(root),
    completed =
      result.exit_code === 0 &&
      result.termination === "exited" &&
      observation.completed &&
      !observation.malformed &&
      !observation.failed &&
      observation.model_identity.status !== "mismatch";
  const logs = path.join(root, ".kineti/routing/output");
  fs.mkdirSync(logs, { recursive: true, mode: 0o700 });
  fs.writeFileSync(path.join(logs, `${runId}.jsonl`), result.stdout_bytes, {
    mode: 0o600,
  });
  fs.writeFileSync(path.join(logs, `${runId}.stderr`), result.stderr, {
    mode: 0o600,
  });
  // Missing usage is never free. Reserve remains unresolved until explicitly reconciled.
  markUnknown(root, runId);
  if (observation.cost.usd !== null)
    settle(root, runId, observation.cost.usd, observation.cost.source);
  const routing = {
    events_sha256: sha256(result.stdout_bytes),
    stderr_sha256: sha256(result.stderr),
    event_log: `.kineti/routing/output/${runId}.jsonl`,
    required_check: policy.required_check,
    host_version: policy.host_version,
    executable_sha256: exeDigest,
    task_sha256: sha256(text),
    manifest_sha256: manifestDigest,
    policy_sha256: policyDigest,
    ...plan,
    execution: {
      status: completed ? "completed" : "failed",
      exit_code: result.exit_code,
      termination: result.termination,
      host_session_id: observation.session_id,
      malformed_events: observation.malformed,
    },
    model_identity: observation.model_identity,
    checks: { status: "missing", receipt_run_id: null },
    budget: {
      cap_usd: plan.cap_usd,
      estimate: plan.estimate,
      cost: observation.cost,
      status: observation.cost.usd === null ? "unknown" : "reported",
      provider_invoice_verified: false,
    },
    outcome: "not-accepted",
  };
  const receipt: EvidenceRecord = {
    receipt_version: 3,
    run_id: runId,
    session_id: sessionId,
    at: new Date().toISOString(),
    label: `route:${runId}`,
    cmd: command.join(" "),
    exit_code: result.exit_code,
    fingerprint: after,
    ...(before !== after ? { mutated_during_run: true } : {}),
    manifest: {
      command,
      dependencies: [],
      toolchain: toolchainVersions(),
      environment: {},
      input_digest: before,
      artifact_digests: [],
      parser_sha256: sha256(
        fs.readFileSync(
          path.resolve(import.meta.dir, "../../bin/kineti-evidence.ts"),
        ),
      ),
    },
    verification: {
      status: "unsupported",
      parser: "host-execution-v1",
      passed: 0,
      failed: completed ? 0 : 1,
      skipped: 0,
      inventory: [],
    },
    routing,
  };
  issueEvidence(receipt, root);
  if (options.probe_model && completed) {
    const live = validateManifest(manifest),
      row = live.models.find((m) => m.id === plan.requested_model)!;
    row.provenance = [
      ...new Set([...row.provenance, "live-call-confirmed" as const]),
    ];
    row.confirmed_at = receipt.at;
    row.expires_at = new Date(
      Date.parse(receipt.at) + live.ttl_hours * 3_600_000,
    ).toISOString();
    row.confirmation_run_id = runId;
    live.revision = crypto.randomUUID();
    writeJson(routingPath(root, host, "manifest"), live);
  }
  return { run_id: runId, session_id: sessionId, ...routing };
}
export function verifyRun(
  root: string,
  runId: string,
  checkRunId?: string,
  taskText?: string,
) {
  if (!id(runId) || (checkRunId !== undefined && !id(checkRunId)))
    throw new Error("invalid execution identity");
  const records = readEvidence(root),
    runs = records.filter((r) => r.run_id === runId);
  if (runs.length !== 1) throw new Error("missing or ambiguous execution");
  const r = runs[0],
    key = fs.readFileSync(
      path.join(root, ".kineti/evidence-issuer.pem"),
      "utf8",
    );
  if (!verifyExecutionSignature(r, key) || !r.routing)
    throw new Error("invalid signed execution");
  const route = r.routing as any,
    failures: string[] = [];
  if (r.receipt_version !== 3 || !id(r.session_id))
    throw new Error("missing signed execution/session identity");
  try {
    const events = fs.readFileSync(
      path.join(root, ".kineti/routing/output", `${runId}.jsonl`),
    );
    if (sha256(events) !== route.events_sha256)
      failures.push("host event evidence altered");
  } catch {
    failures.push("host event evidence missing");
  }

  if (
    !taskText ||
    sha256(taskText) !== route.task_sha256 ||
    JSON.stringify(
      classify(
        taskText,
        route.classification.ambiguous
          ? route.classification.task_class
          : undefined,
      ),
    ) !== JSON.stringify(route.classification)
  )
    failures.push("task input or classification does not match");
  if (route.execution?.status !== "completed")
    failures.push("execution did not complete");
  if (route.model_identity?.status !== "verified")
    failures.push("resolved model unverified or mismatched");
  let checks = "missing";
  if (checkRunId) {
    const matches = records.filter((c) => c.run_id === checkRunId);
    if (matches.length !== 1)
      throw new Error("missing or ambiguous test receipt");
    const c = matches[0];
    if (c.parent_run_id !== runId || c.session_id !== r.session_id)
      throw new Error("foreign execution/session test receipt");
    try {
      if (c.fingerprint !== r.fingerprint)
        throw new Error("test inputs differ from the executed workspace");
      if (!route.required_check || c.label !== route.required_check.label)
        throw new Error("test label does not match owner policy");
      validateCheck(c, {
        root,
        trustedPublicKey: key,
        expectedCommand: route.required_check.command,
      });
      checks = "accepted";
    } catch (error) {
      checks = "failed";
      failures.push((error as Error).message);
    }
  }
  if (checks !== "accepted") failures.push("required checks not accepted");
  const s = withAccountingLock(root, () => readSpending(root));
  const settlements = readSpendLog(root).filter(
    (v: any) =>
      v.kind === "settlement" &&
      v.run_id === runId &&
      v.session_id === r.session_id,
  );
  if (reservations(s)[runId] || settlements.length !== 1)
    failures.push("cost unresolved or spending join missing/ambiguous");
  if (settlements.some((v: any) => v.usd > route.cap_usd))
    failures.push("reported cost exceeds owner run cap");
  return {
    run_id: runId,
    session_id: r.session_id,
    execution: route.execution,
    model_identity: route.model_identity,
    checks,
    accepted: failures.length === 0,
    failures,
  };
}
import {
  validateReceipt as validateCheck,
  configuredCommand as configuredCheck,
} from "../../bin/kineti-evidence.ts";
function readSpendLog(root: string): any[] {
  const file = path.join(root, ".kineti/spend.log.jsonl");
  return fs.existsSync(file)
    ? fs
        .readFileSync(file, "utf8")
        .split("\n")
        .filter(Boolean)
        .map((l) => JSON.parse(l))
    : [];
}

/** Host-reported catalog; no listing is promoted to confirmed model access. */
export async function listHostModels(
  root: string,
  host: Host,
  executable: string = host,
): Promise<{ models: string[]; source: string; host_version: string }> {
  const version = spawnSync(executable, ["--version"], {
    cwd: root,
    encoding: "utf8",
    timeout: 10_000,
  });
  if (version.status !== 0)
    throw new Error("host installation/version unavailable");
  const hostVersion = version.stdout.match(/\d+\.\d+\.\d+/)?.[0];
  if (!hostVersion) throw new Error("unknown host version");
  if (host === "opencode") {
    const r = spawnSync(executable, ["models"], {
      cwd: root,
      encoding: "utf8",
      timeout: 20_000,
      maxBuffer: 2 * 1024 * 1024,
    });
    if (r.status !== 0) throw new Error("OpenCode catalog unavailable");
    const models = r.stdout
      .split("\n")
      .map((v) => v.trim())
      .filter((v) => /^[a-z0-9._-]+\/[^\s]+$/i.test(v));
    if (!models.length) throw new Error("empty OpenCode host catalog");
    return {
      models: [...new Set(models)],
      source: "opencode models",
      host_version: hostVersion,
    };
  }
  return await new Promise((resolve, reject) => {
    const child = spawn(executable, ["app-server", "--listen", "stdio://"], {
      cwd: root,
      stdio: ["pipe", "pipe", "pipe"],
    });
    let buffer = "",
      done = false,
      stderr = "";
    const finish = (error?: Error, models?: string[]) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      child.kill();
      if (error) reject(error);
      else
        resolve({
          models: models!,
          source: "codex app-server model/list",
          host_version: hostVersion,
        });
    };
    const timer = setTimeout(
      () =>
        finish(new Error("Codex catalog timed out; no model call attempted")),
      10_000,
    );
    const send = (v: any) =>
      child.stdin.write(JSON.stringify({ jsonrpc: "2.0", ...v }) + "\n");
    child.stdin.on("error", () => {});
    child.on("error", () => finish(new Error("Codex app-server unavailable")));
    child.stderr.on("data", (chunk) => {
      if (stderr.length < 65536) stderr += chunk.toString();
    });
    child.stdout.on("data", (chunk) => {
      buffer += chunk.toString();
      if (buffer.length > 2 * 1024 * 1024) {
        finish(new Error("oversize catalog response"));
        return;
      }
      let end;
      while ((end = buffer.indexOf("\n")) >= 0) {
        const line = buffer.slice(0, end);
        buffer = buffer.slice(end + 1);
        let v: any;
        try {
          v = JSON.parse(line);
        } catch {
          finish(new Error("malformed catalog event"));
          return;
        }
        if (v.id === 1) {
          if (v.error) {
            finish(new Error("Codex initialization rejected"));
            return;
          }
          send({ method: "initialized" });
          send({
            id: 2,
            method: "model/list",
            params: { limit: 100, includeHidden: false },
          });
        }
        if (v.id === 2) {
          if (v.error || !Array.isArray(v.result?.data)) {
            finish(new Error("Codex model/list unavailable"));
            return;
          }
          if (v.result.nextCursor) {
            finish(
              new Error(
                "catalog pagination requires an explicit complete import; partial catalog rejected",
              ),
            );
            return;
          }
          const models = v.result.data
            .map((r: any) => r.model ?? r.id)
            .filter((m: any) => typeof m === "string" && m.trim());
          if (!models.length) {
            finish(new Error("empty Codex catalog"));
            return;
          }
          finish(undefined, [...new Set<string>(models)]);
        }
      }
    });
    child.on("close", () =>
      finish(
        new Error(
          /read.only|permission denied/i.test(stderr)
            ? "Codex catalog blocked by read-only runtime state"
            : "Codex app-server exited before model/list",
        ),
      ),
    );
    send({
      id: 1,
      method: "initialize",
      params: {
        clientInfo: { name: "kineti", version: "0.5.0" },
        capabilities: { experimentalApi: true },
      },
    });
  });
}
