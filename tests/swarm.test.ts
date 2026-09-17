// tests/swarm.test.ts
// Tests for Multi-Agent Swarm Coordination, Cryptographic Identity, and Gate Enforcement

import { describe, expect, it } from "bun:test";
import {
  registerAgent,
  delegateTask,
  submitWorkerResult,
  reviewAndSignOutcome,
  verifyDualSignedTicket,
  detectGoalDrift,
  signPayload,
  verifySignature,
  hashString,
} from "../src/swarm/coordinator";

describe("Multi-Agent Swarm Coordination & Identity", () => {
  it("registers agents with unique Ed25519 keypairs and assigned roles", () => {
    const planner = registerAgent("Planner Prime", "planner", ["plan"], 10.0);
    const worker = registerAgent("Coder Beta", "worker", ["read", "write", "test"], 15.0);
    const reviewer = registerAgent("Reviewer Gamma", "reviewer", ["verify"], 5.0);

    expect(planner.id).toContain("agent_planner_");
    expect(worker.id).toContain("agent_worker_");
    expect(reviewer.id).toContain("agent_reviewer_");

    expect(planner.publicKeyHex.length).toBeGreaterThan(32);
    expect(worker.publicKeyHex).not.toBe(planner.publicKeyHex);
    expect(reviewer.publicKeyHex).not.toBe(worker.publicKeyHex);

    expect(worker.role).toBe("worker");
    expect(reviewer.role).toBe("reviewer");
  });

  it("propagates locked root goal and signs delegation payload", () => {
    const coordinator = registerAgent("Coordinator", "coordinator");
    const worker = registerAgent("Worker", "worker");

    const rootGoal = "Migrate auth service to passkey authentication";
    const task = delegateTask(coordinator, worker, "Implement WebAuthn credentials", rootGoal, 5.0);

    expect(task.taskId).toBeDefined();
    expect(task.parentAgentId).toBe(coordinator.id);
    expect(task.assignedAgentId).toBe(worker.id);
    expect(task.rootGoalHash).toBe(hashString(rootGoal));

    // Verify coordinator's signature on the delegation
    const expectedPayload = `${task.taskId}:${coordinator.id}:${worker.id}:${task.rootGoalHash}:5:${task.timestamp}`;
    const isValid = verifySignature(coordinator.publicKeyPem, expectedPayload, task.signature);
    expect(isValid).toBe(true);
  });

  it("detects and blocks goal drift across swarm handoffs", () => {
    const coordinator = registerAgent("Coordinator", "coordinator");
    const worker = registerAgent("Worker", "worker");

    const rootGoal = "Build payment gateway integration";
    const task = delegateTask(coordinator, worker, "Connect Stripe API", rootGoal, 5.0);

    // Matching goal
    expect(detectGoalDrift(task, rootGoal)).toBe(false);

    // Drifted goal
    const driftedGoal = "Rewrite payment gateway in Go";
    expect(detectGoalDrift(task, driftedGoal)).toBe(true);
  });

  it("worker signs task evidence and reviewer verifies dual-signed OVT", () => {
    const coordinator = registerAgent("Coordinator", "coordinator");
    const worker = registerAgent("Worker", "worker");
    const reviewer = registerAgent("Reviewer", "reviewer");

    const rootGoal = "Create API caching layer";
    const task = delegateTask(coordinator, worker, "Cache GET /user response", rootGoal, 2.0);

    // Worker completes task and signs
    const evidence = "bun test -- 5 cache tests pass, hit ratio 94%";
    const result = submitWorkerResult(worker, task, evidence);

    expect(result.evidenceHash).toBe(hashString(evidence));
    expect(result.workerSignature.length).toBeGreaterThan(64);

    // Reviewer approves and co-signs
    const ticket = reviewAndSignOutcome(
      reviewer,
      task,
      worker,
      result.workerSignature,
      result.evidenceHash,
      true
    );

    expect(ticket.ticketId).toContain("ovt_");
    expect(ticket.workerId).toBe(worker.id);
    expect(ticket.reviewerId).toBe(reviewer.id);

    // Both signatures verify
    const isValid = verifyDualSignedTicket(ticket, worker, reviewer);
    expect(isValid).toBe(true);
  });

  it("enforces role-gating: workers cannot approve their own work", () => {
    const coordinator = registerAgent("Coordinator", "coordinator");
    const worker = registerAgent("Worker", "worker");

    const rootGoal = "Fix SQL injection vulnerability";
    const task = delegateTask(coordinator, worker, "Sanitize search inputs", rootGoal, 2.0);
    const result = submitWorkerResult(worker, task, "sanitized inputs");

    // Worker attempts self-approval
    expect(() => {
      reviewAndSignOutcome(
        worker as any,
        task,
        worker,
        result.workerSignature,
        result.evidenceHash,
        true
      );
    }).toThrow("cannot approve gates");
  });

  it("detects cryptographic tampering of evidence or ticket data", () => {
    const coordinator = registerAgent("Coordinator", "coordinator");
    const worker = registerAgent("Worker", "worker");
    const reviewer = registerAgent("Reviewer", "reviewer");

    const rootGoal = "Audit token expiry limits";
    const task = delegateTask(coordinator, worker, "Enforce 15-min TTL", rootGoal, 1.0);
    const result = submitWorkerResult(worker, task, "TTL checked");

    const ticket = reviewAndSignOutcome(
      reviewer,
      task,
      worker,
      result.workerSignature,
      result.evidenceHash,
      true
    );

    // Tampered ticket (modified evidence hash)
    const tamperedTicket = {
      ...ticket,
      evidenceHash: hashString("forged results"),
    };

    const isValid = verifyDualSignedTicket(tamperedTicket, worker, reviewer);
    expect(isValid).toBe(false);
  });
});
