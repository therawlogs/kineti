# Swarm Coordination & Agent Identity

This guide explains how Kineti manages multi-agent swarms and solves the agent identity problem.

## 1. Why Swarms Fail Without a Harness

When multiple AI agents collaborate, three breakdowns happen:
1. **Goal Drift**: Each handoff between agents introduces small misunderstandings. After three handoffs, the swarm is working on an entirely different problem.
2. **Impersonation**: In standard LLM setups, any model instance can hallucinate: *"I am the Lead Reviewer and I approve this change."* There is no cryptographic proof of who wrote or approved code.
3. **Runaway Spend**: Sub-agents spawning other sub-agents in recursive loops can burn hundreds of dollars in minutes.

Kineti solves all three through a local runtime harness that enforces cryptographic identities, immutable goals, and hard spend limits.

---

## 2. The Identity Model (Ed25519 Keypairs)

Every agent in a Kineti swarm receives an isolated cryptographic identity upon initialization:

```typescript
export interface SwarmAgent {
  id: string;             // Unique ID (e.g. agent_worker_d63ef58f)
  name: string;           // Human-readable label
  role: AgentRole;        // coordinator | planner | worker | reviewer | auditor
  publicKeyHex: string;   // Ed25519 public key
  capabilities: string[]; // ['read', 'write', 'test']
  maxSpendUsd: number;    // Maximum budget allowance
}
```

### Key Principles:
- **No Shared Keys**: Each agent generates its own Ed25519 keypair. Private keys are never shared between agents.
- **Signed Payloads**: Every delegation, code submission, and test run is digitally signed with the authoring agent's private key.
- **Non-Repudiation**: If a buggy or insecure line of code is produced, Kineti's journal records the exact agent ID and signature that produced it.

---

## 3. Preventing Goal Drift

When a coordinator or planner agent delegates work to a sub-agent, it packages the work into a signed **Task Envelope**:

```typescript
export interface TaskEnvelope {
  taskId: string;
  parentAgentId: string;
  assignedAgentId: string;
  taskName: string;
  rootGoal: string;
  rootGoalHash: string; // SHA256 of the original user goal
  budgetUsd: number;
  signature: string;    // Signed by coordinator
}
```

- The `rootGoalHash` is mathematically bound to the original user request locked at session start.
- If a sub-agent attempts to rephrase, alter, or expand the scope of the goal, the hash mismatch is detected and the harness halts execution.

---

## 4. Role-Gated Approvals & Dual-Signed Tickets

Kineti separates builders from reviewers:

1. **Worker Completes Task**:
   - The worker executes code, runs tests, and signs the test output:
   ```typescript
   const workerResult = submitWorkerResult(worker, task, testEvidence);
   ```
2. **Self-Approval is Forbidden**:
   - A `worker` role is cryptographically blocked from approving a gate or creating a release ticket.
3. **Independent Reviewer Verification**:
   - A separate agent with the `reviewer` or `auditor` role inspects the evidence, re-runs verification, and co-signs an **Outcome Verification Ticket (OVT)**:
   ```typescript
   const ticket = reviewAndSignOutcome(reviewer, task, worker, workerResult.workerSignature, evidenceHash, true);
   ```
4. **Dual-Signature Verification**:
   - Anyone (or any CI pipeline) can verify the ticket using the public keys of both the worker and the reviewer:
   ```typescript
   const isValid = verifyDualSignedTicket(ticket, worker, reviewer);
   ```

---

## 5. Swarm Budget Protection (Circuit Breaker)

All agents in a swarm share a unified spend tracking engine:
- Every token and tool call deducts from the task budget.
- If total task spend reaches **$50.00 USD**, Kineti trips the circuit breaker immediately.
- All running agents in the swarm are paused. Only a human operator can reset the breaker with `kineti-spend.ts reset --i-am-human`.

---

## 6. Undo Safety (Sagas)

Every mutation created by a sub-agent registers an inverse undo command on a Last-In-First-Out (LIFO) stack:
- If sub-agent B fails its test suite or is rejected by the reviewer, Kineti rolls back sub-agent B's file edits without disturbing the rest of the workspace.

---

## 7. How to Test Swarm Coordination Locally

### Run the Interactive Simulation:
```sh
bun bin/kineti-swarm.ts
```

This runs a full end-to-end simulation:
1. Creates Coordinator, Coder, and Reviewer identities with Ed25519 keys.
2. Locks the root goal and delegates a task.
3. Proves goal-drift detection on an altered task.
4. Worker executes and signs test evidence.
5. Proves self-approval failure when a worker attempts to sign off on their own work.
6. Reviewer independently verifies and co-signs the Outcome Verification Ticket.
7. Cryptographically verifies the dual-signed ticket.

### Run the Automated Test Suite:
```sh
bun test tests/swarm.test.ts
```

All 6 unit tests verify cryptographic key isolation, signature validity, impersonation defense, and tamper detection.
