import { afterAll, describe, expect, test } from "bun:test";
import { readFile, rm } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const reportPath = resolve(".kineti/containment-test-report.json");

afterAll(async () => {
  await rm(reportPath, { force: true });
});

describe("containment comparison snapshot", () => {
  test("records missing runtime execution as unsupported, never pass", async () => {
    const run = spawnSync("bash", ["scripts/containment-evaluation.sh", reportPath], {
      encoding: "utf8",
    });
    expect(run.status).toBe(0);
    const report = JSON.parse(await readFile(reportPath, "utf8"));
    expect(report.schema_version).toBe(1);
    expect(report.evaluation).toBe("strands-box-vs-openshell");
    expect(report.status).toBe("unsupported");
    expect(report.scenarios.length).toBeGreaterThanOrEqual(7);
    for (const scenario of report.scenarios) {
      expect(scenario.strands_box).toBe("unsupported");
      expect(scenario.openshell).toBe("unsupported");
      expect(scenario.evidence).toEqual([]);
    }
  });
});
