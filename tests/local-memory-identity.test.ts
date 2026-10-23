import {test,expect,afterEach} from "bun:test";
import fs from "node:fs";
import path from "node:path";
import {ContextMemoryStore,localContextScope} from "../bin/kineti-memory-job.ts";
const repo=path.resolve(import.meta.dir,".."),roots:string[]=[];
function project(){const base=path.join(repo,".kineti/identity-regression");fs.mkdirSync(base,{recursive:true});const p=fs.mkdtempSync(path.join(base,"project-"));fs.mkdirSync(path.join(p,".kineti"));fs.writeFileSync(path.join(p,".kineti/state.json"),JSON.stringify({project:"same-project-name"}),{mode:0o600});roots.push(p);return p;}
afterEach(()=>{for(const p of roots.splice(0))fs.rmSync(p,{recursive:true,force:true});});
test("local memory cannot select another OS identity",()=>{const dir=project(),scope=localContextScope(dir),store=new ContextMemoryStore(dir,repo);expect(()=>store.append({id:"mem-identity-one",scope:{...scope,user:"local-uid-999999"},content:"private",tags:[],sources:[]})).toThrow();});
test("local memory cannot select another project scope",()=>{const dir=project(),scope=localContextScope(dir),store=new ContextMemoryStore(dir,repo);expect(()=>store.append({id:"mem-identity-two",scope:{...scope,project:"other-project"},content:"private",tags:[],sources:[]})).toThrow();});
test("two real folders with the same project label have different current identity",()=>{const a=project(),b=project();expect(localContextScope(a).project).not.toBe(localContextScope(b).project);});
test("legacy scope needs an explicit owner binding and original journal bytes stay unchanged",async()=>{
 const {bindLegacyMemory}=await import("../bin/kineti-memory-job.ts");const {commitCheckpoint}=await import("../bin/ledger-integrity.ts");const {computeDelimitedHash}=await import("../bin/lib.ts");const {legacyCanonicalJson}=await import("../src/security/canonical-json.ts");
 const dir=project(),scope=localContextScope(dir),store=new ContextMemoryStore(dir,repo);store.append({id:"mem-legacy-binding",scope,content:"reviewed legacy note",tags:[],sources:[]});
 const file=path.join(dir,".kineti/journal.jsonl"),record=JSON.parse(fs.readFileSync(file,"utf8"));record.project="same-project-name";record.data.scope.project="same-project-name";record.hash=computeDelimitedHash([record.prev_hash,record.at,record.id,legacyCanonicalJson(record.data)]);const before=JSON.stringify(record)+"\n";fs.writeFileSync(file,before);commitCheckpoint(file);
 expect(store.retrieve(scope).entries).toEqual([]);expect(store.retrieve(scope).unavailable).toEqual(["legacy-scope:mem-legacy-binding"]);expect(()=>bindLegacyMemory(dir,"BIND wrong-project")).toThrow("owner");expect(bindLegacyMemory(dir,`BIND ${scope.project}`).count).toBe(1);expect(fs.readFileSync(file,"utf8").startsWith(before)).toBe(true);expect(store.retrieve(scope).entries[0].content).toBe("reviewed legacy note");
 store.append({id:"mem-legacy-corrected",scope,content:"corrected note",tags:[],sources:[],correction_of:"mem-legacy-binding"});store.delete("mem-legacy-corrected",scope);expect(store.retrieve(scope).entries).toEqual([]);expect(fs.existsSync(path.join(dir,".kineti/memory/keys/mem-legacy-binding.key"))).toBe(false);
});
test("memory storage symlinks cannot write into a different project",()=>{
 const a=project(),b=project();fs.mkdirSync(path.join(a,".kineti/memory"));fs.symlinkSync(b,path.join(a,".kineti/memory/keys"));const store=new ContextMemoryStore(a,repo);expect(()=>store.append({id:"mem-storage-link",scope:localContextScope(a),content:"stay in A",tags:[],sources:[]})).toThrow();expect(fs.existsSync(path.join(b,"mem-storage-link.key"))).toBe(false);
});
test("a refused memory creation preserves an existing orphan key for owner recovery",()=>{
 const dir=project(),keys=path.join(dir,".kineti/memory/keys");fs.mkdirSync(keys,{recursive:true,mode:0o700});const key=path.join(keys,"mem-existing-orphan.key"),before=Buffer.alloc(32,7);fs.writeFileSync(key,before,{mode:0o600});expect(()=>new ContextMemoryStore(dir,repo).append({id:"mem-existing-orphan",scope:localContextScope(dir),content:"new text",tags:[],sources:[]})).toThrow();expect(fs.readFileSync(key)).toEqual(before);
});
