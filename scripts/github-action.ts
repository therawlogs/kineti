/** Runs from an owner-pinned composite action, never from the candidate checkout. */
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { submitServiceRequest } from "../bin/kineti-service-client.ts";
export function validateActionSource(env:NodeJS.ProcessEnv,source=path.resolve(import.meta.dir,"..")) {
 if(!/^[a-f0-9]{40}$/.test(env.KINETI_TRUSTED_COMMIT||"") || env.KINETI_ACTION_REF!==env.KINETI_TRUSTED_COMMIT)throw new Error("action must be called from the exact owner-approved full commit SHA");
 if(!env.KINETI_ACTION_PATH || fs.realpathSync(env.KINETI_ACTION_PATH)!==fs.realpathSync(source))throw new Error("trusted action source mismatch; local PR checkout is not an action authority");
 const client=fs.realpathSync(env.KINETI_CLIENT_PROJECT||"");const stat=fs.statSync(client);if(!stat.isDirectory()||stat.uid!==process.getuid!()||(stat.mode&0o077))throw new Error("client project must be an owner-only directory outside candidate control");
 const workspace=env.GITHUB_WORKSPACE&&fs.realpathSync(env.GITHUB_WORKSPACE);if(workspace&&(client===workspace||client.startsWith(workspace+path.sep)))throw new Error("client authority cannot live inside candidate workspace");
 return client;
}
if(import.meta.main)try {
 const client=validateActionSource(process.env);
 const fd=fs.openSync(process.env.KINETI_ARTIFACT_FILE!,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
 let artifact:Buffer;try {const stat=fs.fstatSync(fd);if(!stat.isFile()||stat.size>1024*1024)throw new Error("candidate must be a bounded regular file");artifact=fs.readFileSync(fd);}finally{fs.closeSync(fd);}
 const manifest=JSON.parse(fs.readFileSync(process.env.KINETI_MANIFEST_FILE!,"utf8"));
 if(crypto.createHash("sha256").update(artifact).digest("hex")!==manifest.artifact_sha256)throw new Error("candidate does not match signed manifest");
 const response=await submitServiceRequest("evaluate",{manifest,artifact_base64:artifact.toString("base64")},client);
 const result=response.result as any;
 if(response.http_status!==200||result.decision!=="accepted")throw new Error("independent evaluation did not authorize this artifact");
 console.log(JSON.stringify({evaluation_id:result.evaluation_id,decision:result.decision,artifact_sha256:result.artifact_sha256,manifest_sha256:result.manifest_sha256}));
 // Export is a distinct, explicitly requested protected action; evaluation alone has no write effect.
}catch(e){console.error((e as Error).message);process.exitCode=1;}
