#!/usr/bin/env bun
import fs from "node:fs";
import path from "node:path";
import { fingerprint, hashFileChunked, verifyReceiptSignature, type EvidenceRecord } from "./kineti-evidence.ts";

const args = process.argv.slice(2);
const get = (name: string) => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const receiptPath = get("--receipt"), keyPath = get("--trusted-key"), workspace = get("--workspace");
if (!receiptPath || !keyPath) { console.error("usage: kineti-evidence-verify --receipt <json> --trusted-key <pem> [--workspace <dir>]"); process.exit(2); }
let receipt: EvidenceRecord;
try { receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8")); } catch { console.error("INVALID: receipt JSON cannot be read"); process.exit(7); }
const trustedKey = fs.readFileSync(keyPath, "utf8");
if (receipt.receipt_version !== 2 || !verifyReceiptSignature(receipt, trustedKey)) { console.error("INVALID: unsupported version, wrong issuer key, or signature mismatch"); process.exit(7); }
if (receipt.verification?.status !== "valid" || !receipt.manifest || receipt.exit_code !== 0) { console.error("INVALID: receipt lacks passing declared evidence"); process.exit(7); }
if (workspace) {
  const root = path.resolve(workspace);
  if (fingerprint(root) !== receipt.fingerprint) { console.error("INVALID: workspace fingerprint differs"); process.exit(4); }
  for (const item of [...receipt.verification.inventory, ...receipt.manifest.dependencies, ...receipt.manifest.artifact_digests]) {
    const target = path.resolve(root, item.path);
    if (!target.startsWith(root + path.sep) || !fs.existsSync(target) || hashFileChunked(target) !== item.sha256) { console.error(`INVALID: declared input differs or is missing: ${item.path}`); process.exit(4); }
  }
}
console.log(`VALID DECLARED EVIDENCE: ${receipt.run_id}; issuer key matched; workspace ${workspace ? "checked" : "not checked"}`);
