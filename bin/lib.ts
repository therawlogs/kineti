import path from "node:path";
import os from "node:os";
import fs from "node:fs";
import crypto from "node:crypto";

export function machineDir(): string {
  return process.env.KINETI_MACHINE_DIR || path.join(os.homedir(), ".kineti");
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
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
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
  fs.appendFileSync(file, JSON.stringify(value) + "\n");
}

export interface Limits {
  globalUsd: number;
  perStageDefaultUsd: number;
  perStage: Record<string, number>;
  safetyFactor: number;
}

export function loadLimits(cwd: string = process.cwd()): Limits {
  const cfg = readJson<any>(path.join(cwd, "kineti.config.json"));
  const s = cfg?.settings?.spend_limit_usd ?? {};
  // Clamp every limit to a positive-finite number, else fall back to defaults.
  // Bad config must never disable the breaker (fail closed).
  const rawGlobal = Number(s.global ?? 50);
  const globalUsd = Number.isFinite(rawGlobal) && rawGlobal > 0 ? rawGlobal : 50;
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
  const envCmd = process.env.KINETI_VERIFY_CMD;
  if (envCmd && envCmd.trim().length > 0) return envCmd.trim();
  const cfg = readJson<any>(path.join(cwd, "kineti.config.json"));
  const cmd = cfg?.settings?.verify_command;
  return typeof cmd === "string" && cmd.trim().length > 0 ? cmd.trim() : null;
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
 * Simple whitespace split. Callers should prefer arrays.
 * If the string contains shell metachars, it will be blocked downstream
 * unless --allow-shell + TTY is given.
 */
export function splitLegacyCommand(cmd: string): string[] {
  const trimmed = cmd.trim();
  if (!trimmed) return [];
  // Simple split on whitespace. No quote handling — quotes are metachars and will block.
  return trimmed.split(/\s+/);
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

