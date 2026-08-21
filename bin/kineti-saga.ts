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

function main() {
  const [cmd, ...rest] = process.argv.slice(2);
  let runId = "", label = "", inverse = "";
  for (let i = 0; i < rest.length; i++) {
    switch (rest[i]) {
      case "--run-id": runId = rest[++i] ?? ""; break;
      case "--label": label = rest[++i] ?? ""; break;
      case "--inverse": inverse = rest[++i] ?? ""; break;
    }
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

  if (cmd === "rollback") {
    openRun(runId);
    const regs = lines().filter((l) => l.kind === "register" && l.run_id === runId);
    const undone = new Set(
      lines().filter((l) => l.kind === "rollback_step" && l.run_id === runId).map((l) => l.label),
    );
    const pending = regs.filter((r) => !undone.has(r.label!)).reverse();
    if (pending.length === 0) { ok(`nothing to roll back for ${runId}`); return; }
    for (const r of pending) {
      const res = spawnSync("bash", ["-lc", r.inverse!], { stdio: "pipe", encoding: "utf8" });
      const code = res.status ?? 1;
      appendJsonl(file(), {
        at: nowIso(), kind: "rollback_step", run_id: runId,
        label: r.label, exit_code: code,
      } satisfies Line);
      if (code !== 0) console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}); continuing`);
      else ok(`undone: ${r.label}`);
    }
    appendJsonl(file(), { at: nowIso(), kind: "rollback_done", run_id: runId } satisfies Line);
    ok(`rollback complete for ${runId} (${pending.length} steps, newest-first)`);
    return;
  }

  die(`unknown command: ${cmd}. Use begin | register | commit | rollback`, 2);
}

main();
