#!/usr/bin/env bun
// bin/kineti-sync.ts
// Phase 5: encrypted multi-device persona sync. Passphrase-encrypted export
// and import of your notes. Off by default. Root goal is never overwritten
// by an import. Every run is audit logged with counts and hashes, never content.

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { die, ok, projectKdir, readJson, readJsonl, writeJson } from "./lib.ts";
import { appendAudit } from "./kineti-audit.ts";

const VERSION = 1;

function switchFile(): string {
  return path.join(projectKdir(), "kineti.json");
}

export function isSyncEnabled(): boolean {
  const s = readJson<{ sync_enabled?: boolean }>(switchFile());
  return s?.sync_enabled === true;
}

function keyFromPassphrase(pass: string, salt: Buffer): Buffer {
  return crypto.scryptSync(pass, salt, 32);
}

function collectSnapshot(): Record<string, unknown> {
  const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
  const journal = readJsonl<any>(path.join(projectKdir(), "journal.jsonl")).slice(-50);
  const prefs = readJson<any>(path.join(projectKdir(), "companion_settings.json")) || {};
  return {
    version: VERSION,
    exported_at: new Date().toISOString(),
    goal: typeof state.root_goal === "string" ? state.root_goal : null,
    stage: state.stage ?? null,
    journal_tail: journal,
    prefs: { user_name: prefs.user_name ?? null },
  };
}

function snapshotHash(snap: Record<string, unknown>): string {
  return crypto.createHash("sha256").update(JSON.stringify(snap)).digest("hex");
}

if (import.meta.main) {
  const [cmd, ...rest] = process.argv.slice(2);

  if (cmd === "on" || cmd === "off") {
    const on = cmd === "on";
    const cur = readJson<Record<string, unknown>>(switchFile()) || {};
    writeJson(switchFile(), { ...cur, sync_enabled: on, updated_by: "user", at: new Date().toISOString() });
    try { appendAudit("user", on ? "sync.on" : "sync.off", "device sync toggled"); } catch {}
    ok(`device sync is now ${on ? "on" : "off"}.`);
  } else if (cmd === "export") {
    let out = "";
    let pass = "";
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--out") out = rest[++i] ?? "";
      else if (rest[i] === "--passphrase-env") pass = process.env[rest[++i] ?? ""] || "";
    }
    if (!out) die("export requires --out FILE", 2);
    if (!isSyncEnabled()) die("device sync is off. Turn it on first, then export.", 2);
    if (!pass) die("export requires --passphrase-env VAR with a passphrase of 12+ chars", 2);
    if (pass.length < 12) die("passphrase must be 12+ chars", 2);
    const snap = collectSnapshot();
    const hash = snapshotHash(snap);
    const salt = crypto.randomBytes(16);
    const iv = crypto.randomBytes(12);
    const key = keyFromPassphrase(pass, salt);
    const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
    const data = Buffer.concat([cipher.update(JSON.stringify(snap), "utf8"), cipher.final()]);
    const tag = cipher.getAuthTag();
    const payload = {
      version: VERSION,
      algo: "aes-256-gcm/scrypt",
      salt: salt.toString("hex"),
      iv: iv.toString("hex"),
      tag: tag.toString("hex"),
      data: data.toString("hex"),
      snapshot_sha256: hash,
    };
    fs.writeFileSync(out, JSON.stringify(payload, null, 2) + "\n", { mode: 0o600 });
    try { appendAudit("user", "sync.export", `notes=${(snap.journal_tail as unknown[]).length} sha=${hash.slice(0, 12)}`); } catch {}
    ok(`exported encrypted notes to ${out} (sha ${hash.slice(0, 12)}).`);
  } else if (cmd === "import") {
    let file = "";
    let pass = "";
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--file") file = rest[++i] ?? "";
      else if (rest[i] === "--passphrase-env") pass = process.env[rest[++i] ?? ""] || "";
    }
    if (!file) die("import requires --file FILE", 2);
    if (!isSyncEnabled()) die("device sync is off. Turn it on first, then import.", 2);
    if (!pass) die("import requires --passphrase-env VAR", 2);
    let payload: any;
    try {
      payload = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch { die(`cannot read ${file}`, 2); }
    if (payload.version !== VERSION || payload.algo !== "aes-256-gcm/scrypt") {
      die("unsupported sync file version", 2);
    }
    let plain: string;
    try {
      const key = keyFromPassphrase(pass, Buffer.from(payload.salt, "hex"));
      const decipher = crypto.createDecipheriv("aes-256-gcm", key, Buffer.from(payload.iv, "hex"));
      decipher.setAuthTag(Buffer.from(payload.tag, "hex"));
      plain = Buffer.concat([
        decipher.update(Buffer.from(payload.data, "hex")),
        decipher.final(),
      ]).toString("utf8");
    } catch { die("wrong passphrase or damaged file", 2); }
    const snap = JSON.parse(plain);
    const hash = snapshotHash({ ...snap });
    if (hash !== payload.snapshot_sha256) die("snapshot hash mismatch, refusing import", 3);
    // Merge journal tail, dedupe by timestamp+text. Never touch root_goal.
    const journalFile = path.join(projectKdir(), "journal.jsonl");
    const existing = readJsonl<any>(journalFile);
    const seen = new Set(existing.map((j) => `${j.at}|${JSON.stringify(j)}`));
    const incoming = Array.isArray(snap.journal_tail) ? snap.journal_tail : [];
    let added = 0;
    for (const j of incoming) {
      const k = `${j.at}|${JSON.stringify(j)}`;
      if (!seen.has(k)) {
        fs.appendFileSync(journalFile, JSON.stringify(j) + "\n");
        seen.add(k);
        added += 1;
      }
    }
    const state = readJson<any>(path.join(projectKdir(), "state.json")) || {};
    const goalMatch = (state.root_goal ?? null) === (snap.goal ?? null);
    try { appendAudit("user", "sync.import", `added=${added} goal_match=${goalMatch} sha=${String(payload.snapshot_sha256).slice(0, 12)}`); } catch {}
    ok(`imported ${added} new notes. Root goal untouched (match=${goalMatch}).`);
  } else {
    die("unknown command: use on | off | export --out FILE --passphrase-env VAR | import --file FILE --passphrase-env VAR", 2);
  }
}
