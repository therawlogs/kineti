#!/usr/bin/env bun
// bin/kineti-pairing.ts
// Local half of the cloud dashboard link (GOOD_ROADMAP section 6).
// Makes single-use pairing codes with a 10 minute life, stored per project.
// No cloud server exists yet: make/status/revoke all work offline.
// Every step is audit logged. Cloud verify + token store land later.

import crypto from "node:crypto";
import path from "node:path";
import { die, ok, projectKdir, readJson, writeJson } from "./lib.ts";
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
    const fs = require("node:fs") as typeof import("node:fs");
    fs.rmSync(pairingFile(), { force: true });
  } catch {}
  try { appendAudit(actor, "pairing.revoke", p ? `project=${p.project}` : "no active pairing"); } catch {}
  return p !== null;
}

function cli(): void {
  const [cmd] = process.argv.slice(2);
  if (cmd === "make") {
    const p = makePairing();
    ok(`${p.code} (open ${PAIR_LINK}, valid ${minutesLeft(p)} min, one use)`);
    return;
  }
  if (cmd === "status") {
    const p = livePairing();
    if (!p) {
      ok("no live pairing code. Make one when you need it.");
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
  die("unknown command: use make | status | revoke", 2);
}

if (import.meta.main) cli();
