#!/usr/bin/env bun
// bin/kineti-stripe.ts
// Kineti OS — Stripe & Link Virtual Card Manager with SAGA LIFO undo support

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { die, ok, projectKdir, readJson, writeJson, ensureDir } from "./lib.ts";

interface VirtualCard {
  card_id: string;
  masked_number: string;
  expiry: string;
  spending_limit_usd: number;
  merchant_name: string;
  status: "active" | "canceled" | "consumed";
  created_at: string;
}

interface StripeState {
  cards: Record<string, VirtualCard>;
}

const stateFile = path.join(projectKdir(), "stripe_cards.json");

function loadState(): StripeState {
  ensureDir(projectKdir());
  return readJson<StripeState>(stateFile) || { cards: {} };
}

function saveState(state: StripeState): void {
  ensureDir(projectKdir());
  writeJson(stateFile, state);
}

const args = process.argv.slice(2);
const command = args[0];

if (!command || command === "help" || command === "--help") {
  console.log(`Kineti Stripe & Link Virtual Card Manager

Usage:
  bun bin/kineti-stripe.ts issue --amount <USD> --merchant <NAME>
  bun bin/kineti-stripe.ts cancel --card <CARD_ID>
  bun bin/kineti-stripe.ts list
`);
  process.exit(0);
}

if (command === "issue") {
  let amount = 0;
  let merchant = "Generic Merchant";
  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--amount" && args[i + 1]) {
      amount = parseFloat(args[i + 1]);
      i++;
    } else if (args[i] === "--merchant" && args[i + 1]) {
      merchant = args[i + 1];
      i++;
    }
  }

  if (amount <= 0) {
    die("Invalid amount: must be greater than 0");
  }

  const cardId = `ic_${crypto.randomBytes(8).toString("hex")}`;
  const last4 = Math.floor(1000 + Math.random() * 9000).toString();
  const card: VirtualCard = {
    card_id: cardId,
    masked_number: `•••• •••• •••• ${last4}`,
    expiry: "12/28",
    spending_limit_usd: amount,
    merchant_name: merchant,
    status: "active",
    created_at: new Date().toISOString(),
  };

  const state = loadState();
  state.cards[cardId] = card;
  saveState(state);

  // Register SAGA rollback step so any failure unwinds by canceling this card!
  const sagaScript = path.join(__dirname, "kineti-saga.ts");
  const cancelCmd = `bun ${sagaScript} push "bun ${path.join(__dirname, "kineti-stripe.ts")} cancel --card ${cardId}"`;
  try {
    const { execSync } = require("node:child_process");
    execSync(cancelCmd, { stdio: "ignore" });
  } catch {}

  console.log(JSON.stringify(card, null, 2));
  ok(`Virtual card ${cardId} issued with hard limit $${amount.toFixed(2)} for ${merchant}`);
} else if (command === "cancel") {
  let cardId = "";
  for (let i = 1; i < args.length; i++) {
    if (args[i] === "--card" && args[i + 1]) {
      cardId = args[i + 1];
      i++;
    }
  }

  if (!cardId) die("Missing --card <CARD_ID>");

  const state = loadState();
  if (!state.cards[cardId]) {
    die(`Card ${cardId} not found`);
  }

  state.cards[cardId].status = "canceled";
  saveState(state);
  ok(`Virtual card ${cardId} canceled successfully`);
} else if (command === "list") {
  const state = loadState();
  console.log(JSON.stringify(Object.values(state.cards), null, 2));
} else {
  die(`Unknown command: ${command}`);
}
