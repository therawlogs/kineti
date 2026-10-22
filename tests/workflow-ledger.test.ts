import { describe, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import {
  addConflict, createOutcomeContract, isApproved, isCurrentApproved, recordApproval, recordArtifact, resolveConflict, validateOutcomeContract,
  type WorkflowLedger, type WorkflowStep,
} from "../bin/kineti-workflow.ts";

const REPO = path.resolve(import.meta.dir, "..");
function makeProject(mode: string | null = "harness") {
  const base = path.join(REPO, ".kineti", "workflow-ledger-tests");
  fs.mkdirSync(base, { recursive: true });
  const cwd = fs.mkdtempSync(path.join(base, "project-"));
  fs.mkdirSync(path.join(cwd, ".kineti"), { recursive: true });
  fs.writeFileSync(path.join(cwd, ".kineti", "state.json"), JSON.stringify({
    version: 1, project: "sample", mode, root_goal: "Build a sample", root_goal_locked_at: "2026-10-16T00:00:00.000Z", stage: 1, gates: {}, history: [],
  }));
  return cwd;
}
function add(ledger: WorkflowLedger, step: WorkflowStep, bytes: string, n: number) {
  return recordArtifact(ledger, step, `docs/${step}.md`, Buffer.from(bytes), `2026-10-16T00:00:${String(n).padStart(2, "0")}.000Z`);
}
function approve(ledger: WorkflowLedger, step: WorkflowStep) {
  const digest = [...ledger.artifacts].reverse().find((item) => item.step === step)!.sha256;
  return recordApproval(ledger, step, `APPROVE ${digest}`, "2026-10-16T01:00:00.000Z");
}
function run(cwd: string, args: string[]) {
  const result = Bun.spawnSync(["bun", path.join(REPO, "bin", "kineti-workflow.ts"), ...args], { cwd, stdout: "pipe", stderr: "pipe" });
  return { status: result.exitCode ?? -1, out: result.stdout.toString(), err: result.stderr.toString() };
}
function approvedInputs(cwd: string) {
  const state = JSON.parse(fs.readFileSync(path.join(cwd, ".kineti", "state.json"), "utf8"));
  const ledger: WorkflowLedger = { schema_version: 1, project: "sample", artifacts: [], approvals: [], conflicts: [] };
  fs.mkdirSync(path.join(cwd, "docs"), { recursive: true });
  const records: Array<[WorkflowStep, string]> = [
    ["discovery", "discovery source bytes\n"], ["design", "design source bytes\n"],
    ["no-data", "no-data source bytes\n"], ["architecture", "architecture source bytes\n"],
  ];
  records.forEach(([step, content]) => {
    const source = `docs/${step}.md`;
    fs.writeFileSync(path.join(cwd, source), content);
    const artifact = recordArtifact(ledger, step, source, Buffer.from(content));
    recordApproval(ledger, step, `APPROVE ${artifact.sha256}`);
  });
  fs.writeFileSync(path.join(cwd, ".kineti", "workflow.json"), JSON.stringify(ledger, null, 2));
  return { state, ledger };
}
function completedOutcome(draft: Record<string, any>) {
  const sections = draft.sections;
  sections.personas = [{ id: "persona-builder", name: "Builder", needs: ["See the accepted task and its checks"] }];
  sections.design = { journey: "Open the project, complete the task, and review the result.", mock_data_shape: "Synthetic local example with no real customer data.", states: ["empty", "error", "complete"] };
  sections.data = { kind: "no-data", inputs: [] };
  sections.features = [{ id: "feature-task", persona_ids: ["persona-builder"], outcome: "The user can verify the agreed result.", tests: ["task-check"] }];
  sections.architecture = { summary: "Local implementation with explicit verification.", decisions: [] };
  sections.tools = { reviewed: true, decisions: [] };
  sections.tests = [{ id: "task-check", command: ["bun", "test", "tests/"], kind: "test", expected_result: "Required tests pass with no skips." }];
  sections.roadmap = [{ id: "task-1", title: "Build the accepted feature", feature_ids: ["feature-task"], priority_id: "task-1" }];
  sections.priorities = { confirmed: true, decision_ref: "approval in this outcome artifact", items: [{ id: "task-1", rank: 1 }] };
  sections.decision_graph = [];
  sections.unknowns = { reviewed: true, items: ["Live host behavior remains unverified."] };
  return draft;
}

describe("durable workflow contract", () => {
  test("requires selected mode and a locked goal before workflow state can be written", () => {
    const cwd = makeProject(null);
    try {
      fs.writeFileSync(path.join(cwd, "brief.md"), "A short project brief\n");
      const result = run(cwd, ["record", "--step", "discovery", "--file", "brief.md"]);
      expect(result.status).toBe(2);
      expect(result.err).toContain("mode is pending");
      expect(fs.existsSync(path.join(cwd, ".kineti", "workflow.json"))).toBe(false);
    } finally { fs.rmSync(path.dirname(cwd), { recursive: true, force: true }); }
  });

  test("records exact artifact bytes, gates order on explicit approvals, and invalidates downstream approvals", () => {
    const ledger: WorkflowLedger = { schema_version: 1, project: "sample", artifacts: [], approvals: [], conflicts: [] };
    add(ledger, "discovery", "goal, personas, constraints\n", 1);
    expect(() => add(ledger, "design", "storyboard\n", 2)).toThrow("blocked until 'discovery' is approved");
    approve(ledger, "discovery");
    expect(isApproved(ledger, "discovery")).toBe(true);
    add(ledger, "design", "journey and mock-data\n", 2);
    approve(ledger, "design");
    add(ledger, "no-data", "No real data is needed for this project.\n", 3);
    approve(ledger, "no-data");
    add(ledger, "architecture", "features, tools, costs, tests\n", 4);
    approve(ledger, "architecture");
    add(ledger, "outcome", "approved scope and priorities\n", 5);
    approve(ledger, "outcome");
    expect(isApproved(ledger, "data")).toBe(true);
    expect(isApproved(ledger, "outcome")).toBe(true);

    add(ledger, "discovery", "revised exact goal bytes\n", 6);
    expect(isApproved(ledger, "discovery")).toBe(false);
    expect(isApproved(ledger, "design")).toBe(false);
    expect(isApproved(ledger, "architecture")).toBe(false);
    expect(isApproved(ledger, "outcome")).toBe(false);
  });

  test("approval requires the exact displayed digest phrase and parent versions", () => {
    const ledger: WorkflowLedger = { schema_version: 1, project: "sample", artifacts: [], approvals: [], conflicts: [] };
    const discovery = add(ledger, "discovery", "exact bytes\n", 1);
    expect(() => recordApproval(ledger, "discovery", "APPROVE wrong-digest")).toThrow("exact phrase");
    expect(recordApproval(ledger, "discovery", `APPROVE ${discovery.sha256}`).decision).toBe("approved");
  });

  test("conflicts keep numbered alternatives and pause only declared affected work", () => {
    const ledger: WorkflowLedger = { schema_version: 1, project: "sample", artifacts: [], approvals: [], conflicts: [] };
    const conflict = addConflict(ledger, { id: "goal-change-1", request: "Replace the approved outcome", affected: ["billing feature"], options: ["Keep current outcome", "Start a new run"] });
    expect(conflict.state).toBe("pending");
    expect(() => resolveConflict(ledger, conflict.id, 3)).toThrow("outside the listed choices");
    expect(resolveConflict(ledger, conflict.id, 2).selected_option).toBe(2);
    expect(() => resolveConflict(ledger, conflict.id, 1)).toThrow("already resolved");
  });

  test("pending step conflicts block that step and downstream work without blocking unrelated work", () => {
    const cwd = makeProject();
    try {
      const ledger: WorkflowLedger = { schema_version: 1, project: "sample", artifacts: [], approvals: [], conflicts: [] };
      fs.writeFileSync(path.join(cwd, "discovery.md"), "Accepted discovery\n");
      fs.writeFileSync(path.join(cwd, "design.md"), "Accepted design\n");
      const discovery = recordArtifact(ledger, "discovery", "discovery.md", Buffer.from("Accepted discovery\n"));
      recordApproval(ledger, "discovery", `APPROVE ${discovery.sha256}`);
      const design = recordArtifact(ledger, "design", "design.md", Buffer.from("Accepted design\n"));
      recordApproval(ledger, "design", `APPROVE ${design.sha256}`);
      fs.writeFileSync(path.join(cwd, ".kineti", "workflow.json"), JSON.stringify(ledger));

      const added = run(cwd, ["conflict", "add", "--id", "design-change-1", "--request", "Change the approved design", "--affected", "design", "--option", "Keep it", "--option", "Revise it"]);
      expect(added.status).toBe(0);
      const approval = run(cwd, ["approve", "design"]);
      expect(approval.status).toBe(2);
      expect(approval.err).toContain("paused by pending conflict(s): design-change-1");
      const outcome = run(cwd, ["record", "--step", "outcome", "--file", "design.md"]);
      expect(outcome.status).toBe(2);
      expect(outcome.err).toContain("paused by pending conflict(s): design-change-1");
      const status = JSON.parse(run(cwd, ["status"]).out);
      expect(status.steps.discovery.blocked_by).toEqual([]);
      expect(status.steps.design.blocked_by).toEqual(["design-change-1"]);
      expect(status.steps.architecture.blocked_by).toEqual(["design-change-1"]);
      expect(status.steps.outcome.blocked_by).toEqual(["design-change-1"]);
    } finally { fs.rmSync(path.dirname(cwd), { recursive: true, force: true }); }
  });

  test("record command rejects traversal and records a saga inverse before writing", () => {
    const cwd = makeProject();
    try {
      fs.writeFileSync(path.join(cwd, "brief.md"), "A short project brief\n");
      const bad = run(cwd, ["record", "--step", "discovery", "--file", "../outside.md"]);
      expect(bad.status).toBe(2);
      const good = run(cwd, ["record", "--step", "discovery", "--file", "brief.md"]);
      expect(good.status).toBe(0);
      const stored = JSON.parse(fs.readFileSync(path.join(cwd, ".kineti", "workflow.json"), "utf8"));
      expect(stored.artifacts[0].path).toBe("brief.md");
      expect(stored.artifacts[0].sha256).toBe(Bun.CryptoHasher.hash("sha256", "A short project brief\n", "hex"));
      const saga = fs.readFileSync(path.join(cwd, ".kineti", "saga.jsonl"), "utf8");
      expect(saga).toContain("workflow-ledger-create");
      expect(saga).toContain("rm -f .kineti/workflow.json");
    } finally { fs.rmSync(path.dirname(cwd), { recursive: true, force: true }); }
  });

  test("a non-interactive approval command fails without changing the ledger", () => {
    const cwd = makeProject();
    try {
      fs.writeFileSync(path.join(cwd, "brief.md"), "A short project brief\n");
      expect(run(cwd, ["record", "--step", "discovery", "--file", "brief.md"]).status).toBe(0);
      const before = fs.readFileSync(path.join(cwd, ".kineti", "workflow.json"), "utf8");
      const result = run(cwd, ["approve", "discovery"]);
      expect(result.status).toBe(2);
      expect(result.err).toContain("requires a human terminal");
      expect(fs.readFileSync(path.join(cwd, ".kineti", "workflow.json"), "utf8")).toBe(before);
    } finally { fs.rmSync(path.dirname(cwd), { recursive: true, force: true }); }
  });

  test("outcome generator links approved versions and refuses a blank draft", () => {
    const cwd = makeProject();
    try {
      const { state, ledger } = approvedInputs(cwd);
      const generated = run(cwd, ["generate-outcome", "--file", "docs/outcome-contract.json"]);
      expect(generated.status).toBe(0);
      const draft = JSON.parse(fs.readFileSync(path.join(cwd, "docs/outcome-contract.json"), "utf8"));
      expect(draft.status).toBe("draft");
      expect(draft.source_refs.discovery.sha256).toBe(ledger.artifacts[0]!.sha256);
      const record = run(cwd, ["record", "--step", "outcome", "--file", "docs/outcome-contract.json"]);
      expect(record.status).toBe(2);
      expect(record.err).toContain("needs named personas");
      expect(ledger.artifacts.some((item) => item.step === "outcome")).toBe(false);
      expect(isCurrentApproved(cwd, ledger, "architecture")).toBe(true);
      expect(() => createOutcomeContract({ ...state, mode: null }, ledger)).toThrow("explicit project mode");
    } finally { fs.rmSync(path.dirname(cwd), { recursive: true, force: true }); }
  });

  test("outcome approval requires confirmed priorities and current source bytes", () => {
    const cwd = makeProject();
    try {
      const { state, ledger } = approvedInputs(cwd);
      const draft = completedOutcome(createOutcomeContract(state, ledger, "2026-10-17T00:00:00.000Z"));
      expect(() => validateOutcomeContract(draft, state, ledger, cwd)).not.toThrow();
      const unconfirmed = structuredClone(draft);
      unconfirmed.sections.priorities.confirmed = false;
      expect(() => validateOutcomeContract(unconfirmed, state, ledger, cwd)).toThrow("priorities must be explicitly confirmed");
      const unsupportedEvidence = structuredClone(draft);
      unsupportedEvidence.sections.decision_graph = [{ from: "feature-task", to: "task-1", relation: "requires", evidence_refs: ["discovery:substituted"] }];
      expect(() => validateOutcomeContract(unsupportedEvidence, state, ledger, cwd)).toThrow("decision graph edges need evidence references");
      const unknownPriority = structuredClone(draft);
      unknownPriority.sections.roadmap[0].priority_id = "missing-priority";
      expect(() => validateOutcomeContract(unknownPriority, state, ledger, cwd)).toThrow("link to a confirmed priority");
      const outcomeBytes = Buffer.from(JSON.stringify(draft));
      fs.writeFileSync(path.join(cwd, "docs/outcome-contract.json"), outcomeBytes);
      const outcome = recordArtifact(ledger, "outcome", "docs/outcome-contract.json", outcomeBytes);
      recordApproval(ledger, "outcome", `APPROVE ${outcome.sha256}`);
      expect(isCurrentApproved(cwd, ledger, "outcome")).toBe(true);
      fs.writeFileSync(path.join(cwd, "docs/discovery.md"), "changed source bytes\n");
      expect(isCurrentApproved(cwd, ledger, "discovery")).toBe(false);
      expect(isCurrentApproved(cwd, ledger, "design")).toBe(false);
      expect(isCurrentApproved(cwd, ledger, "outcome")).toBe(false);
    } finally { fs.rmSync(path.dirname(cwd), { recursive: true, force: true }); }
  });
});
