// src/swarm/coordinator.ts
// Kineti Swarm Coordinator & Cryptographic Agent Identity Engine

import crypto from "node:crypto";

export type AgentRole = "coordinator" | "planner" | "worker" | "reviewer" | "auditor";

export interface SwarmAgent {
  id: string;
  name: string;
  role: AgentRole;
  publicKeyHex: string;
  privateKeyPem: string;
  publicKeyPem: string;
  capabilities: string[];
  maxSpendUsd: number;
  spentUsd: number;
}

export interface TaskEnvelope {
  taskId: string;
  parentAgentId: string;
  assignedAgentId: string;
  taskName: string;
  rootGoal: string;
  rootGoalHash: string;
  budgetUsd: number;
  status: "pending" | "in_progress" | "completed" | "failed" | "rejected";
  timestamp: string;
  signature: string;
}

export interface VerificationTicket {
  ticketId: string;
  taskId: string;
  rootGoalHash: string;
  workerId: string;
  reviewerId: string;
  evidenceHash: string;
  workerSignature: string;
  reviewerSignature: string;
  verifiedAt: string;
}

/**
 * Creates a sha256 hash of a string.
 */
export function hashString(data: string): string {
  return crypto.createHash("sha256").update(data).digest("hex");
}

/**
 * Registers a new agent with an isolated Ed25519 cryptographic keypair and runtime bounds.
 */
export function registerAgent(
  name: string,
  role: AgentRole,
  capabilities: string[] = ["read", "write"],
  maxSpendUsd: number = 10.0
): SwarmAgent {
  const { publicKey, privateKey } = crypto.generateKeyPairSync("ed25519");
  
  const publicKeyPem = publicKey.export({ type: "spki", format: "pem" }).toString();
  const privateKeyPem = privateKey.export({ type: "pkcs8", format: "pem" }).toString();
  const publicKeyDer = publicKey.export({ type: "spki", format: "der" });
  const publicKeyHex = publicKeyDer.toString("hex");

  return {
    id: `agent_${role}_${crypto.randomBytes(4).toString("hex")}`,
    name,
    role,
    publicKeyHex,
    publicKeyPem,
    privateKeyPem,
    capabilities,
    maxSpendUsd,
    spentUsd: 0,
  };
}

/**
 * Signs a string payload using an agent's private key.
 */
export function signPayload(agent: SwarmAgent, payload: string): string {
  const privateKey = crypto.createPrivateKey(agent.privateKeyPem);
  const signature = crypto.sign(null, Buffer.from(payload, "utf8"), privateKey);
  return signature.toString("hex");
}

/**
 * Verifies a signature against an agent's public key.
 */
export function verifySignature(publicKeyPem: string, payload: string, signatureHex: string): boolean {
  try {
    const publicKey = crypto.createPublicKey(publicKeyPem);
    return crypto.verify(null, Buffer.from(payload, "utf8"), publicKey, Buffer.from(signatureHex, "hex"));
  } catch {
    return false;
  }
}

/**
 * Delegates a task from a coordinator/planner to a worker with the immutable root goal locked in.
 */
export function delegateTask(
  coordinator: SwarmAgent,
  worker: SwarmAgent,
  taskName: string,
  rootGoal: string,
  budgetUsd: number
): TaskEnvelope {
  const taskId = `task_${crypto.randomBytes(6).toString("hex")}`;
  const rootGoalHash = hashString(rootGoal);
  const timestamp = new Date().toISOString();

  const payloadToSign = `${taskId}:${coordinator.id}:${worker.id}:${rootGoalHash}:${budgetUsd}:${timestamp}`;
  const signature = signPayload(coordinator, payloadToSign);

  return {
    taskId,
    parentAgentId: coordinator.id,
    assignedAgentId: worker.id,
    taskName,
    rootGoal,
    rootGoalHash,
    budgetUsd,
    status: "pending",
    timestamp,
    signature,
  };
}

/**
 * Checks whether an agent's task has drifted from the immutable root goal.
 */
export function detectGoalDrift(task: TaskEnvelope, expectedRootGoal: string): boolean {
  const expectedHash = hashString(expectedRootGoal);
  return task.rootGoalHash !== expectedHash;
}

/**
 * Worker submits completed work with evidence and a cryptographic signature.
 */
export function submitWorkerResult(
  worker: SwarmAgent,
  task: TaskEnvelope,
  evidence: string
): { evidenceHash: string; workerSignature: string } {
  if (task.assignedAgentId !== worker.id) {
    throw new Error(`Worker ${worker.id} is not authorized for task ${task.taskId}`);
  }

  const evidenceHash = hashString(evidence);
  const payloadToSign = `${task.taskId}:${worker.id}:${task.rootGoalHash}:${evidenceHash}`;
  const workerSignature = signPayload(worker, payloadToSign);

  return {
    evidenceHash,
    workerSignature,
  };
}

/**
 * Reviewer independently verifies the work and signs an Outcome Verification Ticket (OVT).
 * Role-gated: Workers cannot sign verification tickets.
 */
export function reviewAndSignOutcome(
  reviewer: SwarmAgent,
  task: TaskEnvelope,
  worker: SwarmAgent,
  workerSignature: string,
  evidenceHash: string,
  testsPass: boolean
): VerificationTicket {
  if (reviewer.role !== "reviewer" && reviewer.role !== "auditor") {
    throw new Error(`Permission denied: Agent ${reviewer.id} with role "${reviewer.role}" cannot approve gates. Must be reviewer or auditor.`);
  }

  if (reviewer.id === worker.id) {
    throw new Error(`Self-approval forbidden: Worker ${worker.id} cannot act as reviewer for their own work.`);
  }

  // Verify worker's signature first
  const workerPayload = `${task.taskId}:${worker.id}:${task.rootGoalHash}:${evidenceHash}`;
  const workerSigValid = verifySignature(worker.publicKeyPem, workerPayload, workerSignature);
  if (!workerSigValid) {
    throw new Error(`Invalid worker signature on task ${task.taskId}`);
  }

  if (!testsPass) {
    throw new Error(`Reviewer ${reviewer.id} rejected task ${task.taskId}: tests did not pass.`);
  }

  const ticketId = `ovt_${crypto.randomBytes(8).toString("hex")}`;
  const verifiedAt = new Date().toISOString();
  const reviewerPayload = `${ticketId}:${task.taskId}:${reviewer.id}:${evidenceHash}:${verifiedAt}`;
  const reviewerSignature = signPayload(reviewer, reviewerPayload);

  return {
    ticketId,
    taskId: task.taskId,
    rootGoalHash: task.rootGoalHash,
    workerId: worker.id,
    reviewerId: reviewer.id,
    evidenceHash,
    workerSignature,
    reviewerSignature,
    verifiedAt,
  };
}

/**
 * Validates a dual-signed Verification Ticket against the public keys of both agents.
 */
export function verifyDualSignedTicket(
  ticket: VerificationTicket,
  worker: SwarmAgent,
  reviewer: SwarmAgent
): boolean {
  // 1. Verify worker signature
  const workerPayload = `${ticket.taskId}:${ticket.workerId}:${ticket.rootGoalHash}:${ticket.evidenceHash}`;
  const isWorkerValid = verifySignature(worker.publicKeyPem, workerPayload, ticket.workerSignature);

  // 2. Verify reviewer signature
  const reviewerPayload = `${ticket.ticketId}:${ticket.taskId}:${ticket.reviewerId}:${ticket.evidenceHash}:${ticket.verifiedAt}`;
  const isReviewerValid = verifySignature(reviewer.publicKeyPem, reviewerPayload, ticket.reviewerSignature);

  return isWorkerValid && isReviewerValid;
}
