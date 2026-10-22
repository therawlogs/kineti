import crypto from "node:crypto";
import {canonicalJsonV2} from "./canonical-json.ts";
export interface SignedServiceRequest {
  version: 1; operation: "evaluate" | "export" | "inspect"; caller: string; request_id: string; expires_ms: number;
  arguments: Record<string, unknown>; signature: string;
}
export function signServiceRequest(request: Omit<SignedServiceRequest, "signature">, key: string): SignedServiceRequest {
  if(crypto.createPrivateKey(key).asymmetricKeyType!=="ed25519")throw new Error("Ed25519 private key required");
  return { ...request, signature: crypto.sign(null, Buffer.from(canonicalJsonV2(request)), key).toString("base64") };
}
export interface SignedServiceResponse {schema:"kineti-service-response/v1";request_sha256:string;issued_ms:number;result:unknown;signature:string;}
const responseHash=(request:SignedServiceRequest)=>crypto.createHash("sha256").update(canonicalJsonV2(request)).digest("hex");
export function signServiceResponse(request:SignedServiceRequest,result:unknown,key:string):SignedServiceResponse {
 const body={schema:"kineti-service-response/v1" as const,request_sha256:responseHash(request),issued_ms:Date.now(),result};
 return {...body,signature:crypto.sign(null,Buffer.from(canonicalJsonV2(body)),key).toString("base64")};
}
export function verifyServiceResponse(request:SignedServiceRequest,response:SignedServiceResponse,key:string):unknown {
 if(crypto.createPublicKey(key).asymmetricKeyType!=="ed25519")throw new Error("Ed25519 service authority required");
 const {signature,...body}=response||{};
 if(body.schema!=="kineti-service-response/v1"||body.request_sha256!==responseHash(request)||!Number.isSafeInteger(body.issued_ms)||Math.abs(Date.now()-body.issued_ms)>300000||typeof signature!=="string"||!crypto.verify(null,Buffer.from(canonicalJsonV2(body)),key,Buffer.from(signature,"base64")))throw new Error("service response does not match pinned authority and request");
 return body.result;
}
