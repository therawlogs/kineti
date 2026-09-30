import path from "node:path";
import os from "node:os";
import fs from "node:fs";
import crypto from "node:crypto";

export function machineDir(): string {
  if (process.env.KINETI_MACHINE_DIR) return process.env.KINETI_MACHINE_DIR;
  const homeKineti = path.join(os.homedir(), ".kineti");
  try {
    ensureDir(homeKineti);
    fs.accessSync(homeKineti, fs.constants.W_OK);
    return homeKineti;
  } catch {
    const fallback = path.join(process.cwd(), ".kineti");
    ensureDir(fallback);
    return fallback;
  }
}

export function projectKdir(): string {
  return path.join(process.cwd(), ".kineti");
}

export function ensureDir(p: string): void {
  fs.mkdirSync(p, { recursive: true });
}

export function assertWithinProject(targetPath: string, baseDir: string = process.cwd()): string {
  const resolved = path.resolve(baseDir, targetPath);
  const normalizedBase = path.resolve(baseDir);
  if (!resolved.startsWith(normalizedBase + path.sep) && resolved !== normalizedBase) {
    die(`security violation: path '${targetPath}' resolves outside project root '${baseDir}'`, 2);
  }
  return resolved;
}

export function nowIso(): string {
  return new Date().toISOString();
}

export function sha256(text: string | Buffer): string {
  return crypto.createHash("sha256").update(text).digest("hex");
}

export function die(msg: string, code = 1): never {
  console.error(`kineti: ${msg}`);
  process.exit(code);
}

export function ok(msg: string): void {
  console.log(msg);
}

export function readJson<T>(file: string): T | null {
  try {
    return JSON.parse(fs.readFileSync(file, "utf8")) as T;
  } catch {
    return null;
  }
}

export function writeJson(file: string, value: unknown): void {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { mode: 0o600 });
  lockFileMode(file);
}

/**
 * Governance state can contain tokens, spend ledgers, and undo commands.
 * Re-assert owner-only permissions on every write so files created before
 * this helper existed cannot stay world-readable.
 */
function lockFileMode(file: string): void {
  try {
    fs.chmodSync(file, 0o600);
  } catch {
    // Filesystems without POSIX modes (e.g. Windows) ignore this.
  }
}

export function readJsonl<T>(file: string): T[] {
  if (!fs.existsSync(file)) return [];
  try {
    const text = fs.readFileSync(file, "utf8");
    const lines = text.split("\n").filter((l) => l.trim().length > 0);
    const items: T[] = [];
    for (let i = 0; i < lines.length; i++) {
      try {
        items.push(JSON.parse(lines[i]) as T);
      } catch (err) {
        console.error(`kineti: warning: corrupt line ${i + 1} in ${file} skipped: ${(err as Error).message}`);
      }
    }
    return items;
  } catch (err) {
    console.error(`kineti: error reading ${file}: ${(err as Error).message}`);
    return [];
  }
}

export function appendJsonl(file: string, value: unknown): void {
  ensureDir(path.dirname(file));
  fs.appendFileSync(file, JSON.stringify(value) + "\n", { mode: 0o600 });
  lockFileMode(file);
}

export interface Limits {
  globalUsd: number;
  perStageDefaultUsd: number;
  perStage: Record<string, number>;
  safetyFactor: number;
}

/** Per-project ceiling bounds. Only the human sets it, every change is audit logged. */
export const MIN_PROJECT_CEILING_USD = 1;
export const MAX_PROJECT_CEILING_USD = 1000;
export const DEFAULT_PROJECT_CEILING_USD = 50;

export function loadLimits(cwd: string = process.cwd()): Limits {
  warnOnUnknownConfigKeys(cwd);
  const cfg = readJson<any>(path.join(cwd, "kineti.config.json"));
  const s = cfg?.settings?.spend_limit_usd ?? {};
  // Clamp every limit to a positive-finite number, else fall back to defaults.
  // Bad config must never disable the breaker (fail closed).
  const rawGlobal = Number(s.global ?? 50);
  let globalUsd = Number.isFinite(rawGlobal) && rawGlobal > 0 ? rawGlobal : 50;
  // Per-project ceiling set at mirror time overrides the repo default.
  // Invalid or missing mirror ceiling falls back to the config value (fail closed).
  const mirror = readJson<{ ceiling?: unknown }>(path.join(cwd, ".kineti", "mirror.json"));
  const rawCeiling = Number(mirror?.ceiling);
  if (
    Number.isFinite(rawCeiling) &&
    rawCeiling >= MIN_PROJECT_CEILING_USD &&
    rawCeiling <= MAX_PROJECT_CEILING_USD
  ) {
    globalUsd = rawCeiling;
  }
  const rawStageDefault = Number(s.per_stage_default ?? 10);
  const perStageDefaultUsd = Number.isFinite(rawStageDefault) && rawStageDefault > 0 ? rawStageDefault : 10;
  const perStage: Record<string, number> = {};
  const rawPerStage = s.per_stage;
  if (rawPerStage && typeof rawPerStage === "object" && !Array.isArray(rawPerStage)) {
    for (const [k, v] of Object.entries(rawPerStage)) {
      const n = Number(v);
      if (Number.isFinite(n) && n > 0) perStage[k] = n;
      // Non-finite or non-positive per-stage caps are dropped (falls back to default).
    }
  }
  const rawSafety = Number(s.safety_factor ?? 0.95);
  const safetyFactor = Number.isFinite(rawSafety) && rawSafety > 0 && rawSafety <= 1 ? rawSafety : 0.95;
  return { globalUsd, perStageDefaultUsd, perStage, safetyFactor };
}

export function loadVerifyCommand(cwd: string = process.cwd()): string | null {
  warnOnUnknownConfigKeys(cwd);
  const envCmd = process.env.KINETI_VERIFY_CMD;
  if (envCmd && envCmd.trim().length > 0) return envCmd.trim();
  const cfg = readJson<any>(path.join(cwd, "kineti.config.json"));
  const cmd = cfg?.settings?.verify_command;
  return typeof cmd === "string" && cmd.trim().length > 0 ? cmd.trim() : null;
}

const KNOWN_CONFIG_TOP_KEYS = new Set([
  "version", "name", "description", "root_goal", "pipeline",
  "settings", "memory", "asset_paths", "hosts",
]);

const KNOWN_CONFIG_SETTINGS_KEYS = new Set([
  "spend_limit_usd", "qa_max_self_repair_attempts", "data_quality_threshold",
  "hurdle_rate_min", "verify_command", "ci_stage", "freeze_during",
]);

const warnedConfigKeys = new Set<string>();

/**
 * Warn about unrecognized kineti.config.json keys (typos fail silently
 * otherwise). Lists unknown top-level keys and unknown settings.* keys.
 * Never throws and never exits.
 */
export function warnOnUnknownConfigKeys(cwd: string = process.cwd()): void {
  try {
    const cfg = readJson<any>(path.join(cwd, "kineti.config.json"));
    if (!cfg || typeof cfg !== "object" || Array.isArray(cfg)) return;
    const unknownTop = Object.keys(cfg).filter((k) => !KNOWN_CONFIG_TOP_KEYS.has(k));
    const s = (cfg as any).settings;
    const unknownSettings =
      s && typeof s === "object" && !Array.isArray(s)
        ? Object.keys(s).filter((k) => !KNOWN_CONFIG_SETTINGS_KEYS.has(k))
        : [];
    if (unknownTop.length === 0 && unknownSettings.length === 0) return;
    const dedupe = JSON.stringify([path.resolve(cwd), unknownTop, unknownSettings]);
    if (warnedConfigKeys.has(dedupe)) return;
    warnedConfigKeys.add(dedupe);
    const parts: string[] = [];
    if (unknownTop.length > 0) parts.push(`unknown top-level keys: ${unknownTop.join(", ")}`);
    if (unknownSettings.length > 0) parts.push(`unknown settings keys: ${unknownSettings.join(", ")}`);
    console.warn(`kineti: warning: unrecognized kineti.config.json keys (${parts.join("; ")}); check spelling`);
  } catch {
    // Warning must never break config loading.
  }
}

/**
 * Strict subcommand allowlist for model-supplied evidence commands.
 * The base binary allowlist permits package runners, but those can fetch
 * and run remote code (bun x, npx <pkg>), so evidence recording restricts
 * them to local test subcommands only. Returns a deny reason when refused.
 */
export function isAllowedEvidenceCommand(
  argv: string[],
  verifyCmd: string | null = null,
): { allowed: boolean; reason: string } {
  if (argv.length === 0) return { allowed: false, reason: "empty command" };
  if (hasShellMetachars(argv)) {
    return { allowed: false, reason: `shell metachars detected in: ${argv.join(" ")}` };
  }
  const bin = argv[0].split("/").pop() || argv[0];
  const rest = argv.slice(1);
  let verifyBin: string | null = null;
  try {
    const t = (verifyCmd || "").trim();
    if (t) verifyBin = t.split(/\s+/)[0]?.split("/").pop() || null;
  } catch {
    verifyBin = null;
  }
  switch (bin) {
    case "bun":
      if (rest[0] === "test" || rest[0] === "run") {
        return { allowed: true, reason: "bun test/run" };
      }
      return { allowed: false, reason: "bun only allows `test` and `run` subcommands (never `bun x`)" };
    case "pytest":
      return { allowed: true, reason: "pytest" };
    case "python":
    case "python3":
      if (rest[0] === "-m" && String(rest[1] || "").startsWith("pip")) {
        return { allowed: false, reason: "python -m pip is not allowed" };
      }
      if (rest[0] === "-c") {
        return { allowed: false, reason: "python -c is not allowed" };
      }
      return { allowed: true, reason: "python script" };
    case "npm":
      if (rest[0] === "test") return { allowed: true, reason: "npm test" };
      if (rest[0] === "exec" || rest[0] === "dlx") {
        return { allowed: false, reason: `npm ${rest[0]} is not allowed` };
      }
      return { allowed: false, reason: "npm only allows the `test` subcommand" };
    case "node": {
      const target = rest[0];
      if (!target || target.startsWith("-")) {
        return { allowed: false, reason: "node only allows a repo-relative .js/.ts file" };
      }
      if (!/\.[cm]?[jt]s$/.test(target)) {
        return { allowed: false, reason: "node only allows repo-relative .js/.ts files" };
      }
      if (path.isAbsolute(target) || target.split("/").includes("..")) {
        return { allowed: false, reason: "node target must stay inside the project" };
      }
      return { allowed: true, reason: "node script" };
    }
    case "npx":
      return { allowed: false, reason: "npx is not allowed (it fetches remote packages)" };
    case "echo":
    case "true":
    case "false":
      return { allowed: true, reason: `${bin} (no side effects)` };
    default:
      if (verifyBin && bin === verifyBin) {
        return { allowed: true, reason: "project verify command binary" };
      }
      return { allowed: false, reason: `binary not in evidence allowlist: ${bin}` };
  }
}

/**
 * Length-prefixed and null-byte delimited hash to eliminate second-preimage
 * delimiter collisions (RFC 8785 and cryptographic domain separation).
 */
export function computeDelimitedHash(parts: (string | Buffer)[]): string {
  const bufs: Buffer[] = [];
  for (const p of parts) {
    const b = Buffer.isBuffer(p) ? p : Buffer.from(String(p), "utf8");
    const lenBuf = Buffer.alloc(4);
    lenBuf.writeUInt32BE(b.length, 0);
    bufs.push(lenBuf, b, Buffer.from("\x00", "utf8"));
  }
  return crypto.createHash("sha256").update(Buffer.concat(bufs)).digest("hex");
}

export function usdToMicrocents(usd: number): number {
  return Math.round(usd * 1_000_000);
}

export function microcentsToUsd(microcents: number): number {
  return Math.round((microcents / 1_000_000) * 1e4) / 1e4;
}

/**
 * Phase 0.1 — Safe execution without shell.
 * Plain words: run commands directly, no shell, unless human allows it.
 */

export const SAFE_EXEC_ALLOWLIST_BINARIES = new Set([
  "cargo", "bun", "npm", "npx", "pytest", "python", "python3", "node",
  "true", "false", "echo", "sleep", "git", "rm", "touch", "ls", "cat",
]);

const SHELL_META_CHARS = [";", "&", "|", ">", "<", "$", "`", "\\", "'", '"', "*", "?", "~", "#", "(", ")", "{", "}", "[", "]", "!", "\n", "\r"];

export function hasShellMetachars(argv: string[]): boolean {
  const joined = argv.join(" ");
  for (const c of SHELL_META_CHARS) {
    if (joined.includes(c)) return true;
  }
  return false;
}

export function isAllowlistedExec(argv: string[], verifyCmd: string | null = null): boolean {
  if (argv.length === 0) return false;
  const bin = argv[0].split("/").pop() || argv[0];
  if (!SAFE_EXEC_ALLOWLIST_BINARIES.has(bin)) {
    // Allow project verify command binary as well
    if (verifyCmd) {
      const vBin = verifyCmd.trim().split(/\s+/)[0]?.split("/").pop();
      if (vBin && bin === vBin) return true;
    }
    return false;
  }
  // bun: allow test, run, x, --version etc. Block bun -e with shell chars unless explicitly allowed.
  // For now allow bun, npm, pytest family; metachar check happens separately.
  return true;
}

export interface SafeExecResult {
  exitCode: number;
  stdout: string;
  stderr: string;
  blocked?: boolean;
  reason?: string;
}

export function jailCwdToWorkspace(cwd: string, workspaceRoot: string): string {
  const resolvedRoot = path.resolve(workspaceRoot);
  const resolvedCwd = path.resolve(cwd);
  // cwd must be inside workspace root. If not, force to root.
  if (resolvedCwd !== resolvedRoot && !resolvedCwd.startsWith(resolvedRoot + path.sep)) {
    return resolvedRoot;
  }
  return resolvedCwd;
}

export function assertInsideProject(targetPath: string, rootDir: string = process.cwd()): string {
  const resolvedRoot = path.resolve(rootDir);
  const resolvedTarget = path.resolve(targetPath);
  if (resolvedTarget !== resolvedRoot && !resolvedTarget.startsWith(resolvedRoot + path.sep)) {
    die(`Path traversal rejected: '${targetPath}' is outside project root '${resolvedRoot}'`, 2);
  }
  return resolvedTarget;
}

function appendExecToEgressLedger(argv: string[], cwd: string): void {
  try {
    const mDir = machineDir();
    const ledgerFile = path.join(mDir, "egress.jsonl");
    const stateFile = path.join(mDir, "egress.state.json");
    let chain: any[] = [];
    try {
      if (fs.existsSync(ledgerFile)) {
        const text = fs.readFileSync(ledgerFile, "utf8");
        chain = text.split("\n").filter((l) => l.trim().length > 0).map((l) => JSON.parse(l));
      }
    } catch { chain = []; }
    const prev = chain.length > 0 ? chain[chain.length - 1].hash : "GENESIS";
    const at = nowIso();
    const host = `local-exec:${argv[0]?.split("/").pop() || "unknown"}`;
    const description = `exec: ${argv.join(" ")} in ${cwd}`.slice(0, 500);
    const seq = chain.length;
    const hash = computeDelimitedHash([String(seq), at, host, description, prev]);
    const receipt = { seq, at, host, description, prev_hash: prev, hash };
    ensureDir(mDir);
    fs.appendFileSync(ledgerFile, JSON.stringify(receipt) + "\n");
    try {
      fs.writeFileSync(stateFile, JSON.stringify({ count: chain.length + 1, last_hash: hash }));
    } catch (writeErr) {
      console.warn(`kineti: warning: failed to update ledger state file: ${(writeErr as Error).message}`);
    }
  } catch (appendErr) {
    // Ledger write must never break exec, but log warning
    console.warn(`kineti: warning: failed to append to egress ledger: ${(appendErr as Error).message}`);
  }
}

/**
 * Run a command array directly with no shell.
 * - Blocks shell metachars unless allowShell + TTY.
 * - Blocks non-allowlisted binaries unless allowShell + TTY.
 * - 60s timeout, cwd jailed to workspaceRoot.
 * - Every run is appended to the egress ledger.
 */
export function runSafeCommand(
  argv: string[],
  opts: { cwd?: string; workspaceRoot?: string; timeoutMs?: number; allowShell?: boolean; verifyCmd?: string | null } = {},
): SafeExecResult {
  const workspaceRoot = opts.workspaceRoot ? path.resolve(opts.workspaceRoot) : process.cwd();
  const cwd = jailCwdToWorkspace(opts.cwd || workspaceRoot, workspaceRoot);
  const timeoutMs = opts.timeoutMs ?? 60000;
  const allowShell = opts.allowShell ?? false;

  if (argv.length === 0) {
    return { exitCode: 2, stdout: "", stderr: "kineti: blocked: empty command", blocked: true, reason: "empty" };
  }

  const meta = hasShellMetachars(argv);
  const allowlisted = isAllowlistedExec(argv, opts.verifyCmd ?? null);

  if ((meta || !allowlisted) && !allowShell) {
    const reason = meta ? `shell metachars detected in: ${argv.join(" ")}` : `binary not in allowlist: ${argv[0]}`;
    return { exitCode: 2, stdout: "", stderr: `kineti: blocked: ${reason}. Use --allow-shell in a human TTY to run it.`, blocked: true, reason };
  }

  if (allowShell) {
    const isTTY = !!process.stdin.isTTY;
    if (!isTTY) {
      return { exitCode: 2, stdout: "", stderr: "kineti: blocked: --allow-shell needs a human TTY. Refused.", blocked: true, reason: "no-tty" };
    }
  }

  // Log before run
  appendExecToEgressLedger(argv, cwd);

  try {
    // Use node spawnSync with no shell, timeout 60s.
    // Dynamic import to avoid top-level cycle.
    const { spawnSync } = require("node:child_process") as typeof import("node:child_process");
    const res = spawnSync(argv[0], argv.slice(1), {
      cwd,
      encoding: "utf8",
      timeout: timeoutMs,
      killSignal: "SIGKILL",
      shell: false,
    } as any);
    let exitCode = res.status ?? 1;
    let stderr = res.stderr?.toString() ?? "";
    const stdout = res.stdout?.toString() ?? "";
    // Timeout detection: node sets error with ETIMEDOUT and status null
    if ((res as any).error && String((res as any).error).includes("ETIMEDOUT")) {
      stderr = (stderr ? stderr + "\n" : "") + `kineti: timeout after ${timeoutMs}ms`;
      exitCode = 124;
    }
    if (res.signal) {
      stderr = (stderr ? stderr + "\n" : "") + `kineti: killed by ${res.signal}`;
      if (exitCode === 0) exitCode = 124;
    }
    return { exitCode, stdout, stderr };
  } catch (err: any) {
    return { exitCode: 1, stdout: "", stderr: `kineti: exec failed: ${err?.message || err}` };
  }
}

/**
 * Split a legacy single-string command into argv without shell.
 * Quote-aware: single quotes, double quotes, and backslash escapes group
 * words that contain spaces (for example file paths). Quotes are consumed,
 * so the result has no quote characters for the metachar check to trip on.
 * Unquoted input splits on whitespace exactly as before.
 */
export function splitLegacyCommand(cmd: string): string[] {
  const trimmed = cmd.trim();
  if (!trimmed) return [];
  const out: string[] = [];
  let cur = "";
  let hasToken = false;
  let inSingle = false;
  let inDouble = false;
  for (let i = 0; i < trimmed.length; i++) {
    const c = trimmed[i];
    if (inSingle) {
      if (c === "'") inSingle = false;
      else cur += c;
      continue;
    }
    if (inDouble) {
      if (c === '"') inDouble = false;
      else if (c === "\\" && i + 1 < trimmed.length) { cur += trimmed[i + 1]; i++; }
      else cur += c;
      continue;
    }
    if (c === "'") { inSingle = true; hasToken = true; }
    else if (c === '"') { inDouble = true; hasToken = true; }
    else if (c === "\\" && i + 1 < trimmed.length) { cur += trimmed[i + 1]; i++; hasToken = true; }
    else if (/\s/.test(c)) {
      if (hasToken) { out.push(cur); cur = ""; hasToken = false; }
    } else { cur += c; hasToken = true; }
  }
  if (hasToken) out.push(cur);
  return out;
}

/**
 * Scaffolds zero-touch root governance files for Claude Code, Antigravity,
 * Cursor, and OpenAI Codex into a project repository root.
 */
export function scaffoldRootHooks(targetDir: string = process.cwd()): string[] {
  const installed: string[] = [];

  // 1. CLAUDE.md
  const claudePath = path.join(targetDir, "CLAUDE.md");
  if (!fs.existsSync(claudePath)) {
    fs.writeFileSync(
      claudePath,
      `# Kineti OS — Rules for Claude Code\n\nThis project uses Kineti OS to keep work safe and organized.\n\n## Rules\n1. Plain words and numbered choices: Speak in plain English without metaphors or jargon. Give numbered options (1, 2, 3).\n2. Check the current task: Read .kineti/state.json before starting.\n3. Plan before building new features: Get user approval before writing code in src/.\n4. Run tests: Save test proofs with bun bin/kineti-evidence.ts run.\n5. Spending limit: Stop right away if spending reaches $50.00.\n6. Save undo steps: Save an undo command before making changes.\n`
    );
    installed.push("CLAUDE.md");
  }

  // 2. AGENTS.md
  const agentsPath = path.join(targetDir, "AGENTS.md");
  if (!fs.existsSync(agentsPath)) {
    fs.writeFileSync(
      agentsPath,
      `# Kineti OS — Rules for AI Agents\n\nThis project uses Kineti OS to keep work safe and organized.\n\n## Rules\n1. Plain words and numbered choices: Speak in simple English without metaphors.\n2. Check the current task: Read .kineti/state.json before starting.\n3. Plan before building new features: Get approval before writing code in src/.\n4. Run tests: Save test proofs before claiming work is finished.\n5. Spending limit: Stop right away if spending reaches $50.00.\n6. Save undo steps: Save undo commands before making changes.\n`
    );
    installed.push("AGENTS.md");
  }

  // 3. .cursor/rules/kineti.mdc
  const cursorRulesDir = path.join(targetDir, ".cursor", "rules");
  const cursorRulePath = path.join(cursorRulesDir, "kineti.mdc");
  if (!fs.existsSync(cursorRulePath)) {
    ensureDir(cursorRulesDir);
    fs.writeFileSync(
      cursorRulePath,
      `---\ndescription: Kineti OS Safety Rules\nglobs: *\nalwaysApply: true\n---\n\n# Kineti OS — Rules for Cursor\n\n- Plain words and numbered choices: Speak in simple English without metaphors.\n- Check .kineti/state.json to see the current goal and task.\n- For new features, get user approval before writing code in src/.\n- Stop right away if spending reaches $50.00.\n`
    );
    installed.push(".cursor/rules/kineti.mdc");
  }

  // 4. CODEX.md
  const codexPath = path.join(targetDir, "CODEX.md");
  if (!fs.existsSync(codexPath)) {
    fs.writeFileSync(
      codexPath,
      `# Kineti OS — Rules for OpenAI Codex\n\nThis project uses Kineti OS to keep work safe and organized.\n\n1. Plain words and numbered choices (1, 2, 3).\n2. Read .kineti/state.json to see the current task.\n3. For new features, get approval before writing code in src/.\n4. Stop if spending reaches $50.00.\n5. Run and check tests.\n`
    );
    installed.push("CODEX.md");
  }

  return installed;
}

