#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { appendJsonl, die, loadVerifyCommand, nowIso, ok, projectKdir, readJson, readJsonl, runSafeCommand, sha256, splitLegacyCommand, writeJson } from "./lib.ts";

interface Line {
  at: string; kind: "begin" | "register" | "commit" | "rollback_step" | "rollback_done";
  run_id: string; label?: string; inverse?: string; inverse_hash?: string; exit_code?: number | null;
}

function file(): string { return path.join(projectKdir(), "saga.jsonl"); }
function guardFile(): string { return path.join(projectKdir(), "saga.guard.json"); }

function lines(): Line[] { return readJsonl<Line>(file()); }

function updateGuard(): void {
  try {
    const st = fs.statSync(file());
    writeJson(guardFile(), { uid: (st as any).uid ?? null, gid: (st as any).gid ?? null, mtimeMs: st.mtimeMs, size: st.size, at: nowIso() });
  } catch { /* file may not exist yet; guard created on first write */ }
}

function checkGuard(): void {
  let guard: { uid: number | null; gid: number | null; mtimeMs: number; size: number } | null = null;
  try {
    guard = readJson<any>(guardFile());
  } catch { guard = null; }
  let st: fs.Stats;
  try {
    st = fs.statSync(file());
  } catch {
    die("saga ledger missing; refusing rollback", 3);
  }
  if (!guard) {
    // First run after upgrade (no guard yet): trust current file once, create guard.
    console.error("kineti: note: no saga guard yet; trusting current ledger once and creating guard");
    updateGuard();
    return;
  }
  const uid = (st! as any).uid ?? null, gid = (st! as any).gid ?? null;
  if (guard.uid !== null && uid !== null && guard.uid !== uid) {
    die(`saga ledger owner changed (was ${guard.uid}, now ${uid}); refusing rollback. Inspect ${file()} before retrying.`, 3);
  }
  if (guard.gid !== null && gid !== null && guard.gid !== gid) {
    die(`saga ledger group changed; refusing rollback. Inspect ${file()} before retrying.`, 3);
  }
  if (guard.mtimeMs !== st!.mtimeMs || guard.size !== st!.size) {
    die(`saga ledger changed outside push/register (mtime/size mismatch); refusing rollback. Inspect ${file()} before retrying.`, 3);
  }
  const mode = st!.mode & 0o777;
  if (mode & 0o022) {
    console.error(`kineti: warning: saga ledger is group/world-writable (${mode.toString(8)}); proceeding but fix permissions`);
  }
}

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
  let allowShell = false;
  let yes = false;
  const positional: string[] = [];
  for (let i = 0; i < rest.length; i++) {
    if (rest[i] === "--run-id") runId = rest[++i] ?? "";
    else if (rest[i] === "--label") label = rest[++i] ?? "";
    else if (rest[i] === "--inverse") inverse = rest[++i] ?? "";
    else if (rest[i] === "--allow-shell") allowShell = true;
    else if (rest[i] === "--yes") yes = true;
    else positional.push(rest[i]);
  }

  if (cmd === "begin") {
    if (!runId) die("begin requires --run-id R");
    appendJsonl(file(), { at: nowIso(), kind: "begin", run_id: runId } satisfies Line);
    updateGuard();
    ok(`run ${runId} open`);
    return;
  }

  if (cmd === "register") {
    openRun(runId);
    if (!label || !inverse) die("register requires --label L --inverse \"command\"");
    appendJsonl(file(), { at: nowIso(), kind: "register", run_id: runId, label, inverse, inverse_hash: sha256(inverse) } satisfies Line);
    updateGuard();
    ok(`undo step registered: ${label}`);
    return;
  }

  if (cmd === "commit") {
    openRun(runId);
    appendJsonl(file(), { at: nowIso(), kind: "commit", run_id: runId } satisfies Line);
    updateGuard();
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
    appendJsonl(file(), { at: nowIso(), kind: "register", run_id: targetRun, label: pLabel, inverse: pInverse, inverse_hash: sha256(pInverse) } satisfies Line);
    updateGuard();
    ok(`undo step registered: ${pLabel}`);
    return;
  }

  if (cmd === "rollback") {
    const targetRun = runId || latestOpenRun() || "default";
    openRun(targetRun);
    checkGuard();
    const regs = lines().filter((l) => l.kind === "register" && l.run_id === targetRun);
    const undone = new Set(
      lines().filter((l) => l.kind === "rollback_step" && l.run_id === targetRun).map((l) => l.label),
    );
    const pending = regs.filter((r) => !undone.has(r.label!)).reverse();
    if (pending.length === 0) { ok(`nothing to roll back for ${targetRun}`); return; }
    // Print every inverse with its hash so a human sees planted commands before confirming.
    console.log(`Pending undo steps for ${targetRun} (newest-first, ${pending.length}):`);
    for (const r of pending) {
      const h = r.inverse_hash || sha256(r.inverse || "");
      const stored = r.inverse_hash ? "" : " (legacy: no stored hash)";
      console.log(`  - [${r.label}] hash:${h.slice(0, 12)} :: ${r.inverse}${stored}`);
      if (r.inverse_hash && r.inverse_hash !== sha256(r.inverse || "")) {
        die(`saga ledger TAMPER at "${r.label}": inverse hash mismatch; refusing rollback. Inspect ${file()} before retrying.`, 3);
      }
    }
    if (!yes) {
      if (process.stdin.isTTY) {
        let answer: string | null = null;
        try {
          answer = prompt(`Type y to run ${pending.length} undo steps (newest-first), or N to abort: `);
        } catch { answer = null; }
        if ((answer || "").trim().toLowerCase() !== "y") {
          die("rollback aborted: type y to confirm. Nothing was undone.", 2);
        }
      } else {
        die(`rollback needs a human: ${pending.length} steps listed above. Re-run in a TTY and type y, or pass --yes for automation after reviewing the list. Nothing was undone.`, 2);
      }
    } else {
      console.error(`kineti: confirmed via --yes (automation); running ${pending.length} steps newest-first`);
    }
    const verifyCmd = loadVerifyCommand();
    for (const r of pending) {
      const cmdArgv = splitLegacyCommand(r.inverse || "");
      const res = runSafeCommand(cmdArgv, { cwd: process.cwd(), workspaceRoot: process.cwd(), timeoutMs: 60000, allowShell, verifyCmd });
      if (res.stdout) process.stdout.write(res.stdout);
      if (res.stderr) process.stderr.write(res.stderr + (res.stderr.endsWith("\n") ? "" : "\n"));
      const code = res.exitCode;
      appendJsonl(file(), {
        at: nowIso(), kind: "rollback_step", run_id: targetRun,
        label: r.label, exit_code: code,
      } satisfies Line);
      updateGuard();
      if (code !== 0) console.error(`kineti: CRITICAL undo failed for "${r.label}" (exit ${code}); continuing`);
      else ok(`undone: ${r.label}`);
    }
    appendJsonl(file(), { at: nowIso(), kind: "rollback_done", run_id: targetRun } satisfies Line);
    updateGuard();
    ok(`rollback complete for ${targetRun} (${pending.length} steps, newest-first)`);
    return;
  }

  die(`unknown command: ${cmd}. Use begin | register | push | commit | rollback [--yes] [--allow-shell]`, 2);
}

main();
