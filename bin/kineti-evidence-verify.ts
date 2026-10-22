#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { validateReceipt, verifyReceiptSignature, type EvidenceRecord } from "./kineti-evidence.ts";

const args = process.argv.slice(2);
const get = (name: string) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const receiptPath = get("--receipt"), keyPath = get("--trusted-key"), workspace = get("--workspace");
if (!receiptPath || !keyPath) { console.error("usage: kineti-evidence-verify --receipt <json> --trusted-key <pem> [--workspace <dir>]"); process.exit(2); }
let receipt: EvidenceRecord;
try { receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8")); } catch { console.error("INVALID: receipt JSON cannot be read"); process.exit(7); }
const trustedKey = fs.readFileSync(keyPath, "utf8");
if (receipt.receipt_version !== 2 || !verifyReceiptSignature(receipt, trustedKey)) { console.error("INVALID: unsupported version, wrong issuer key, or signature mismatch"); process.exit(7); }
if (!workspace) {
  console.log("SIGNATURE VALID: workspace and current input inventory were not checked; acceptance unsupported");
  process.exit(4);
}
try {
  validateReceipt(receipt, { root: path.resolve(workspace), trustedPublicKey: trustedKey });
  console.log(`VALID DECLARED EVIDENCE: ${receipt.run_id}; local issuer and current workspace checked`);
} catch (error) { console.error(`INVALID: ${(error as Error).message}`); process.exit(7); }
