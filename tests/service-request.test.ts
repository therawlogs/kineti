import {test,expect} from "bun:test";
import crypto from "node:crypto";
import {signServiceRequest,signServiceResponse,verifyServiceResponse} from "../src/security/service-request.ts";
const pair=()=>{const p=crypto.generateKeyPairSync("ed25519");return {private:p.privateKey.export({type:"pkcs8",format:"pem"}).toString(),public:p.publicKey.export({type:"spki",format:"pem"}).toString()};};
test("service response binds exact signed request and independently pinned authority",()=>{
 const caller=pair(),service=pair(),other=pair();const request=signServiceRequest({version:1,operation:"export",caller:"client-one",request_id:"request-one",expires_ms:Date.now()+10000,arguments:{destination:"release/app.sh",digest:"abc"}},caller.private);const response=signServiceResponse(request,{state:"applied"},service.private);
 expect(verifyServiceResponse(request,response,service.public)).toEqual({state:"applied"});expect(()=>verifyServiceResponse(request,response,other.public)).toThrow();expect(()=>verifyServiceResponse({...request,request_id:"request-two"},response,service.public)).toThrow();expect(()=>verifyServiceResponse(request,{...response,result:{state:"denied"}},service.public)).toThrow();expect(()=>verifyServiceResponse(request,{...response,issued_ms:Date.now()-600000},service.public)).toThrow();
});
test("service request signing rejects ambiguous JSON values before any request",()=>{
 const caller=pair();expect(()=>signServiceRequest({version:1,operation:"evaluate",caller:"client-one",request_id:"request-one",expires_ms:Date.now()+10000,arguments:{value:NaN}},caller.private)).toThrow();
});
