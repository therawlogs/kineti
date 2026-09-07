#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { die, loadVerifyCommand, machineDir, nowIso, ok, readJson, sha256, writeJson } from "./lib.ts";

interface Trust { [repo: string]: { cmd_hash: string; at: string } }

function trustFile(): string { return path.join(machineDir(), "trust.json"); }

function repoKey(): string {
  return process.env.KINETI_REPO_KEY || process.cwd();
}

function main() {
  const argv = process.argv.slice(2);
  const cmd = argv[0];
  const declared = loadVerifyCommand();

  if (cmd === "--trust") {
    if (!declared) die("no verify command declared (kineti.config.json settings.verify_command or KINETI_VERIFY_CMD)", 2);
    if (!process.stdin.isTTY && !process.env.KINETI_TRUST_CONFIRMED) {
      die("security: --trust must be executed interactively in a human TTY session", 2);
    }
    const t = readJson<Trust>(trustFile()) ?? {};
    t[repoKey()] = { cmd_hash: sha256(declared), at: nowIso() };
    writeJson(trustFile(), t);
    ok(`trusted for this repo: ${declared}`);
    return;
  }

  if (cmd === "--status") {
    const t = readJson<Trust>(trustFile()) ?? {};
    const e = t[repoKey()];
    if (!declared) { ok("no verify command declared"); return; }
    if (!e) { ok("declared but NOT trusted"); return; }
    ok(e.cmd_hash === sha256(declared) ? "trusted and current" : "trust stale: command changed, re-run --trust");
    return;
  }

  if (cmd !== undefined) die(`unknown option: ${cmd}. Gate takes no command; it reads the declared verify command.`, 2);

  if (!declared) {
    ok("no verify command declared; gate passes open");
    process.exit(0);
  }

  const t = readJson<Trust>(trustFile()) ?? {};
  const entry = t[repoKey()];
  if (!entry || entry.cmd_hash !== sha256(declared)) {
    console.error("kineti: verify-gate blocked. This repo has not trusted the current verify command.");
    console.error(`Command: ${declared}`);
    console.error("Review it, then run once by hand: kineti-verify-gate --trust");
    process.exit(9);
  }

  const res = spawnSync("bash", ["-lc", declared], { stdio: "inherit" });
  if ((res.status ?? 1) !== 0) {
    console.error("kineti: verify FAILED; session must not end on red.");
    process.exit(1);
  }
  fs.appendFileSync(path.join(machineDir(), "alerts.log"), `${nowIso()} verify passed: ${declared}\n`);
  ok("verify passed");
}

main();
