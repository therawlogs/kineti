// Exact original signer-byte and verifier functions; test fixture only.
import crypto from "node:crypto";
export type EvidenceRecord = { [key: string]: any };
export function unsignedRecord(record: EvidenceRecord): string {
  const { signature: _sig, ...unsigned } = record;
  return JSON.stringify(unsigned);
}
export function verifyReceiptSignature(record: EvidenceRecord, trustedPublicKey?: string): boolean {
  if (typeof record.signature !== "string" || typeof record.issuer_key !== "string" || (trustedPublicKey && record.issuer_key !== trustedPublicKey) || !/^[A-Za-z0-9+/]{86}==$/.test(record.signature)) return false;
  try { const key = crypto.createPublicKey(trustedPublicKey ?? record.issuer_key); return key.asymmetricKeyType === "ed25519" && crypto.verify(null, Buffer.from(unsignedRecord(record)), key, Buffer.from(record.signature, "base64")); } catch { return false; }
}
