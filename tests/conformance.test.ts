import { describe, expect, test } from "bun:test";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { executeConformance } from "../bin/kineti-conformance.ts";

describe("portable gate conformance corpus", () => {
  test("both adapters consume the same fixture bytes and report decisions with reason codes", () => {
    const fixturePath = path.resolve(import.meta.dir, "..", "hosts", "conformance.v2.json");
    const bytes = fs.readFileSync(fixturePath);
    const digest = crypto.createHash("sha256").update(bytes).digest("hex");
    const local = executeConformance(bytes, "local-protocol");
    const secondHost = executeConformance(bytes, "second-host-stub");

    expect(local.fixture_version).toBe(2);
    expect(local.fixture_sha256).toBe(digest);
    expect(secondHost.fixture_sha256).toBe(digest);
    expect(local.outcome).toBe("unsupported");
    expect(local.results.every((item) => item.status === "passed" || item.status === "unsupported")).toBe(true);
    const localById = new Map(local.results.map((item) => [item.case_id, item]));
    expect(localById.get("authentic-control")?.actual).toEqual({ decision: "accepted", reason_code: "all-required-checks-passed" });
    expect(localById.get("wrong-command")?.actual).toEqual({ decision: "denied", reason_code: "inventory-substitution" });
    expect(localById.get("wrong-issuer")?.actual).toEqual({ decision: "denied", reason_code: "untrusted-owner" });
    expect(localById.get("wrong-key")?.actual).toEqual({ decision: "denied", reason_code: "untrusted-evaluator" });
    expect(localById.get("wrong-destination")?.status).toBe("unsupported");
    expect(secondHost.outcome).toBe("unsupported");
    expect(secondHost.results.every((item) => item.status === "unsupported" && item.actual.reason_code === "adapter-unavailable")).toBe(true);

    const repoRoot = path.resolve(import.meta.dir, "..");
    for (const adapter of ["local-protocol", "second-host-stub"] as const) {
      const run = Bun.spawnSync(["bun", "bin/kineti-conformance.ts", "--adapter", adapter], { cwd: repoRoot, stdout: "pipe", stderr: "pipe" });
      expect(run.exitCode).toBe(4);
      const report = JSON.parse(run.stdout.toString());
      expect(report.fixture_sha256).toBe(digest);
      expect(report.adapter).toBe(adapter);
    }
  });

  test("rejects malformed or version-mismatched fixture bytes", () => {
    expect(() => executeConformance(Buffer.from("not json"), "local-protocol")).toThrow("not valid JSON");
    expect(() => executeConformance(Buffer.from(JSON.stringify({ schema_version: 1, cases: [] })), "local-protocol")).toThrow("unsupported or malformed");
  });
});
