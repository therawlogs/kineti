#!/usr/bin/env bun
/** Native graph/vector adapter over the same encrypted current-memory authority. JSON enters through stdin. */
import {ContextMemoryStore,localContextScope} from "./kineti-memory-job.ts";
const TAG="kineti-native-memory-v1";
export async function nativeMemoryRequest(input:any,root=process.cwd()) {
 const scope=localContextScope(root),store=new ContextMemoryStore(root);
 if(!input||input.version!==1||!["ingest","correct","delete","snapshot"].includes(input.operation)||Object.hasOwn(input,"scope")||Object.hasOwn(input,"user"))throw new Error("native memory requires versioned operation; OS identity is not an input");
 if(input.operation==="delete") {if(input.confirm!==input.id)throw new Error("deletion requires matching confirmation");store.delete(input.id,scope);}
 if(input.operation==="ingest"||input.operation==="correct") {
  const p=input.payload;
  if(!p||[p.category,p.key,p.value].some(x=>typeof x!=="string"||!x.trim())||!Array.isArray(p.embedding)||!p.embedding.length||p.embedding.length>256||p.embedding.some((x:any)=>typeof x!=="number"||!Number.isFinite(x)))throw new Error("native payload requires category, key, value and 1-256 finite embedding values");
  if(input.operation==="correct"&&(typeof input.corrects!=="string"||!input.corrects))throw new Error("correction needs the existing record id");
  const content=JSON.stringify({schema:TAG,category:p.category,key:p.key,value:p.value,embedding:p.embedding,created_at_ms:Date.now()});
  store.append({id:input.id,scope,content,tags:[TAG],sources:[],...(input.operation==="correct"?{correction_of:input.corrects}:{})});
 }
 const snapshot=store.nativeSnapshot(scope);if(snapshot.unavailable.length)throw new Error("native current-memory snapshot contains unavailable encrypted records");
 const records=snapshot.entries.filter(x=>x.tags.includes(TAG)).map(x=>{const payload=JSON.parse(x.content);if(payload.schema!==TAG||!Number.isSafeInteger(payload.created_at_ms))throw new Error("invalid native memory payload");return {id:x.id,...payload};});
 return {version:1,scope,records};
}
if(import.meta.main && process.argv.includes("--help")){console.log("native-memory: versioned JSON on stdin; operations ingest, correct, delete, snapshot; identity derives from the actual OS account/project");}
else if(import.meta.main)try {const text=await Bun.stdin.text();if(Buffer.byteLength(text)>65536)throw new Error("native request too large");console.log(JSON.stringify(await nativeMemoryRequest(JSON.parse(text))));}catch(e){console.error((e as Error).message);process.exitCode=2;}
