#!/usr/bin/env bun
/** Durable, version-linked project workflow artifacts and explicit approvals. */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createInterface } from "node:readline/promises";
import { stdin, stdout } from "node:process";
import { assertInsideProject, nowIso, projectKdir, readJson, writeJson } from "./lib.ts";

export type WorkflowStep = "discovery" | "design" | "data-intake" | "no-data" | "architecture" | "outcome";
export type ContractStep = "discovery" | "design" | "data" | "architecture" | "outcome";
export interface WorkflowArtifact {
  step: WorkflowStep;
  path: string;
  sha256: string;
  recorded_at: string;
  depends_on: Record<string, string>;
}
export interface WorkflowApproval {
  step: WorkflowStep;
  sha256: string;
  decision: "approved" | "rejected";
  confirmed_at: string;
  depends_on: Record<string, string>;
}
export interface WorkflowConflict {
  id: string;
  request: string;
  affected: string[];
  options: string[];
  state: "pending" | "resolved";
  selected_option?: number;
  created_at: string;
  resolved_at?: string;
}
export interface WorkflowLedger {
  schema_version: 1;
  project: string;
  artifacts: WorkflowArtifact[];
  approvals: WorkflowApproval[];
  conflicts: WorkflowConflict[];
}
export interface ProjectState { project: string; mode?: "harness" | "gate" | "both" | null; root_goal: string | null; }
export interface OutcomeSourceRef { step: WorkflowStep; path: string; sha256: string; }

const WORKFLOW_STEPS: WorkflowStep[] = ["discovery", "design", "data-intake", "no-data", "architecture", "outcome"];
const STEP_ALIASES: Record<ContractStep, WorkflowStep[]> = {
  discovery: ["discovery"], design: ["design"], data: ["data-intake", "no-data"],
  architecture: ["architecture"], outcome: ["outcome"],
};

function sha256(bytes: Buffer | string): string { return crypto.createHash("sha256").update(bytes).digest("hex"); }
function ledgerPath(cwd = process.cwd()): string { return path.join(cwd, ".kineti", "workflow.json"); }
function currentArtifact(ledger: WorkflowLedger, step: ContractStep): WorkflowArtifact | undefined {
  const allowed = STEP_ALIASES[step];
  return [...ledger.artifacts].reverse().find((item) => allowed.includes(item.step));
}
function prerequisites(step: WorkflowStep): ContractStep[] {
  if (step === "design") return ["discovery"];
  if (step === "data-intake" || step === "no-data") return ["design"];
  if (step === "architecture") return ["design", "data"];
  if (step === "outcome") return ["architecture"];
  return [];
}
function latestApproval(ledger: WorkflowLedger, artifact: WorkflowArtifact): WorkflowApproval | undefined {
  return [...ledger.approvals].reverse().find((approval) => approval.step === artifact.step && approval.sha256 === artifact.sha256);
}
function dependencySnapshot(ledger: WorkflowLedger, step: WorkflowStep, requireApproval: boolean): Record<string, string> {
  const deps: Record<string, string> = {};
  for (const predecessor of prerequisites(step)) {
    const artifact = currentArtifact(ledger, predecessor);
    if (!artifact) throw new Error(`workflow step '${step}' requires a recorded '${predecessor}' artifact`);
    if (requireApproval && !isApproved(ledger, predecessor)) throw new Error(`workflow step '${step}' is blocked until '${predecessor}' is approved`);
    deps[predecessor] = artifact.sha256;
  }
  return deps;
}
export function isApproved(ledger: WorkflowLedger, step: ContractStep): boolean {
  const artifact = currentArtifact(ledger, step);
  if (!artifact) return false;
  const approval = latestApproval(ledger, artifact);
  if (!approval || approval.decision !== "approved") return false;
  try { return JSON.stringify(approval.depends_on) === JSON.stringify(dependencySnapshot(ledger, artifact.step, true)); }
  catch { return false; }
}

function artifactBytesAreCurrent(cwd: string, artifact: WorkflowArtifact): boolean {
  try {
    const source = validateRelativeSource(artifact.path, cwd);
    return source.relative === artifact.path && sha256(source.bytes) === artifact.sha256;
  } catch { return false; }
}
export function isCurrentApproved(cwd: string, ledger: WorkflowLedger, step: ContractStep): boolean {
  const artifact = currentArtifact(ledger, step);
  if (!artifact || !isApproved(ledger, step) || !artifactBytesAreCurrent(cwd, artifact)) return false;
  return prerequisites(artifact.step).every((parent) => isCurrentApproved(cwd, ledger, parent));
}
const DOWNSTREAM_STEPS: Record<WorkflowStep, WorkflowStep[]> = {
  discovery: ["discovery", "design", "data-intake", "no-data", "architecture", "outcome"],
  design: ["design", "data-intake", "no-data", "architecture", "outcome"],
  "data-intake": ["data-intake", "architecture", "outcome"],
  "no-data": ["no-data", "architecture", "outcome"],
  architecture: ["architecture", "outcome"],
  outcome: ["outcome"],
};
function pendingConflictsForStep(ledger: WorkflowLedger, step: WorkflowStep): WorkflowConflict[] {
  return ledger.conflicts.filter((conflict) => {
    if (conflict.state !== "pending") return false;
    const affected = new Set(conflict.affected.map((item) => item.trim().toLocaleLowerCase()));
    if (affected.has("*") || affected.has("all")) return true;
    const roots = new Set<WorkflowStep>();
    for (const item of affected) {
      if (item === "data") { roots.add("data-intake"); roots.add("no-data"); continue; }
      if ((WORKFLOW_STEPS as string[]).includes(item)) roots.add(item as WorkflowStep);
    }
    return [...roots].some((root) => DOWNSTREAM_STEPS[root].includes(step));
  });
}
function assertNoPendingConflict(ledger: WorkflowLedger, step: WorkflowStep): void {
  const conflicts = pendingConflictsForStep(ledger, step);
  if (conflicts.length) throw new Error(`workflow step '${step}' is paused by pending conflict(s): ${conflicts.map((item) => item.id).join(", ")}`);
}
function assertCurrentPrerequisites(cwd: string, ledger: WorkflowLedger, step: WorkflowStep): void {
  for (const parent of prerequisites(step)) {
    if (!isCurrentApproved(cwd, ledger, parent)) throw new Error(`workflow step '${step}' is blocked until '${parent}' is approved at its current source bytes`);
  }
}

function readState(cwd: string): ProjectState {
  const stateFile = path.join(cwd, ".kineti", "state.json");
  const state = readJson<ProjectState>(stateFile);
  if (!state || typeof state.project !== "string" || !state.project || !["harness", "gate", "both"].includes(state.mode ?? "")) {
    throw new Error("project mode is pending or state is missing; select harness, gate, or both before starting this workflow");
  }
  if (typeof state.root_goal !== "string" || !state.root_goal.trim()) throw new Error("lock the project goal before starting this workflow");
  return state;
}
function emptyLedger(project: string): WorkflowLedger { return { schema_version: 1, project, artifacts: [], approvals: [], conflicts: [] }; }
function loadLedger(cwd: string, project: string): WorkflowLedger {
  const file = ledgerPath(cwd);
  if (!fs.existsSync(file)) return emptyLedger(project);
  let value: unknown;
  try { value = JSON.parse(fs.readFileSync(file, "utf8")); }
  catch { throw new Error("workflow ledger is malformed; preserve it for owner recovery"); }
  const ledger = value as WorkflowLedger;
  if (!ledger || ledger.schema_version !== 1 || ledger.project !== project || !Array.isArray(ledger.artifacts) || !Array.isArray(ledger.approvals) || !Array.isArray(ledger.conflicts)) {
    throw new Error("workflow ledger does not match the current project; preserve it for owner recovery");
  }
  return ledger;
}
function registerUndo(cwd: string, label: string, inverse: string): void {
  const saga = path.join(import.meta.dir, "kineti-saga.ts");
  const result = spawnSync(process.execPath, [saga, "push", label, inverse], { cwd, encoding: "utf8" });
  if (result.status !== 0) throw new Error(`could not register undo step: ${(result.stderr || "").trim()}`);
}
function persistLedger(cwd: string, ledger: WorkflowLedger): void {
  const target = ledgerPath(cwd);
  const relativeTarget = path.relative(cwd, target).split(path.sep).join("/");
  if (!fs.existsSync(target)) {
    registerUndo(cwd, `workflow-ledger-create-${Date.now()}`, `rm -f ${relativeTarget}`);
  } else {
    const backup = path.join(cwd, ".kineti", `workflow-backup-${Date.now()}-${process.pid}.json`);
    const relativeBackup = path.relative(cwd, backup).split(path.sep).join("/");
    registerUndo(cwd, `workflow-backup-remove-${Date.now()}`, `rm -f ${relativeBackup}`);
    fs.copyFileSync(target, backup);
    registerUndo(cwd, `workflow-ledger-restore-${Date.now()}`, `cp ${relativeBackup} ${relativeTarget}`);
  }
  writeJson(target, ledger);
}

function validateStep(value: string): WorkflowStep {
  if (!(WORKFLOW_STEPS as string[]).includes(value)) throw new Error(`step must be one of: ${WORKFLOW_STEPS.join(", ")}`);
  return value as WorkflowStep;
}
function validateRelativeSource(source: string, cwd: string): { absolute: string; relative: string; bytes: Buffer } {
  if (!source || path.isAbsolute(source) || source.split(/[\\/]/).some((part) => part === "..")) throw new Error("artifact path must be a relative project path");
  const absolute = assertInsideProject(path.resolve(cwd, source), cwd);
  const realRoot = fs.realpathSync(cwd);
  const realFile = fs.realpathSync(absolute);
  if (realFile !== realRoot && !realFile.startsWith(realRoot + path.sep)) throw new Error("artifact symlink escapes the project");
  if (!fs.statSync(realFile).isFile()) throw new Error("artifact must be a regular file");
  const bytes = fs.readFileSync(realFile);
  if (bytes.length === 0 || bytes.length > 5 * 1024 * 1024) throw new Error("artifact must contain 1 byte to 5 MiB");
  return { absolute: realFile, relative: path.relative(cwd, realFile).split(path.sep).join("/"), bytes };
}

function sourceRef(ledger: WorkflowLedger, step: ContractStep): OutcomeSourceRef {
  const artifact = currentArtifact(ledger, step);
  if (!artifact) throw new Error(`outcome contract requires a recorded '${step}' artifact`);
  return { step: artifact.step, path: artifact.path, sha256: artifact.sha256 };
}

export function createOutcomeContract(state: ProjectState, ledger: WorkflowLedger, generatedAt = nowIso()): Record<string, unknown> {
  if (!["harness", "gate", "both"].includes(state.mode ?? "") || !state.root_goal?.trim()) throw new Error("outcome contract requires an explicit project mode and locked goal");
  for (const step of ["discovery", "design", "data", "architecture"] as const) {
    if (!isApproved(ledger, step)) throw new Error(`outcome contract requires approved '${step}' inputs`);
  }
  const data = sourceRef(ledger, "data");
  return {
    schema_version: 1, status: "draft", project: state.project, mode: state.mode, root_goal: state.root_goal, generated_at: generatedAt,
    source_refs: { discovery: sourceRef(ledger, "discovery"), design: sourceRef(ledger, "design"), data, architecture: sourceRef(ledger, "architecture") },
    sections: {
      goal: state.root_goal, personas: [], design: null,
      data: { kind: data.step === "no-data" ? "no-data" : "real", inputs: [], purpose_approval_ref: null },
      features: [], architecture: null, tools: { reviewed: false, decisions: [] }, tests: [], roadmap: [],
      priorities: { confirmed: false, decision_ref: null, items: [] }, decision_graph: [], unknowns: { reviewed: false, items: [] },
    },
  };
}

function isObject(value: unknown): value is Record<string, any> { return !!value && typeof value === "object" && !Array.isArray(value); }
function hasText(value: unknown): value is string { return typeof value === "string" && !!value.trim(); }
function hasTextArray(value: unknown, min = 0): value is string[] { return Array.isArray(value) && value.length >= min && value.every(hasText); }
function uniqueIds(value: unknown, min = 0): value is Array<Record<string, any>> {
  if (!Array.isArray(value) || value.length < min) return false;
  const ids = value.map((item) => item?.id);
  return ids.every(hasText) && new Set(ids).size === ids.length;
}

/** Validate user-completed outcome content against the currently approved artifact versions. */
export function validateOutcomeContract(value: unknown, state: ProjectState, ledger: WorkflowLedger, cwd: string): void {
  if (!isObject(value) || value.schema_version !== 1 || value.status !== "draft"
    || value.project !== state.project || value.mode !== state.mode || value.root_goal !== state.root_goal) {
    throw new Error("outcome contract identity, version, or draft status does not match this project");
  }
  for (const step of ["discovery", "design", "data", "architecture"] as const) {
    if (!isCurrentApproved(cwd, ledger, step)) throw new Error(`outcome contract source '${step}' is not approved at its current bytes`);
    const expected = sourceRef(ledger, step), actual = value.source_refs?.[step];
    if (!isObject(actual) || actual.step !== expected.step || actual.path !== expected.path || actual.sha256 !== expected.sha256) {
      throw new Error(`outcome contract source reference '${step}' is stale or substituted`);
    }
  }
  const sections = value.sections;
  if (!isObject(sections) || sections.goal !== state.root_goal) throw new Error("outcome contract goal must equal the locked project goal");
  if (!uniqueIds(sections.personas, 1) || sections.personas.some((item) => !hasText(item.name) || !hasTextArray(item.needs, 1))) {
    throw new Error("outcome contract needs named personas with at least one stated need");
  }
  const personaIds = new Set(sections.personas.map((item) => item.id));
  if (!isObject(sections.design) || !hasText(sections.design.journey) || !hasText(sections.design.mock_data_shape) || !hasTextArray(sections.design.states, 1)) {
    throw new Error("outcome contract needs a storyboard journey, mock-data shape, and relevant screen states");
  }
  const dataArtifact = currentArtifact(ledger, "data");
  if (!isObject(sections.data) || !["no-data", "real", "mock"].includes(sections.data.kind) || !Array.isArray(sections.data.inputs)) {
    throw new Error("outcome contract needs an explicit real-data, mock-data, or no-data decision");
  }
  if (dataArtifact?.step === "no-data" && (sections.data.kind !== "no-data" || sections.data.inputs.length !== 0)) throw new Error("the approved no-data path cannot declare real or mock inputs");
  if (dataArtifact?.step === "data-intake" && sections.data.kind === "no-data") throw new Error("the approved data-intake artifact cannot be recorded as no-data");
  if (sections.data.kind === "no-data" && sections.data.inputs.length !== 0) throw new Error("no-data decision must have an empty input list");
  if (sections.data.kind !== "no-data") {
    if (!uniqueIds(sections.data.inputs, 1) || sections.data.inputs.some((item) => !hasText(item.provenance) || !hasText(item.purpose)
      || !hasText(item.sensitivity) || !hasText(item.retention) || !["real", "mock"].includes(item.real_or_mock))) {
      throw new Error("each input needs provenance, purpose, sensitivity, retention, and real/mock status");
    }
    if (sections.data.inputs.some((item) => item.real_or_mock === "real")
      && (dataArtifact?.step !== "data-intake" || sections.data.purpose_approval_ref !== `data-intake:${dataArtifact.sha256}`)) {
      throw new Error("real data needs a purpose approval reference to the approved data-intake artifact digest");
    }
  }
  if (!uniqueIds(sections.features, 1) || sections.features.some((item) => !hasText(item.outcome) || !hasTextArray(item.persona_ids, 1)
    || item.persona_ids.some((id: string) => !personaIds.has(id)) || !hasTextArray(item.tests, 1))) {
    throw new Error("each accepted feature must link to a persona, outcome, and test");
  }
  if (!isObject(sections.architecture) || !hasText(sections.architecture.summary) || !Array.isArray(sections.architecture.decisions)) {
    throw new Error("outcome contract needs an architecture summary and explicit decision list");
  }
  if (!isObject(sections.tools) || sections.tools.reviewed !== true || !Array.isArray(sections.tools.decisions)
    || sections.tools.decisions.some((item: any) => !isObject(item) || !hasText(item.name) || !hasText(item.reason) || !hasText(item.costs)
      || !hasText(item.limits) || !hasText(item.failure_behavior) || !hasText(item.exit_cost))) {
    throw new Error("tool choices must be reviewed; selected tools need reason, costs, limits, failure behavior, and exit cost");
  }
  if (!uniqueIds(sections.tests, 1) || sections.tests.some((item) => !Array.isArray(item.command) || item.command.length === 0
    || !item.command.every(hasText) || !hasText(item.expected_result) || !["test", "typecheck", "package"].includes(item.kind))) {
    throw new Error("outcome contract needs executable checks with expected results");
  }
  if (!uniqueIds(sections.roadmap, 1) || sections.roadmap.some((item) => !hasText(item.title) || !hasTextArray(item.feature_ids, 1)
    || item.feature_ids.some((id: string) => !sections.features.some((feature: any) => feature.id === id)) || !hasText(item.priority_id))) {
    throw new Error("roadmap items must link to accepted features and confirmed priorities");
  }
  if (!isObject(sections.priorities) || sections.priorities.confirmed !== true || !hasText(sections.priorities.decision_ref)
    || !uniqueIds(sections.priorities.items, 1) || sections.priorities.items.some((item) => !Number.isInteger(item.rank) || item.rank < 1)) {
    throw new Error("priorities must be explicitly confirmed and list ranked items with a decision reference");
  }
  const roadmapIds = new Set(sections.roadmap.map((item: any) => item.id));
  const priorityIds = sections.priorities.items.map((item: any) => item.id), ranks = sections.priorities.items.map((item: any) => item.rank);
  if (roadmapIds.size !== priorityIds.length || priorityIds.some((id: string) => !roadmapIds.has(id)) || new Set(ranks).size !== ranks.length) {
    throw new Error("confirmed priority items must rank every roadmap item exactly once");
  }
  const approvedEvidence = new Set(Object.values(value.source_refs).map((ref: any) => `${ref.step}:${ref.sha256}`));
  if (sections.roadmap.some((item: any) => !priorityIds.includes(item.priority_id))) {
    throw new Error("each roadmap item must link to a confirmed priority item");
  }
  if (!Array.isArray(sections.decision_graph) || sections.decision_graph.some((edge: any) => !isObject(edge) || !hasText(edge.from)
    || !hasText(edge.to) || !hasText(edge.relation) || !hasTextArray(edge.evidence_refs)
    || edge.evidence_refs.some((ref: string) => !approvedEvidence.has(ref)) || (edge.relation === "caused" && edge.evidence_refs.length === 0))) {
    throw new Error("decision graph edges need evidence references; causal edges need supporting evidence");
  }
  if (!isObject(sections.unknowns) || sections.unknowns.reviewed !== true || !hasTextArray(sections.unknowns.items)) {
    throw new Error("unknowns must be reviewed and either listed or explicitly empty");
  }
}

function shellQuote(value: string): string { return `'${value.replace(/'/g, "'\\''")}'`; }
function writeOutcomeDraft(cwd: string, destination: string, contents: string): string {
  if (!destination || path.isAbsolute(destination) || destination.split(/[\\/]/).some((part) => part === "..")) throw new Error("outcome path must be relative to the project");
  const absolute = assertInsideProject(path.resolve(cwd, destination), cwd);
  const parent = fs.realpathSync(path.dirname(absolute)), realRoot = fs.realpathSync(cwd);
  if (parent !== realRoot && !parent.startsWith(realRoot + path.sep)) throw new Error("outcome directory symlink escapes the project");
  if (fs.existsSync(absolute)) throw new Error("outcome file already exists; preserve it and choose another path");
  const relative = path.relative(cwd, absolute).split(path.sep).join("/");
  registerUndo(cwd, `outcome-contract-remove-${Date.now()}`, `rm -f ${shellQuote(relative)}`);
  fs.writeFileSync(absolute, contents, { encoding: "utf8", flag: "wx", mode: 0o600 });
  return relative;
}

export function recordArtifact(ledger: WorkflowLedger, step: WorkflowStep, sourcePath: string, bytes: Buffer, recordedAt = nowIso()): WorkflowArtifact {
  const dependencies = dependencySnapshot(ledger, step, true);
  const artifact: WorkflowArtifact = { step, path: sourcePath, sha256: sha256(bytes), recorded_at: recordedAt, depends_on: dependencies };
  ledger.artifacts.push(artifact);
  return artifact;
}
export function recordApproval(ledger: WorkflowLedger, step: WorkflowStep, confirmation: string, confirmedAt = nowIso()): WorkflowApproval {
  const artifact = currentArtifact(ledger, step === "data-intake" || step === "no-data" ? "data" : step as ContractStep);
  if (!artifact || artifact.step !== step) throw new Error(`no current '${step}' artifact is recorded`);
  const dependencies = dependencySnapshot(ledger, step, true);
  const expected = `APPROVE ${artifact.sha256}`;
  if (confirmation !== expected) throw new Error(`approval requires the exact phrase: ${expected}`);
  const approval: WorkflowApproval = { step, sha256: artifact.sha256, decision: "approved", confirmed_at: confirmedAt, depends_on: dependencies };
  ledger.approvals.push(approval);
  return approval;
}
export function addConflict(ledger: WorkflowLedger, value: Omit<WorkflowConflict, "state" | "created_at">, at = nowIso()): WorkflowConflict {
  if (!/^[a-z0-9][a-z0-9-]{2,80}$/i.test(value.id) || ledger.conflicts.some((item) => item.id === value.id)) throw new Error("conflict id must be unique and use letters, numbers, or dashes");
  if (!value.request.trim() || value.affected.length === 0 || value.affected.some((item) => !item.trim())) throw new Error("conflict request and affected work are required");
  if (value.options.length < 2 || value.options.some((item) => !item.trim())) throw new Error("provide at least two numbered options");
  const conflict: WorkflowConflict = { ...value, state: "pending", created_at: at };
  ledger.conflicts.push(conflict);
  return conflict;
}
export function resolveConflict(ledger: WorkflowLedger, id: string, option: number, at = nowIso()): WorkflowConflict {
  const conflict = ledger.conflicts.find((item) => item.id === id);
  if (!conflict || conflict.state !== "pending") throw new Error("conflict is missing or already resolved");
  if (!Number.isInteger(option) || option < 1 || option > conflict.options.length) throw new Error("selected option is outside the listed choices");
  conflict.state = "resolved"; conflict.selected_option = option; conflict.resolved_at = at;
  return conflict;
}

function parseArgs(args: string[]): { positional: string[]; options: Map<string, string[]> } {
  const positional: string[] = [], options = new Map<string, string[]>();
  for (let i = 0; i < args.length; i++) {
    const item = args[i]!;
    if (item.startsWith("--")) {
      const value = args[++i];
      if (!value || value.startsWith("--")) throw new Error(`${item} requires a value`);
      options.set(item, [...(options.get(item) ?? []), value]);
    } else positional.push(item);
  }
  return { positional, options };
}
function option(parsed: ReturnType<typeof parseArgs>, key: string): string | undefined { return parsed.options.get(key)?.[0]; }
async function humanApproval(artifact: WorkflowArtifact): Promise<string> {
  if (!stdin.isTTY || !stdout.isTTY) throw new Error("approval requires a human terminal; no approval was recorded");
  const rl = createInterface({ input: stdin, output: stdout });
  try { return (await rl.question(`Review ${artifact.path} (SHA-256 ${artifact.sha256}). Type APPROVE ${artifact.sha256} to approve these exact bytes: `)).trim(); }
  finally { rl.close(); }
}
function usage(): void {
  console.log(`kineti workflow - version-linked project decisions and approvals

Commands:
  status
  record --step <discovery|design|data-intake|no-data|architecture|outcome> --file <relative-path>
  generate-outcome [--file <relative-path>]  Write an unapproved draft linked to current accepted artifacts
  approve <step>        Human terminal approval for the current exact artifact digest
  conflict add --id <id> --request <text> --affected <work> --option <choice> --option <choice> [--option <choice> ...]
  conflict list
  conflict resolve --id <id>   Human selects a listed numbered option

Required order: discovery → design → data-intake or no-data → architecture → outcome.
Re-recording an upstream artifact makes approvals tied to its old digest inapplicable.`);
}

async function main(args = process.argv.slice(2), cwd = process.cwd()): Promise<void> {
  if (args.includes("--help") || args.includes("-h")) { usage(); return; }
  const parsed = parseArgs(args), [command, subcommand] = parsed.positional;
  if (!command || command === "--help" || command === "-h") { usage(); return; }
  const state = readState(cwd), ledger = loadLedger(cwd, state.project);
  if (command === "status") {
    const steps: ContractStep[] = ["discovery", "design", "data", "architecture", "outcome"];
    console.log(JSON.stringify({ project: state.project, mode: state.mode, root_goal: state.root_goal,
      steps: Object.fromEntries(steps.map((step) => {
        const workflowSteps = step === "data" ? ["data-intake", "no-data"] as const : [step as WorkflowStep];
        const blockers = [...new Set(workflowSteps.flatMap((item) => pendingConflictsForStep(ledger, item).map((conflict) => conflict.id)))];
        return [step, { artifact_sha256: currentArtifact(ledger, step)?.sha256 ?? null, approved: isCurrentApproved(cwd, ledger, step), blocked_by: blockers }];
      })),
      pending_conflicts: ledger.conflicts.filter((item) => item.state === "pending").map((item) => ({ id: item.id, affected: item.affected })) }, null, 2));
    return;
  }
  if (command === "generate-outcome") {
    assertNoPendingConflict(ledger, "outcome");
    assertCurrentPrerequisites(cwd, ledger, "outcome");
    const output = option(parsed, "--file") ?? "docs/outcome-contract.json";
    const contract = createOutcomeContract(state, ledger);
    const relative = writeOutcomeDraft(cwd, output, `${JSON.stringify(contract, null, 2)}\n`);
    console.log(`DRAFT GENERATED ${relative}: fill the reviewed sections, then record and approve the exact bytes`);
    return;
  }
  if (command === "record") {
    const step = validateStep(option(parsed, "--step") ?? "");
    const source = option(parsed, "--file");
    if (!source) throw new Error("record requires --file <relative-path>");
    const artifact = validateRelativeSource(source, cwd);
    assertNoPendingConflict(ledger, step);
    if (step !== "discovery") assertCurrentPrerequisites(cwd, ledger, step);
    if (step === "outcome") {
      let contract: unknown;
      try { contract = JSON.parse(artifact.bytes.toString("utf8")); } catch { throw new Error("outcome contract must be valid JSON"); }
      validateOutcomeContract(contract, state, ledger, cwd);
    }
    const result = recordArtifact(ledger, step, artifact.relative, artifact.bytes);
    persistLedger(cwd, ledger);
    console.log(`RECORDED ${step}: ${result.sha256}`);
    return;
  }
  if (command === "approve") {
    const step = validateStep(subcommand ?? "");
    const artifact = currentArtifact(ledger, step === "data-intake" || step === "no-data" ? "data" : step as ContractStep);
    if (!artifact || artifact.step !== step) throw new Error(`no current '${step}' artifact is recorded`);
    assertNoPendingConflict(ledger, step);
    if (!artifactBytesAreCurrent(cwd, artifact)) throw new Error(`'${step}' source bytes changed after recording; record the current file again`);
    if (step !== "discovery") assertCurrentPrerequisites(cwd, ledger, step);
    if (step === "outcome") {
      const source = validateRelativeSource(artifact.path, cwd);
      validateOutcomeContract(JSON.parse(source.bytes.toString("utf8")), state, ledger, cwd);
    }
    const confirmation = await humanApproval(artifact);
    const approval = recordApproval(ledger, step, confirmation);
    persistLedger(cwd, ledger);
    console.log(`APPROVED ${step}: ${approval.sha256}`);
    return;
  }
  if (command === "conflict" && subcommand === "add") {
    const affected = option(parsed, "--affected")?.split(",").map((item) => item.trim()).filter(Boolean) ?? [];
    const request = option(parsed, "--request") ?? "";
    const id = option(parsed, "--id") ?? "";
    const conflict = addConflict(ledger, { id, request, affected, options: parsed.options.get("--option") ?? [] });
    persistLedger(cwd, ledger);
    console.log(`PENDING ${conflict.id}: affected work paused; ${conflict.options.length} choices recorded`);
    return;
  }
  if (command === "conflict" && subcommand === "list") {
    console.log(JSON.stringify(ledger.conflicts, null, 2)); return;
  }
  if (command === "conflict" && subcommand === "resolve") {
    if (!stdin.isTTY || !stdout.isTTY) throw new Error("conflict resolution requires a human terminal; no choice was recorded");
    const id = option(parsed, "--id");
    const conflict = ledger.conflicts.find((item) => item.id === id && item.state === "pending");
    if (!conflict) throw new Error("pending conflict not found");
    conflict.options.forEach((choice, index) => console.log(`${index + 1}. ${choice}`));
    const rl = createInterface({ input: stdin, output: stdout });
    let answer: string;
    try { answer = await rl.question("Choose an option number: "); } finally { rl.close(); }
    const resolution = resolveConflict(ledger, conflict.id, Number(answer.trim()));
    persistLedger(cwd, ledger);
    console.log(`RESOLVED ${resolution.id}: option ${resolution.selected_option}`);
    return;
  }
  usage();
  throw new Error("unknown workflow command");
}

if (import.meta.main) main().catch((error) => { console.error(`kineti workflow: ${error instanceof Error ? error.message : String(error)}`); process.exitCode = 2; });
