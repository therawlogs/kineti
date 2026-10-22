import {test,expect} from "bun:test";
import fs from "node:fs";
import path from "node:path";
if(process.platform!=="linux"||process.getuid!()!==0)throw new Error("This acceptance requires a Linux root supervisor and two real, unprivileged test accounts.");
const repo=process.cwd(),binary=process.env.KINETI_NATIVE_CLI||path.join(repo,".kineti/linux-target/debug/kineti-cli"),bridge=path.join(repo,"bin/kineti-native-memory.ts"),base=path.join(repo,".kineti/memory-os-users");
fs.mkdirSync(base,{recursive:true,mode:0o755});fs.chmodSync(path.join(repo,".kineti"),0o755);fs.chmodSync(base,0o755);
function project(uid:number,label:string){const root=fs.mkdtempSync(path.join(base,`uid-${uid}-`));fs.mkdirSync(path.join(root,".kineti"),{mode:0o700});fs.mkdirSync(path.join(root,"home"),{mode:0o700});fs.writeFileSync(path.join(root,".kineti/state.json"),JSON.stringify({project:label}),{mode:0o600});for(const p of [root,path.join(root,".kineti"),path.join(root,"home"),path.join(root,".kineti/state.json")])fs.chownSync(p,uid,uid);fs.chmodSync(root,0o700);return root;}
function asUser(uid:number,root:string,body:any,home=path.join(root,"home")){const r=Bun.spawnSync(["/bin/setpriv",`--reuid=${uid}`,`--regid=${uid}`,"--clear-groups",binary,"memory"],{cwd:root,env:{...process.env,HOME:home,KINETI_BUN:process.execPath,KINETI_NATIVE_BRIDGE:bridge,KINETI_MACHINE_DIR:path.join(root,".kineti/machine")},stdin:Buffer.from(JSON.stringify({version:1,...body})),stdout:"pipe",stderr:"pipe"});return {code:r.exitCode,text:new TextDecoder().decode(r.stdout),error:new TextDecoder().decode(r.stderr)};}
function call(uid:number,root:string,body:any,home?:string){const r=asUser(uid,root,body,home);if(r.code!==0)throw new Error(r.error);return JSON.parse(r.text);}
test("two real OS accounts and two projects each stay separate through restart, native graph/vector, correction and deletion",()=>{
 for(const uid of [2501,2502]){const account=Bun.spawnSync(["/usr/bin/id",uid===2501?"kineti-test-one":"kineti-test-two"],{stdout:"pipe",stderr:"pipe"});expect(account.exitCode).toBe(0);expect(new TextDecoder().decode(account.stdout)).toContain(`uid=${uid}(`);}
 const roots=[project(2501,"same-name"),project(2501,"same-name"),project(2502,"same-name"),project(2502,"same-name")];
 const users=[2501,2501,2502,2502];
 for(let i=0;i<roots.length;i++)call(users[i],roots[i],{operation:"ingest",id:"mem-os-initial",category:"synthetic",key:"marker",value:`private synthetic marker ${i}`,embedding:[1,0]});
 for(let i=0;i<roots.length;i++){
  const uid=users[i],root=roots[i],other=roots[(i+2)%roots.length];expect(call(uid,root,{operation:"graph"})[0].value).toBe(`private synthetic marker ${i}`);expect(call(uid,root,{operation:"vectors",embedding:[1,0]})[0].snippet).toBe(`private synthetic marker ${i}`);
  expect(asUser(uid,other,{operation:"graph"}).code).not.toBe(0);expect(asUser(uid,root,{operation:"graph",scope:{user:`local-uid-${uid===2501?2502:2501}`,project:"same-name"}}).code).not.toBe(0);
  expect(call(uid,root,{operation:"graph"},path.join(other,"home"))[0].value).toBe(`private synthetic marker ${i}`);
  const journal=fs.readFileSync(path.join(root,".kineti/journal.jsonl"),"utf8");expect(journal).not.toContain(`private synthetic marker ${i}`);expect(JSON.parse(journal.split("\n")[0]).data.scope.user).toBe(`local-uid-${uid}`);
  call(uid,root,{operation:"correct",corrects:"mem-os-initial",id:"mem-os-corrected",category:"synthetic",key:"marker",value:`corrected marker ${i}`,embedding:[0,1]});expect(call(uid,root,{operation:"graph"}).map((r:any)=>r.value)).toEqual([`corrected marker ${i}`]);expect(JSON.stringify(call(uid,root,{operation:"vectors",embedding:[1,0]}))).not.toContain("private synthetic");
  call(uid,root,{operation:"delete",id:"mem-os-corrected",confirm:"mem-os-corrected"});expect(call(uid,root,{operation:"graph"})).toEqual([]);expect(call(uid,root,{operation:"vectors",embedding:[0,1]})).toEqual([]);expect(fs.existsSync(path.join(root,".kineti/memory/keys/mem-os-initial.key"))).toBe(false);expect(fs.existsSync(path.join(root,".kineti/memory/keys/mem-os-corrected.key"))).toBe(false);
 }
},60000);
