#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import path from "node:path";
import { appendJsonl, die, nowIso, ok, projectKdir, readJsonl } from "./lib.ts";

interface Line {
  at: string; kind: "begin" | "register" | "commit" | "rollback_step" | "rollback_done";
  run_id: string; label?: string; inverse?: string; exit_code?: number | null;
}

function file(): string { return path.join(projectKdir(), "saga.jsonl"); }

function lines(): Line[] { return readJsonl<Line>(file()); }

function openRun(runId: string): void {
  const ls = lines();
  const begin = ls.find((l) => l.kind === "begin" && l.run_id === runId);
  if (!begin) die(`unknown run: ${runId}. Begin it first.`, 2);
  if (ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
}

function ensureRun(runId: string): void {
  const ls = lines();
  const begin = ls.find((l) => l.kind === "begin" && l.run_id === runId);
  if (!begin) {
    appendJsonl(file(), { at: nowIso(), kind: "begin", run_id: runId } satisfies Line);
    return;
  }
  if (ls.some((l) => l.kind === "commit" && l.run_id === runId)) die(`run ${runId} already committed`, 2);
}

function latestOpenRun(): string | null {
  const ls = lines();
  const committed = new Set(ls.filter((l) => l.kind === "commit").map((l) => l.run_id));
  const begins = ls.filter((l) => l.kind === "begin" && !committed.has(l.run_id));
  if (begins.length === 0) return null;
  return begins[begins.length - 1].run_id;
}

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  let runId = "", label = "", inverse = "";
  const positional: string[] = [];
  for (let i = 0; i < rest.length; i++) {
    if (rest[i] === "--run-id") runId = rest[++i] ?? "";
    else if (rest[i] === "--label") label = rest[++i] ?? "";
    else if (rest[i] === "--inverse") inverse = rest[++i] ?? "";
    else positional.push(rest[i]);
  }

  if (cmd === "begin") {
    if (!runId) die("begin requires --run-id R");
    appendJsonl(file(), { at: nowIso(), kind: "begin", run_id: runId } satisfies Line);
    ok(`run ${runId} open`);
    return;
  }

  if (cmd === "register") {
    openRun(runId);
    if (!label || !inverse) die("register requires --label L --inverse \"command\"");
    appendJsonl(file(), { at: nowIso(), kind: "register", run_id: runId, label, inverse } satisfies Line);
    ok(`undo step registered: ${label}`);
    return;
  }

  if (cmd === "commit") {
    openRun(runId);
    appendJsonl(file(), { at: nowIso(), kind: "commit", run_id: runId } satisfies Line);
    ok(`run ${runId} committed`);
    return;
  }

  if (cmd === "push") {
    const targetRun = runId || "default";
    let pLabel = label;
    let pInverse = inverse;
    if (!pLabel && positional.length >= 1) pLabel = positional[0];
    if (!pInverse && positional.length >= 2) pInverse = positional[1];
    if (!pInverse && positional.length === 1) {
      pInverse = positional[0];
      pLabel = `manual-${Date.now()}`;
    }
    if (!pLabel || !pInverse) die(`push requires <label> <undo-command> OR --label L --inverse "command"`);
    ensureRun(targetRun);
    appendJsonl(file(), { at: nowIso(), kind: "register", run_id: targetRun, label: pLabel, inverse: pInverse } satisfies Line);
    ok(`undo step registered: ${pLabel}`);
    return;
  }

  if (cmd === "rollback") {
    const targetRun = runId || latestOpenRun() || "default";
    openRun(targetRun);
    const regs = lines().filter((l) => l.kind === "register" && l.run_id === targetRun);
    const undone = new Set(
      lines().filter((l) => l.kind === "rollback_step" && l.run_id === targetRun).map((l) => l.label),
    );
    const pending = regs.filter((r) => !undone.has(r.label!)).reverse();
    if (pending.length === 0) { ok(`nothing to roll back for ${targetRun}`); return; }
    for (const r of pending) {
      const res = spawnSync("bash", ["-lc", r.inverse!], { stdio: "pipe", encoding: "utf8" });
      const code = res.status ?? 1;
      appendJsonl(file(), {
        at: nowIso(), kind: "rollback_step", run_id: targetRun,
        label: r.label, exit_code: code,
      } satisfies Line);
      if (code !== 0) console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}); continuing`);
      else ok(`undone: ${r.label}`);
    }
    appendJsonl(file(), { at: nowIso(), kind: "rollback_done", run_id: targetRun } satisfies Line);
    ok(`rollback complete for ${targetRun} (${pending.length} steps, newest-first)`);
    return;
  }

  die(`unknown command: ${cmd}. Use begin | register | push | commit | rollback`, 2);
}

main();
