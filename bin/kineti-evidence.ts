#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { appendJsonl, die, nowIso, ok, projectKdir, readJsonl, sha256 } from "./lib.ts";

interface Record {
  at: string; label: string; cmd: string; exit_code: number | null;
  fingerprint: string;
}

const EXCLUDE_DIRS = new Set([
  ".git", ".kineti", "node_modules", "dist", "build", ".next",
  "coverage", "tmp", ".cache", "legacy",
]);
const EXCLUDE_FILES = new Set([".DS_Store"]);
const MAX_FILE_BYTES = 4 * 1024 * 1024;

function file(): string { return path.join(projectKdir(), "evidence.jsonl"); }

export function fingerprint(root: string = process.cwd()): string {
  const parts: string[] = [];
  const walk = (dir: string) => {
    let entries: fs.Dirent[];
    try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return; }
    for (const e of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const full = path.join(dir, e.name);
      if (e.isDirectory()) {
        if (!EXCLUDE_DIRS.has(e.name)) walk(full);
        continue;
      }
      if (e.isFile() && !EXCLUDE_FILES.has(e.name)) {
        try {
          const st = fs.statSync(full);
          if (st.size > MAX_FILE_BYTES) continue;
          const rel = path.relative(root, full);
          parts.push(`${rel}:${sha256(fs.readFileSync(full))}`);
        } catch { /* unreadable: skip */ }
      }
    }
  };
  walk(root);
  return sha256(parts.join("\n"));
}

function records(): Record[] { return readJsonl<Record>(file()); }

function main() {
  const argv = process.argv.slice(2);
  const cmd0 = argv[0];

  if (cmd0 === "fingerprint") {
    ok(fingerprint());
    return;
  }

  if (cmd0 === "run") {
    let label = "";
    const dd = argv.indexOf("--");
    if (dd === -1) die("run requires: run --label L -- <command...>");
    for (let i = 1; i < dd; i++) if (argv[i] === "--label") label = argv[i + 1] ?? "";
    const command = argv.slice(dd + 1).join(" ");
    if (!label || command.length === 0) die("run requires --label and a command after --");
    const fpBefore = fingerprint();
    const res = Bun.spawnSync(["bash", "-lc", command], { stdout: "pipe", stderr: "pipe" });
    const code = res.exitCode;
    const fpAfter = fingerprint();
    appendJsonl(file(), {
      at: nowIso(), label, cmd: command, exit_code: code,
      fingerprint: fpAfter,
    } satisfies Record);
    if (code !== 0) die(`command failed (exit ${code}); recorded as proof anyway`, 1);
    ok(`proof recorded: ${label} ${fpBefore === fpAfter ? "(code unchanged during run)" : "(note: code changed during run)"}`);
    return;
  }

  if (cmd0 === "check") {
    let label = "", maxAgeMin = 240, expectCmd: string | null = null;
    for (let i = 1; i < argv.length; i++) {
      switch (argv[i]) {
        case "--label": label = argv[++i] ?? ""; break;
        case "--max-age": maxAgeMin = Number(argv[++i]); break;
        case "--expect-cmd": expectCmd = argv[++i] ?? null; break;
      }
    }
    if (!label) die("check requires --label L");
    const rs = records().filter((r) => r.label === label);
    if (rs.length === 0) { console.error("kineti: MISSING"); process.exit(5); }
    const last = rs[rs.length - 1];
    if (expectCmd && !last.cmd.includes(expectCmd)) { console.error("kineti: STALE (different command)"); process.exit(4); }
    const ageMin = (Date.now() - new Date(last.at).getTime()) / 60000;
    if (ageMin > maxAgeMin) { console.error(`kineti: STALE (${Math.round(ageMin)} min old)`); process.exit(4); }
    if (last.fingerprint !== fingerprint()) { console.error("kineti: STALE (code changed after the run)"); process.exit(4); }
    if (last.exit_code !== 0) { console.error("kineti: STALE (recorded run failed)"); process.exit(4); }
    ok(`FRESH: ${label}`);
    return;
  }

  die(`unknown command: ${cmd0}. Use fingerprint | run | check`, 2);
}

main();
