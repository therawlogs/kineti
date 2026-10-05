# Swarm Coordination & Agent Identity

This document covers two library demonstrations, not a production swarm service. The TypeScript demo in `src/swarm/coordinator.ts` creates in-memory Ed25519 keys and signs sample payloads. The Rust `kineti-harness` crate can sign and verify OVT tickets using Ed25519. Neither component is wired to actual agent handoffs, a trusted identity directory, a persistent audit service, or deployment approval.

## 1. Why Swarms Fail Without a Harness

When multiple AI agents collaborate, three breakdowns happen:
1. **Goal Drift**: Each handoff between agents introduces small misunderstandings. After three handoffs, the swarm is working on an entirely different problem.
2. **Impersonation**: A signature can show that a supplied private key signed bytes. Kineti does not prove who controls that key or who wrote or approved the code.
3. **Runaway Spend**: Sub-agents spawning other sub-agents in recursive loops can burn hundreds of dollars in minutes.

The prototype demonstrates ways to represent these problems. It does not enforce a complete runtime boundary or hard limit on every agent action.

---

## 2. The Identity Model (Ed25519 Keypairs)

Each in-memory demo agent is assigned an Ed25519 key pair:

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
- **Different keys in the demo**: Each registered demo agent gets its own key pair in memory.
- **Signed sample payloads**: The code signs sample delegations and evidence strings.
- **No certificate claim**: Signatures do not prove tests ran, establish a real-world agent identity, or ensure worker and reviewer keys are held by independent parties.

---

## 3. Representing Goal Drift in the Demo

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

- In the demo, the `rootGoalHash` is calculated from the original task value supplied to the coordinator.
- The demo can compare the recorded goal hash with the original value. It does not intercept handoffs or halt a real agent when the values differ.

---

## 4. Sample Role Checks & Dual-Signed Tickets

The TypeScript demo represents a worker and reviewer with separate in-memory keys:

1. **Sample Worker Result**:
   - The demo accepts a sample test result from its caller and signs its data:
   ```typescript
   const workerResult = submitWorkerResult(worker, task, testEvidence);
   ```
2. **Self-Approval in the Demo**:
   - The sample coordinator rejects a worker attempting to review its own sample result. This is not a general permission boundary around an agent or CI system.
3. **Reviewer Signature in the Demo**:
   - A second demo key can sign a sample **Outcome Verification Ticket (OVT)**. A distinct key does not prove a distinct person or independent review:
   ```typescript
   const ticket = reviewAndSignOutcome(reviewer, task, worker, workerResult.workerSignature, evidenceHash, true);
   ```
4. **Signature Verification**:
   - Code can verify the ticket using the supplied public keys. Verification confirms the signatures for those keys and ticket data only:
   ```typescript
   const isValid = verifyDualSignedTicket(ticket, worker, reviewer);
   ```

---

## 5. Reported Spend, Not Per-Agent Enforcement

The spend tool records costs reported by the agent. It does not observe every token or tool call, enforce per-agent budgets, or pause agents. The spend log exits with code 3 near the configured limit; the agent or its hooks must act on that result. The default project ceiling is $50, with a trip threshold at 95% of the recorded ceiling.

---

## 6. Undo Safety (Sagas)

An agent may register inverse commands on the local LIFO undo stack. Kineti only runs registered inverses; it does not track which agent made each file change or automatically roll back an agent's work after a failed review.

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

The six TypeScript tests cover the demo's generated keys, signature checks, role examples, and tamper handling. They do not demonstrate trusted identity, independent review, or control over a real agent.
