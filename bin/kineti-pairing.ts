#!/usr/bin/env bun
// bin/kineti-pairing.ts
// Local pairing-code prototype only. There is no cloud service in this release.

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { die, ok, projectKdir, readJson, writeJson, machineDir, MIN_PROJECT_CEILING_USD, MAX_PROJECT_CEILING_USD, DEFAULT_PROJECT_CEILING_USD } from "./lib.ts";
import { appendAudit } from "./kineti-audit.ts";

export const PAIR_TTL_MS = 10 * 60 * 1000;

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

/** Marks a local prototype code used; no cloud service calls this function. */
export function markUsed(actor = "local-prototype"): boolean {
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

function cloudFile(): string {
  return path.join(machineDir(), "cloud.json");
}

/** Cloud linking is unavailable; locally stored placeholder data is never treated as a link. */
export function cloudStatus(): { linked: false; available: false } {
  return { linked: false, available: false };
}

/** Clears local pairing prototype files. No cloud service is contacted. */
export function dropLink(actor = "user"): { hadPairing: boolean; hadCloud: boolean } {
  const hadPairing = revokePairing(actor);
  const hadCloud = fs.existsSync(cloudFile());
  try { fs.rmSync(cloudFile(), { force: true }); } catch {}
  try { appendAudit(actor, "pairing.local_clear", hadCloud ? "removed local placeholder link data; no cloud request sent" : "removed local pairing data; no cloud service exists"); } catch {}
  return { hadPairing, hadCloud };
}

/** Waits for local prototype state only; it does not contact a service. */
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
  ceiling: number;
  updated_at: string | null;
}

function mirrorFile(): string {
  return path.join(projectKdir(), "mirror.json");
}

/** Local project budget preference. It does not mirror data to a service. */
export function getMirror(): MirrorState {
  const state = readJson<{ project?: string }>(path.join(projectKdir(), "state.json")) || {};
  const project = typeof state.project === "string" ? state.project : "kineti";
  const disk = readJson<{ enabled?: boolean; note_sync?: boolean; ceiling?: unknown; updated_at?: string }>(mirrorFile());
  const rawCeiling = Number(disk?.ceiling);
  return {
    project,
    enabled: disk?.enabled === true,
    note_sync: disk?.note_sync === true,
    ceiling:
      Number.isFinite(rawCeiling) && rawCeiling >= MIN_PROJECT_CEILING_USD && rawCeiling <= MAX_PROJECT_CEILING_USD
        ? rawCeiling
        : DEFAULT_PROJECT_CEILING_USD,
    updated_at: typeof disk?.updated_at === "string" ? disk.updated_at : null,
  };
}

export function setMirror(enabled: boolean, noteSync: boolean, actor = "user", ceiling = DEFAULT_PROJECT_CEILING_USD): MirrorState {
  const cur = getMirror();
  const rawCeiling = Number(ceiling);
  const next: MirrorState = {
    project: cur.project,
    enabled,
    note_sync: noteSync,
    ceiling:
      Number.isFinite(rawCeiling) && rawCeiling >= MIN_PROJECT_CEILING_USD && rawCeiling <= MAX_PROJECT_CEILING_USD
        ? rawCeiling
        : cur.ceiling,
    updated_at: new Date().toISOString(),
  };
  writeJson(mirrorFile(), next);
  try { appendAudit(actor, "project.budget", `project=${next.project} ceiling=$${next.ceiling}; cloud mirror unavailable`); } catch {}
  return next;
}

function cli(): void {
  const [cmd, ...rest] = process.argv.slice(2);
  if (cmd === "make") {
    die("cloud pairing is not available in this release; no code was created", 2);
  }
  if (cmd === "status") {
    const mirror = getMirror();
    ok(`cloud pairing is unavailable; no remote link exists. Local budget ceiling is $${mirror.ceiling}.`);
    return;
  }
  if (cmd === "revoke") {
    const had = revokePairing();
    ok(had ? "local pairing code removed; no cloud service was contacted." : "no local pairing code was found.");
    return;
  }
  if (cmd === "claim") {
    die("cloud pairing is not available in this release; no link was created", 2);
  }
  if (cmd === "drop") {
    const d = dropLink();
    ok(d.hadPairing || d.hadCloud ? "local pairing data removed; no cloud service was contacted." : "no local pairing data was found.");
    return;
  }
  if (cmd === "poll") {
    die("cloud pairing is not available in this release; nothing can claim a local code", 2);
  }
  if (cmd === "mirror") {
    const on = rest[0] === "on";
    if (rest[0] !== "on" && rest[0] !== "off") die("mirror requires on or off", 2);
    const notes = rest.includes("--with-notes");
    let ceiling = DEFAULT_PROJECT_CEILING_USD;
    const ci = rest.indexOf("--ceiling");
    if (ci >= 0) ceiling = Number(rest[ci + 1] ?? DEFAULT_PROJECT_CEILING_USD);
    const m = setMirror(on, notes, "user", ceiling);
    ok(`saved local budget preference for ${m.project} with a $${m.ceiling} ceiling. No data was mirrored or sent.`);
    return;
  }
  die("unknown command: use status | revoke | drop | mirror <on|off>", 2);
}

if (import.meta.main) cli();
