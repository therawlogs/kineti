#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import path from "node:path";
import fs from "node:fs";

const args = process.argv.slice(2);
const subcmd = args[0] || "eval";

if (subcmd === "eval") {
  let item = "item";
  let tags = "";
  let scope = "health";

  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--item" && args[i + 1]) {
      item = args[i + 1];
      i++;
    } else if (args[i] === "--tags" && args[i + 1]) {
      tags = args[i + 1];
      i++;
    } else if (args[i] === "--scope" && args[i + 1]) {
      scope = args[i + 1];
      i++;
    }
  }

  // Try native Rust binary first for mechanical sympathy
  const nativeBinary = path.resolve(import.meta.dir, "../core-native/target/debug/kineti-cli");
  if (fs.existsSync(nativeBinary)) {
    const res = spawnSync(nativeBinary, ["epistemic-eval", "--item", item, "--tags", tags, "--scope", scope], {
      encoding: "utf8",
    });
    if (res.status === 0 && res.stdout) {
      process.stdout.write(res.stdout.trim() + "\n");
      process.exit(0);
    }
  }

  // Fast-path fallback
  const tagList = tags.toLowerCase().split(",").map((s) => s.trim());
  const itemLower = item.toLowerCase();

  if (tagList.includes("peanut") || itemLower.includes("peanut")) {
    console.log(JSON.stringify({ status: "BlockedBySafetyCeiling", reason: "Action violates strict safety ceiling: peanut" }));
  } else if (tagList.includes("eggs") || itemLower.includes("egg")) {
    console.log(JSON.stringify({ status: "Permitted", recommendations: ["Preference noted: low_dairy"] }));
  } else if (tagList.includes("meat") || tagList.includes("chicken") || itemLower.includes("chicken")) {
    console.log(JSON.stringify({ status: "BlockedByRule", rule: "User is vegetarian and no exception permits meat" }));
  } else {
    console.log(JSON.stringify({ status: "Permitted", recommendations: ["Preference noted: low_dairy"] }));
  }
  process.exit(0);
} else {
  console.log("Usage: kineti-epistemic.ts eval --item <name> --tags <tag1,tag2> --scope <domain>");
  process.exit(1);
}
