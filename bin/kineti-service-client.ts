#!/usr/bin/env bun
/** Signed loopback client; configure an SSH tunnel for a remote user-run Linux service. */
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { evaluationDigest, validateTrustedEvaluation } from "../src/security/evaluation-contract.ts";
import { signServiceRequest, verifyServiceResponse, type SignedServiceRequest } from "../src/security/service-request.ts";
export async function submitServiceRequest(operation:SignedServiceRequest["operation"],args:Record<string,unknown>,root=process.cwd(),requestId=crypto.randomUUID()) {
 const readOwned=(file:string)=>{const real=fs.realpathSync(file);if(!real.startsWith(fs.realpathSync(root)+path.sep))throw new Error("client configuration escapes project");const s=fs.lstatSync(file);if(!s.isFile()||s.isSymbolicLink()||s.uid!==process.getuid!()||(s.mode&0o077))throw new Error("client credential/config must be an owner-only regular file");return fs.readFileSync(file,"utf8");};
 const c=JSON.parse(readOwned(path.join(root,".kineti/service-client.json")));
 const url=new URL(c.endpoint);if(c.version!==1||url.protocol!=="http:"||url.hostname!=="127.0.0.1"||url.username||url.password||url.pathname!=="/v1/request"||url.search||url.hash||!url.port)throw new Error("explicit loopback service endpoint required; use an SSH tunnel");
 if(typeof c.private_key_file!=="string"||path.isAbsolute(c.private_key_file)||c.private_key_file.split(/[\\/]/).some((x:string)=>x===".."))throw new Error("private key file must stay inside the project");
 const keyPath=fs.realpathSync(path.join(root,c.private_key_file)), realRoot=fs.realpathSync(root);if(!keyPath.startsWith(realRoot+path.sep))throw new Error("private key path escapes the project");
 if(typeof c.evaluator_public_key!=="string"||typeof c.owner_public_key!=="string"||!c.pins)throw new Error("owner-pinned evaluation authorities are required");
 const request=signServiceRequest({version:1,operation,caller:c.caller,request_id:requestId,expires_ms:Date.now()+60000,arguments:args},readOwned(keyPath));
 const logged=spawnSync(process.execPath,[path.join(import.meta.dir,"kineti-egress.ts"),"record","--host","127.0.0.1","--description",`signed Linux service ${operation}; request ${requestId}; body_sha256=${crypto.createHash("sha256").update(JSON.stringify(request)).digest("hex")}`],{cwd:root,encoding:"utf8"});
 if(logged.status!==0)throw new Error("egress history unavailable; request was not sent");
 try {const response=await fetch(url,{method:"POST",redirect:"error",headers:{"content-type":"application/json"},body:JSON.stringify(request),signal:AbortSignal.timeout(30000)});const text=await response.text();if(text.length>2*1024*1024)throw new Error("oversized service response");let result=verifyServiceResponse(request,JSON.parse(text),c.evaluator_public_key) as any;
 if(operation==="evaluate"&&response.status===200){const {evaluation_id,...signed}=result;validateTrustedEvaluation(args.manifest as any,signed,{pins:c.pins,trustedOwnerKeys:{[c.pins.owner.key_id]:c.owner_public_key},trustedEvaluatorKeys:{[c.pins.evaluator.key_id]:c.evaluator_public_key}});result={...result,evaluation_sha256:evaluationDigest(signed)};}
 return {request_id:requestId,http_status:response.status,result};}
 catch {return {request_id:requestId,http_status:null,result:{state:"unknown",reason:"service-response-unavailable; inspect request before retrying"}};}
}
if(import.meta.main){const [operation,file]=process.argv.slice(2);if(operation==="--help"||!operation)console.log("service-client evaluate|export|inspect REQUEST_ARGUMENTS.json; reads owner-only .kineti/service-client.json and its project-local Ed25519 key file");else try{if(!["evaluate","export","inspect"].includes(operation))throw new Error("unsupported operation");const args=JSON.parse(fs.readFileSync(file!,"utf8"));console.log(JSON.stringify(await submitServiceRequest(operation as SignedServiceRequest["operation"],args)));}catch(e){console.error((e as Error).message);process.exitCode=1;}}
