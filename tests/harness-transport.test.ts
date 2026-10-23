import { afterEach, expect, test } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { CodexClient } from "../src/harness-runtime/codex.ts";
const roots: string[] = [];
afterEach(() => {
    for (const root of roots.splice(0))
        fs.rmSync(root, { recursive: true, force: true });
});
function clientFor(body: string) {
    const base = path.join(process.cwd(), ".kineti/acceptance/harness-transport");
    fs.mkdirSync(base, { recursive: true });
    const root = fs.mkdtempSync(path.join(base, "wire-"));
    roots.push(root);
    const exe = path.join(root, "fixture-codex");
    fs.writeFileSync(exe, `#!${process.execPath}\nif(process.argv.includes('--version')){console.log('codex 0.163.0');process.exit(0);}\nlet input='';process.stdin.on('data',chunk=>{input+=chunk;let n;while((n=input.indexOf('\\n'))>=0){const request=JSON.parse(input.slice(0,n));input=input.slice(n+1);${body}}});\n`, { mode: 0o700 });
    return new CodexClient(exe, path.join(root, "host"), new AbortController().signal);
}
test("malformed, foreign and invalid UTF-8 host responses fail without retry", async () => {
    for (const body of ["process.stdout.write('{broken\\n');", "console.log(JSON.stringify({id:request.id+1,result:{}}));", "console.log(JSON.stringify({id:request.id,result:{},error:{message:'ambiguous'}}));", "process.stdout.write(Buffer.from([255,10]));"]) {
        const client = clientFor(body);
        try {
            await expect(client.request("initialize", {})).rejects.toThrow();
        }
        finally {
            client.close();
        }
    }
});
test("duplicate server request identities stop the transport before a second dispatch", async () => {
    const client = clientFor("const r={id:777,method:'item/tool/call',params:{}};console.log(JSON.stringify(r));console.log(JSON.stringify(r));");
    let calls = 0;
    client.onRequest = async () => { calls++; return await new Promise(() => { }); };
    try {
        await expect(client.request("initialize", {})).rejects.toThrow(/duplicate/);
        expect(calls).toBe(1);
    }
    finally {
        client.close();
    }
});
test("host diagnostics are bounded and drained without becoming protocol evidence", async () => {
    const client = clientFor("process.stderr.write('x'.repeat(131072));console.log(JSON.stringify({id:request.id,result:{ok:true}}));");
    try {
        expect(await client.request("initialize", {})).toEqual({ ok: true });
        await Bun.sleep(30);
        const root = roots.at(-1)!;
        expect(fs.statSync(path.join(root, "host/host.stderr")).size).toBe(65536);
    }
    finally {
        client.close();
    }
});
