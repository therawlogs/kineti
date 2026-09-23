#!/usr/bin/env bun
// bin/kineti-schema.ts
// Two-pass schema standardizer, v1 (GOOD_ROADMAP section 9).
// Pass 1: check any input (text, image ref, audio ref, tool output) into
// fixed fields and drop the rest. Pass 2 (later): feed cleaned fields to
// the model picker and load memory first so repeat tasks send less.
// Every check and memory hit is audit logged with token estimates as proof.
// Estimates are chars/4; real counts come from the spend log when wired.

import crypto from "node:crypto";
import path from "node:path";
import { die, ok, projectKdir, readJson, writeJson } from "./lib.ts";
import { appendAudit } from "./kineti-audit.ts";
import { classifyTask, type TaskType } from "./kineti-models.ts";

export interface RawInput {
  text?: string;
  imageRef?: string;
  audioRef?: string;
  toolOutput?: string;
}

export interface CheckedInput {
  kind: TaskType;
  task: string;
  entities: string[];
  images: string[];
  audio: string[];
  droppedChars: number;
  tokensBefore: number;
  tokensAfter: number;
  memoryHit: boolean;
}

export interface SchemaMemoryEntry {
  sig: string;
  kind: TaskType;
  task: string;
  entities: string[];
  before: number;
  after: number;
  hits: number;
  updated_at: string;
}

const MAX_TEXT = 2000;
const MAX_TOOL_TAIL = 1500;

export function estimateTokens(s: string): number {
  return Math.max(0, Math.ceil(s.length / 4));
}

function schemaFile(): string {
  return path.join(projectKdir(), "schema.json");
}

function readMemory(): SchemaMemoryEntry[] {
  const disk = readJson<{ entries?: SchemaMemoryEntry[] }>(schemaFile());
  return Array.isArray(disk?.entries) ? disk.entries : [];
}

function writeMemory(entries: SchemaMemoryEntry[]): void {
  writeJson(schemaFile(), { entries: entries.slice(-100) });
}

/** Stable signature for a raw input: normalized text plus ref presence. */
export function signatureOf(raw: RawInput): string {
  const norm = `${raw.text || ""}`.toLowerCase().replace(/\s+/g, " ").trim().slice(0, 200);
  const refs = `|img:${raw.imageRef ? 1 : 0}|aud:${raw.audioRef ? 1 : 0}|tool:${raw.toolOutput ? 1 : 0}`;
  return crypto.createHash("sha256").update(norm + refs).digest("hex").slice(0, 16);
}

function cleanText(t: string): string {
  return t.replace(/\s+/g, " ").trim().slice(0, MAX_TEXT);
}

function extractEntities(t: string): string[] {
  const found = new Set<string>();
  for (const m of t.matchAll(/`([^`]{1,64})`/g)) found.add(m[1].trim());
  for (const m of t.matchAll(/\b(?:src|bin|tests|docs|core-native)\/\S{1,80}/g)) {
    found.add(m[0].replace(/[.,;:!?)]+$/, ""));
  }
  return [...found].slice(0, 12);
}

/**
 * Pass 1: standardize any input into fixed fields.
 * Repeats hit memory and reuse the stored cleaned fields.
 */
export function checkInput(raw: RawInput, actor = "user"): CheckedInput {
  const text = String(raw.text || "");
  const tool = String(raw.toolOutput || "");
  const before = estimateTokens(text + tool);
  const sig = signatureOf(raw);

  const entries = readMemory();
  const known = entries.find((e) => e.sig === sig);
  if (known) {
    known.hits += 1;
    known.updated_at = new Date().toISOString();
    writeMemory(entries);
    try { appendAudit(actor, "schema.hit", `${known.kind} sig=${sig} saved~${before - known.after} tokens`); } catch {}
    return {
      kind: known.kind,
      task: known.task,
      entities: known.entities,
      images: raw.imageRef ? [raw.imageRef.slice(0, 256)] : [],
      audio: raw.audioRef ? [raw.audioRef.slice(0, 256)] : [],
      droppedChars: Math.max(0, text.length + tool.length - known.task.length),
      tokensBefore: before,
      tokensAfter: known.after,
      memoryHit: true,
    };
  }

  const toolTail = tool.length > MAX_TOOL_TAIL ? tool.slice(-MAX_TOOL_TAIL) : tool;
  const task = cleanText(text || toolTail);
  const kind = classifyTask(task);
  const entities = extractEntities(`${text} ${toolTail}`);
  const after = estimateTokens(task);
  const entry: SchemaMemoryEntry = {
    sig,
    kind,
    task,
    entities,
    before,
    after,
    hits: 0,
    updated_at: new Date().toISOString(),
  };
  entries.push(entry);
  writeMemory(entries);
  try { appendAudit(actor, "schema.check", `${kind} sig=${sig} ${before}->${after} tokens`); } catch {}
  return {
    kind,
    task,
    entities,
    images: raw.imageRef ? [raw.imageRef.slice(0, 256)] : [],
    audio: raw.audioRef ? [raw.audioRef.slice(0, 256)] : [],
    droppedChars: Math.max(0, text.length + tool.length - task.length),
    tokensBefore: before,
    tokensAfter: after,
    memoryHit: false,
  };
}

/** Run 1 vs run 2 proof for the same input. Second run must cost less or equal. */
export function proveRepeat(raw: RawInput, actor = "user"): { first: CheckedInput; second: CheckedInput; saved: number } {
  const first = checkInput(raw, actor);
  const second = checkInput(raw, actor);
  return { first, second, saved: second.tokensBefore - second.tokensAfter };
}

function cli(): void {
  const argv = process.argv.slice(2);
  const cmd = argv[0];
  if (cmd === "check") {
    let text = "";
    let imageRef = "";
    let audioRef = "";
    let toolOutput = "";
    for (let i = 1; i < argv.length; i++) {
      if (argv[i] === "--image") imageRef = argv[++i] ?? "";
      else if (argv[i] === "--audio") audioRef = argv[++i] ?? "";
      else if (argv[i] === "--tool") toolOutput = argv[++i] ?? "";
      else text += (text ? " " : "") + argv[i];
    }
    if (!text && !toolOutput) die("check requires words or --tool output", 2);
    const c = checkInput({ text, imageRef, audioRef, toolOutput });
    ok(`${c.kind}: "${c.task.slice(0, 120)}" ${c.tokensBefore}->${c.tokensAfter} tokens${c.memoryHit ? " (memory hit)" : ""}. Entities: ${c.entities.join(", ") || "none"}.`);
    return;
  }
  if (cmd === "stats") {
    const entries = readMemory();
    const hits = entries.reduce((n, e) => n + e.hits, 0);
    const saved = entries.reduce((n, e) => n + (e.before - e.after) * e.hits, 0);
    ok(`${entries.length} learned inputs, ${hits} memory hits, ~${saved} tokens saved on repeats.`);
    return;
  }
  die("unknown command: use check <words> [--image REF] [--audio REF] [--tool OUT] | stats", 2);
}

if (import.meta.main) cli();
