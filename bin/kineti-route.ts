#!/usr/bin/env bun
/** Machine-readable routing operations. No live operation is implied by planning or importing policy. */
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { strictOptions } from "./strict-options.ts";
import { writeJson } from "./lib.ts";
import { withAccountingLock } from "./spending-state.ts";
import {
  classify,
  listHostModels,
  hostName,
  routingPath,
  validateManifest,
  validatePolicy,
  loadConfig,
  planRoute,
  execute,
  settle,
  verifyRun,
  type AccessManifest,
} from "../src/routing/index.ts";

async function main(raw: string[]) {
  const [cmd, sub] = raw;
  if (!cmd || cmd === "--help" || cmd === "-h") {
    console.log(
      JSON.stringify({
        commands: [
          "classify --task-file FILE",
          "manifest declare|revoke --host HOST --model MODEL",
          "manifest import|validate --host HOST --file FILE",
          "manifest list --host HOST",
          "manifest refresh --host HOST --consent",
          "policy import|validate --host HOST --file FILE",
          "plan|execute|probe --host HOST --task-file FILE",
          "verify --run-id UUID --task-file FILE [--check-run-id UUID]",
          "settle --run-id UUID --usd USD --source SOURCE",
        ],
        live_requirements:
          "explicit --consent, owner policy cap, model permission, supported estimate and available credentials; probe also needs --model",
        guarantee:
          "requested model is separate from resolved evidence; provider invoice is not capped or independently verified",
      }),
    );
    return;
  }
  const nested = cmd === "manifest" || cmd === "policy";
  const specs: Record<string, string[]> = {
    classify: ["--task-file", "--default-class"],
    settle: ["--run-id", "--usd", "--source"],
    verify: ["--run-id", "--check-run-id", "--task-file"],
    plan: ["--host", "--task-file"],
    execute: ["--host", "--task-file", "--session-id"],
    probe: ["--host", "--task-file", "--session-id", "--model"],
  };
  const values = nested
    ? sub === "import" || sub === "validate"
      ? ["--host", "--file"]
      : sub === "declare" || sub === "revoke"
        ? ["--host", "--model"]
        : ["--host"]
    : (specs[cmd] ?? []);
  const flags =
    cmd === "execute" ||
    cmd === "probe" ||
    (cmd === "manifest" && sub === "refresh")
      ? ["--consent"]
      : [];
  const args = strictOptions(raw.slice(nested ? 2 : 1), values, flags, [], 0);
  const value = (key: string) => {
    const i = args.indexOf(key);
    return i < 0 ? undefined : args[i + 1];
  };
  const required = (key: string) => {
    const v = value(key);
    if (v === undefined) throw new Error(`${key} required`);
    return v;
  };
  const root = process.cwd(),
    output = (v: unknown) => console.log(JSON.stringify(v));
  const text = () => {
    const p = path.resolve(required("--task-file"));
    if (!fs.statSync(p).isFile() || fs.statSync(p).size > 1024 * 1024)
      throw new Error("task must be a file up to 1 MiB");
    return fs.readFileSync(p, "utf8");
  };
  if (cmd === "classify") {
    const fallback = value("--default-class");
    if (fallback && !["code", "plan", "chat", "fix"].includes(fallback))
      throw new Error("invalid default class");
    output(classify(text(), fallback as any));
    return;
  }
  if (cmd === "settle") {
    settle(
      root,
      required("--run-id"),
      Number(required("--usd")),
      required("--source"),
    );
    output({
      status: "reported",
      run_id: value("--run-id"),
      provider_invoice_verified: false,
    });
    return;
  }
  if (cmd === "verify") {
    const result = verifyRun(
      root,
      required("--run-id"),
      value("--check-run-id"),
      text(),
    );
    output(result);
    if (!result.accepted) process.exitCode = 1;
    return;
  }
  const host = hostName(required("--host"));
  if (cmd === "policy" || cmd === "manifest") {
    if (sub === "validate" || sub === "import") {
      const data = JSON.parse(fs.readFileSync(required("--file"), "utf8"));
      const validated =
        cmd === "manifest" ? validateManifest(data) : validatePolicy(data);
      if (validated.host !== host)
        throw new Error("foreign host configuration");
      if (sub === "import")
        withAccountingLock(root, () =>
          writeJson(routingPath(root, host, cmd), validated),
        );
      output({
        status: sub === "import" ? "saved" : "valid",
        host,
        revision: validated.revision,
      });
      return;
    }
    if (cmd !== "manifest") throw new Error("unknown policy command");
    const file = routingPath(root, host, "manifest");
    const read = (): AccessManifest =>
      fs.existsSync(file)
        ? validateManifest(JSON.parse(fs.readFileSync(file, "utf8")))
        : {
            schema_version: 1,
            revision: crypto.randomUUID(),
            host,
            ttl_hours: 24,
            models: [],
          };
    if (sub === "list") {
      output(read());
      return;
    }
    if (sub === "declare" || sub === "revoke") {
      const model = required("--model");
      withAccountingLock(root, () => {
        const m = read();
        let row = m.models.find((v) => v.id === model);
        if (!row) {
          row = { id: model, provenance: ["user-declared"] };
          m.models.push(row);
        }
        if (sub === "revoke") row.revoked = true;
        m.revision = crypto.randomUUID();
        validateManifest(m);
        writeJson(file, m);
      });
      output({
        status: sub === "revoke" ? "revoked" : "declared",
        host,
        model,
      });
      return;
    }
    if (sub === "refresh") {
      if (!args.includes("--consent"))
        throw new Error("catalog inspection requires --consent");
      const policyFile = routingPath(root, host, "policy");
      const binary = fs.existsSync(policyFile)
        ? validatePolicy(JSON.parse(fs.readFileSync(policyFile, "utf8")))
            .executable
        : undefined;
      const destinations =
        host === "codex" ? ["api.openai.com", "chatgpt.com"] : ["models.dev"];
      for (const domain of destinations) {
        const audit = spawnSync(
          "bun",
          [
            path.resolve(import.meta.dir, "kineti-egress.ts"),
            "record",
            "--host",
            domain,
            "--description",
            `Owner-consented ${host} catalog inspection; no task or secret values recorded`,
          ],
          { cwd: root, encoding: "utf8" },
        );
        if (audit.status !== 0)
          throw new Error("catalog egress recording unavailable");
      }
      const catalog = await listHostModels(root, host, binary),
        models = catalog.models;
      withAccountingLock(root, () => {
        const m = read();
        for (const model of models) {
          let row = m.models.find((v) => v.id === model);
          if (!row) {
            row = { id: model, provenance: [] };
            m.models.push(row);
          }
          row.provenance = [
            ...new Set([...row.provenance, "host-listed" as const]),
          ];
          row.listed_at = new Date().toISOString();
        }
        m.revision = crypto.randomUUID();
        writeJson(file, m);
      });
      output({
        status: "host-listed",
        host,
        count: models.length,
        source: catalog.source,
        host_version: catalog.host_version,
        access_confirmed: false,
      });
      return;
    }
    throw new Error("unknown manifest command");
  }
  if (!["plan", "execute", "probe"].includes(cmd))
    throw new Error("unknown routing command");
  const { manifest, policy } = loadConfig(root, host),
    task = text();
  if (cmd === "plan") {
    output(planRoute(root, host, task, manifest, policy));
    return;
  }
  if (!args.includes("--consent"))
    throw new Error("live execution requires --consent");
  console.error(
    "Owner cap limits admission and further work using recorded/estimated costs. It cannot guarantee the provider invoice or prevent an in-flight overrun. No automatic retries.",
  );
  const result = await execute(root, host, task, manifest, policy, {
    consent: true,
    session_id: value("--session-id"),
    probe_model: cmd === "probe" ? required("--model") : undefined,
  });
  output(result);
  if (result.outcome !== "accepted") process.exitCode = 1;
}
if (import.meta.main) {
  try {
    await main(process.argv.slice(2));
  } catch (error) {
    console.log(
      JSON.stringify({ status: "blocked", reason: (error as Error).message }),
    );
    process.exitCode = 2;
  }
}
