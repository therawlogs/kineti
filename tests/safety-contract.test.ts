import { afterEach, expect, test } from "bun:test";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { strictOptions } from "../bin/strict-options.ts";
import { readSpending, spendingStop } from "../bin/spending-state.ts";
import { loadLimits } from "../bin/lib.ts";
import { unsignedRecord, type EvidenceRecord } from "../bin/kineti-evidence.ts";

const repo = process.cwd(), roots: string[] = [];
function fixture() {
  const base = path.join(repo, ".kineti/acceptance/safety-contract"); fs.mkdirSync(base,{recursive:true});
  const root = fs.mkdtempSync(path.join(base,"run-")); roots.push(root);
  fs.mkdirSync(path.join(root,".kineti")); fs.mkdirSync(path.join(root,"tests"));
  fs.writeFileSync(path.join(root,"kineti.config.json"), JSON.stringify({root_goal:"Safety contract",settings:{verify_command_argv:["bun","test","tests/unit.test.ts"]}}));
  fs.writeFileSync(path.join(root,"tests/unit.test.ts"),'import {test,expect} from "bun:test";test("addition",()=>expect(2+3).toBe(5));\n');
  const seed = run(root,"seed",[]); expect(seed.exitCode).toBe(0); return root;
}
function run(root: string, tool: string, args: string[]) {
  return Bun.spawnSync(["bun",path.join(repo,`bin/kineti-${tool}.ts`),...args],{cwd:root,env:{...process.env,KINETI_MACHINE_DIR:path.join(root,".kineti/machine")},stdout:"pipe",stderr:"pipe"});
}
afterEach(()=>{for(const root of roots.splice(0)) fs.rmSync(root,{recursive:true,force:true});});
test("option normalization rejects typos, duplicate flags, missing values and preserves literal command argv",()=>{
  expect(strictOptions(["run","--label=x","--","bun","test","--timeout=3"],["--label"], [], [], 1, true)).toEqual(["run","--label","x","--","bun","test","--timeout=3"]);
  for(const args of [["--typo"],["--label"],["--label=x","--label","y"],["--yes=false"]]) expect(()=>strictOptions(args,["--label"],["--yes"])).toThrow();
});
test("invalid spending arguments leave accounting and configuration unchanged",()=>{
  const root=fixture(),file=path.join(root,".kineti/spend.json"),before=fs.readFileSync(file,"utf8");
  for(const args of [["--typo"],["--usd","1","--usd=2"],["--usd"],["--usd=NaN"],["--rate-in=2"]]) {
    const result=run(root,"spend",["log","--stage=build","--tokens-in=0","--tokens-out=0",...args]);
    expect(result.exitCode).toBe(2); expect(fs.readFileSync(file,"utf8")).toBe(before);
    expect(fs.existsSync(path.join(root,".kineti/spend.log.jsonl"))).toBe(false);
  }
});
test("corrupt accounting fails closed; resetting a threshold does not release it",()=>{
  const root=fixture(),file=path.join(root,".kineti/spend.json");
  fs.writeFileSync(file,JSON.stringify({total_microcents:null,tripped:false}));
  expect(run(root,"spend",["check"]).exitCode).not.toBe(0);
  fs.writeFileSync(file,JSON.stringify({total_microcents:48_000_000,tripped:true}));
  expect(run(root,"spend",["reset","--i-am-human"]).exitCode).toBe(0);
  expect(run(root,"spend",["check"]).exitCode).toBe(3);
  const state=readSpending(root);expect(spendingStop(state,loadLimits(root))).toContain("threshold");
  fs.writeFileSync(file,JSON.stringify({total_microcents:9_500_000,by_stage_microcents:{build:9_500_000},tripped:false}));
  expect(run(root,"spend",["check"]).exitCode).toBe(3);
});
test("simultaneous spending updates are not lost",async()=>{
  const root=fixture();
  const children=Array.from({length:16},()=>Bun.spawn(["bun",path.join(repo,"bin/kineti-spend.ts"),"log","--stage=build","--tokens-in=0","--tokens-out=0","--usd=0.01"],{cwd:root,env:{...process.env,KINETI_MACHINE_DIR:path.join(root,".kineti/machine")},stdout:"ignore",stderr:"pipe"}));
  expect(await Promise.all(children.map(c=>c.exited))).toEqual(Array(16).fill(0));
  const state=readSpending(root);expect(state.entries).toBe(16);expect(state.total_microcents).toBe(160_000);
});
test("checker and CI agree on exact argv; signed session identity cannot be altered; v2 stays readable",()=>{
  const root=fixture(),session=crypto.randomUUID(),parent=crypto.randomUUID();
  const recorded=run(root,"evidence",["run","--label=unit","--session-id",session,"--parent-run-id",parent,"--required-test=tests/unit.test.ts","--","bun","test","tests/unit.test.ts"]);
  expect(recorded.exitCode).toBe(0);
  const evidence=path.join(root,".kineti/evidence.jsonl"),receipt=JSON.parse(fs.readFileSync(evidence,"utf8")) as EvidenceRecord;
  expect(receipt.receipt_version).toBe(3);expect(receipt.session_id).toBe(session);expect(receipt.parent_run_id).toBe(parent);
  expect(run(root,"evidence",["check","--label=unit"]).exitCode).toBe(0);
  expect(run(root,"ci",["--require-evidence=unit","--require-command-mappings","--json"]).exitCode).toBe(1);
  const exact='unit=["bun","test","tests/unit.test.ts"]',wrong='unit=["bun","test","other.test.ts"]';
  expect(run(root,"ci",["--require-evidence=unit","--require-command-mappings","--expect-command",exact,"--json"]).exitCode).toBe(0);
  const mismatch=run(root,"ci",["--require-evidence=unit","--require-command-mappings","--expect-command",wrong,"--json"]);
  expect(mismatch.exitCode).toBe(1);expect(mismatch.stdout.toString()).toContain("recorded arguments");
  expect(run(root,"evidence",["check","--label=unit","--expect-command",'["bun","test","other.test.ts"]']).exitCode).not.toBe(0);
  fs.writeFileSync(evidence,JSON.stringify({...receipt,session_id:crypto.randomUUID()})+"\n");
  expect(run(root,"evidence",["check","--label=unit"]).exitCode).toBe(7);
  const legacy={...receipt,receipt_version:2 as const};delete legacy.session_id;delete legacy.parent_run_id;
  legacy.signature=crypto.sign(null,Buffer.from(unsignedRecord(legacy)),fs.readFileSync(path.join(root,".kineti/evidence-issuer.pem.private"))).toString("base64");
  fs.writeFileSync(evidence,JSON.stringify(legacy)+"\n");expect(run(root,"evidence",["check","--label=unit"]).exitCode).toBe(0);
},20_000);

test("governance commands reject unknown and duplicate flags before creating records",()=>{
 const root=fixture();
 const cases:[string,string[]][]=[["state",["new-run","--goal=a","--goal=b"]],["egress",["record","--host=example.com","--desc=a","--description=b"]],["saga",["push","--label=a","--inverse=x","--typo"]],["spend",["log","--stage=build","--tokens-in=0","--tokens-out=0","--usd=1","--","--ignored"]]];
 for(const [tool,args] of cases) expect(run(root,tool,args).exitCode).toBe(2);
 expect(fs.existsSync(path.join(root,".kineti/saga.jsonl"))).toBe(false);expect(fs.existsSync(path.join(root,".kineti/spend.log.jsonl"))).toBe(false);
});
