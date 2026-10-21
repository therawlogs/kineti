#!/usr/bin/env bun
/** User-operated Linux service. Candidate code runs only inside Bubblewrap. */
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawn } from "node:child_process";
import { Database } from "bun:sqlite";
import { canonicalJsonV2 } from "../src/security/canonical-json.ts";
import { evaluationDigest, evaluationManifestDigest, signEvaluationResult, validateTrustedEvaluation,
  type ApprovedEvaluationPins, type OwnerApprovedManifest, type SignedEvaluationResult, type CheckResult } from "../src/security/evaluation-contract.ts";

const sha = (v: string | Uint8Array) => crypto.createHash("sha256").update(v).digest("hex");
const digestFile = (p: string) => sha(fs.readFileSync(p));
const fail = (message: string): never => { throw new Error(message); };
const id = (x: unknown): x is string => typeof x === "string" && /^[a-zA-Z0-9][a-zA-Z0-9._-]{2,100}$/.test(x);
const same = (a: unknown, b: unknown) => canonicalJsonV2(a) === canonicalJsonV2(b);
function bytes64(value: unknown): Buffer {
  if (typeof value !== "string" || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(value)) fail("invalid-base64");
  const bytes = Buffer.from(value as string, "base64");
  if (bytes.length > 1024 * 1024) fail("input-too-large");
  return bytes;
}
function owned(p: string, directory = false, readable = false): void {
  const s = fs.lstatSync(p);
  if (s.isSymbolicLink() || s.uid !== process.getuid!() || (s.mode & (readable ? 0o022 : 0o077)) || (directory ? !s.isDirectory() : !s.isFile())) fail("unsafe-service-owned-path");
}
function syncDir(p: string): void { const fd = fs.openSync(p, "r"); try { fs.fsyncSync(fd); } finally { fs.closeSync(fd); } }
function durableNew(p: string, bytes: Uint8Array): void {
  const fd = fs.openSync(p, "wx", 0o600);
  try { fs.writeFileSync(fd, bytes); fs.fsyncSync(fd); } finally { fs.closeSync(fd); }
  syncDir(path.dirname(p));
}
export interface TrustedCliCheck { id: string; argv: string[]; stdin_base64: string; expected_exit: number; stdout_sha256: string; stderr_sha256: string; timeout_ms: number; }
export interface LinuxServiceConfig {
  version: 1;
  owner_public_key: string;
  evaluator_private_key: string;
  pins: ApprovedEvaluationPins;
  contract: { version: 1; environment: Record<string, string>; checks: TrustedCliCheck[] };
  policy: { version: 1; destinations: string[]; caller_keys: Record<string, string>; revoked_callers: string[]; revoked_evaluations: string[]; max_age_ms: number };
  runtime: { bwrap: string; prlimit: string; busybox: string; library_dir: string; cgroup_root: string; sha256: Record<string, string> };
}
export { signServiceRequest, type SignedServiceRequest } from "../src/security/service-request.ts";
import { signServiceResponse, type SignedServiceRequest } from "../src/security/service-request.ts";
function unsigned(r: SignedServiceRequest) { const { signature: _, ...body } = r; return body; }

/** Bounded pipes, process-group termination, namespaces, no network, no authority mounts. */
async function isolated(config: LinuxServiceConfig, artifact: string, check: TrustedCliCheck) {
  const rt = config.runtime;
  const group = path.join(rt.cgroup_root, crypto.randomUUID());
  fs.mkdirSync(group);
  try {
    fs.writeFileSync(path.join(group,"memory.max"),"134217728");
    fs.writeFileSync(path.join(group,"memory.swap.max"),"0");
    fs.writeFileSync(path.join(group,"pids.max"),"16");
    fs.writeFileSync(path.join(group,"cpu.max"),"100000 100000");
  } catch(e) { fs.rmdirSync(group); throw e; }
  const args = ["--cpu=5", "--as=268435456", "--fsize=1048576", "--nofile=64", "--nproc=32", "--", rt.bwrap,
    "--unshare-all", "--die-with-parent", "--new-session", "--clearenv", "--setenv", "PATH", "/bin",
    "--ro-bind", rt.busybox, "/bin/busybox", "--ro-bind", rt.library_dir, "/lib", "--proc", "/proc", "--dev", "/dev", "--size", "16777216", "--tmpfs", "/tmp",
    "--ro-bind", artifact, "/candidate", "--chdir", "/tmp", "--uid", "65534", "--gid", "65534", "--cap-drop", "ALL"];
  for (const [key, value] of Object.entries(config.contract.environment)) args.push("--setenv", key, value);
  args.push("--", ...check.argv);
  return await new Promise<{ exit: number | null; stdout: Buffer; stderr: Buffer; fault: string | null }>((resolve) => {
    const child = spawn(rt.busybox, ["sh","-c",'echo $$ > "$1/cgroup.procs" || exit 125; shift; exec "$@"',"kineti-launch",group,rt.prlimit,...args], { detached: true, env: { PATH: "/usr/bin:/bin" }, stdio: ["pipe", "pipe", "pipe"] });
    const out: Buffer[] = [], err: Buffer[] = []; let count = 0, fault: string | null = null;
    const kill = (why: string) => { fault = why; try { fs.writeFileSync(path.join(group,"cgroup.kill"),"1"); } catch {} try { process.kill(-child.pid!, "SIGKILL"); } catch {} };
    const timer = setTimeout(() => kill("timeout"), check.timeout_ms);
    const collect = (list: Buffer[], b: Buffer) => { count += b.length; if (count > 2 * 1024 * 1024) kill("output-limit"); else list.push(b); };
    child.stdout.on("data", b => collect(out, b)); child.stderr.on("data", b => collect(err, b));
    child.stdin.on("error", () => {}); child.stdin.end(bytes64(check.stdin_base64));
    child.on("error", () => { fault = "runner-unavailable"; });
    child.on("close", async code => { clearTimeout(timer);
      try { if (/oom_kill [1-9]/.test(fs.readFileSync(path.join(group,"memory.events"),"utf8"))) fault="memory-limit"; fs.writeFileSync(path.join(group,"cgroup.kill"),"1"); } catch { fault ||= "cgroup-inspection-failed"; }
      let cleaned=false;
      for(let attempt=0;attempt<50;attempt++){try{fs.rmdirSync(group);cleaned=true;break;}catch{await new Promise(r=>setTimeout(r,10));}}
      if(!cleaned)fault ||= "cgroup-cleanup-pending";
      resolve({ exit: code, stdout: Buffer.concat(out), stderr: Buffer.concat(err), fault }); });
  });
}

export class LinuxProtectedService {
  readonly root: string; readonly config: LinuxServiceConfig; readonly db!: Database;
  private lock: string; private evaluatorPublic: string; private closed = false;
  constructor(root: string, options: { initialize?: boolean } = {}) {
    if (process.platform !== "linux") fail("linux-required");
    this.root = fs.realpathSync(root); owned(this.root, true);
    this.lock = path.join(this.root, "service.lock");
    const lockFd = fs.openSync(this.lock, "wx", 0o600); fs.writeFileSync(lockFd, String(process.pid)); fs.closeSync(lockFd);
    try {
      const configPath = path.join(this.root, "config.json"); owned(configPath);
      this.config = JSON.parse(fs.readFileSync(configPath, "utf8")); this.validateConfig();
      this.evaluatorPublic = crypto.createPublicKey(this.config.evaluator_private_key).export({ type: "spki", format: "pem" }).toString();
      if (crypto.createPublicKey(this.config.owner_public_key).export({type:"spki",format:"der"}).equals(crypto.createPublicKey(this.evaluatorPublic).export({type:"spki",format:"der"}))) fail("shared-authority");
      for (const dir of ["artifacts", "output"]) { const p = path.join(this.root, dir); if (options.initialize) fs.mkdirSync(p, {mode:0o700}); owned(p, true); }
      const dbPath = path.join(this.root, "requests.sqlite"), marker = path.join(this.root, "requests.initialized");
      if (options.initialize) {
        if (fs.existsSync(dbPath) || fs.existsSync(marker)) fail("already-initialized");
      } else { owned(dbPath); owned(marker); }
      if(!options.initialize) {
        const header=fs.readFileSync(dbPath).subarray(0,16).toString("binary");
        if(header!=="SQLite format 3\0")fail("damaged-request-store-owner-recovery-required");
        const inspection=new Database(dbPath,{readonly:true});
        try {
          const stored=inspection.query("SELECT value FROM metadata WHERE id='identity'").get() as {value:string}|null;
          if(!stored||stored.value!==fs.readFileSync(marker,"utf8")||(inspection.query("PRAGMA quick_check").get() as any).quick_check!=="ok")fail("damaged-request-store-owner-recovery-required");
        } finally {inspection.close();}
      }
      this.db = new Database(dbPath, { create: !!options.initialize, strict: true }); fs.chmodSync(dbPath, 0o600);
      this.db.exec("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA busy_timeout=5000;");
      if (options.initialize) {
        const uuid = crypto.randomUUID();
        this.db.exec("CREATE TABLE metadata (id TEXT PRIMARY KEY, value TEXT NOT NULL); CREATE TABLE evaluations (id TEXT PRIMARY KEY, result TEXT NOT NULL, artifact TEXT NOT NULL, at_ms INTEGER NOT NULL); CREATE TABLE requests (id TEXT PRIMARY KEY, caller TEXT NOT NULL, binding TEXT NOT NULL, evaluation TEXT NOT NULL UNIQUE, destination TEXT NOT NULL, digest TEXT NOT NULL, previous TEXT, state TEXT NOT NULL, reason TEXT NOT NULL); CREATE UNIQUE INDEX destination_pending ON requests(destination) WHERE state='pending';");
        this.db.query("INSERT INTO metadata VALUES ('identity', ?)").run(uuid);
        this.db.exec("PRAGMA wal_checkpoint(FULL)"); durableNew(marker, Buffer.from(uuid));
      }
      const identity = this.db.query("SELECT value FROM metadata WHERE id='identity'").get() as {value:string} | null;
      if (!identity || fs.readFileSync(marker,"utf8") !== identity.value || (this.db.query("PRAGMA quick_check").get() as any).quick_check !== "ok") fail("damaged-request-store-owner-recovery-required");
      if (options.initialize) durableNew(path.join(this.root,"revocations.json"),Buffer.from(JSON.stringify({version:1,callers:[],evaluations:[]})));
      this.revocations(); this.reconcile();
    } catch (e) { try { (this.db! as Database | undefined)?.close(); } catch {} fs.unlinkSync(this.lock); throw e; }
  }
  private validateConfig() {
    const c = this.config;
    if (!path.isAbsolute(c.runtime?.cgroup_root||"") || fs.lstatSync(c.runtime.cgroup_root).isSymbolicLink() || fs.statSync(c.runtime.cgroup_root).uid!==process.getuid!() || (fs.statSync(c.runtime.cgroup_root).mode&0o022)) fail("unsafe-cgroup-delegation");
    if(crypto.createPrivateKey(c.evaluator_private_key).asymmetricKeyType!=="ed25519" || crypto.createPublicKey(c.owner_public_key).asymmetricKeyType!=="ed25519" || Object.values(c.policy.caller_keys).some(k=>crypto.createPublicKey(k).asymmetricKeyType!=="ed25519"))fail("Ed25519-authorities-required");
    const authorityKeys=[c.owner_public_key,crypto.createPublicKey(c.evaluator_private_key).export({type:"spki",format:"pem"}).toString(),...Object.values(c.policy.caller_keys)].map(k=>crypto.createPublicKey(k).export({type:"spki",format:"der"}).toString("hex"));
    if(new Set(authorityKeys).size!==authorityKeys.length)fail("owner-evaluator-and-caller-keys-must-be-distinct");
    if (c.version !== 1 || c.contract?.version !== 1 || c.policy?.version !== 1 || !c.pins || !c.runtime || !Array.isArray(c.contract.checks) || !c.contract.checks.length || c.contract.checks.length > 100) fail("invalid-service-config");
    if (evaluationDigest(c.contract) !== c.pins.contract_sha256 || evaluationDigest(c.policy) !== c.pins.policy.sha256) fail("config-pin-mismatch");
    if (!Number.isSafeInteger(c.policy.max_age_ms) || c.policy.max_age_ms < 1000 || c.policy.max_age_ms > 86400000) fail("invalid-expiry-policy");
    if (!Array.isArray(c.policy.destinations) || !c.policy.destinations.length || c.policy.destinations.some(p=>!this.safeRelative(p))) fail("invalid-destination-policy");
    if (!Array.isArray(c.policy.revoked_callers) || !Array.isArray(c.policy.revoked_evaluations) || !c.policy.caller_keys || !Object.keys(c.policy.caller_keys).every(id)) fail("invalid-caller-policy");
    if (!c.contract.environment || Object.entries(c.contract.environment).some(([k,v])=>! /^[A-Z][A-Z0-9_]{0,63}$/.test(k) || ["PATH","LD_PRELOAD","LD_LIBRARY_PATH"].includes(k) || typeof v!=="string" || v.includes("\0"))) fail("invalid-environment");
    if (c.contract.checks.length !== c.pins.required_checks.length) fail("inventory-mismatch");
    c.contract.checks.forEach((x,i)=>{
      if (!id(x.id) || !Array.isArray(x.argv) || !x.argv.length || x.argv[0]!=="/bin/busybox" || x.argv.some(a=>typeof a!=="string"||a.includes("\0")) || !Number.isInteger(x.expected_exit) || x.expected_exit<0 || x.expected_exit>255 || !Number.isSafeInteger(x.timeout_ms) || x.timeout_ms<10 || x.timeout_ms>10000 || !/^[a-f0-9]{64}$/.test(x.stdout_sha256) || !/^[a-f0-9]{64}$/.test(x.stderr_sha256)) fail("invalid-trusted-check");
      if (!same(c.pins.required_checks[i], {id:x.id, argv:x.argv, input_sha256:[sha(bytes64(x.stdin_base64))]})) fail("inventory-mismatch");
    });
    for (const p of [c.runtime.bwrap,c.runtime.prlimit,c.runtime.busybox]) if (!path.isAbsolute(p) || digestFile(p)!==c.runtime.sha256[p]) fail("runtime-substitution");
    if (c.pins.runner.image_sha256 !== evaluationDigest(c.runtime.sha256)) fail("runtime-image-mismatch");
    owned(c.runtime.library_dir, true, true); // Provision a private, read-only copy of the runtime libraries.
    for (const entry of fs.readdirSync(c.runtime.library_dir)) {
      const p = path.join(c.runtime.library_dir,entry); if (!fs.lstatSync(p).isFile() || digestFile(p)!==c.runtime.sha256[p]) fail("runtime-library-substitution");
    }
    if (Object.keys(c.runtime.sha256).length !== 3 + fs.readdirSync(c.runtime.library_dir).length) fail("runtime-inventory-mismatch");
  }
  private revocations(): {version:1;callers:string[];evaluations:string[]} {
    const p=path.join(this.root,"revocations.json");owned(p);const r=JSON.parse(fs.readFileSync(p,"utf8"));
    if(r.version!==1 || !Array.isArray(r.callers)||!Array.isArray(r.evaluations)||!r.callers.every(id)||!r.evaluations.every(id))fail("invalid-revocations");return r;
  }
  private safeRelative(p: unknown): p is string { return typeof p === "string" && p.length < 200 && p.split("/").every(s=>/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(s) && s!==".." && s!=="."); }
  private destination(relative: string): string {
    if (!this.safeRelative(relative) || !this.config.policy.destinations.includes(relative)) fail("destination-denied");
    const parts=relative.split("/"), leaf=parts.pop()!; let parent=path.join(this.root,"output"); owned(parent,true);
    for(const part of parts) { parent=path.join(parent,part); if(!fs.existsSync(parent)) {fs.mkdirSync(parent,{mode:0o700});syncDir(path.dirname(parent));} owned(parent,true); }
    const dest=path.join(parent,leaf); if(fs.existsSync(dest)) owned(dest); else if(fs.lstatSync(dest,{throwIfNoEntry:false})) fail("destination-symlink"); return dest;
  }
  private verifyRequest(r: SignedServiceRequest) {
    if (!r || r.version!==1 || !id(r.caller) || !id(r.request_id) || !["evaluate","export","inspect"].includes(r.operation) || !Number.isSafeInteger(r.expires_ms) || r.expires_ms<Date.now() || r.expires_ms>Date.now()+300000 || !r.arguments || typeof r.arguments!=="object" || Array.isArray(r.arguments)) fail("invalid-or-expired-request");
    if (this.config.policy.revoked_callers.includes(r.caller) || this.revocations().callers.includes(r.caller)) fail("caller-revoked");
    const key = Object.hasOwn(this.config.policy.caller_keys,r.caller) && this.config.policy.caller_keys[r.caller];
    if (!key || !crypto.verify(null,Buffer.from(canonicalJsonV2(unsigned(r))),key,bytes64(r.signature))) fail("caller-signature-denied");
  }
  async handle(r: SignedServiceRequest): Promise<unknown> {
    this.verifyRequest(r);
    if (r.operation==="evaluate") return this.evaluate(r);
    if (r.operation==="export") return this.export(r);
    const row=this.db.query("SELECT id, caller, state, reason, digest, destination FROM requests WHERE id=?").get(String(r.arguments.request_id)) as any;
    if (!row || row.caller!==r.caller) fail("request-not-found"); return row;
  }
  private async evaluate(r: SignedServiceRequest) {
    this.validateConfig();
    const manifest=r.arguments.manifest as OwnerApprovedManifest, bytes=bytes64(r.arguments.artifact_base64);
    if (!manifest || !id(manifest.evaluation_id) || sha(bytes)!==this.config.pins.artifact_sha256) fail("artifact-substitution");
    if(this.db.query("SELECT id FROM evaluations WHERE id=?").get(manifest.evaluation_id)) fail("evaluation-already-exists");
    const base={schema:manifest.schema,manifest_sha256:evaluationManifestDigest(manifest),artifact_sha256:sha(bytes),contract_sha256:this.config.pins.contract_sha256,policy_sha256:this.config.pins.policy.sha256,evaluator:this.config.pins.evaluator,runner:this.config.pins.runner};
    const verify=(result:SignedEvaluationResult)=>validateTrustedEvaluation(manifest,result,{pins:this.config.pins,trustedOwnerKeys:{[this.config.pins.owner.key_id]:this.config.owner_public_key},trustedEvaluatorKeys:{[this.config.pins.evaluator.key_id]:this.evaluatorPublic}});
    verify(signEvaluationResult({...base,checks:this.config.contract.checks.map(x=>({id:x.id,status:"unrun",reason_code:"not-executed"})),decision:"denied",reason_code:"required-check-incomplete"},this.config.evaluator_private_key));
    const artifact=path.join(this.root,"artifacts",sha(bytes));
    if (!fs.existsSync(artifact)) { durableNew(artifact,bytes); fs.chmodSync(artifact,0o444); syncDir(path.dirname(artifact)); } owned(artifact,false,true); if(digestFile(artifact)!==sha(bytes)) fail("immutable-artifact-damaged");
    const checks: CheckResult[]=[]; const observations: unknown[]=[];
    for(const check of this.config.contract.checks) {
      const result=await isolated(this.config,artifact,check);
      const passed=!result.fault && result.exit===check.expected_exit && sha(result.stdout)===check.stdout_sha256 && sha(result.stderr)===check.stderr_sha256;
      checks.push({id:check.id,status:result.fault?"unknown":passed?"passed":"failed",reason_code:result.fault|| (passed?"trusted-output-match":"trusted-output-mismatch")});
      observations.push({id:check.id,argv:check.argv,stdin_sha256:sha(bytes64(check.stdin_base64)),stdout_sha256:sha(result.stdout),stderr_sha256:sha(result.stderr),exit:result.exit,fault:result.fault});
    }
    if(digestFile(artifact)!==sha(bytes)) fail("immutable-artifact-damaged"); this.validateConfig();
    const decision=checks.some(x=>x.status==="failed")?"denied":checks.some(x=>x.status==="unknown")?"unknown":"accepted";
    const at=Date.now();
    const unsignedResult={...base,checks,decision,reason_code:decision==="accepted"?"all-required-checks-passed":decision==="unknown"?"required-check-outcome-unknown":"required-check-failed",execution:{schema:"kineti-linux-execution/v1",at_ms:at,expires_ms:at+this.config.policy.max_age_ms,kernel:process.platform+"/"+process.arch+"/"+require("node:os").release(),evaluator_sha256:digestFile(import.meta.path),environment_sha256:evaluationDigest(this.config.contract.environment),isolation:{driver:"bubblewrap",network:"unshared",uid:65534,memory_bytes:134217728,pids:16,cpu_quota:"100000/100000",tmpfs_bytes:16777216},observations,candidate_tests_authority:"diagnostic-only"}};
    const result=signEvaluationResult(unsignedResult as any,this.config.evaluator_private_key); verify(result);
    this.db.query("INSERT INTO evaluations VALUES (?,?,?,?)").run(manifest.evaluation_id,JSON.stringify({manifest,result}),artifact,at);
    return {evaluation_id:manifest.evaluation_id,...result};
  }
  private fault(point:string) { if(process.env.KINETI_SERVICE_FAULT===point) process.kill(process.pid,"SIGKILL"); }
  private export(r: SignedServiceRequest) {
    this.validateConfig(); const a=r.arguments as Record<string, any>;
    if (!id(a.evaluation_id) || typeof a.destination!=="string" || typeof a.evaluation_sha256!=="string" || a.policy_sha256!==this.config.pins.policy.sha256 || a.action!=="export-artifact" || Object.keys(a).sort().join(",")!=="action,destination,digest,evaluation_id,evaluation_sha256,policy_sha256") fail("export-binding-denied");
    if(this.config.policy.revoked_evaluations.includes(a.evaluation_id) || this.revocations().evaluations.includes(a.evaluation_id)) fail("evaluation-revoked");
    const stored=this.db.query("SELECT result, artifact, at_ms FROM evaluations WHERE id=?").get(a.evaluation_id) as any;
    if(!stored) fail("evaluation-not-found"); const {manifest,result}=JSON.parse(stored.result);
    validateTrustedEvaluation(manifest,result,{pins:this.config.pins,trustedOwnerKeys:{[this.config.pins.owner.key_id]:this.config.owner_public_key},trustedEvaluatorKeys:{[this.config.pins.evaluator.key_id]:this.evaluatorPublic}});
    if(result.decision!=="accepted" || evaluationDigest(result)!==a.evaluation_sha256 || result.execution.expires_ms<Date.now() || result.execution.at_ms>Date.now() || a.digest!==result.artifact_sha256) fail("evaluation-denied-or-expired");
    owned(stored.artifact,false,true); if(digestFile(stored.artifact)!==a.digest) fail("artifact-substitution");
    const dest=this.destination(a.destination), previous=fs.existsSync(dest)?digestFile(dest):null;
    const binding=sha(canonicalJsonV2(unsigned(r)));
    this.db.exec("BEGIN IMMEDIATE");
    try { this.db.query("INSERT INTO requests VALUES (?,?,?,?,?,?,?,?,?)").run(r.request_id,r.caller,binding,a.evaluation_id,a.destination,a.digest,previous,"pending","durably-reserved"); this.db.exec("COMMIT"); }
    catch(e) { this.db.exec("ROLLBACK"); fail("replay-or-destination-busy"); }
    this.fault("after-pending"); const temp=path.join(path.dirname(dest),`.kineti-${r.request_id}.pending`);
    try {
      durableNew(temp,fs.readFileSync(stored.artifact)); this.fault("after-temp");
      if(digestFile(temp)!==a.digest) fail("wrong-export-bytes"); fs.renameSync(temp,dest); syncDir(path.dirname(dest)); this.fault("after-rename");
      this.db.query("UPDATE requests SET state='applied', reason='durable-rename' WHERE id=?").run(r.request_id); this.fault("after-applied");
      return {request_id:r.request_id,state:"applied",digest:a.digest};
    } catch { this.db.query("UPDATE requests SET state='unknown', reason='inspect-effect-before-owner-recovery' WHERE id=?").run(r.request_id); return {request_id:r.request_id,state:"unknown"}; }
  }
  private reconcile() {
    const rows=this.db.query("SELECT id,destination,digest,previous FROM requests WHERE state='pending'").all() as any[];
    for(const row of rows) {
      let matches=false; try { const dest=this.destination(row.destination); matches=fs.existsSync(dest)&&digestFile(dest)===row.digest&&row.previous!==row.digest; } catch {}
      this.db.query("UPDATE requests SET state=?,reason=? WHERE id=?").run(matches?"applied":"unknown",matches?"startup-confirmed-bytes":"startup-effect-unproven",row.id);
    }
  }
  close() { if(!this.closed) {this.db.close();fs.unlinkSync(this.lock);this.closed=true;} }
}

if(import.meta.main) {
  const [command,root,...rest]=process.argv.slice(2);
  if(command==="--help"||!command) { console.log("Linux protected service: init ROOT | serve ROOT PORT | request ROOT FILE | recover-lock ROOT --owner-inspected. Provision owner-only config.json first. Abandoned service.lock requires owner inspection; never delete request history to retry."); }
  else try {
    if(command==="recover-lock") {
      if(rest[0]!=="--owner-inspected")fail("explicit-owner-inspection-required");owned(root!,true);const lock=path.join(root!,"service.lock");owned(lock);const pid=Number(fs.readFileSync(lock,"utf8"));if(!Number.isSafeInteger(pid)||pid<1)fail("malformed-lock-owner-inspection-required");
      try {process.kill(pid,0);fail("service-process-still-alive");}catch(e){if((e as NodeJS.ErrnoException).code!=="ESRCH")throw e;}
      fs.unlinkSync(lock);syncDir(root!);const recovered=new LinuxProtectedService(root!);recovered.close();console.log("reconciled; request history retained");process.exit(0);
    }
    const service=new LinuxProtectedService(root!,{initialize:command==="init"});
    if(command==="init") {service.close();console.log("initialized");}
    else if(command==="request") {try{console.log(JSON.stringify(await service.handle(JSON.parse(fs.readFileSync(rest[0]!,"utf8")))));}finally{service.close();}}
    else if(command==="serve") {
      const port=Number(rest[0]);if(!Number.isInteger(port)||port<1024||port>65535) fail("explicit-local-port-required");
      Bun.serve({hostname:"127.0.0.1",port,maxRequestBodySize:2*1024*1024,async fetch(req){
        if(req.method!=="POST"||new URL(req.url).pathname!=="/v1/request")return new Response("not found",{status:404});
        let request:SignedServiceRequest;try{request=await req.json() as SignedServiceRequest;}catch{return Response.json({state:"denied",reason:"invalid-json"},{status:400});}
        try{return Response.json(signServiceResponse(request,await service.handle(request),service.config.evaluator_private_key));}catch(e){return Response.json(signServiceResponse(request,{state:"denied",reason:(e as Error).message},service.config.evaluator_private_key),{status:403});}
      }}); console.log(`listening 127.0.0.1:${port}`);
      process.on("SIGTERM",()=>{service.close();process.exit(0);});
    } else {service.close();fail("unknown-service-command");}
  } catch(e) {console.error(JSON.stringify({state:"denied",reason:(e as Error).message}));process.exitCode=1;}
}
