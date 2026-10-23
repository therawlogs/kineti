import {afterAll, beforeAll, expect, test} from "bun:test";
import crypto from "node:crypto";
import path from "node:path";
const containers: string[] = [];
const container=`kineti-evaluator-${crypto.randomUUID()}`;
const env={...process.env,DOCKER_CONFIG:process.env.DOCKER_CONFIG??path.join(process.cwd(),".kineti/docker")};
function docker(args:string[],input?:string){const r=Bun.spawnSync(["docker",...args],{env,stdin:input===undefined?"ignore":Buffer.from(input),stdout:"pipe",stderr:"pipe"});return {code:r.exitCode,text:r.stdout.toString(),error:r.stderr.toString()};}
async function ready(name=container){for(let i=0;i<8;i++){if(docker(["exec",name,"test","-S","/run/kineti/evaluator.sock"]).code===0)return;await Bun.sleep(100);}throw new Error("Evaluator startup failed: "+JSON.stringify(docker(["logs",name])));}
async function start(name:string, failpoint?:string){
 containers.push(name);
 const args=["run","--init","-d","--name",name,"--network=none","--read-only","--memory=256m","--pids-limit=64","--cap-drop=ALL","--cap-add=SETUID","--cap-add=SETGID","--cap-add=CHOWN","--cap-add=DAC_OVERRIDE","--mount",`type=volume,source=${name}-state,target=/var/lib/kineti`,"--tmpfs","/var/lib/kineti-jobs:mode=0711","--tmpfs","/run:mode=0755","--tmpfs","/tmp:mode=1777,noexec,nosuid"];
 if(failpoint)args.push("--env",`KINETI_EVALUATOR_FAILPOINT=${failpoint}`);
 args.push("kineti-evaluator:acceptance");
 const r=docker(args);if(r.code!==0)throw new Error(r.error);await ready(name);
}
beforeAll(async()=>{
 await start(container);
 const sentinel=docker(["exec",container,"sh","-c","printf unchanged > /var/lib/kineti/state/outside-sentinel"]);expect(sentinel.code).toBe(0);
});
afterAll(()=>{for(const name of containers){docker(["rm","-f",name]);docker(["volume","rm",`${name}-state`]);}});
const genuine='#!/bin/sh\nprintf "%s\\n" "$(( $1 + $2 ))"\n';
function request(artifact=genuine){return {operation_id:crypto.randomUUID(),run_id:crypto.randomUUID(),session_id:crypto.randomUUID(),artifact_hex:Buffer.from(artifact).toString("hex"),artifact_sha256:crypto.createHash("sha256").update(artifact).digest("hex"),expires_at:Math.floor(Date.now()/1000)+120};}
function submit(req:ReturnType<typeof request>,name=container){const r=docker(["exec","-i","--user","10003:10003",name,"sh","-c",`cat > /tmp/${req.operation_id}.json; /usr/local/bin/kineti-cli evaluator submit /run/kineti/evaluator.sock /tmp/${req.operation_id}.json`],JSON.stringify(req));if(!r.text)return {code:r.code,value:{status:"unknown",reason:r.error}};return {code:r.code,value:JSON.parse(r.text)};}
function destination(req:ReturnType<typeof request>){return docker(["exec",container,"cat",`/var/lib/kineti/state/exports/${req.operation_id}.artifact`]);}
function verifySignature(value:any){const {signature, ...unsigned}=value;const sort=(v:any):any=>Array.isArray(v)?v.map(sort):v&&typeof v==="object"?Object.fromEntries(Object.keys(v).sort().map(k=>[k,sort(v[k])])):v;const publicKey=crypto.createPublicKey({key:Buffer.concat([Buffer.from("302a300506032b6570032100","hex"),Buffer.from(value.issuer,"hex")]),type:"spki",format:"der"});expect(crypto.verify(null,Buffer.from(JSON.stringify(sort(unsigned))),publicKey,Buffer.from(signature,"hex"))).toBe(true);}
test("fabricated summaries, early exit and incomplete checks deny export",()=>{
 for(const artifact of ['echo "4 pass, 0 fail"\n','exit 0\n','[ "$1" = 2 ] && echo 5\n']){const req=request(artifact),r=submit(req);expect(r.code).toBe(1);expect(r.value.status).toBe("denied");expect(destination(req).code).not.toBe(0);}
 expect(docker(["exec",container,"cat","/var/lib/kineti/state/outside-sentinel"]).text).toBe("unchanged");
});
test("genuine execution exports exact bytes with a verifiable service signature; replay is idempotent",()=>{
 const req=request(),r=submit(req);expect(r.value).toMatchObject({status:"applied"});expect(r.code).toBe(0);expect(r.value.status).toBe("applied");expect(docker(["exec",container,"cat",`/var/lib/kineti/state/artifacts/${req.operation_id}.artifact`]).text).toBe(genuine);expect(r.value.checks).toHaveLength(4);expect(r.value.checks[0].command.slice(-2)).toEqual(["2","3"]);expect(r.value.previous.status).toBe("pending");verifySignature(r.value.previous);verifySignature(r.value);expect(destination(req).text).toBe(genuine);
 const replay=submit(req);expect(replay.value).toEqual(r.value);
 const changed=submit({...req,session_id:crypto.randomUUID()});expect(changed.code).toBe(1);expect(changed.value.reason).toContain("substitution");expect(destination(req).text).toBe(genuine);
});
test("expired, changed artifacts, unknown fields and revocations deny without effects",()=>{
 const expired=request();expired.expires_at=1;expect(submit(expired).value.status).toBe("denied");expect(destination(expired).code).not.toBe(0);
 const altered=request();altered.artifact_hex=Buffer.from("exit 0").toString("hex");expect(submit(altered).value.reason).toContain("digest");
 const unknown={...request(),destination:"/outside"};expect(submit(unknown).value.reason).toContain("unknown");
 const revoked=request();const policy={version:1,inventory:"sum-v1",revoked_operations:[revoked.operation_id],revoked_runs:[],revoked_issuers:[]};
 expect(docker(["exec","-i",container,"sh","-c","cat > /var/lib/kineti/state/policy.json"],JSON.stringify(policy)).code).toBe(0);
 expect(submit(revoked).value.reason).toContain("revoked");expect(destination(revoked).code).not.toBe(0);
});
test("candidate cannot alter trust, export, outside sentinel, or gain privileges",()=>{
 for(const target of ["/var/lib/kineti/state/signing.key","/var/lib/kineti/state/policy.json","/var/lib/kineti/state/outside-sentinel","/var/lib/kineti/state/exports/escape"]){expect(docker(["exec","--user","10002:10002",container,"sh","-c",`printf hacked > ${target}`]).code).not.toBe(0);}
 expect(docker(["exec","--user","10002:10002",container,"test","-e","/var/run/docker.sock"]).code).not.toBe(0);
 const req=request('id -u\n');expect(submit(req).value.status).toBe("denied");
 expect(docker(["exec",container,"cat","/var/lib/kineti/state/outside-sentinel"]).text).toBe("unchanged");
});

test("concurrent submissions apply once and preserve execution identity",async()=>{
 const req=request();
 expect(docker(["exec","-i",container,"sh","-c","cat > /tmp/concurrent.json"],JSON.stringify(req)).code).toBe(0);
 const children=Array.from({length:2},()=>Bun.spawn(["docker","exec","--user","10003:10003",container,"/usr/local/bin/kineti-cli","evaluator","submit","/run/kineti/evaluator.sock","/tmp/concurrent.json"],{env,stdout:"pipe",stderr:"pipe"}));
 const results=await Promise.all(children.map(async child=>({code:await child.exited,value:JSON.parse(await new Response(child.stdout).text())})));
 expect(results.map(r=>r.code)).toEqual([0,0]);expect(results[0].value).toEqual(results[1].value);expect(destination(req).text).toBe(genuine);
});
test("crash after rename reconciles the same durable operation without another export",async()=>{
 const name=`kineti-crash-${crypto.randomUUID()}`;await start(name,"after-rename");const req=request();
 expect(submit(req,name).value.status).toBe("unknown");
 expect(docker(["start",name]).code).toBe(0);await ready(name);
 const reconciled=submit(req,name);expect(reconciled.code).toBe(0);expect(reconciled.value.status).toBe("applied");verifySignature(reconciled.value);
 expect(submit(req,name).value).toEqual(reconciled.value);
},15_000);
test("crash before export stays unknown and blocks new operations",async()=>{
 const name=`kineti-pending-${crypto.randomUUID()}`;await start(name,"after-intent");const req=request();
 expect(submit(req,name).value.status).toBe("unknown");expect(docker(["start",name]).code).toBe(0);await ready(name);
 expect(submit(req,name).value.status).toBe("unknown");expect(submit(request(),name).value.reason).toContain("unreconciled");
},15_000);
