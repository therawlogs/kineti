#!/usr/bin/env bun
import fs from "node:fs";
import { checkedOptions } from "./strict-options.ts";
import path from "node:path";
import crypto from "node:crypto";
import os from "node:os";
import { assertJson } from "../src/security/canonical-json.ts";
import { checkCheckpoint, commitCheckpoint, readStrictLines, withFileLock } from "./ledger-integrity.ts";
import { spawnSync } from "node:child_process";
import { appendJsonl, assertInsideProject, computeDelimitedHash, die, nowIso, ok, readJson, readJsonl, sha256 } from "./lib.ts";

interface Link {
  word: string;
  from_id: string; from_at: string;
  to_id: string; to_at: string;
  status: "candidate" | "hypothesis" | "validated" | "rejected";
  proof_id?: string | null;
}

interface Rec {
  at: string;
  type: "run-record" | "learning" | "dossier" | "incident" | "memory" | "memory-tombstone" | "memory-scope-binding";
  state: "active" | "warm" | "cold" | "archive";
  project: string;
  id: string;
  data: any;
  links?: Link[];
  expires?: string | null;
  prev_hash?: string;
  hash?: string;
}

const CORE_WORDS = new Set([
  "caused", "triggers", "blocks", "enables", "requires", "supports",
  "indicates", "contributes_to", "remediates", "contradicts",
  "supersedes", "resolves", "duplicates",
]);
const TIME_ORDERED = new Set(["caused", "triggers", "blocks"]);
const WARM_DAYS = 90, COLD_DAYS = 275;

function journalFile(dir: string): string {
  return path.join(dir, ".kineti", "journal.jsonl");
}

function ownedStoragePath(dir:string,target:string,directory=false,privateMode=false):void {
  const root=fs.realpathSync(dir),relative=path.relative(root,path.resolve(target));
  if(relative.startsWith("..")||path.isAbsolute(relative))throw new Error("storage path escapes actual project");
  const uid=process.getuid?.()??os.userInfo().uid;
  let current=root;const parts=relative.split(path.sep).filter(Boolean);
  for(let i=0;i<parts.length;i++){
    current=path.join(current,parts[i]);const stat=fs.lstatSync(current,{throwIfNoEntry:false});if(!stat)return;
    const final=i===parts.length-1;
    if(stat.isSymbolicLink()||stat.uid!==uid||(stat.mode&0o022)||(!final&&!stat.isDirectory())||(final&&(directory?!stat.isDirectory():!stat.isFile()))||(final&&privateMode&&(stat.mode&0o077)))throw new Error("memory/journal storage must stay in owned project paths without symlinks");
  }
}
function ownedProjectDir(dir:string,allowed:string):string {
  const target=fs.realpathSync(assertInsideProject(path.resolve(dir),path.resolve(allowed))),root=fs.realpathSync(allowed);
  if(target!==root&&!target.startsWith(root+path.sep))throw new Error("project symlink escapes allowed root");return target;
}

function load(dir: string): Rec[] {
  for(const file of [journalFile(dir),`${journalFile(dir)}.checkpoint.json`,`${journalFile(dir)}.initialized`])ownedStoragePath(dir,file);
  try { checkCheckpoint(journalFile(dir)); } catch (error) { throw new Error(`invalid journal: ${(error as Error).message}`); }
  const records = readStrictLines<Rec>(journalFile(dir));
  const problem = chainProblem(records);
  if (problem) throw new Error(`invalid journal: ${problem}`);
  return records;
}

function save(dir: string, recs: Rec[]): void {
  const body = recs.map((r) => JSON.stringify(r)).join("\n") + (recs.length ? "\n" : "");
  const file = journalFile(dir);
  const temp = `${file}.${crypto.randomUUID()}.tmp`;
  fs.writeFileSync(temp, body, { flag:"wx", mode: 0o600 });
  fs.renameSync(temp, file);
  commitCheckpoint(file);
}

// Canonicalization: recursively key-sorted JSON; stable=true converts
// every finite number to fixed 6-decimal strings while false keeps raw numbers.
function canonStable(v: any, stable: boolean): string {
  assertJson(v);
  const norm = (x: any): any => {
    if (x === null || typeof x !== "object") {
      return (stable && typeof x === "number" && Number.isFinite(x)) ? x.toFixed(6) : x;
    }
    if (Array.isArray(x)) return x.map(norm);
    const o: any = Object.create(null);
    for (const k of Object.keys(x).sort()) o[k] = norm(x[k]);
    return o;
  };
  return JSON.stringify(norm(v));
}

function recordHash(r: Rec, stable: boolean): string {
  return computeDelimitedHash([r.prev_hash ?? "", r.at, r.id, canonStable(r.data, stable)]);
}

function recordHashLegacy(r: Rec, stable: boolean): string {
  return sha256(`${r.prev_hash}${r.at}${r.id}${canonStable(r.data, stable)}`);
}

function chainProblem(records: Rec[]): string | undefined {
  let prev = "GENESIS";
  for (const record of records) {
    if (!record || typeof record.id !== "string" || typeof record.at !== "string" || !Number.isFinite(Date.parse(record.at)) || typeof record.project !== "string" || !record.data || typeof record.data !== "object") return "malformed journal record";
    if (!record.prev_hash || !record.hash) return `missing prev_hash/hash at ${record.id}`;
    if (record.prev_hash !== prev) return `broken previous hash at ${record.id}`;
    if (recordHash(record, true) !== record.hash && recordHash(record, false) !== record.hash
      && recordHashLegacy(record, true) !== record.hash && recordHashLegacy(record, false) !== record.hash) return `content hash mismatch at ${record.id}`;
    prev = record.hash;
  }
  return undefined;
}

export function journalSnapshot(dir: string): Rec[] {
  dir=ownedProjectDir(dir,process.env.KINETI_WORKSPACE_ROOT||process.cwd());
  return withFileLock(path.join(dir, ".kineti", "journal.lock"), () => {
    const records = load(dir), problem = chainProblem(records);
    if (problem) throw new Error(`invalid export journal: ${problem}`);
    return records.slice(-50);
  });
}

export function importJournalTail(dir: string, incoming: unknown, snapshotSha256: string): number {
  if (!Array.isArray(incoming) || incoming.length > 50) throw new Error("invalid snapshot journal inventory");
  const records = incoming as Rec[];
  let parent: string | undefined;
  const ids = new Set<string>();
  for (const record of records) {
    if (!record || typeof record.id !== "string" || !record.id || ids.has(record.id) || typeof record.at !== "string"
      || !Number.isFinite(Date.parse(record.at)) || typeof record.project !== "string"
      || !record.prev_hash || !record.hash || (parent !== undefined && record.prev_hash !== parent)
      || (recordHash(record, true) !== record.hash && recordHash(record, false) !== record.hash
        && recordHashLegacy(record, true) !== record.hash && recordHashLegacy(record, false) !== record.hash)) throw new Error("snapshot contains invalid journal history");
    ids.add(record.id); parent = record.hash;
  }
  return withFileLock(path.join(dir, ".kineti", "journal.lock"), () => {
    const local = load(dir), problem = chainProblem(local);
    if (problem) throw new Error(`invalid destination journal: ${problem}`);
    const seen = new Set(local.map(r => r.id));
    let added = 0;
    const state = readJson<{ project?: string }>(path.join(dir, ".kineti", "state.json"));
    for (const original of records) {
      const id = `import-${sha256(JSON.stringify(original))}`;
      if (seen.has(id) || local.some(item => JSON.stringify(item) === JSON.stringify(original))) continue;
      const record: Rec = {
        at: nowIso(), type: "run-record", state: "active", project: state?.project ?? "import", id,
        data: { import_provenance: { version: 1, snapshot_sha256: snapshotSha256, original_record: structuredClone(original), authority: "historical source record only; no local approval or current-memory authority" } },
        links: [], expires: null, prev_hash: local.at(-1)?.hash ?? "GENESIS", hash: "",
      };
      record.hash = recordHash(record, true); local.push(record); seen.add(id); added++;
    }
    if (added) save(dir, local);
    return added;
  });
}

export interface RunRecordInput {
  project: string;
  id: string;
  data: { root_goal: string; stage_outcomes: string[]; costs_usd: number | null; failed_approaches: string[] };
  at?: string;
}

export interface IncidentEvidenceRef { source_id: string; path: string; sha256: string; }
export interface IncidentEvent { at: string; actor: string; task_id: string; observation: string; evidence: IncidentEvidenceRef[]; }
export interface IncidentClaim { claim: string; reported_by?: string; evidence_ids: string[]; }
export interface IncidentInput {
  id: string;
  scope: { user: string; project: string };
  title: string;
  events: IncidentEvent[];
  reported_causes: (IncidentClaim & { reported_by: string })[];
  hypotheses: IncidentClaim[];
  dependencies?: string[];
  missing_evidence: string[];
  conflicting_explanations: string[];
  correction_of?: string;
  at?: string;
}

function resolveIncidentEvidence(dir: string, refs: IncidentEvidenceRef[]): { source_id: string; path: string; expected_sha256: string; status: "verified" | "missing" | "changed" }[] {
  return refs.map((ref) => {
    const base = path.resolve(dir);
    if (!ref || typeof ref.source_id !== "string" || !ref.source_id.trim() || typeof ref.path !== "string"
      || path.isAbsolute(ref.path) || ref.path.split(/[\\/]/).some((part) => part === "..")
      || !/^[a-f0-9]{64}$/i.test(ref.sha256)) {
      return { source_id: String(ref?.source_id ?? "invalid"), path: String(ref?.path ?? ""), expected_sha256: String(ref?.sha256 ?? ""), status: "missing" as const };
    }
    try {
      const real = fs.realpathSync(path.resolve(base, ref.path));
      if (!real.startsWith(base + path.sep) || !fs.statSync(real).isFile()) throw new Error("outside");
      const actual = sha256(fs.readFileSync(real));
      return { source_id: ref.source_id, path: path.relative(base, real).split(path.sep).join("/"), expected_sha256: ref.sha256, status: actual === ref.sha256 ? "verified" as const : "changed" as const };
    } catch {
      return { source_id: ref.source_id, path: ref.path, expected_sha256: ref.sha256, status: "missing" as const };
    }
  });
}

/** Append-only incident records in the same hash-linked project journal as run records. */
export class IncidentJournalWriter {
  constructor(private readonly dir: string, private readonly allowedRoot: string = process.cwd()) {}

  append(input: IncidentInput): Rec {
    const projectDir = ownedProjectDir(this.dir,this.allowedRoot);
    assertLocalContextScope(projectDir,input.scope);
    if (!input || !/^ir-[a-z0-9][a-z0-9-]{2,100}$/i.test(input.id)
      || !input.scope || typeof input.scope.user !== "string" || !input.scope.user.trim()
      || typeof input.scope.project !== "string" || !input.scope.project.trim()
      || typeof input.title !== "string" || !input.title.trim() || !Array.isArray(input.events) || input.events.length === 0
      || !Array.isArray(input.reported_causes) || !Array.isArray(input.hypotheses)
      || (input.dependencies !== undefined && (!Array.isArray(input.dependencies) || !input.dependencies.every((item) => typeof item === "string" && item.trim())))
      || !Array.isArray(input.missing_evidence) || !Array.isArray(input.conflicting_explanations)) {
      throw new Error("incident requires id, user/project scope, title, observed events, reported causes, hypotheses, and evidence notes");
    }
    const sourceIds = new Set<string>();
    const allRefs: IncidentEvidenceRef[] = [];
    for (const event of input.events) {
      if (!Number.isFinite(Date.parse(event.at)) || new Date(Date.parse(event.at)).toISOString() !== event.at
        || !event.actor?.trim() || !event.task_id?.trim() || !event.observation?.trim() || !Array.isArray(event.evidence)) {
        throw new Error("each observed event needs an ISO time, actor, task id, observation, and evidence list");
      }
      allRefs.push(...event.evidence);
    }
    for (const claim of [...input.reported_causes, ...input.hypotheses]) {
      if (!claim.claim?.trim() || !Array.isArray(claim.evidence_ids) || claim.evidence_ids.some((id) => typeof id !== "string")) throw new Error("incident explanations need text and evidence id lists");
      if ("reported_by" in claim && !claim.reported_by?.trim()) throw new Error("reported causes must identify who reported them");
    }
    for (const ref of allRefs) {
      if (!ref || !/^src-[a-z0-9][a-z0-9-]{1,80}$/i.test(ref.source_id) || sourceIds.has(ref.source_id)) throw new Error("evidence source ids must be unique and use src-* names");
      sourceIds.add(ref.source_id);
    }
    for (const claim of [...input.reported_causes, ...input.hypotheses]) {
      if (claim.evidence_ids.some((id) => !sourceIds.has(id))) throw new Error("incident claim references an unknown evidence source id");
    }
    const sourceStatus = resolveIncidentEvidence(projectDir, allRefs);
    const broken = sourceStatus.find((ref) => ref.status !== "verified");
    if (broken) throw new Error(`incident source ${broken.source_id} is ${broken.status}; evidence must be inspectable and match its digest`);
    const at = input.at ?? nowIso();
    if (!Number.isFinite(Date.parse(at)) || new Date(Date.parse(at)).toISOString() !== at) throw new Error("incident timestamp must be an ISO instant");
    const file = journalFile(projectDir);
    fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
    const lockFile = path.join(path.dirname(file), "journal.lock");
    let lockFd: number;
    try { lockFd = fs.openSync(lockFile, "wx", 0o600); }
    catch (error) { if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new Error("journal is locked by another writer"); throw error; }
    try {
      const records = load(projectDir);
      const problem = chainProblem(records);
      if (problem) throw new Error(`refusing to append to an invalid journal: ${problem}`);
      if (records.some((record) => record.id === input.id)) throw new Error(`incident record id already exists: ${input.id}`);
      if (input.correction_of) {
        const prior = records.find((record) => record.type === "incident" && record.id === input.correction_of);
        if (!prior || prior.data?.scope?.user !== input.scope.user || prior.project !== input.scope.project) throw new Error("correction must reference an existing incident in the same user and project scope");
      }
      const record: Rec = {
        at, type: "incident", state: "active", project: input.scope.project, id: input.id,
        data: { scope: input.scope, title: input.title, events: structuredClone(input.events), reported_causes: structuredClone(input.reported_causes),
          hypotheses: structuredClone(input.hypotheses), missing_evidence: [...input.missing_evidence], conflicting_explanations: [...input.conflicting_explanations],
          dependencies: [...(input.dependencies ?? [])],
          ...(input.correction_of ? { correction_of: input.correction_of } : {}) },
        links: [], expires: null, prev_hash: records.at(-1)?.hash ?? "GENESIS", hash: "",
      };
      record.hash = recordHash(record, true);
      appendJsonl(file, record);
      commitCheckpoint(file);
      return record;
    } finally { fs.closeSync(lockFd); fs.rmSync(lockFile, { force: true }); }
  }
}

export function incidentReport(dir: string, id: string, scope: { user: string; project: string }): {
  record: Rec; source_status: ReturnType<typeof resolveIncidentEvidence>; notice: string;
} {
  if (!scope.user?.trim() || !scope.project?.trim()) throw new Error("incident retrieval requires both user and project scope");
  const projectDir = ownedProjectDir(dir,process.env.KINETI_WORKSPACE_ROOT || process.cwd());
  assertLocalContextScope(projectDir,scope);
  const records = withFileLock(path.join(projectDir, ".kineti", "journal.lock"), () => load(projectDir));
  const problem = chainProblem(records);
  if (problem) throw new Error(`incident journal integrity failure: ${problem}`);
  const record = records.find((item) => item.type === "incident" && item.id === id);
  if (!record || record.project !== scope.project || record.data?.scope?.user !== scope.user || record.data?.scope?.project !== scope.project) throw new Error("incident not found in this user and project scope");
  const refs = (record.data.events as IncidentEvent[]).flatMap((event) => event.evidence ?? []);
  return { record, source_status: resolveIncidentEvidence(projectDir, refs), notice: "Observed sequence and dependency links do not by themselves prove causation." };
}

export interface ContextMemoryInput {
  id: string;
  scope: { user: string; project: string };
  content: string;
  tags: string[];
  sources: IncidentEvidenceRef[];
  correction_of?: string;
  at?: string;
}
export interface ContextMemoryEntry { id: string; content: string; tags: string[]; sources: ReturnType<typeof resolveIncidentEvidence>; at: string; }

function appendMemoryRecord(projectDir: string, record: Rec): void {
  const file = journalFile(projectDir);
  fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
  const lockFile = path.join(path.dirname(file), "journal.lock");
  let lockFd: number;
  try { lockFd = fs.openSync(lockFile, "wx", 0o600); }
  catch (error) { if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new Error("journal is locked by another writer"); throw error; }
  try {
    const records = load(projectDir), problem = chainProblem(records);
    if (problem) throw new Error(`refusing to append to an invalid journal: ${problem}`);
    if (records.some((item) => item.id === record.id)) throw new Error(`journal record id already exists: ${record.id}`);
    record.prev_hash = records.at(-1)?.hash ?? "GENESIS";
    record.hash = recordHash(record, true);
    appendJsonl(file, record);
      commitCheckpoint(file);
  } finally { fs.closeSync(lockFd); fs.rmSync(lockFile, { force: true }); }
}

function currentScopeRecords(dir:string,records:Rec[],scope:{user:string;project:string}):Rec[] {
  const legacyProject=readJson<{project:string}>(path.join(dir,".kineti/state.json"))?.project;
  const allowed=new Map<string,string>();
  for(const record of records.filter(r=>r.type==="memory-scope-binding")) {
    const b=record.data;
    if(b.version!==1||b.current_scope?.user!==scope.user||b.current_scope?.project!==scope.project||b.legacy_scope?.user!==scope.user||b.legacy_scope?.project!==legacyProject||!Array.isArray(b.record_refs))continue;
    for(const ref of b.record_refs)if(typeof ref.id==="string"&&typeof ref.hash==="string")allowed.set(ref.id,ref.hash);
  }
  return records.filter(r=>r.data?.scope?.user===scope.user && (r.data?.scope?.project===scope.project || (r.data?.scope?.project===legacyProject&&allowed.get(r.id)===r.hash)));
}
/** Explicit owner binding; legacy records retain their original bytes and authority. */
export function bindLegacyMemory(dir:string,confirmation:string):{count:number;scope:{user:string;project:string}} {
  dir=ownedProjectDir(dir,process.env.KINETI_WORKSPACE_ROOT||process.cwd());
  const scope=localContextScope(dir);
  if(confirmation!==`BIND ${scope.project}`)throw new Error("exact owner scope confirmation required");
  const state=readJson<{project:string}>(path.join(dir,".kineti/state.json"))!;
  const records=withFileLock(path.join(dir,".kineti/journal.lock"),()=>load(dir));
  const eligible=records.filter(r=>["memory","memory-tombstone"].includes(r.type)&&r.data?.scope?.user===scope.user&&r.data?.scope?.project===state.project);
  if(!eligible.length)throw new Error("no eligible legacy OS-user memory records to bind");
  const record:Rec={id:`bind-${crypto.randomUUID()}`,at:nowIso(),type:"memory-scope-binding",state:"active",project:scope.project,data:{version:1,current_scope:scope,legacy_scope:{user:scope.user,project:state.project},record_refs:eligible.map(r=>({id:r.id,hash:r.hash})),authority:"explicit owner-approved local scope binding; original records unchanged"},links:[],expires:null};
  appendMemoryRecord(dir,record);return {count:eligible.length,scope};
}

/** Scoped encrypted memory. Only the current live version is returned; deletion destroys its key and blob. */
export class ContextMemoryStore {
  constructor(private readonly dir: string, private readonly allowedRoot: string = process.cwd()) {}

  append(input: ContextMemoryInput): Rec {
    const projectDir = ownedProjectDir(this.dir,this.allowedRoot);
    assertLocalContextScope(projectDir,input.scope);
    if (!input || !/^mem-[a-z0-9][a-z0-9-]{2,100}$/i.test(input.id)
      || !input.scope?.user?.trim() || !input.scope?.project?.trim()
      || typeof input.content !== "string" || !input.content.trim() || Buffer.byteLength(input.content) > 16_384
      || !Array.isArray(input.tags) || input.tags.length > 32 || !input.tags.every((tag) => typeof tag === "string" && tag.trim() && tag.length < 80)
      || !Array.isArray(input.sources)) throw new Error("memory requires scoped id, content up to 16 KiB, tags, and source references");
    const statuses = resolveIncidentEvidence(projectDir, input.sources);
    if (statuses.some((ref) => ref.status !== "verified")) throw new Error("memory source references must resolve to exact verified project files");
    const at = input.at ?? nowIso();
    if (!Number.isFinite(Date.parse(at)) || new Date(Date.parse(at)).toISOString() !== at) throw new Error("memory timestamp must be an ISO instant");
    const file = journalFile(projectDir);
    fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
    const lockFile = path.join(path.dirname(file), "journal.lock");
    let lockFd: number;
    try { lockFd = fs.openSync(lockFile, "wx", 0o600); }
    catch (error) { if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new Error("journal is locked by another writer"); throw error; }
    let keyPath = "", blobPath = "", keyCreated=false,blobCreated=false,journalAttempted=false;
    try {
      const records = load(projectDir), problem = chainProblem(records);
      if (problem) throw new Error(`refusing to append to an invalid journal: ${problem}`);
      if (records.some((record) => record.id === input.id)) throw new Error(`memory id already exists: ${input.id}`);
      if (input.correction_of) {
        const prior = records.find((record) => record.type === "memory" && record.id === input.correction_of);
        if (!prior || !currentScopeRecords(projectDir,records,input.scope).some(r=>r.id===prior.id)) throw new Error("correction must reference existing memory in the same scope");
      }
      const memoryDir = path.join(projectDir, ".kineti", "memory");
      const keysDir = path.join(memoryDir, "keys"), blobsDir = path.join(memoryDir, "blobs");
      ownedStoragePath(projectDir,keysDir,true,true);ownedStoragePath(projectDir,blobsDir,true,true);
      fs.mkdirSync(keysDir, { recursive: true, mode: 0o700 }); fs.mkdirSync(blobsDir, { recursive: true, mode: 0o700 });
      keyPath = path.join(keysDir, `${input.id}.key`); blobPath = path.join(blobsDir, `${input.id}.enc`);
      const key = crypto.randomBytes(32), nonce = crypto.randomBytes(12);
      const tags = [...new Set(input.tags.map((tag) => tag.trim()))];
      const payload = Buffer.from(JSON.stringify({ content: input.content, tags, sources: input.sources }), "utf8");
      const cipher = crypto.createCipheriv("aes-256-gcm", key, nonce);
      const encrypted = Buffer.concat([cipher.update(payload), cipher.final()]);
      const ciphertext = Buffer.concat([nonce, cipher.getAuthTag(), encrypted]);
      const userScope = input.scope.user, projectScope = input.scope.project;
      const sagaPath = path.join(import.meta.dir, "kineti-saga.ts");
      const jobPath = JSON.stringify(path.join(import.meta.dir, "kineti-memory-job.ts"));
      const inverse = `bun ${jobPath} undo-memory --dir . --id ${input.id}`;
      const pushed = spawnSync(process.execPath, [sagaPath, "push", `memory-store-${input.id}`, inverse], { cwd: projectDir, encoding: "utf8" });
      if (pushed.status !== 0) throw new Error(`could not register memory undo: ${(pushed.stderr || "").trim()}`);
      fs.writeFileSync(keyPath, key, { flag: "wx", mode: 0o600 }); keyCreated=true;
      fs.writeFileSync(blobPath, ciphertext, { flag: "wx", mode: 0o600 }); blobCreated=true;
      const data = { scope: { user: userScope, project: projectScope }, memory_id: input.id,
        content_sha256: sha256(Buffer.from(input.content, "utf8")), ciphertext_sha256: sha256(ciphertext),
        blob_path: path.relative(projectDir, blobPath).split(path.sep).join("/"),
        ...(input.correction_of ? { correction_of: input.correction_of } : {}) };
      const record: Rec = { at, type: "memory", state: "active", project: projectScope, id: input.id, data,
        links: [], expires: null, prev_hash: records.at(-1)?.hash ?? "GENESIS", hash: "" };
      record.hash = recordHash(record, true);
      journalAttempted=true;appendJsonl(file, record);
      commitCheckpoint(file);
      return record;
    } catch (error) {
      if (!journalAttempted&&keyCreated) fs.rmSync(keyPath, { force: true });
      if (!journalAttempted&&blobCreated) fs.rmSync(blobPath, { force: true });
      throw error;
    } finally { fs.closeSync(lockFd); fs.rmSync(lockFile, { force: true }); }
  }

  retrieve(scope: { user: string; project: string }, query = "", limit = 8): { entries: ContextMemoryEntry[]; unavailable: string[] } {
    if(!Number.isInteger(limit)||limit<1||limit>20)throw new Error("memory limit must be from 1 to 20");
    return this.currentEntries(scope,query,limit);
  }
  /** Current encrypted records for the native adapter; never a historical-context view. */
  nativeSnapshot(scope:{user:string;project:string}):{entries:ContextMemoryEntry[];unavailable:string[]} {
    const result=this.currentEntries(scope,"",1001);
    if(result.entries.length>1000)throw new Error("native memory snapshot exceeds 1000 current entries");
    return result;
  }
  private currentEntries(scope: { user: string; project: string }, query: string, limit: number): { entries: ContextMemoryEntry[]; unavailable: string[] } {
    if (!scope.user?.trim() || !scope.project?.trim()) throw new Error("memory retrieval requires both user and project scope");
    const projectDir = ownedProjectDir(this.dir,this.allowedRoot);
    assertLocalContextScope(projectDir,scope);
    return withFileLock(path.join(projectDir,".kineti/journal.lock"),()=>{
    const records = load(projectDir), problem = chainProblem(records);
    if (problem) throw new Error(`memory journal integrity failure: ${problem}`);
    const scoped = currentScopeRecords(projectDir,records,scope);
    const tombstoned = new Set(scoped.filter((record) => record.type === "memory-tombstone").map((record) => record.data?.memory_id));
    const superseded = new Set(scoped.filter((record) => record.type === "memory" && record.data?.correction_of).map((record) => record.data.correction_of));
    const terms = query.toLocaleLowerCase().split(/[^\p{L}\p{N}]+/u).filter((item) => item.length >= 2);
    const legacyProject=readJson<{project:string}>(path.join(projectDir,".kineti/state.json"))?.project;
    const entries: ContextMemoryEntry[] = [], unavailable: string[] = records.filter(r=>r.type==="memory"&&r.data?.scope?.user===scope.user&&r.data?.scope?.project===legacyProject&&!scoped.some(x=>x.id===r.id)).map(r=>`legacy-scope:${r.id}`);
    const candidates = scoped.filter((record) => record.type === "memory" && !tombstoned.has(record.id) && !superseded.has(record.id)).reverse();
    for (const record of candidates) {
      if (entries.length >= limit) break;
      const id = record.id, base = path.join(projectDir, ".kineti", "memory");
      const keyPath = path.join(base, "keys", `${id}.key`), blobPath = path.join(base, "blobs", `${id}.enc`);
      try {
        if(!/^mem-[a-z0-9][a-z0-9-]{2,100}$/i.test(id))throw new Error("invalid memory id in journal");
        ownedStoragePath(projectDir,keyPath,false,true);ownedStoragePath(projectDir,blobPath,false,true);
        if (!fs.existsSync(keyPath) || !fs.existsSync(blobPath)) { unavailable.push(id); continue; }
        const blob = fs.readFileSync(blobPath), key = fs.readFileSync(keyPath);
        if (key.length !== 32 || sha256(blob) !== record.data.ciphertext_sha256 || blob.length < 28) { unavailable.push(id); continue; }
        const nonce = blob.subarray(0, 12), tag = blob.subarray(12, 28), body = blob.subarray(28);
        const decipher = crypto.createDecipheriv("aes-256-gcm", key, nonce); decipher.setAuthTag(tag);
        const payload = JSON.parse(Buffer.concat([decipher.update(body), decipher.final()]).toString("utf8")) as { content: string; tags: string[]; sources: IncidentEvidenceRef[] };
        if (typeof payload.content !== "string" || !Array.isArray(payload.tags) || !Array.isArray(payload.sources)
          || sha256(Buffer.from(payload.content, "utf8")) !== record.data.content_sha256) { unavailable.push(id); continue; }
        const content = payload.content, tags = payload.tags;
        if (terms.length && !terms.some((term) => content.toLocaleLowerCase().includes(term) || tags.some((tag) => tag.toLocaleLowerCase().includes(term)))) continue;
        entries.push({ id, content, tags, sources: resolveIncidentEvidence(projectDir, payload.sources), at: record.at });
      } catch { unavailable.push(id); }
    }
    return { entries, unavailable };
    });
  }

  delete(id: string, scope: { user: string; project: string }): void {
    if (!/^mem-[a-z0-9][a-z0-9-]{2,100}$/i.test(id) || !scope.user?.trim() || !scope.project?.trim()) throw new Error("delete requires a memory id and explicit user/project scope");
    const projectDir = ownedProjectDir(this.dir,this.allowedRoot);
    assertLocalContextScope(projectDir,scope);
    const records = withFileLock(path.join(projectDir, ".kineti", "journal.lock"), () => load(projectDir)), problem = chainProblem(records);
    if (problem) throw new Error(`memory journal integrity failure: ${problem}`);
    const memory = records.find((record) => record.type === "memory" && record.id === id);
    if (!memory || !currentScopeRecords(projectDir,records,scope).some(r=>r.id===memory.id)) throw new Error("memory not found in this user and project scope");
    if (records.some((record) => record.type === "memory-tombstone" && record.data?.scope?.user === scope.user && record.data?.scope?.project === scope.project && record.data?.memory_id === id)) throw new Error("memory is already deleted");
    const file = journalFile(projectDir), lockFile = path.join(path.dirname(file), "journal.lock");
    let lockFd: number;
    try { lockFd = fs.openSync(lockFile, "wx", 0o600); }
    catch (error) { if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new Error("journal is locked by another writer"); throw error; }
    try {
      const fresh = load(projectDir), freshProblem = chainProblem(fresh);
      if (freshProblem) throw new Error(`memory journal integrity failure: ${freshProblem}`);
      if (fresh.some((record) => record.type === "memory-tombstone" && record.data?.memory_id === id)) throw new Error("memory is already deleted");
      const tombstone: Rec = { at: nowIso(), type: "memory-tombstone", state: "active", project: scope.project, id: `tomb-${id}-${Date.now()}`,
        data: { scope: { ...scope }, memory_id: id, content_sha256: memory.data.content_sha256 }, links: [], expires: null,
        prev_hash: fresh.at(-1)?.hash ?? "GENESIS", hash: "" };
      tombstone.hash = recordHash(tombstone, true);
      appendJsonl(file, tombstone);
      commitCheckpoint(file);
      const base = path.join(projectDir, ".kineti", "memory");
      const seen=new Set<string>(); let target:Rec|undefined=memory;
      while(target && !seen.has(target.id) && target.type==="memory" && currentScopeRecords(projectDir,fresh,scope).some(r=>r.id===target!.id)) {
        if(!/^mem-[a-z0-9][a-z0-9-]{2,100}$/i.test(target.id))throw new Error("invalid correction ancestry");
        ownedStoragePath(projectDir,path.join(base,"keys",`${target.id}.key`),false,true);ownedStoragePath(projectDir,path.join(base,"blobs",`${target.id}.enc`),false,true);
        seen.add(target.id);
        fs.rmSync(path.join(base,"keys",`${target.id}.key`),{force:true});
        fs.rmSync(path.join(base,"blobs",`${target.id}.enc`),{force:true});
        target=target.data.correction_of?fresh.find(r=>r.id===target!.data.correction_of):undefined;
      }
    } finally { fs.closeSync(lockFd); fs.rmSync(lockFile, { force: true }); }
  }
}

export function localContextScope(dir = process.cwd()): { user: string; project: string } {
  const root=fs.realpathSync(dir),stateFile=path.join(root,".kineti/state.json");
  const uid = typeof process.getuid === "function" ? process.getuid() : os.userInfo().uid;
  const stat=fs.lstatSync(stateFile);
  if(!stat.isFile()||stat.isSymbolicLink()||stat.uid!==uid)throw new Error("project state must belong to the actual OS user");
  if(!fs.realpathSync(stateFile).startsWith(root+path.sep))throw new Error("project state escapes its actual project");
  const state = readJson<{ project?: string }>(stateFile);
  if (typeof state?.project!=="string"||!state.project.trim()) throw new Error("memory needs a current project state");
  return { user: `local-uid-${uid}`, project: `local-project-v2-${sha256(root)}` };
}
function assertLocalContextScope(dir:string,scope:{user:string;project:string}):void {
  const actual=localContextScope(dir);
  if(!scope||scope.user!==actual.user||scope.project!==actual.project)throw new Error("scope does not match actual OS user and project; caller-selected identity is not authorization");
}

/** Append one scoped run record without rewriting or repairing older journal history. */
export class JournalWriter {
  constructor(private readonly dir: string, private readonly allowedRoot: string = process.cwd()) {}

  appendRunRecord(input: RunRecordInput): Rec {
    const projectDir = ownedProjectDir(this.dir,this.allowedRoot);
    if (!input || typeof input.project !== "string" || !input.project.trim() || /[\r\n\0]/.test(input.project)
      || typeof input.id !== "string" || !/^rr-[a-z0-9][a-z0-9-]{2,100}$/i.test(input.id)
      || !input.data || typeof input.data.root_goal !== "string" || !input.data.root_goal.trim()
      || !Array.isArray(input.data.stage_outcomes) || !input.data.stage_outcomes.every((item) => typeof item === "string")
      || !(input.data.costs_usd === null || (typeof input.data.costs_usd === "number" && Number.isFinite(input.data.costs_usd) && input.data.costs_usd >= 0))
      || !Array.isArray(input.data.failed_approaches) || !input.data.failed_approaches.every((item) => typeof item === "string")) {
      throw new Error("run record requires project, rr-* id, root_goal, stage_outcomes, costs_usd, and failed_approaches");
    }
    const at = input.at ?? nowIso();
    const timestamp = Date.parse(at);
    if (!Number.isFinite(timestamp) || new Date(timestamp).toISOString() !== at) throw new Error("run record timestamp must be an ISO instant");

    const file = journalFile(projectDir);
    fs.mkdirSync(path.dirname(file), { recursive: true, mode: 0o700 });
    const lockFile = path.join(path.dirname(file), "journal.lock");
    let lockFd: number;
    try { lockFd = fs.openSync(lockFile, "wx", 0o600); }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code === "EEXIST") throw new Error("journal is locked by another writer");
      throw error;
    }
    try {
      const records = load(projectDir);
      const problem = chainProblem(records);
      if (problem) throw new Error(`refusing to append to an invalid journal: ${problem}`);
      if (records.some((record) => record.id === input.id)) throw new Error(`run record id already exists: ${input.id}`);
      const record: Rec = {
        at,
        type: "run-record",
        state: "active",
        project: input.project,
        id: input.id,
        data: structuredClone(input.data),
        links: [],
        expires: null,
        prev_hash: records.at(-1)?.hash ?? "GENESIS",
        hash: "",
      };
      record.hash = recordHash(record, true);
      appendJsonl(file, record);
      commitCheckpoint(file);
      return record;
    } finally {
      fs.closeSync(lockFd);
      fs.rmSync(lockFile, { force: true });
    }
  }
}

function ageDays(iso: string): number {
  return (Date.now() - new Date(iso).getTime()) / 86400000;
}

function main() {
  process.argv = [...process.argv.slice(0,2),...checkedOptions(process.argv.slice(2),["--dir","--user","--project","--id","--record","--data"],[],[],3)];
  const [cmd] = process.argv.slice(2);
  const di = process.argv.indexOf("--dir");
  const rawDir = di > -1 ? path.resolve(process.argv[di + 1]) : process.cwd();
  const dir = ownedProjectDir(rawDir, process.env.KINETI_WORKSPACE_ROOT || process.cwd());

  if (cmd === "incident") {
    const action = process.argv[3];
    const option = (name: string) => {
      const index = process.argv.indexOf(name, 4);
      return index >= 0 ? process.argv[index + 1] : undefined;
    };
    const user = option("--user"), project = option("--project"), id = option("--id");
    if (action === "add") {
      const raw = option("--record");
      if (!raw) die("incident add requires --record <json>", 2);
      let input: IncidentInput;
      try { input = JSON.parse(raw) as IncidentInput; }
      catch { die("incident add requires valid JSON after --record", 2); }
      try {
        const record = new IncidentJournalWriter(dir).append(input);
        ok(`incident appended: ${record.id}; hash ${record.hash}`);
      } catch (error) { die(`incident refused: ${(error as Error).message}`, 3); }
      return;
    }
    if (action === "show") {
      if (!id || !user || !project) die("incident show requires --id, --user, and --project scope", 2);
      try { console.log(JSON.stringify(incidentReport(dir, id, { user, project }), null, 2)); }
      catch (error) { die(`incident unavailable: ${(error as Error).message}`, 3); }
      return;
    }
    if (action === "list") {
      if (!user || !project) die("incident list requires --user and --project scope", 2);
      assertLocalContextScope(dir,{user,project});
      const records = withFileLock(path.join(dir,".kineti/journal.lock"),()=>load(dir));
      const problem = chainProblem(records);
      if (problem) die(`incident journal integrity failure: ${problem}`, 3);
      const scoped = records.filter((record) => record.type === "incident" && record.project === project && record.data?.scope?.user === user && record.data?.scope?.project === project)
        .map((record) => ({ id: record.id, at: record.at, title: record.data.title, correction_of: record.data.correction_of ?? null }));
      console.log(JSON.stringify(scoped, null, 2));
      return;
    }
    die("incident command is add | show | list", 2);
  }

  if (cmd === "undo-memory") {
    const index = process.argv.indexOf("--id", 3), id = index >= 0 ? process.argv[index + 1] : undefined;
    if (!id) die("undo-memory requires --id", 2);
    try { new ContextMemoryStore(dir).delete(id, localContextScope(dir)); ok(`memory creation rolled back: ${id}`); }
    catch (error) { die(`memory rollback refused: ${(error as Error).message}`, 3); }
    return;
  }

  if (cmd === "append-run") {
    const option = (name: string) => {
      const index = process.argv.indexOf(name);
      return index >= 0 ? process.argv[index + 1] : undefined;
    };
    const project = option("--project"), id = option("--id"), rawData = option("--data");
    if (!project || !id || rawData === undefined) die("append-run requires --project, --id, and --data <json>", 2);
    let data: RunRecordInput["data"];
    try { data = JSON.parse(rawData) as RunRecordInput["data"]; }
    catch { die("append-run requires valid JSON after --data", 2); }
    try {
      const record = new JournalWriter(dir).appendRunRecord({ project, id, data });
      ok(`run record appended: ${record.id}; hash ${record.hash}`);
    } catch (error) {
      die(`run record refused: ${(error as Error).message}`, 3);
    }
    return;
  }

  if (cmd === "sweep") {
    return withFileLock(path.join(dir, ".kineti", "journal.lock"), () => {
    const recs = load(dir);
    const problem = chainProblem(recs);
    if (problem) throw new Error(`refusing maintenance of invalid journal: ${problem}`);
    let moved = 0;
    for (const r of recs) {
      if (r.state === "archive") continue;
      const age = ageDays(r.at);
      let target: Rec["state"] = r.state;
      if (r.state === "active") {
        const expired = r.expires ? new Date(r.expires).getTime() < Date.now() : false;
        if (expired || (r.type === "learning" && age > 90)) target = "warm";
      }
      if (r.state === "warm" && age > WARM_DAYS + 90) target = "cold";
      if (r.state === "cold" && age > WARM_DAYS + COLD_DAYS) target = "archive";
      if (target !== r.state) { r.state = target; moved++; }
    }
    save(dir, recs);
    ok(`sweep complete: ${moved} record(s) moved`);
    });
  }

  if (cmd === "verify-chain") {
    return withFileLock(path.join(dir, ".kineti", "journal.lock"), () => {
    // ONE chain over ALL record types in file order, float-stable
    // canonicalization, with legacy hash acceptance for old journals.
    const chain = load(dir);
    let prev = "GENESIS";
    for (const r of chain) {
      if (!r.prev_hash || !r.hash) {
        console.error(`kineti: CHAIN BROKEN at ${r.id}: missing prev_hash/hash — append via JournalWriter, not raw JSONL`);
        process.exit(3);
      }
      if (r.prev_hash !== prev) {
        console.error(`kineti: CHAIN BROKEN at ${r.id}: expected prev ${prev.slice(0, 8)}, found ${r.prev_hash.slice(0, 8)}`);
        process.exit(3);
      }
      if (recordHash(r, true) !== r.hash && recordHash(r, false) !== r.hash && recordHashLegacy(r, true) !== r.hash && recordHashLegacy(r, false) !== r.hash) {
        console.error(`kineti: TAMPER at ${r.id}: content hash mismatch`);
        process.exit(3);
      }
      prev = r.hash;
    }
    ok(`chain intact: ${chain.length} record(s), head ${prev.slice(0, 12)}`);
    });
  }

  if (cmd === "time-order") {
    return withFileLock(path.join(dir, ".kineti", "journal.lock"), () => {
    const byId = new Map(load(dir).map((r) => [r.id, r]));
    const bad: string[] = [];
    for (const r of load(dir)) {
      for (const l of r.links ?? []) {
        if (!TIME_ORDERED.has(l.word)) continue;
        const cause = byId.get(l.from_id), effect = byId.get(l.to_id);
        if (!cause || !effect) continue;
        if (new Date(effect.at) < new Date(cause.at)) {
          bad.push(`${l.word}: effect ${l.to_id} (${effect.at}) precedes cause ${l.from_id} (${cause.at})`);
        }
      }
    }
    if (bad.length) {
      console.error(`kineti: ${bad.length} time-order violation(s):`);
      for (const b of bad) console.error(`  - ${b}`);
      throw Object.assign(new Error("journal time-order check failed"), { exitCode: 1 });
    }
    ok("time order holds for all caused/triggers/blocks links");
    });
  }

  if (cmd === "promote") {
    return withFileLock(path.join(dir, ".kineti", "journal.lock"), () => {
    const counts = new Map<string, number>();
    for (const r of load(dir)) {
      for (const l of r.links ?? []) {
        if (!CORE_WORDS.has(l.word)) counts.set(l.word, (counts.get(l.word) ?? 0) + 1);
      }
    }
    const candidates = [...counts.entries()].filter(([, n]) => n >= 3);
    if (candidates.length === 0) { ok("no promotion candidates (non-core words used fewer than 3 times)"); return; }
    for (const [w, n] of candidates.sort((a, b) => b[1] - a[1])) {
      console.log(`PROMOTE? "${w}" used ${n}x — add to core vocabulary in AGENTS.md and kineti.config.json`);
    }
    });
  }

  die(`unknown command: ${cmd}. Use append-run | sweep | verify-chain | time-order | promote`, 2);
}

if (import.meta.main) { try { main(); } catch (error) { console.error(`kineti: ${(error as Error).message}`); process.exitCode = (error as { exitCode?: number }).exitCode ?? 3; } }
