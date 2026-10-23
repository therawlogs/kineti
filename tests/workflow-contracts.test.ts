import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const read = (path: string) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

describe("workflow contracts", () => {
  test("design approval precedes real-data intake and diagnosis stays evidence-led", () => {
    const design = read("skills/design/SKILL.md");
    const diagnose = read("skills/diagnose/SKILL.md");
    expect(design).toContain("Get human approval of the storyboard and mock-data");
    expect(diagnose).toContain("requires a separately approved purpose");
    expect(diagnose).toContain("Time order alone does not prove cause");
    expect(diagnose).not.toContain("Convert to dollars (P90");
  });

  test("continuity and product mode require explicit choices", () => {
    const spec = read("skills/spec/SKILL.md");
    expect(spec).toContain("continue\n   within contract, revise the spec within the same goal, or stop and start");
    expect(spec).toContain("`both`, with no preselected default");
    expect(spec).toContain("Installing a package does not activate a project");
    expect(spec).toContain("Never edit the locked\n   goal or silently reinterpret it");
    expect(spec).not.toContain("No file under `src/` may exist");
  });

  test("workflow index orders decisions and separates host capabilities", () => {
    const workflows = read("docs/WORKFLOW-CONTRACTS.md");
    expect(workflows).toContain("explicit mode selection → 01 discovery → 02 design");
    expect(workflows).toContain("installation, workflow guidance, background behavior,");
    expect(workflows).toContain("Slash commands may be optional");
    expect(workflows).toContain("time order or dependency alone does not establish cause");
  });

  test("host guidance does not require slash commands or claim enforcement", () => {
    for (const host of ["cursor", "codex", "claude", "fx", "opencode", "gemini", "antigravity"]) {
      const instructions = read(`hooks/${host}.txt`);
      expect(instructions).not.toContain("run /kineti-officehours first");
      expect(instructions).toContain("guidance");
    }
  });

  test("capability declaration is versioned and has no unsupported evidence claims", () => {
    const contract = JSON.parse(read("hosts/capabilities.v1.json"));
    const expected = ["installation", "workflow_guidance", "background_behavior", "natural_language_routing", "protected_action"];
    expect(contract.schema_version).toBe(1);
    expect(contract.capabilities).toEqual(expected);
    expect(contract.surfaces.map((surface: { id: string }) => surface.id)).toEqual([
      "github-action", "codex", "claude-code", "mcp", "cursor", "antigravity",
    ]);
    for (const surface of contract.surfaces) {
      expect(Object.keys(surface.claims).sort()).toEqual([...expected].sort());
      for (const claim of Object.values(surface.claims) as Array<{ status: string; reason: string; evidence?: string[] }>) {
        expect(["supported", "stub", "unsupported"]).toContain(claim.status);
        expect(claim.reason.length).toBeGreaterThan(0);
        if (claim.status === "supported") expect(claim.evidence?.length ?? 0).toBeGreaterThan(0);
      }
    }
  });
});
