/** Copy and verify the exact trusted suite, then execute it in the provisioned local Linux VM. */
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
const root=process.cwd(),out=path.join(root,".kineti/completion-b");fs.mkdirSync(out,{recursive:true});
const config=path.join(root,".kineti/linux/kineti/ssh.config"), host="lima-kineti";
const files=["bin/kineti-linux-service.ts","src/security/evaluation-contract.ts","src/security/canonical-json.ts","src/security/service-request.ts","acceptance/linux-service.test.ts"];
function run(args:string[],capture=false){const r=Bun.spawnSync(args,{cwd:root,stdout:capture?"pipe":"inherit",stderr:capture?"pipe":"inherit"});if(r.exitCode!==0)throw new Error(`Linux acceptance command failed: ${args[0]} (exit ${r.exitCode})${capture?new TextDecoder().decode(r.stderr):""}`);return capture?new TextDecoder().decode(r.stdout):"";}
const expected=files.map(f=>crypto.createHash("sha256").update(fs.readFileSync(f)).digest("hex")+"  "+f).join("\n");
const tar=path.join(out,"service-source.tar.gz");run(["tar","-czf",tar,...files]);run(["scp","-F",config,tar,`${host}:service-source.tar.gz`]);
const ssh=["ssh","-F",config,"-o","ConnectTimeout=10",host];run([...ssh,"mkdir -p kineti && tar -xzf service-source.tar.gz -C kineti"]);
const check=()=>{const actual=run([...ssh,`cd kineti && sha256sum ${files.join(" ")}`],true).trim();if(actual!==expected)throw new Error("Linux source substitution detected");};check();
run([...ssh,"cd kineti && uname -a && sudo ../bun test --timeout 30000 acceptance/linux-service.test.ts"]);check();
console.log("Linux source digests matched before and after real acceptance.");
