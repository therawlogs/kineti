#!/usr/bin/env bun
// bin/kineti-swarm.ts
// Kineti OS — Local Swarm Coordinator & Multi-Agent Identity Harness

import {
  registerAgent,
  delegateTask,
  submitWorkerResult,
  reviewAndSignOutcome,
  verifyDualSignedTicket,
  detectGoalDrift,
} from "../src/swarm/coordinator.ts";
import { ok } from "./lib.ts";

function runSwarmSimulation(rootGoalOverride?: string) {
  console.log("\n=======================================================");
  console.log("  Kineti OS — Multi-Agent Swarm & Identity Simulation");
  console.log("=======================================================\n");

  // Step 1: Define Immutable Root Goal (pass as first CLI arg to customize)
  const rootGoal = rootGoalOverride?.trim() || "Refactor database query layer to support connection pooling and sub-5ms latency";
  console.log(`Locked Root Goal: "${rootGoal}"`);

  // Step 2: Register Swarm Agents with isolated Ed25519 Keypairs
  console.log("\n[1] Registering Swarm Agents with Cryptographic Identities...");
  const coordinator = registerAgent("Coordinator Alpha", "coordinator", ["delegate", "audit"], 25.0);
  const coder = registerAgent("Coder Beta", "worker", ["read", "write", "test"], 15.0);
  const reviewer = registerAgent("Reviewer Gamma", "reviewer", ["read", "verify"], 10.0);

  console.log(`  • Coordinator: ${coordinator.id} (Role: ${coordinator.role}, Key: ${coordinator.publicKeyHex.slice(0, 16)}...)`);
  console.log(`  • Coder:       ${coder.id} (Role: ${coder.role}, Key: ${coder.publicKeyHex.slice(0, 16)}...)`);
  console.log(`  • Reviewer:    ${reviewer.id} (Role: ${reviewer.role}, Key: ${reviewer.publicKeyHex.slice(0, 16)}...)`);

  // Step 3: Coordinator delegates task with Immutable Root Goal
  console.log("\n[2] Coordinator Delegating Task to Worker...");
  const task = delegateTask(coordinator, coder, "Optimize DB Pool Queries", rootGoal, 5.0);
  console.log(`  • Task ID:        ${task.taskId}`);
  console.log(`  • Goal Hash:      ${task.rootGoalHash.slice(0, 16)}...`);
  console.log(`  • Coordinator Sig:${task.signature.slice(0, 20)}...`);

  // Step 4: Anti-Drift Check
  console.log("\n[3] Testing Goal Drift Protection...");
  const isDrift = detectGoalDrift(task, rootGoal);
  console.log(`  • Goal drift check against root goal: ${isDrift ? "DRIFT DETECTED" : "ALIGNED (0% drift)"}`);

  const driftedGoal = "Rewrite UI in Flutter";
  const wouldDrift = detectGoalDrift(task, driftedGoal);
  console.log(`  • Goal drift check against altered goal: ${wouldDrift ? "DRIFT CAUGHT & BLOCKED" : "UNCAUGHT"}`);

  // Step 5: Coder writes code, runs tests, and signs output
  console.log("\n[4] Worker Executing Code & Signing Outcome Evidence...");
  const testEvidence = "bun test -- 14 tests passing, 0 regressions, pool latency 3.8ms";
  const workerResult = submitWorkerResult(coder, task, testEvidence);
  console.log(`  • Evidence Hash:    ${workerResult.evidenceHash.slice(0, 16)}...`);
  console.log(`  • Worker Signature: ${workerResult.workerSignature.slice(0, 20)}...`);

  // Step 6: Test Impersonation Defense (Coder attempts to approve own gate)
  console.log("\n[5] Testing Impersonation Defense (Self-Approval Prevention)...");
  try {
    reviewAndSignOutcome(coder as any, task, coder, workerResult.workerSignature, workerResult.evidenceHash, true);
    console.log("  • Warning: Self-approval was not blocked!");
  } catch (err: any) {
    console.log(`  • Security Defense: ${err.message}`);
  }

  // Step 7: Reviewer independently inspects and co-signs Verification Ticket
  console.log("\n[6] Reviewer Signing Dual-Signed Outcome Verification Ticket (OVT)...");
  const ticket = reviewAndSignOutcome(reviewer, task, coder, workerResult.workerSignature, workerResult.evidenceHash, true);
  console.log(`  • Ticket ID:          ${ticket.ticketId}`);
  console.log(`  • Worker Signature:   ${ticket.workerSignature.slice(0, 20)}...`);
  console.log(`  • Reviewer Signature: ${ticket.reviewerSignature.slice(0, 20)}...`);

  // Step 8: Verify Dual Signature Cryptographically
  console.log("\n[7] Verifying Ticket Cryptographic Proof...");
  const isTicketValid = verifyDualSignedTicket(ticket, coder, reviewer);
  console.log(`  • Dual-Signature Verified: ${isTicketValid ? "VALID (Non-repudiable)" : "INVALID"}`);

  ok("Swarm coordination and cryptographic identity simulation completed successfully.\n");
}

if (import.meta.main) {
  const customGoal = process.argv.slice(2).join(" ").trim() || undefined;
  if (customGoal) console.log(`Using custom goal from CLI args.`);
  else console.log(`No goal arg given — running fixed demo. Usage: kineti swarm "Your goal here"`);
  runSwarmSimulation(customGoal);
}

export { runSwarmSimulation };
