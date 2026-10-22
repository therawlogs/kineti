import { test, expect } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { JournalWriter } from "../bin/kineti-memory-job.ts";
import { runSafeCommand } from "../bin/lib.ts";
import { canonicalJsonV2, legacyCanonicalJson } from "../src/security/canonical-json.ts";

const repo = path.resolve(import.meta.dir, "..");
const scratch = path.join(repo, ".kineti", "checkpoint-a2");
fs.mkdirSync(scratch, { recursive: true });
function fixture() { return fs.mkdtempSync(path.join(scratch, "case-")); }
function cli(root: string, script: string, args: string[], stdin?: string) {
  return Bun.spawnSync([process.execPath, path.join(repo, "bin", script), ...args], {
    cwd: root, env: { ...process.env, KINETI_MACHINE_DIR: path.join(root, ".kineti", "machine"), TEST_SYNC_PASS: "synthetic-only-password" }, stdin: stdin === undefined ? undefined : Buffer.from(stdin),
  });
}
function append(root: string, id: string) {
  return new JournalWriter(root, repo).appendRunRecord({ project: "test", id, data: { root_goal: "synthetic", stage_outcomes: [], costs_usd: null, failed_approaches: [] } });
}

test("MCP undo preserves spaces, empty arguments and Unicode through execution", () => {
  const root = fixture();
  try {
    fs.writeFileSync(path.join(root, "capture.ts"), 'await Bun.write("captured.json", JSON.stringify(process.argv.slice(2)));');
    const argv = ["bun", "capture.ts", "two words", "", "λ東京"];
    const rpc = { jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: "kineti_saga_push", arguments: { step: "capture", inverse: argv } } };
    const pushed = cli(root, "kineti-mcp.ts", ["--workspace-root", root], JSON.stringify(rpc) + "\n");
    expect(pushed.exitCode).toBe(0);
    expect(cli(root, "kineti-saga.ts", ["rollback", "--yes"]).exitCode).toBe(0);
    expect(JSON.parse(fs.readFileSync(path.join(root, "captured.json"), "utf8"))).toEqual(argv.slice(2));
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("journal truncation blocks verification and append without changing remaining bytes", () => {
  const root = fixture();
  try {
    append(root, "rr-first-001"); append(root, "rr-second-001");
    const file = path.join(root, ".kineti/journal.jsonl");
    const first = fs.readFileSync(file, "utf8").split("\n")[0] + "\n";
    fs.writeFileSync(file, first);
    expect(cli(root, "kineti-memory-job.ts", ["verify-chain"]).exitCode).not.toBe(0);
    expect(() => append(root, "rr-third-001")).toThrow();
    expect(fs.readFileSync(file, "utf8")).toBe(first);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("audit writers reject changed history and detect a removed final record", () => {
  for (const mode of ["changed", "truncated"]) {
    const root = fixture();
    try {
      expect(cli(root, "kineti-audit.ts", ["log", "--action", "first"]).exitCode).toBe(0);
      expect(cli(root, "kineti-audit.ts", ["log", "--action", "second"]).exitCode).toBe(0);
      const file = path.join(root, ".kineti/machine/audit.log.jsonl");
      const lines = fs.readFileSync(file, "utf8").trim().split("\n");
      const bytes = mode === "changed" ? lines.join("\n").replace('"action":"first"', '"action":"other"') + "\n" : lines[0] + "\n";
      fs.writeFileSync(file, bytes);
      expect(cli(root, "kineti-audit.ts", ["verify"]).exitCode).not.toBe(0);
      expect(cli(root, "kineti-audit.ts", ["log", "--action", "third"]).exitCode).not.toBe(0);
      expect(fs.readFileSync(file, "utf8")).toBe(bytes);
    } finally { fs.rmSync(root, { recursive: true, force: true }); }
  }
});

test("encrypted sync merges valid histories with verifiable import provenance", () => {
  const source = fixture(), destination = fixture();
  try {
    append(source, "rr-source-001"); append(destination, "rr-dest-001");
    expect(cli(source, "kineti-sync.ts", ["on"]).exitCode).toBe(0);
    expect(cli(destination, "kineti-sync.ts", ["on"]).exitCode).toBe(0);
    const snapshot = path.join(source, "snapshot.json");
    expect(cli(source, "kineti-sync.ts", ["export", "--out", snapshot, "--passphrase-env", "TEST_SYNC_PASS"]).exitCode).toBe(0);
    fs.copyFileSync(snapshot, path.join(destination, "snapshot.json"));
    const args = ["import", "--file", "snapshot.json", "--passphrase-env", "TEST_SYNC_PASS"];
    expect(cli(destination, "kineti-sync.ts", args).exitCode).toBe(0);
    expect(cli(destination, "kineti-memory-job.ts", ["verify-chain"]).exitCode).toBe(0);
    const file = path.join(destination, ".kineti/journal.jsonl"), bytes = fs.readFileSync(file, "utf8");
    const imported = bytes.trim().split("\n").map(line => JSON.parse(line)).find(record => record.data?.import_provenance);
    expect(imported.data.import_provenance.original_record.id).toBe("rr-source-001");
    expect(cli(destination, "kineti-sync.ts", args).exitCode).toBe(0);
    expect(fs.readFileSync(file, "utf8")).toBe(bytes);
  } finally { fs.rmSync(source, { recursive: true, force: true }); fs.rmSync(destination, { recursive: true, force: true }); }
});

test("execution logging binds argv by digest without storing secret argument values", () => {
  const root = fixture(), previous = process.env.KINETI_MACHINE_DIR;
  process.env.KINETI_MACHINE_DIR = path.join(root, ".kineti/machine");
  try {
    const secret = "synthetic-secret-42";
    const result = runSafeCommand(["echo", secret], { cwd: root, workspaceRoot: root });
    expect(result.exitCode).toBe(0);
    const ledger = fs.readFileSync(path.join(root, ".kineti/machine/egress.jsonl"), "utf8");
    expect(ledger).not.toContain(secret);
    expect(ledger).toContain("argv_sha256=");
  } finally {
    if (previous === undefined) delete process.env.KINETI_MACHINE_DIR; else process.env.KINETI_MACHINE_DIR = previous;
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("egress recovery preserves damaged history and links a distinct replacement", () => {
  const root = fixture();
  try {
    const source = path.join(root, "damaged.jsonl"), bytes = Buffer.from('{"old":"damaged"}\n{bad');
    fs.writeFileSync(source, bytes);
    expect(cli(root, "kineti-egress.ts", ["recover", "--source", source, "--destination", ".kineti/replacement", "--reason", "synthetic recovery", "--i-am-human"]).exitCode).toBe(0);
    expect(fs.readFileSync(source)).toEqual(bytes);
    expect(fs.readFileSync(path.join(root, ".kineti/replacement/historical-damaged-egress.jsonl"))).toEqual(bytes);
    const run = (args: string[]) => Bun.spawnSync([process.execPath, path.join(repo, "bin/kineti-egress.ts"), ...args], { cwd: root, env: { ...process.env, KINETI_MACHINE_DIR: path.join(root, ".kineti/replacement") } });
    expect(run(["verify"]).exitCode).toBe(0);
    const metadata = JSON.parse(fs.readFileSync(path.join(root, ".kineti/replacement/recovery.json"), "utf8"));
    expect(metadata.unresolved_historical_damage).toBe(true);
    fs.unlinkSync(path.join(root, ".kineti/replacement/egress.state.json"));
    expect(run(["record", "--host", "test", "--description", "must block"]).exitCode).not.toBe(0);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("canonical v2 normalizes number spellings and UTF-16 key order without accepting invalid JSON", () => {
  expect(canonicalJsonV2({ "\ue000": null, "😀": 1 })).toBe('["kineti-canonical-v2",["object",[["f09f9880",["f64","3ff0000000000000"]],["ee8080",["null"]]]]]');
  expect(canonicalJsonV2(-0)).toBe(canonicalJsonV2(0));
  expect(canonicalJsonV2(0.1)).toBe('["kineti-canonical-v2",["f64","3fb999999999999a"]]');
  for (const invalid of [NaN, Infinity, 9007199254740992, undefined, () => 1, 1n, new Date(), "\ud800", [, 1]]) {
    expect(() => canonicalJsonV2(invalid)).toThrow();
    expect(() => legacyCanonicalJson(invalid)).toThrow();
  }
  const getter = Object.defineProperty({}, "key", { enumerable: true, get() { throw new Error("must not execute getter"); } });
  expect(() => canonicalJsonV2(getter)).toThrow("accessors");
  expect(legacyCanonicalJson({ z: 1, a: 0.1 })).toBe('{"a":0.1,"z":1}');
});

test("malformed undo is refused before registration or execution", () => {
  const root = fixture();
  try {
    expect(cli(root, "kineti-saga.ts", ["push", "bad", 'echo "unclosed']).exitCode).not.toBe(0);
    expect(fs.existsSync(path.join(root, ".kineti/saga.jsonl"))).toBe(false);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("weekly maintenance enters each approved project from a separate launch folder", () => {
  const root = fixture();
  try {
    const first = path.join(root, "project one"), second = path.join(root, "project two"), launch = path.join(root, "launch"), home = path.join(root, "home");
    for (const dir of [first, second, launch, path.join(home, ".kineti")]) fs.mkdirSync(dir, { recursive: true });
    append(first, "rr-weekly-one"); append(second, "rr-weekly-two");
    fs.writeFileSync(path.join(home, ".kineti/repo"), repo + "\n");
    const env: NodeJS.ProcessEnv = { ...process.env, HOME: home, KINETI_PROJECTS: `${first}:${second}`, KINETI_BUN: process.execPath };
    delete env.KINETI_WORKSPACE_ROOT;
    const result = Bun.spawnSync(["bash", path.join(repo, "scripts/weekly.sh")], { cwd: launch, env });
    expect(result.exitCode).toBe(0);
    expect(result.stdout.toString()).toContain("=== done ===");
    expect(cli(first, "kineti-memory-job.ts", ["verify-chain"]).exitCode).toBe(0);
    expect(cli(second, "kineti-memory-job.ts", ["verify-chain"]).exitCode).toBe(0);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test("PR reporting selects the exact author and marker and preserves stale and error states", async () => {
  const workflow = fs.readFileSync(path.join(repo, ".github/workflows/kineti-gate.yml"), "utf8");
  const lines = workflow.slice(workflow.lastIndexOf("          script: |") + "          script: |".length).split("\n");
  const end = lines.findIndex((line, i) => i > 0 && line.trim() !== "" && !line.startsWith("            "));
  const code = lines.slice(0, end < 0 ? undefined : end).map(line => line.replace(/^ {12}/, "")).join("\n");
  const execute = new Function("github", "context", "core", "process", "Buffer", `return (async () => {${code}\n})();`);
  const head = "a".repeat(40), other = "b".repeat(40), marker = "<!-- kineti-ci-report:v1 -->";
  for (const mode of ["current", "stale", "race", "missing", "api-error"]) {
    const writes: any[] = [], failures: string[] = [];
    let reads = 0;
    const github = {
      rest: {
        pulls: { get: async () => { reads++; if (mode === "api-error") throw new Error("offline"); return { data: { head: { sha: mode === "stale" || (mode === "race" && reads === 2) ? other : head } } }; } },
        issues: { listComments() {}, updateComment: async (value: any) => writes.push(value), createComment: async () => { throw new Error("must update only the matching owned comment"); } },
      },
      paginate: async () => [
        { id: 1, user: { login: "other[bot]", type: "Bot" }, body: marker + " spoof" },
        { id: 3, user: { login: "github-actions[bot]", type: "Bot" }, body: "Kineti CI Checks from another tool" },
        { id: 2, user: { login: "github-actions[bot]", type: "Bot" }, body: marker + " old owned report" },
      ],
    };
    const promise = execute(github, { repo: { owner: "test", repo: "test" }, issue: { number: 1 } }, { setFailed: (value: string) => failures.push(value) }, { env: { PR_EXPECTED_HEAD: head, PR_COMMENT_B64: mode === "missing" ? "" : Buffer.from("PASSED synthetic control").toString("base64") } }, Buffer);
    if (mode === "api-error") {
      await expect(promise).rejects.toThrow("offline"); expect(writes.length).toBe(0); expect(failures[0]).toContain("ERROR"); continue;
    }
    await promise;
    expect(writes[0].comment_id).toBe(2);
    expect(writes[0].body).toContain(`evaluated-head:${head}`);
    if (mode === "current") { expect(writes[0].body).toContain("state:current"); expect(failures.length).toBe(0); }
    else { expect(writes[0].body).not.toContain("PASSED"); expect(failures.length).toBe(1); }
  }
});

test("concurrent audit processes preserve every entry in a valid chain", async () => {
  const root = fixture();
  try {
    const runs = Array.from({ length: 24 }, (_, index) => Bun.spawn([process.execPath, path.join(repo, "bin/kineti-audit.ts"), "log", "--action", `case-${index}`], { cwd: root, env: { ...process.env, KINETI_MACHINE_DIR: path.join(root, ".kineti/machine") }, stdout: "ignore", stderr: "ignore" }));
    expect(await Promise.all(runs.map(run => run.exited))).toEqual(Array(24).fill(0));
    expect(cli(root, "kineti-audit.ts", ["verify"]).exitCode).toBe(0);
    expect(fs.readFileSync(path.join(root, ".kineti/machine/audit.log.jsonl"), "utf8").trim().split("\n").length).toBe(24);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});
