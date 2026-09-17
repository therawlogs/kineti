#!/usr/bin/env bun
// bin/kineti-privacy.ts
// Kineti OS — Privacy & External Data Purge CLI

import { die, ok } from "./lib.ts";
import { PrivacyGovernanceManager } from "../src/privacy/governance.ts";

const args = process.argv.slice(2);
const command = args[0];

const manager = new PrivacyGovernanceManager();

if (!command || command === "help" || command === "--help") {
  console.log(`Kineti Privacy & Data Purge CLI

Usage:
  bun bin/kineti-privacy.ts status
  bun bin/kineti-privacy.ts opt-out --enable | --disable
  bun bin/kineti-privacy.ts purge --yes
`);
  process.exit(0);
}

if (command === "status") {
  const settings = manager.getSettings();
  console.log(JSON.stringify(settings, null, 2));
} else if (command === "opt-out") {
  if (args.includes("--enable")) {
    manager.setImproveKineti(true);
    ok("Model training opt-in enabled ('Improve Kineti for everyone' is ON)");
  } else if (args.includes("--disable")) {
    manager.setImproveKineti(false);
    ok("Model training opt-out enabled ('Improve Kineti for everyone' is OFF)");
  } else {
    die("Specify --enable or --disable");
  }
} else if (command === "purge") {
  if (!args.includes("--yes")) {
    die("Safety check: run with --yes to confirm purging all external third-party cached data");
  }
  const result = manager.executeExternalDataPurge();
  console.log(JSON.stringify(result, null, 2));
  ok(`External data purged: ${result.records_invalidated} records tombstoned across ${result.sources_cleared.length} sources`);
} else {
  die(`Unknown command: ${command}`);
}
