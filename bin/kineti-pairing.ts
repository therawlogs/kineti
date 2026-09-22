#!/usr/bin/env bun
// bin/kineti-pairing.ts
// Local half of the cloud dashboard link (GOOD_ROADMAP section 6).
// Makes single-use pairing codes with a 10 minute life, stored per project.
// No cloud server exists yet: make/status/revoke all work offline.
// Every step is audit logged. Cloud verify + token store land later.

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { die, ok, projectKdir, readJson, writeJson, machineDir, ensureDir } from "./lib.ts";
import { appendAudit } from "./kineti-audit.ts";

export const PAIR_TTL_MS = 10 * 60 * 1000;
export const PAIR_LINK = "app.getkineti.com/pair";

export interface Pairing {
  code: string;
  created_at: string;
  expires_at: string;
  used: boolean;
  project: string;
}

function pairingFile(): string {
  return path.join(projectKdir(), "pairing.json");
}

function readPairing(): Pairing | null {
  return readJson<Pairing>(pairingFile());
}

function isLive(p: Pairing | null): p is Pairing {
  if (!p || p.used) return false;
  return Date.now() < new Date(p.expires_at).getTime();
}

/** Random code like KIN-7F2A-91QD. Skips confusing chars (0/O, 1/I). */
export function makeCode(): string {
  const alpha = "ABCDEFGHJKMNPQRSTUVWXYZ23456789";
  const part = (n: number) => {
    const bytes = crypto.randomBytes(n);
    let s = "";
    for (let i = 0; i < n; i++) s += alpha[bytes[i] % alpha.length];
    return s;
  };
  return `KIN-${part(4)}-${part(4)}`;
}

/** Makes a fresh code, replacing any older one. Returns the pairing. */
export function makePairing(actor = "user"): Pairing {
  const now = new Date();
  const state = readJson<{ project?: string }>(path.join(projectKdir(), "state.json")) || {};
  const p: Pairing = {
    code: makeCode(),
    created_at: now.toISOString(),
    expires_at: new Date(now.getTime() + PAIR_TTL_MS).toISOString(),
    used: false,
    project: typeof state.project === "string" ? state.project : "kineti",
  };
  writeJson(pairingFile(), p);
  try { appendAudit(actor, "pairing.make", `project=${p.project} expires=${p.expires_at}`); } catch {}
  return p;
}

/** Current live pairing, or null when none, expired, or used. */
export function livePairing(): Pairing | null {
  const p = readPairing();
  return isLive(p) ? p : null;
}

/** Minutes left on the live code, or 0. */
export function minutesLeft(p: Pairing): number {
  return Math.max(0, Math.ceil((new Date(p.expires_at).getTime() - Date.now()) / 60000));
}

/** Marks the code used after the cloud claims it. Kept for the future server hook. */
export function markUsed(actor = "cloud"): boolean {
  const p = readPairing();
  if (!isLive(p)) return false;
  writeJson(pairingFile(), { ...p, used: true });
  try { appendAudit(actor, "pairing.used", `project=${p.project}`); } catch {}
  return true;
}

/** Deletes the pairing and drops the link. Safe to call when none exists. */
export function revokePairing(actor = "user"): boolean {
  const p = readPairing();
  try {
    fs.rmSync(pairingFile(), { force: true });
  } catch {}
  try { appendAudit(actor, "pairing.revoke", p ? `project=${p.project}` : "no active pairing"); } catch {}
  return p !== null;
}

export interface CloudLink {
  linked_at: string;
  project: string;
}

function cloudFile(): string {
  return path.join(machineDir(), "cloud.json");
}

function writeOwnerOnly(file: string, value: unknown): void {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { mode: 0o600 });
  try { fs.chmodSync(file, 0o600); } catch {}
}

/**
 * Simulates the cloud claiming a live code. Marks it used, stores tokens
 * owner-only on this device, audit logs the pairing without token values.
 * The real server verify call lands later; shape stays the same.
 */
export function claimPairing(actor = "cloud"): CloudLink | null {
  const p = readPairing();
  if (!isLive(p)) return null;
  markUsed(actor);
  const tokens = {
    access_token: crypto.randomBytes(32).toString("hex"),
    refresh_token: crypto.randomBytes(32).toString("hex"),
    project: (p as Pairing).project,
    linked_at: new Date().toISOString(),
  };
  writeOwnerOnly(cloudFile(), tokens);
  try { appendAudit(actor, "cloud.linked", `project=${(p as Pairing).project}`); } catch {}
  return { linked_at: tokens.linked_at, project: tokens.project };
}

/** Linked state without ever exposing token values. */
export function cloudStatus(): CloudLink & { linked: true } | { linked: false } {
  const disk = readJson<{ linked_at?: string; project?: string; access_token?: string }>(cloudFile());
  if (disk && typeof disk.linked_at === "string" && typeof disk.access_token === "string") {
    return { linked: true as const, linked_at: disk.linked_at, project: typeof disk.project === "string" ? disk.project : "kineti" };
  }
  return { linked: false as const };
}

/**
 * Drops everything on this device: pairing file plus token store.
 * Audit logs both sides. Safe to call when nothing exists.
 */
export function dropLink(actor = "user"): { hadPairing: boolean; hadCloud: boolean } {
  const hadPairing = revokePairing(actor);
  const hadCloud = cloudStatus().linked;
  try { fs.rmSync(cloudFile(), { force: true }); } catch {}
  try { appendAudit(actor, "cloud.unlinked", hadCloud ? "stored tokens deleted on this device" : "no stored tokens"); } catch {}
  return { hadPairing, hadCloud };
}

/** Waits until the live code is used or expires. Outbound-only shape: reads local state, opens nothing. */
export async function pollPairing(timeoutMs = 60000, intervalMs = 2000): Promise<{ outcome: "used" | "expired" | "timeout" }> {
  const deadline = Date.now() + Math.max(1000, Math.min(600000, timeoutMs));
  const step = Math.max(250, Math.min(10000, intervalMs));
  for (;;) {
    const p = readPairing();
    if (!p) return { outcome: "expired" };
    if (p.used) return { outcome: "used" };
    if (Date.now() >= new Date(p.expires_at).getTime()) return { outcome: "expired" };
    if (Date.now() >= deadline) return { outcome: "timeout" };
    await new Promise((r) => setTimeout(r, step));
  }
}

export interface MirrorState {
  project: string;
  enabled: boolean;
  note_sync: boolean;
  updated_at: string | null;
}

function mirrorFile(): string {
  return path.join(projectKdir(), "mirror.json");
}

/** Per-project mirror consent. Off by default. Journal notes excluded unless note_sync is on. */
export function getMirror(): MirrorState {
  const state = readJson<{ project?: string }>(path.join(projectKdir(), "state.json")) || {};
  const project = typeof state.project === "string" ? state.project : "kineti";
  const disk = readJson<{ enabled?: boolean; note_sync?: boolean; updated_at?: string }>(mirrorFile());
  return {
    project,
    enabled: disk?.enabled === true,
    note_sync: disk?.note_sync === true,
    updated_at: typeof disk?.updated_at === "string" ? disk.updated_at : null,
  };
}

export function setMirror(enabled: boolean, noteSync: boolean, actor = "user"): MirrorState {
  const cur = getMirror();
  const next: MirrorState = {
    project: cur.project,
    enabled,
    note_sync: noteSync,
    updated_at: new Date().toISOString(),
  };
  writeJson(mirrorFile(), next);
  try { appendAudit(actor, enabled ? "mirror.on" : "mirror.off", `project=${next.project} notes=${noteSync ? "included" : "excluded"}`); } catch {}
  return next;
}

function cli(): void {
  const [cmd, ...rest] = process.argv.slice(2);
  if (cmd === "make") {
    const p = makePairing();
    ok(`${p.code} (open ${PAIR_LINK}, valid ${minutesLeft(p)} min, one use)`);
    return;
  }
  if (cmd === "status") {
    const p = livePairing();
    const link = cloudStatus();
    const mirror = getMirror();
    if (!p) {
      ok(`no live pairing code. Make one when you need it. Cloud link is ${link.linked ? "on" : "off"}. Mirror is ${mirror.enabled ? "on" : "off"}.`);
      return;
    }
    ok(`${p.code} (open ${PAIR_LINK}, ${minutesLeft(p)} min left, project=${p.project})`);
    return;
  }
  if (cmd === "revoke") {
    const had = revokePairing();
    ok(had ? "cloud link dropped." : "nothing to drop.");
    return;
  }
  if (cmd === "claim") {
    const c = claimPairing();
    if (!c) {
      die("no live code to claim. Make one first.", 2);
      return;
    }
    ok(`linked project=${c.project} at ${c.linked_at}. Tokens stored owner-only on this device.`);
    return;
  }
  if (cmd === "drop") {
    const d = dropLink();
    ok(d.hadPairing || d.hadCloud ? "cloud link dropped. Stored tokens deleted on this device." : "nothing to drop.");
    return;
  }
  if (cmd === "poll") {
    let timeoutMs = 60000;
    for (let i = 0; i < rest.length; i++) {
      if (rest[i] === "--timeout-ms") timeoutMs = Number(rest[++i] ?? 60000);
    }
    if (!Number.isFinite(timeoutMs)) timeoutMs = 60000;
    pollPairing(timeoutMs).then((r) => {
      if (r.outcome === "used") ok("paired. Cloud link is ready.");
      else if (r.outcome === "expired") ok("code expired or dropped. Make a fresh one when you need it.");
      else ok("still waiting. Code is live. Try again.");
    });
    return;
  }
  if (cmd === "mirror") {
    const on = rest[0] === "on";
    if (rest[0] !== "on" && rest[0] !== "off") die("mirror requires on or off", 2);
    const notes = rest.includes("--with-notes");
    const m = setMirror(on, notes);
    ok(`mirror for ${m.project} is now ${on ? "on" : "off"}. Journal notes ${m.note_sync ? "included" : "excluded"}. Files and vault secrets never leave this device.`);
    return;
  }
  die("unknown command: use make | status | revoke | claim | drop | poll | mirror <on|off>", 2);
}

if (import.meta.main) cli();
