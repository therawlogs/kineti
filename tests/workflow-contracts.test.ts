import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

describe("workflow contracts", () => {
  test("data intake requires approved storyboard and mock-data shape first", () => {
    const design = read("skills/design/SKILL.md");
    const diagnose = read("skills/diagnose/SKILL.md");
    expect(design).toContain("Get human approval of the storyboard and mock-data");
    expect(diagnose).toContain("If either is absent, stop and route back");
    expect(diagnose).toContain("Get approval for real-data use before");
  });

  test("continuity and product mode require explicit choices", () => {
    const spec = read("skills/spec/SKILL.md");
    expect(spec).toContain("continue\n   within contract, revise the spec within the same goal, or stop and start");
    expect(spec).toContain("protected-action checks), or `both`");
    expect(spec).toContain("Never edit the locked\n   goal or silently reinterpret it");
  });

  test("workflow index marks untested adapters and routes unsupported", () => {
    const workflows = read("docs/WORKFLOW-CONTRACTS.md");
    expect(workflows).toContain("01 → 02 → 03 → 04 → 06 → 07 → 11 → 12 → 13");
    expect(workflows).toContain("GitHub Action,");
    expect(workflows).toContain("Codex, Claude Code,");
    expect(workflows).toContain("Untested integrations are `stub` or `unsupported`");
    expect(workflows).toContain("Natural-language skill routing is host-specific");
  });
});
