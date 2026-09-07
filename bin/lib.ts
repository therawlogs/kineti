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
  return {
    globalUsd: Number(s.global ?? 50),
    perStageDefaultUsd: Number(s.per_stage_default ?? 10),
    perStage: (s.per_stage ?? {}) as Record<string, number>,
    safetyFactor: Number(s.safety_factor ?? 0.95),
  };
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
      `# Kineti OS — Autonomous Directives for Claude Code\n\nThis repository is governed by Kineti OS v3.\n\n## Rules\n1. Plain English and numbered choices (1, 2, 3).\n2. Inspect .kineti/state.json on every prompt to track the active task or stage (flexible: bugfix, feature, refactor, audit, or stages 1-13).\n3. Hard gate: For new features, do NOT write application code in src/ before Spec approval.\n4. Check spend status before heavy tasks ($50.00 limit).\n5. Record test proofs via bun bin/kineti-evidence.ts run.\n`
    );
    installed.push("CLAUDE.md");
  }

  // 2. AGENTS.md
  const agentsPath = path.join(targetDir, "AGENTS.md");
  if (!fs.existsSync(agentsPath)) {
    fs.writeFileSync(
      agentsPath,
      `# Kineti OS — Autonomous Agent Standing Directives\n\nThis repository is governed by Kineti OS v3.\n\n## Rules\n1. Plain English interaction with numbered choices.\n2. Read .kineti/state.json. Supports stage-agnostic tasks (bugfix, refactor, etc.) and full 13-stage pipelines.\n3. Verify test proofs with code fingerprints.\n4. Pause if spend reaches $50 USD.\n`
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
      `---\ndescription: Kineti OS Autonomous Runtime Governance\nglobs: *\nalwaysApply: true\n---\n\n# Kineti OS — Cursor Autonomous Governance\n\n- Inspect .kineti/state.json on every task (flexible stages: bugfix, feature, refactor, audit, 1-13).\n- Spec gate blocks code generation for new features until approved.\n- Use registered kineti_* MCP tools.\n- Respect spend circuit breaker ($50 ceiling).\n`
    );
    installed.push(".cursor/rules/kineti.mdc");
  }

  // 4. CODEX.md
  const codexPath = path.join(targetDir, "CODEX.md");
  if (!fs.existsSync(codexPath)) {
    fs.writeFileSync(
      codexPath,
      `# Kineti OS — Autonomous Directives for OpenAI Codex\n\nGoverned by Kineti OS v3.\n\n1. Plain English and numbered options.\n2. Read .kineti/state.json. Supports flexible stage entry (bugfix, refactor, spec, build).\n3. Verify evidence with code fingerprints.\n`
    );
    installed.push("CODEX.md");
  }

  return installed;
}

