import { test, expect } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { LinuxProtectedService, signServiceRequest, type LinuxServiceConfig, type SignedServiceRequest } from "../bin/kineti-linux-service.ts";
import { evaluationDigest, signOwnerManifest } from "../src/security/evaluation-contract.ts";
if(process.platform!=="linux")throw new Error("This acceptance suite requires real Linux isolation; no mock fallback.");
fs.mkdirSync("/sys/fs/cgroup/kineti-acceptance",{recursive:true});fs.writeFileSync("/sys/fs/cgroup/cgroup.subtree_control","+cpu +memory +pids");fs.writeFileSync("/sys/fs/cgroup/kineti-acceptance/cgroup.subtree_control","+cpu +memory +pids");
const base=path.resolve(".kineti/linux-service-acceptance"); fs.mkdirSync(base,{recursive:true,mode:0o700});
const sha=(x:string|Uint8Array)=>crypto.createHash("sha256").update(x).digest("hex");
const pair=()=>{const p=crypto.generateKeyPairSync("ed25519");return {private:p.privateKey.export({type:"pkcs8",format:"pem"}).toString(),public:p.publicKey.export({type:"spki",format:"pem"}).toString()};};
let n=0;
function fixture(script="printf 'hello\\n'\n", checks?: any[]) {
 const root=path.join(base,`case-${++n}`);fs.mkdirSync(root,{mode:0o700});const lib=path.join(root,"runtime-lib");fs.mkdirSync(lib,{mode:0o755});
 const loader=fs.readdirSync("/lib").find(x=>/^ld-musl.*\.so\.1$/.test(x));if(!loader)throw new Error("musl-runtime-required");
 fs.copyFileSync(`/lib/${loader}`,path.join(lib,loader));fs.chmodSync(path.join(lib,loader),0o555);fs.copyFileSync(`/lib/${loader}`,path.join(lib,"libc.musl-aarch64.so.1"));fs.chmodSync(path.join(lib,"libc.musl-aarch64.so.1"),0o555);
 const owner=pair(), evaluator=pair(),caller=pair(); const runtime={bwrap:"/usr/bin/bwrap",prlimit:"/usr/bin/prlimit",busybox:"/bin/busybox",library_dir:lib,cgroup_root:"/sys/fs/cgroup/kineti-acceptance",sha256:{} as Record<string,string>};
 for(const p of [runtime.bwrap,runtime.prlimit,runtime.busybox,...fs.readdirSync(lib).map(x=>path.join(lib,x))])runtime.sha256[p]=sha(fs.readFileSync(p));
 const artifact=Buffer.from(script);const contract={version:1 as const,environment:{LANG:"C"},checks:checks||[{id:"hello-check",argv:["/bin/busybox","sh","/candidate"],stdin_base64:"",expected_exit:0,stdout_sha256:sha("hello\n"),stderr_sha256:sha(""),timeout_ms:3000}]};
 const policy={version:1 as const,destinations:["release/app.sh"],caller_keys:{"client-one":caller.public},revoked_callers:[] as string[],revoked_evaluations:[] as string[],max_age_ms:60000};
 const pins={artifact_sha256:sha(artifact),contract_sha256:evaluationDigest(contract),policy:{id:"export-policy",version:"1",sha256:evaluationDigest(policy)},owner:{id:"owner-service",key_id:"owner-key"},evaluator:{id:"linux-evaluator",key_id:"eval-key"},runner:{id:"linux-bwrap",version:"1",image_sha256:evaluationDigest(runtime.sha256)},required_checks:contract.checks.map(x=>({id:x.id,argv:x.argv,input_sha256:[sha(Buffer.from(x.stdin_base64,"base64"))]}))};
 const config:LinuxServiceConfig={version:1,owner_public_key:owner.public,evaluator_private_key:evaluator.private,pins,contract,policy,runtime};fs.writeFileSync(path.join(root,"config.json"),JSON.stringify(config),{mode:0o600});
 const service=new LinuxProtectedService(root,{initialize:true});
 const manifest=signOwnerManifest({schema:"kineti-independent-evaluation/v1",evaluation_id:`evaluation-${n}`,artifact_sha256:pins.artifact_sha256,contract_sha256:pins.contract_sha256,policy:pins.policy,owner:pins.owner,runner:pins.runner,required_checks:pins.required_checks},owner.private);
 const request=(operation:SignedServiceRequest["operation"],args:Record<string,unknown>,overrides={})=>signServiceRequest({version:1,operation,caller:"client-one",request_id:crypto.randomUUID(),expires_ms:Date.now()+60000,arguments:args,...overrides},caller.private);
 const evaluate=()=>service.handle(request("evaluate",{manifest,artifact_base64:artifact.toString("base64")})) as Promise<any>;
 const exportRequest=(evaluation:any)=>request("export",{action:"export-artifact",evaluation_id:manifest.evaluation_id,evaluation_sha256:evaluationDigest((({evaluation_id:_,...result})=>result)(evaluation)),policy_sha256:pins.policy.sha256,destination:"release/app.sh",digest:sha(artifact)});
 return {root,service,config,manifest,artifact,caller,owner,request,evaluate,exportRequest};
}
function reset(){fs.rmSync(base,{recursive:true,force:true});fs.mkdirSync(base,{mode:0o700});} reset();
test("real isolated control executes trusted checks, signs observations and exports exact bytes",async()=>{
 const f=fixture();try {const e=await f.evaluate();expect(e.decision).toBe("accepted");expect(e.execution.observations[0].stdout_sha256).toBe(sha("hello\n"));const r=f.exportRequest(e);expect(await f.service.handle(r)).toMatchObject({state:"applied"});expect(fs.readFileSync(path.join(f.root,"output/release/app.sh"))).toEqual(f.artifact);await expect(f.service.handle(r)).rejects.toThrow("replay");await expect(f.service.handle({...r,request_id:crypto.randomUUID()})).rejects.toThrow("signature");const newRequest=f.request("export",r.arguments);await expect(f.service.handle(newRequest)).rejects.toThrow("replay");}finally{f.service.close();}
});
test("candidate has UID 65534 and cannot read service policy or signing authority",async()=>{
 const script=`/bin/busybox id -u\ntest ! -e '${base}' || exit 4\ntest ! -e /home || exit 5\ntest ! -e /etc/passwd || exit 6\n`;
 const f=fixture(script,[{id:"isolation-check",argv:["/bin/busybox","sh","/candidate"],stdin_base64:"",expected_exit:0,stdout_sha256:sha("65534\n"),stderr_sha256:sha(""),timeout_ms:3000}]);try {expect((await f.evaluate()).decision).toBe("accepted");}finally{f.service.close();}
});
test("failed output and timed-out checks cannot export",async()=>{
 for(const script of ["printf forged\\n\n","while :; do :; done\n"]){const f=fixture(script);try{const e=await f.evaluate();expect(e.decision).not.toBe("accepted");await expect(f.service.handle(f.exportRequest(e))).rejects.toThrow("evaluation-denied");expect(fs.readdirSync(path.join(f.root,"output"))).toEqual([]);}finally{f.service.close();}}
},15000);
test("artifact, owner, contract, inventory, runtime and request substitutions are denied",async()=>{
 const f=fixture();try{
 await expect(f.service.handle(f.request("evaluate",{manifest:f.manifest,artifact_base64:Buffer.from("changed").toString("base64")}))).rejects.toThrow("artifact-substitution");
 for(const edit of [{contract_sha256:sha("wrong")},{policy:{...f.manifest.policy,sha256:sha("wrong")}},{required_checks:[]},{runner:{...f.manifest.runner,version:"other"}}]){const m=signOwnerManifest({...f.manifest,...edit},f.owner.private);await expect(f.service.handle(f.request("evaluate",{manifest:m,artifact_base64:f.artifact.toString("base64")}))).rejects.toThrow();}
 const attacker=pair();const m=signOwnerManifest(f.manifest,attacker.private);await expect(f.service.handle(f.request("evaluate",{manifest:m,artifact_base64:f.artifact.toString("base64")}))).rejects.toThrow("owner");
 const e=await f.evaluate();const r=f.exportRequest(e);
 for(const edit of [{destination:"../sentinel"},{digest:sha("wrong")},{evaluation_sha256:sha("other")},{policy_sha256:sha("changed")},{action:"delete"}])await expect(f.service.handle(f.request("export",{...r.arguments,...edit}))).rejects.toThrow();
 await expect(f.service.handle(f.request("export",r.arguments,{expires_ms:Date.now()-1}))).rejects.toThrow("expired");
 await expect(f.service.handle(f.request("export",r.arguments,{caller:"other-client"}))).rejects.toThrow("signature");
 expect(fs.readdirSync(path.join(f.root,"output"))).toEqual([]);
 }finally{f.service.close();}
});
test("two simultaneous exports consume an authorization once",async()=>{
 const f=fixture();try {const e=await f.evaluate(),r=f.exportRequest(e);const results=await Promise.allSettled([f.service.handle(r),f.service.handle(f.request("export",r.arguments))]);expect(results.filter(x=>x.status==="fulfilled")).toHaveLength(1);expect(results.filter(x=>x.status==="rejected")).toHaveLength(1);expect((f.service.db.query("SELECT count(*) AS n FROM requests").get() as any).n).toBe(1);}finally{f.service.close();}
});
test("restart preserves replay, symlink sentinels, missing and corrupt initialized state block",async()=>{
 const f=fixture();const e=await f.evaluate(),r=f.exportRequest(e);fs.writeFileSync(path.join(f.root,"sentinel"),"unchanged",{mode:0o600});fs.symlinkSync(path.join(f.root,"sentinel"),path.join(f.root,"output/release"));await expect(f.service.handle(r)).rejects.toThrow();expect(fs.readFileSync(path.join(f.root,"sentinel"),"utf8")).toBe("unchanged");fs.unlinkSync(path.join(f.root,"output/release"));await f.service.handle(r);f.service.close();
 const reopened=new LinuxProtectedService(f.root);await expect(reopened.handle(r)).rejects.toThrow("replay");reopened.close();
 fs.renameSync(path.join(f.root,"requests.sqlite"),path.join(f.root,"requests.saved"));expect(()=>new LinuxProtectedService(f.root)).toThrow();fs.renameSync(path.join(f.root,"requests.saved"),path.join(f.root,"requests.sqlite"));fs.writeFileSync(path.join(f.root,"requests.sqlite"),"corrupt");expect(()=>new LinuxProtectedService(f.root)).toThrow();
});
test("deterministic crashes reconcile only proved bytes and never reset authorization",async()=>{
 for(const point of ["after-pending","after-temp","after-rename","after-applied"]){const f=fixture();const e=await f.evaluate(),r=f.exportRequest(e);f.service.close();const requestPath=path.join(f.root,"request.json");fs.writeFileSync(requestPath,JSON.stringify(r),{mode:0o600});const proc=Bun.spawnSync([process.execPath,path.resolve("bin/kineti-linux-service.ts"),"request",f.root,requestPath],{env:{...process.env,KINETI_SERVICE_FAULT:point},stdout:"pipe",stderr:"pipe"});expect(proc.exitCode).not.toBe(0);if(!fs.existsSync(path.join(f.root,"service.lock")))throw new Error(new TextDecoder().decode(proc.stderr));expect(fs.existsSync(path.join(f.root,"service.lock"))).toBe(true);const recovery=Bun.spawnSync([process.execPath,path.resolve("bin/kineti-linux-service.ts"),"recover-lock",f.root,"--owner-inspected"],{stdout:"pipe",stderr:"pipe"});expect(recovery.exitCode).toBe(0);
 const service=new LinuxProtectedService(f.root);try {const row=await service.handle(f.request("inspect",{request_id:r.request_id})) as any;expect(row.state).toBe(point==="after-rename"||point==="after-applied"?"applied":"unknown");await expect(service.handle(f.request("export",r.arguments))).rejects.toThrow("replay");}finally{service.close();}}
},30000);
test("revocation is read from owner-only storage for each request",async()=>{
 const f=fixture();try{const e=await f.evaluate(),r=f.exportRequest(e);const p=path.join(f.root,"revocations.json");fs.writeFileSync(p,JSON.stringify({version:1,callers:[],evaluations:[f.manifest.evaluation_id]}),{mode:0o600});await expect(f.service.handle(r)).rejects.toThrow("revoked");fs.writeFileSync(p,JSON.stringify({version:1,callers:["client-one"],evaluations:[]}),{mode:0o600});await expect(f.service.handle(r)).rejects.toThrow("revoked");fs.unlinkSync(p);await expect(f.service.handle(r)).rejects.toThrow();expect(fs.readdirSync(path.join(f.root,"output"))).toEqual([]);}finally{f.service.close();}
});
test("memory exhaustion is contained by a real cgroup",async()=>{
 const f=fixture("/bin/busybox awk 'BEGIN { while(1) a[++i]=sprintf(\"%0100000d\",i) }'\n");try{const e=await f.evaluate();expect(e.decision).not.toBe("accepted");expect(e.execution.observations[0].fault).toBe("memory-limit");}finally{f.service.close();}
},15000);
test("a second Linux UID cannot directly read authority or change protected output",async()=>{
 if(process.getuid!()!==0)throw new Error("Root test supervisor required to exercise a second real UID.");
 const f=fixture();try{const e=await f.evaluate();await f.service.handle(f.exportRequest(e));for(const file of ["config.json","requests.sqlite","output/release/app.sh"]){const p=Bun.spawnSync(["/bin/setpriv","--reuid=2501","--regid=2501","--clear-groups","/bin/busybox","cat",path.join(f.root,file)],{stdout:"pipe",stderr:"pipe"});expect(p.exitCode).not.toBe(0);expect(p.stdout.length).toBe(0);}const write=Bun.spawnSync(["/bin/setpriv","--reuid=2501","--regid=2501","--clear-groups","/bin/busybox","sh","-c",'printf changed > "$1"',"client",path.join(f.root,"output/release/app.sh")],{stdout:"pipe",stderr:"pipe"});expect(write.exitCode).not.toBe(0);expect(fs.readFileSync(path.join(f.root,"output/release/app.sh"))).toEqual(f.artifact);}finally{f.service.close();}
});
test("real HTTP calls from another Linux UID allow authentic evaluation/export and deny substitution",async()=>{
 const f=fixture();f.service.close();
 const reserve=Bun.serve({hostname:"127.0.0.1",port:0,fetch:()=>new Response("reserve")});const port=reserve.port!;reserve.stop(true);
 const {spawn}=await import("node:child_process");
 const child=spawn(process.execPath,[path.resolve("bin/kineti-linux-service.ts"),"serve",f.root,String(port)],{stdio:["ignore","pipe","pipe"]});
 try {
  await new Promise<void>((resolve,reject)=>{const timer=setTimeout(()=>reject(new Error("service did not become ready")),5000);child.stdout.on("data",b=>{if(String(b).includes("listening")){clearTimeout(timer);resolve();}});child.once("exit",()=>{clearTimeout(timer);reject(new Error("service exited before readiness"));});});
  const clientCode=`import {signServiceRequest,verifyServiceResponse} from ${JSON.stringify(path.resolve("src/security/service-request.ts"))};const input=JSON.parse(await Bun.stdin.text());const request=signServiceRequest(input.request,input.key);const response=await fetch(input.url,{method:"POST",redirect:"error",headers:{"content-type":"application/json"},body:JSON.stringify(request)});console.log(JSON.stringify({status:response.status,result:verifyServiceResponse(request,await response.json(),input.service_key)}));`;
  function call(operation:SignedServiceRequest["operation"],args:Record<string,unknown>){const input={url:`http://127.0.0.1:${port}/v1/request`,key:f.caller.private,service_key:crypto.createPublicKey(f.config.evaluator_private_key).export({type:"spki",format:"pem"}).toString(),request:{version:1,operation,caller:"client-one",request_id:crypto.randomUUID(),expires_ms:Date.now()+60000,arguments:args}};const p=Bun.spawnSync(["/bin/setpriv","--reuid=2501","--regid=2501","--clear-groups",process.execPath,"-e",clientCode],{stdin:Buffer.from(JSON.stringify(input)),stdout:"pipe",stderr:"pipe"});if(p.exitCode!==0)throw new Error(new TextDecoder().decode(p.stderr));return JSON.parse(new TextDecoder().decode(p.stdout));}
  const evaluated=call("evaluate",{manifest:f.manifest,artifact_base64:f.artifact.toString("base64")});expect(evaluated.status).toBe(200);expect(evaluated.result.decision).toBe("accepted");const args=f.exportRequest(evaluated.result).arguments;
  const denied=call("export",{...args,destination:"../../sentinel"});expect(denied.status).toBe(403);expect(denied.result.state).toBe("denied");expect(fs.readdirSync(path.join(f.root,"output"))).toEqual([]);
  expect(call("export",args).result.state).toBe("applied");expect(fs.readFileSync(path.join(f.root,"output/release/app.sh"))).toEqual(f.artifact);
 }finally{child.kill("SIGTERM");await new Promise<void>(r=>child.once("exit",()=>r()));}
},15000);
