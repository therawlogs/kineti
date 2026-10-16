import { describe, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";

describe("portable gate conformance declarations", () => {
  test("fixture cases declare unsupported behavior instead of passing it", () => {
    const fixture = JSON.parse(fs.readFileSync(path.join(import.meta.dir, "fixtures", "gate-conformance.json"), "utf8"));
    expect(fixture.fixture_version).toBe(1);
    expect(fixture.cases.every((item: any) => ["implemented", "implemented-single-use-run-id", "not-implemented-no-revocation-store", "scripted-stub-only", "not-yet-exercised-by-fixture"].includes(item.support))).toBe(true);
    for (const id of ["wrong-command-valid-label", "forged-runner-output", "modified-parser", "changed-package-output", "wrong-issuer", "wrong-key", "wrong-destination", "revoked-proof", "replayed-proof", "second-host-stub"]) {
      expect(fixture.cases.some((item: any) => item.id === id)).toBe(true);
    }
    const stub = Bun.spawnSync(["bash", path.join(import.meta.dir, "fixtures", "second-host-stub.sh")], { stdout: "pipe" });
    expect(stub.exitCode).toBe(78);
    expect(new TextDecoder().decode(stub.stdout)).toContain("unsupported");
  });
});
