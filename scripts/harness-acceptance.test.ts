import { expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { prepareCandidate, runCheck, verifyScope } from "../src/harness-runtime/worker.ts";
import { EDITABLE, sha } from "../src/harness-runtime/types.ts";
const base = path.join(process.cwd(), ".kineti/harness-acceptance");
fs.mkdirSync(base, { recursive: true });
test("real Linux worker reproduces the pinned main bug and rejects unchanged or fabricated summaries", async () => {
    const root = fs.mkdtempSync(path.join(base, "baseline-"));
    const state = prepareCandidate(process.cwd(), path.join(root, "candidate"));
    const before = await runCheck(path.join(root, "candidate"));
    expect(before.passed).toBe(false);
    expect(before.cases.filter(c => !c.id.startsWith("unknown-")).every(c => c.passed)).toBe(true);
    expect(before.cases.some(c => c.id.startsWith("unknown-") && !c.passed)).toBe(true);
    expect(verifyScope(path.join(root, "candidate"), state.manifest)).toBe(state.artifact_sha256);
    fs.writeFileSync(path.join(root, "candidate", EDITABLE), 'console.log("7 pass, 0 fail");process.exit(0);\n');
    const fake = await runCheck(path.join(root, "candidate"));
    expect(fake.passed).toBe(false);
    expect(fake.cases.find(c => c.id === "unknown-log")?.passed).toBe(false);
}, 60000);
test("explicit non-model control patch passes trusted black-box CLI checks", async () => {
    const root = fs.mkdtempSync(path.join(base, "control-")), candidate = path.join(root, "candidate"), state = prepareCandidate(process.cwd(), candidate), file = path.join(candidate, EDITABLE);
    const original = fs.readFileSync(file, "utf8");
    // This hand-written control is a checker test, never a claimed live coding proof.
    const patch = 'function main() {\n  const raw=process.argv.slice(2);const known=new Set(["--help","-h","--economics","--stage","--model","--tokens-in","--tokens-out","--usd","--rate-in","--rate-out","--estimate","--i-am-human","--extend-usd"]);if(raw.some(a=>a.startsWith("--")&&!known.has(a)))process.exit(2);';
    fs.writeFileSync(file, original.replace("async function main() {", "async " + patch));
    expect(sha(fs.readFileSync(file))).not.toBe(state.artifact_sha256);
    const result = await runCheck(candidate);
    expect(result.passed).toBe(true);
    expect(result.cases).toHaveLength(11);
    expect(verifyScope(candidate, state.manifest)).toBe(result.artifact_sha256);
}, 60000);
test("candidate cannot mutate source, read trusted checks, use host sockets or reach the network", async () => {
    const root = fs.mkdtempSync(path.join(base, "boundary-")), candidate = path.join(root, "candidate"), state = prepareCandidate(process.cwd(), candidate);
    fs.writeFileSync(path.join(candidate, EDITABLE), `import fs from 'node:fs';
const result={uid:process.getuid?.(),source:false,checks:false,socket:false,network:false};
try{fs.writeFileSync('/candidate/bin/lib.ts','tampered');result.source=true;}catch{}
try{fs.readFileSync('/trusted/check-spend.ts');result.checks=true;}catch{}
try{fs.statSync('/var/run/docker.sock');result.socket=true;}catch{}
try{await Bun.connect({hostname:'1.1.1.1',port:443,socket:{data(){},open(s){s.end();}}});result.network=true;}catch{}
console.log(JSON.stringify(result));process.exit(0);
`);
    const result = await runCheck(candidate);
    expect(result.passed).toBe(false);
    expect(JSON.parse(result.observations![0].stdout)).toEqual({ uid: 10002, source: false, checks: false, socket: false, network: false });
    expect(verifyScope(candidate, state.manifest)).toBe(result.artifact_sha256);
}, 60000);
