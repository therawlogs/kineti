/** Shared locked accounting snapshots. Declared usage is not a provider invoice. */
import fs from "node:fs";
import path from "node:path";
import { loadLimits, writeJson } from "./lib.ts";
import { withFileLock, readStrictLines } from "./ledger-integrity.ts";
export interface SpendSnapshot {
  routing_reservations?: Record<string, { session_id: string; stage: string; microcents: number; status: "reserved" | "unknown" }>;
  tripped:boolean; reason:string|null; total_microcents:number; total_usd:number;
  by_stage:Record<string,number>; by_stage_microcents:Record<string,number>; entries:number;
}
export function withAccountingLock<T>(root:string,fn:()=>T):T{return withFileLock(path.join(root,".kineti/spend.lock"),fn);}
const uuid=(v:unknown):v is string=>typeof v==="string"&&/^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(v);
const money=(v:unknown):v is number=>typeof v==="number"&&Number.isFinite(v)&&v>=0;
const integer=(v:unknown):v is number=>money(v)&&Number.isSafeInteger(v);
const object=(v:unknown):v is Record<string,any>=>!!v&&typeof v==="object"&&!Array.isArray(v);
const stageOK=(s:unknown):s is string=>typeof s==="string"&&!!s.trim()&&!s.includes("\0")&&!Object.hasOwn(Object.prototype,s)&&s!=="prototype";
const close=(usd:number,micro:number)=>Math.abs(usd-micro/1e6)<=0.000051;
export function readSpending(root:string,allowMissing=false):SpendSnapshot{
 const file=path.join(root,".kineti/spend.json"),log=path.join(root,".kineti/spend.log.jsonl"),marker=path.join(root,".kineti/spend.initialized.json");
 const damaged=():never=>{throw new Error("spend accounting is damaged or incomplete; owner inspection and explicit recovery required");};
 if(!fs.existsSync(file)){
  if(!allowMissing||fs.existsSync(log)||fs.existsSync(marker))return damaged();
  return {tripped:false,reason:null,total_microcents:0,total_usd:0,by_stage:{},by_stage_microcents:{},entries:0};
 }
 let s:any;try{s=JSON.parse(fs.readFileSync(file,"utf8"));}catch{return damaged();}
 if(!object(s)||typeof s.tripped!=="boolean")return damaged();
 const total=s.total_microcents===undefined&&money(s.total_usd)?Math.round(s.total_usd*1e6):s.total_microcents;
 if(!integer(total))return damaged();
 // A verified empty legacy seed is readable without silently repairing stored history.
 const totalUsd=s.total_usd??(total===0?0:undefined),entries=s.entries??(total===0?0:undefined),usdStages=s.by_stage??(total===0?{}:undefined);
 if(!money(totalUsd)||!integer(entries)||!close(totalUsd,total)||!object(usdStages)||!Object.values(usdStages).every(money)||(s.reason!==undefined&&s.reason!==null&&typeof s.reason!=="string"))return damaged();
 const stages=s.by_stage_microcents??Object.fromEntries(Object.entries(usdStages).map(([k,v])=>[k,Math.round((v as number)*1e6)]));
 if(!object(stages)||!Object.keys(stages).every(stageOK)||!Object.values(stages).every(integer))return damaged();
 const pending=s.routing_reservations??{};
 if(!object(pending))return damaged();
 const opened=new Map<string,any>();let sum=0,count=0;const stageSums:Record<string,number>=Object.create(null);
 const records=readStrictLines<any>(log);
 for(const r of records){
  if(!object(r))return damaged();
  if(r.kind==="reservation"){
   if(!uuid(r.run_id)||!uuid(r.session_id)||!stageOK(r.stage)||!integer(r.microcents)||r.microcents===0||opened.has(r.run_id))return damaged();
   opened.set(r.run_id,r);continue;
  }
  if(r.kind==="settlement"||r.kind==="reservation-cancelled"){
   const prior=opened.get(r.run_id);if(!prior||prior.session_id!==r.session_id||prior.stage!==r.stage)return damaged();opened.delete(r.run_id);
   if(r.kind==="reservation-cancelled")continue;
  }else if(r.kind!==undefined)return damaged();
  if(!stageOK(r.stage)||!money(r.usd))return damaged();
  if(r.kind===undefined&&(typeof r.model!=="string"||!money(r.tokens_in)||!money(r.tokens_out)))return damaged();
  const micro=r.microcents??Math.round(r.usd*1e6);
  if(!integer(micro)||!close(r.usd,micro))return damaged();sum+=micro;count++;stageSums[r.stage]=(stageSums[r.stage]??0)+micro;
  if(!integer(sum)||!integer(stageSums[r.stage]))return damaged();
 }
 if(sum!==total||count!==entries||Object.keys(pending).length!==opened.size)return damaged();
 for(const [key,r]of Object.entries(pending)){
  const original=opened.get(key);if(!object(r)||!original||!uuid(r.session_id)||!stageOK(r.stage)||!integer(r.microcents)||r.microcents<=0||!["reserved","unknown"].includes(r.status)||r.microcents!==original.microcents||r.stage!==original.stage||r.session_id!==original.session_id)return damaged();
 }
 for(const stage of new Set([...Object.keys(stageSums),...Object.keys(stages),...Object.keys(usdStages)]))if(!stageOK(stage)||(stageSums[stage]??0)!==(stages[stage]??0)||!close(usdStages[stage]??0,stages[stage]??0))return damaged();
 return {...s,tripped:s.tripped,reason:s.reason??null,total_microcents:total,total_usd:total/1e6,entries,by_stage_microcents:stages,by_stage:Object.fromEntries(Object.entries(stages).map(([k,v])=>[k,(v as number)/1e6]))};
}
/** Must be called under withAccountingLock; marker makes deleted initialized state fail closed. */
export function saveSpending(root:string,snapshot:SpendSnapshot):void{
 const dir=path.join(root,".kineti"),marker=path.join(dir,"spend.initialized.json");
 if(!fs.existsSync(marker))writeJson(marker,{version:1,at:new Date().toISOString()});
 writeJson(path.join(dir,"spend.json"),snapshot);
}
export function spendingStop(s:SpendSnapshot,limits:ReturnType<typeof loadLimits>):string|null{
 const pending=(s as any).routing_reservations;
 if(pending!==undefined){if(!object(pending))return "invalid routing reservations";if(Object.keys(pending).length)return "routing work or cost remains unresolved";}
 if(s.tripped)return s.reason||"spending circuit breaker is tripped";
 if(s.total_microcents>=Math.round(limits.globalUsd*limits.safetyFactor*1e6))return "global spending stop threshold reached";
 for(const[stage,value]of Object.entries(s.by_stage_microcents))if(value>=Math.round((limits.perStage[stage]??limits.perStageDefaultUsd)*limits.safetyFactor*1e6))return `stage ${stage} spending stop threshold reached`;
 return null;
}
