#!/usr/bin/env bun
// bin/kineti-invite.ts
// Kineti OS — Viral Invite & Referral Quota CLI

import { die, ok } from "./lib.ts";
import { ViralInviteEngine } from "../src/growth/viral_invites.ts";

const args = process.argv.slice(2);
const command = args[0];

const engine = new ViralInviteEngine();

if (!command || command === "help" || command === "--help") {
  console.log(`Kineti Viral Invite CLI

Usage:
  bun bin/kineti-invite.ts list
  bun bin/kineti-invite.ts quota
  bun bin/kineti-invite.ts create [--recipient <EMAIL_OR_PHONE>]
  bun bin/kineti-invite.ts revoke --code <INVITE_CODE>
`);
  process.exit(0);
}

if (command === "list") {
  const list = engine.listInvites();
  console.log(JSON.stringify(list, null, 2));
} else if (command === "quota") {
  const tier = engine.getTier();
  const max = engine.getMaxQuota();
  const used = engine.getUsedQuota();
  const remaining = engine.getRemainingQuota();
  console.log(
    JSON.stringify(
      {
        tier,
        max_quota: max === -1 ? "unlimited" : max,
        used,
        remaining: max === -1 ? "unlimited" : remaining,
      },
      null,
      2,
    ),
  );
} else if (command === "create") {
  let recipient: string | undefined;
  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--recipient" && args[i + 1]) {
      recipient = args[i + 1];
      i++;
    }
  }

  try {
    const invite = engine.createInvite(recipient);
    console.log(JSON.stringify(invite, null, 2));
    ok(`Invite created! Vanity Link: ${invite.vanity_url}`);
  } catch (err: any) {
    die(err.message);
  }
} else if (command === "revoke") {
  let code = "";
  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--code" && args[i + 1]) {
      code = args[i + 1];
      i++;
    }
  }

  if (!code) die("Missing --code <INVITE_CODE>");
  if (engine.revokeInvite(code)) {
    ok(`Invite ${code} revoked`);
  } else {
    die(`Could not revoke invite ${code} (already claimed or not found)`);
  }
} else {
  die(`Unknown command: ${command}`);
}
