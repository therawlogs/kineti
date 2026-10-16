import { describe, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";

describe("portable gate conformance declarations", () => {
  test("fixture cases declare unsupported behavior instead of passing it", () => {
    const fixture = JSON.parse(fs.readFileSync(path.join(import.meta.dir, "fixtures", "gate-conformance.json"), "utf8"));
    expect(fixture.fixture_version).toBe(1);
    expect(fixture.cases.every((item: any) => ["implemented", "implemented-single-use-run-id", "reproduced-untrusted-output", "scripted-stub-only", "injected-failure-consumes-proof"].includes(item.support))).toBe(true);
    for (const id of ["wrong-command-valid-label", "forged-runner-output", "modified-parser", "changed-package-output", "wrong-issuer", "wrong-key", "wrong-destination", "revoked-proof", "replayed-proof", "second-host-stub"]) {
      expect(fixture.cases.some((item: any) => item.id === id)).toBe(true);
    }
    expect(fixture.cases.find((item: any) => item.id === "forged-runner-output").expected).toBe("unsupported");
    expect(fixture.cases.find((item: any) => item.id === "revoked-proof").expected).toBe("deny");
    const stub = Bun.spawnSync(["bash", path.join(import.meta.dir, "fixtures", "second-host-stub.sh")], { stdout: "pipe" });
    expect(stub.exitCode).toBe(78);
    expect(new TextDecoder().decode(stub.stdout)).toContain("unsupported");
  });
});
