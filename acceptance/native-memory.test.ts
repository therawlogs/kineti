import {test,expect,afterEach} from "bun:test";
import fs from "node:fs";
import path from "node:path";
const repo=process.cwd(),binary=process.env.KINETI_NATIVE_CLI||path.join(repo,"core-native/target/debug/kineti-cli"),bridge=path.join(repo,"bin/kineti-native-memory.ts");
if(!fs.existsSync(binary))throw new Error("Build the real native CLI first; this acceptance has no mock fallback.");
const roots:string[]=[];function project(){const base=path.join(repo,".kineti/native-memory-acceptance");fs.mkdirSync(base,{recursive:true});const root=fs.mkdtempSync(path.join(base,"project-"));fs.mkdirSync(path.join(root,".kineti"));fs.writeFileSync(path.join(root,".kineti/state.json"),JSON.stringify({project:"same-display-label"}),{mode:0o600});roots.push(root);return root;}
afterEach(()=>{for(const root of roots.splice(0))fs.rmSync(root,{recursive:true,force:true});});
function call(root:string,input:any){const p=Bun.spawnSync([binary,"memory"],{cwd:root,env:{...process.env,KINETI_BUN:process.execPath,KINETI_NATIVE_BRIDGE:bridge},stdin:Buffer.from(JSON.stringify({version:1,...input})),stdout:"pipe",stderr:"pipe"});if(p.exitCode!==0)throw new Error(new TextDecoder().decode(p.stderr));return JSON.parse(new TextDecoder().decode(p.stdout));}
test("native graph and vector retrieval persist across processes, correction and deletion share one current view",()=>{
 const root=project();expect(call(root,{operation:"ingest",id:"mem-native-one",category:"synthetic",key:"color",value:"old green marker",embedding:[1,0]}).state).toBe("stored");
 const before=call(root,{operation:"graph"});expect(before[0].value).toBe("old green marker");expect(call(root,{operation:"graph"})[0].id).toBe(before[0].id);expect(call(root,{operation:"vectors",embedding:[1,0]})[0].id).toBe("mem-native-one");
 call(root,{operation:"correct",corrects:"mem-native-one",id:"mem-native-two",category:"synthetic",key:"color",value:"new blue marker",embedding:[0,1]});
 expect(call(root,{operation:"graph"}).map((x:any)=>x.value)).toEqual(["new blue marker"]);expect(JSON.stringify(call(root,{operation:"vectors",embedding:[1,0]}))).not.toContain("old green");
 call(root,{operation:"delete",id:"mem-native-two",confirm:"mem-native-two"});expect(call(root,{operation:"graph"})).toEqual([]);expect(call(root,{operation:"vectors",embedding:[0,1]})).toEqual([]);expect(fs.existsSync(path.join(root,".kineti/memory/keys/mem-native-one.key"))).toBe(false);expect(fs.existsSync(path.join(root,".kineti/memory/keys/mem-native-two.key"))).toBe(false);
},30000);
test("same project labels do not share native memory, and identity cannot be an API input",()=>{
 const a=project(),b=project();call(a,{operation:"ingest",id:"mem-native-private",category:"synthetic",key:"note",value:"project A only",embedding:[1,0]});expect(call(b,{operation:"graph"})).toEqual([]);expect(()=>call(a,{operation:"graph",scope:{user:"another-user",project:"another-project"}})).toThrow("identity");
 const raw=fs.readFileSync(path.join(a,".kineti/journal.jsonl"),"utf8");expect(raw).not.toContain("project A only");
},30000);
