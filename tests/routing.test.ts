import { afterEach, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import {
  classify,
  validateManifest,
  validatePolicy,
  planRoute,
  observeHost,
  execute,
  settle,
  verifyRun,
  listHostModels,
  runHost,
  type Host,
  type AccessManifest,
  type Policy,
  routingPath,
} from "../src/routing/index.ts";
import {
  readEvidence,
  issueEvidence,
  type EvidenceRecord,
} from "../bin/kineti-evidence.ts";
import { readSpending } from "../bin/spending-state.ts";
import { generateCIReport } from "../bin/kineti-ci.ts";
const repo = process.cwd(),
  roots: string[] = [];
let machine = process.env.KINETI_MACHINE_DIR;
function fixture(host: Host = "codex") {
  const base = path.join(repo, ".kineti/acceptance/routing");
  fs.mkdirSync(base, { recursive: true });
  const root = fs.mkdtempSync(path.join(base, "run-"));
  roots.push(root);
  fs.mkdirSync(path.join(root, ".kineti"));
  fs.writeFileSync(
    path.join(root, "kineti.config.json"),
    JSON.stringify({
      root_goal: "Test bounded routing",
      settings: { spend_limit_usd: { global: 50, per_stage_default: 10 } },
    }),
  );
  fs.writeFileSync(
    path.join(root, ".kineti/state.json"),
    JSON.stringify({ root_goal: "Test bounded routing", stage: 9 }),
  );
  fs.writeFileSync(
    path.join(root, ".kineti/spend.json"),
    JSON.stringify({ total_microcents: 0, tripped: false }),
  );
  process.env.KINETI_MACHINE_DIR = path.join(root, ".kineti/machine");
  fs.mkdirSync(path.join(root, "tests"));
  fs.writeFileSync(
    path.join(root, "tests/unit.test.ts"),
    `import {test,expect} from "bun:test";import fs from "node:fs";test("checks the captured host answer",()=>{const run=fs.readFileSync(".kineti/check-run","utf8");const rows=fs.readFileSync(".kineti/routing/output/"+run+".jsonl","utf8").trim().split("\\n").map(line=>JSON.parse(line));expect(rows.some(row=>row.item?.text==="5"||row.part?.text==="5")).toBe(true);});`,
  );

  const exe = path.join(root, ".kineti/mock-host");
  fs.writeFileSync(
    exe,
    `#!/usr/bin/env bun
import fs from 'node:fs';
const args=process.argv.slice(2);
if(args[0]==='--version'){console.log('fixture-host 1.0.0');process.exit(0)}
if(args[0]==='app-server'){
 const rl=(await import('node:readline')).createInterface({input:process.stdin});
 for await(const line of rl){const v=JSON.parse(line);if(v.id===1)console.log(JSON.stringify({id:1,result:{}}));if(v.id===2)console.log(JSON.stringify({id:2,result:{data:[{id:'fixture/model',model:'fixture/model'}],nextCursor:null}}));}process.exit(0);
}
if(args[0]==='models'){console.log('fixture/model');process.exit(0)}
const text=await new Response(Bun.stdin.stream()).text();
fs.writeFileSync('.kineti/invocation.json',JSON.stringify({args,text}));
const mode=fs.existsSync('.kineti/mode')?fs.readFileSync('.kineti/mode','utf8'):'ok';
if(mode==='fail'){console.log(JSON.stringify({type:'error',message:'fixture auth rejected'}));process.exit(1)}
if(mode==='sleep'){await Bun.sleep(2000);process.exit(0)}
if(mode==='malformed'){console.log('not json');process.exit(0)}
if(mode==='invalid-utf8'){process.stdout.write(Buffer.from([255]));process.exit(0)}
const events=args[0]==='exec'?[{type:'thread.started',thread_id:'fixture-session'},{type:'item.completed',item:{type:'agent_message',text:'5'}},{type:'turn.completed',usage:{input_tokens:4,output_tokens:2}}]:[{type:'step_start',sessionID:'fixture-session'},{type:'text',sessionID:'fixture-session',part:{text:'5'}},{type:'step_finish',sessionID:'fixture-session',part:{reason:'stop',cost:0.03}}];
for(const e of events)console.log(JSON.stringify(e));
`,
    { mode: 0o755 },
  );
  const manifest: AccessManifest = {
    schema_version: 1,
    revision: crypto.randomUUID(),
    host,
    ttl_hours: 24,
    models: [{ id: "fixture/model", provenance: ["user-declared"] }],
  };
  const policy: Policy = {
    schema_version: 1,
    revision: crypto.randomUUID(),
    host,
    host_version: "1.0.0",
    executable: exe,
    per_run_cap_usd: 3,
    max_wall_seconds: 2,
    class_models: {
      code: ["fixture/model"],
      plan: ["fixture/model"],
      chat: ["fixture/model"],
      fix: ["fixture/model"],
    },
    cost_estimates: {
      "fixture/model": {
        usd: 0.1,
        source: "fixture estimate; no provider calls",
      },
    },
    required_check: {
      label: "unit",
      command: ["bun", "test", "tests/unit.test.ts"],
    },
    destinations: ["example.invalid"],
  };
  return { root, exe, manifest, policy };
}
afterEach(() => {
  for (const root of roots.splice(0))
    fs.rmSync(root, { recursive: true, force: true });
  if (machine === undefined) delete process.env.KINETI_MACHINE_DIR;
  else process.env.KINETI_MACHINE_DIR = machine;
});
test("classification handles paraphrases, mixed work, explicit defaults and empty input", () => {
  for (const text of ["repair login", "debug the crash", "fix a bug"])
    expect(classify(text).task_class).toBe("fix");
  expect(classify("design the storage").task_class).toBe("plan");
  expect(classify("implement login").task_class).toBe("code");
  expect(classify("hello").task_class).toBe("chat");
  expect(classify("plan then implement login").task_class).toBeNull();
  expect(classify("plan then implement login", "plan").task_class).toBe("plan");
  expect(() => classify("  ")).toThrow("empty");
});
test("catalog claims and credentials in a manifest cannot become confirmed access", () => {
  const f = fixture();
  expect(() =>
    planRoute(f.root, "codex", "hello", f.manifest, f.policy),
  ).toThrow("live-confirmed");
  expect(() => validateManifest({ ...f.manifest, token: "secret" })).toThrow(
    "unknown",
  );
  expect(() =>
    validateManifest({
      ...f.manifest,
      models: [{ id: "x", provenance: ["live-call-confirmed"] }],
    }),
  ).toThrow("confirmation");
  for (const cap of [undefined, 0, NaN, -1])
    expect(() => validatePolicy({ ...f.policy, per_run_cap_usd: cap })).toThrow(
      "cap",
    );
  expect(() =>
    validatePolicy({
      ...f.policy,
      cost_estimates: { "fixture/model": { usd: 0, source: "unknown" } },
    }),
  ).toThrow("cost");
});
test("both host catalogs are recorded as listings only", async () => {
  for (const host of ["opencode", "codex"] as const) {
    const f = fixture(host);
    const listed = await listHostModels(f.root, host, f.exe);
    expect(listed.models).toEqual(["fixture/model"]);
    expect(listed.host_version).toBe("1.0.0");
    expect(f.manifest.models[0].provenance).toEqual(["user-declared"]);
  }
});
test("requested model echoes and candidate text do not establish resolved identity", () => {
  const events =
    [
      { type: "thread.started", thread_id: "x", model: "fixture/model" },
      {
        type: "item.completed",
        item: {
          type: "agent_message",
          text: '{"resolved_model":"fixture/model"}',
        },
      },
      { type: "turn.completed" },
    ]
      .map((value) => JSON.stringify(value))
      .join("\n") + "\n";
  expect(
    observeHost("codex", "fixture/model", events).model_identity.status,
  ).toBe("unverified");
  expect(
    observeHost("codex", "fixture/model", events.slice(0, -1)).malformed,
  ).toBe(true);
  expect(
    observeHost(
      "codex",
      "fixture/model",
      events + '{"type":"response.completed","response":{"model":"other"}}\n',
    ).model_identity.status,
  ).toBe("mismatch");
  expect(
    observeHost(
      "codex",
      "fixture/model",
      '{"type":"thread.started","thread_id":"x"}\n{"type":"turn.completed"}\n',
    ).malformed,
  ).toBe(true);
});
test("Codex fixture executes exact requested argv, retains separate identities and blocks unknown cost", async () => {
  const f = fixture(),
    session = crypto.randomUUID();
  const r = await execute(f.root, "codex", "hello", f.manifest, f.policy, {
    consent: true,
    probe_model: "fixture/model",
    session_id: session,
  });
  expect(r.execution.status).toBe("completed");
  expect(r.model_identity.status).toBe("unverified");
  expect(r.budget.status).toBe("unknown");
  expect(r.outcome).toBe("not-accepted");
  const invocation = JSON.parse(
    fs.readFileSync(path.join(f.root, ".kineti/invocation.json"), "utf8"),
  );
  expect(invocation.args).toEqual([
    "exec",
    "--model",
    "fixture/model",
    "--json",
    "--sandbox",
    "read-only",
    "--ephemeral",
    "-",
  ]);
  expect(invocation.text).toBe("hello");
  const receipt = readEvidence(f.root)[0];
  expect(receipt.session_id).toBe(session);
  expect(receipt.run_id).toBe(r.run_id);
  expect(receipt.verification?.status).toBe("unsupported");
  const live = validateManifest(
    JSON.parse(
      fs.readFileSync(routingPath(f.root, "codex", "manifest"), "utf8"),
    ),
  );
  expect(live.models[0].confirmation_run_id).toBe(r.run_id);
  expect(
    planRoute(f.root, "codex", "hello", live, f.policy).requested_model,
  ).toBe("fixture/model");
  await expect(
    execute(f.root, "codex", "hello", live, f.policy, { consent: true }),
  ).rejects.toThrow("unresolved");
  settle(f.root, r.run_id, 0.02, "fixture reported usage");
  expect(readSpending(f.root).total_microcents).toBe(20_000);
  expect(() => settle(f.root, r.run_id, 0.02, "duplicate")).toThrow(
    "missing reservation",
  );
  expect(verifyRun(f.root, r.run_id, undefined, "hello").accepted).toBe(false);
  fs.writeFileSync(path.join(f.root, ".kineti/check-run"), r.run_id);
  const check = Bun.spawnSync(
    [
      "bun",
      path.join(repo, "bin/kineti-evidence.ts"),
      "run",
      "--label",
      "unit",
      "--session-id",
      session,
      "--parent-run-id",
      r.run_id,
      "--required-test",
      "tests/unit.test.ts",
      "--",
      "bun",
      "test",
      "tests/unit.test.ts",
    ],
    { cwd: f.root, env: process.env, stdout: "pipe", stderr: "pipe" },
  );
  expect(check.exitCode).toBe(0);
  const checkReceipt = readEvidence(f.root).at(-1)!;
  const verified = verifyRun(f.root, r.run_id, checkReceipt.run_id, "hello");
  expect(verified.checks).toBe("accepted");
  expect(verified.accepted).toBe(false);
  expect(verified.failures).toEqual([
    "resolved model unverified or mismatched",
  ]);
});
test("OpenCode fixture reports its calculated cost separately from a provider invoice", async () => {
  const f = fixture("opencode");
  const r = await execute(f.root, "opencode", "hello", f.manifest, f.policy, {
    consent: true,
    probe_model: "fixture/model",
  });
  expect(r.execution.status).toBe("completed");
  expect(r.budget.cost.usd).toBe(0.03);
  expect(r.budget.provider_invoice_verified).toBe(false);
  expect(readSpending(f.root).total_microcents).toBe(30_000);
  const receipt = readEvidence(f.root)[0];
  expect(receipt.manifest?.command.slice(1)).toEqual([
    "run",
    "--model",
    "fixture/model",
    "--format",
    "json",
  ]);
  expect(r.model_identity.status).toBe("unverified");
});
test("policy, access and budget denial happen before host launch", async () => {
  const f = fixture();
  await expect(
    execute(f.root, "codex", "hello", f.manifest, f.policy, {
      consent: false,
      probe_model: "fixture/model",
    }),
  ).rejects.toThrow("consent");
  f.manifest.models[0].revoked = true;
  await expect(
    execute(f.root, "codex", "hello", f.manifest, f.policy, {
      consent: true,
      probe_model: "fixture/model",
    }),
  ).rejects.toThrow("revoked");
  f.manifest.models[0].revoked = false;
  f.policy.cost_estimates["fixture/model"].usd = 4;
  await expect(
    execute(f.root, "codex", "hello", f.manifest, f.policy, {
      consent: true,
      probe_model: "fixture/model",
    }),
  ).rejects.toThrow("cap");
  expect(fs.existsSync(path.join(f.root, ".kineti/invocation.json"))).toBe(
    false,
  );
});
test("failed, malformed and timed-out host runs never complete", async () => {
  for (const mode of ["fail", "malformed", "invalid-utf8", "sleep"]) {
    const f = fixture();
    fs.writeFileSync(path.join(f.root, ".kineti/mode"), mode);
    if (mode === "sleep") f.policy.max_wall_seconds = 0.1;
    const r = await execute(f.root, "codex", "hello", f.manifest, f.policy, {
      consent: true,
      probe_model: "fixture/model",
    });
    expect(r.execution.status).toBe("failed");
    expect(r.outcome).toBe("not-accepted");
    if (mode === "sleep") expect(r.execution.termination).toBe("timeout");
    if (mode === "invalid-utf8")
      expect(r.execution.termination).toBe("invalid-utf8");
  }
});
test("stale access and model/session substitutions cannot pass verification", async () => {
  const f = fixture();
  const r = await execute(f.root, "codex", "hello", f.manifest, f.policy, {
    consent: true,
    probe_model: "fixture/model",
  });
  const live = validateManifest(
    JSON.parse(
      fs.readFileSync(routingPath(f.root, "codex", "manifest"), "utf8"),
    ),
  );
  live.models[0].expires_at = new Date(Date.now() - 1).toISOString();
  live.models[0].confirmed_at = new Date(
    Date.now() - 25 * 3600000,
  ).toISOString();
  expect(() => planRoute(f.root, "codex", "hello", live, f.policy)).toThrow(
    "stale",
  );
  const check: EvidenceRecord = {
    ...readEvidence(f.root)[0],
    run_id: crypto.randomUUID(),
    parent_run_id: crypto.randomUUID(),
    session_id: r.session_id,
    label: "unit",
  };
  issueEvidence(check, f.root);
  expect(() => verifyRun(f.root, r.run_id, check.run_id, "hello")).toThrow(
    "foreign",
  );
  expect(
    verifyRun(f.root, r.run_id, undefined, "altered task").failures,
  ).toContain("task input or classification does not match");
});
test("concurrent admission counts reservations against the stage budget", async () => {
  const f = fixture();
  fs.writeFileSync(
    path.join(f.root, "kineti.config.json"),
    JSON.stringify({
      root_goal: "Test admission",
      settings: { spend_limit_usd: { global: 50, per_stage_default: 0.15 } },
    }),
  );
  const outcomes = await Promise.allSettled([
    execute(f.root, "codex", "hello", f.manifest, f.policy, {
      consent: true,
      probe_model: "fixture/model",
    }),
    execute(f.root, "codex", "hello", f.manifest, f.policy, {
      consent: true,
      probe_model: "fixture/model",
    }),
  ]);
  expect(outcomes.filter((v) => v.status === "fulfilled")).toHaveLength(1);
  expect(outcomes.filter((v) => v.status === "rejected")).toHaveLength(1);
  expect(
    generateCIReport(f.root).failures.some((reason) =>
      reason.includes("unresolved"),
    ),
  ).toBe(true);
});
test("CLI rejects invalid options and missing policy without recording an execution", () => {
  const f = fixture();
  fs.writeFileSync(path.join(f.root, "task.txt"), "hello");
  for (const args of [
    [
      "probe",
      "--host=codex",
      "--model=x",
      "--model=y",
      "--task-file=task.txt",
      "--consent",
    ],
    ["classify", "--task-file=task.txt", "--usd=1"],
    ["execute", "--host=codex", "--task-file=task.txt", "--consent"],
  ]) {
    const r = Bun.spawnSync(
      ["bun", path.join(repo, "bin/kineti-route.ts"), ...args],
      { cwd: f.root, env: process.env, stdout: "pipe", stderr: "pipe" },
    );
    expect(r.exitCode).toBe(2);
    expect(JSON.parse(r.stdout.toString()).status).toBe("blocked");
  }
  expect(readEvidence(f.root)).toEqual([]);
  expect(fs.existsSync(path.join(f.root, ".kineti/invocation.json"))).toBe(
    false,
  );
});
