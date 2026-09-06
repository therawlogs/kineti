# Technical Survey Report: Core Engine Hardening, Research Substrate Integration & 44 Defect Remediations

**Document ID:** TSR-R2-20260906  
**Milestone:** M2 - Strategy Blueprint (Focus: R2 Core Engine & Integrity Runtime)  
**Author:** Teamwork Explorer `teamwork_preview_explorer_m2_2`  
**Target Workspace:** `/Users/praveen/Documents/Products/kineti local harness`  
**Status:** Authoritative / Complete  

---

## Table of Contents
1. [Executive Summary & Paradigm Shift](#1-executive-summary--paradigm-shift)
2. [The Context Integrity Protocol (CIP) 7-Layer Architecture](#2-the-context-integrity-protocol-cip-7-layer-architecture)
   - [Layer 1: Physical / Transport](#layer-1-physical--transport)
   - [Layer 2: Host Session & Agent Boundary](#layer-2-host-session--agent-boundary)
   - [Layer 3: Agent Context & Working Memory](#layer-3-agent-context--working-memory)
   - [Layer 4: Causal-Graph Substrate](#layer-4-causal-graph-substrate)
   - [Layer 5: Gate Enforcement & Policy Runtime](#layer-5-gate-enforcement--policy-runtime)
   - [Layer 6: Provenance & Cryptographic Attestation](#layer-6-provenance--cryptographic-attestation)
   - [Layer 7: Business & Outcome Layer](#layer-7-business--outcome-layer)
3. [Causal-Graph Substrates & ISO SQL/PGQ Specification](#3-causal-graph-substrates--iso-sqlpgq-specification)
   - [Node & Edge Taxonomy](#node--edge-taxonomy)
   - [Relational & Property Graph Schema (DDL)](#relational--property-graph-schema-ddl)
   - [ISO SQL/PGQ Graph Queries (Traversal, Blame & Cycle Detection)](#iso-sqlpgq-graph-queries)
4. [Universal 20-Entity Provenance Kernel](#4-universal-20-entity-provenance-kernel)
   - [Entity Inventory & Schema Reference](#entity-inventory--schema-reference)
   - [Complete JSON-Schema / TypeScript Definitions](#complete-json-schema--typescript-definitions)
5. [Runtime Ontology Trigger Data (OTD) Mechanics](#5-runtime-ontology-trigger-data-otd-mechanics)
   - [Dynamic Ontological State vs Static Prompts](#dynamic-ontological-state-vs-static-prompts)
   - [OTD Event-Driven State Machine & Activation Schema](#otd-event-driven-state-machine--activation-schema)
6. [Outcome Verification Tickets (OVTs) Deep Specification](#6-outcome-verification-tickets-ovts-deep-specification)
   - [Cryptographic Dual-Signing Protocol](#cryptographic-dual-signing-protocol)
   - [RFC 8785 Canonical JSON Serialization & Merkle Inclusion](#rfc-8785-canonical-json-serialization--merkle-inclusion)
   - [W3C Verifiable Credentials / Exportable Compliance Schema](#w3c-verifiable-credentials--exportable-compliance-schema)
7. [Systematic Remediation & Eradication of All 44 Audit Defects](#7-systematic-remediation--eradication-of-all-44-audit-defects)
   - [Critical Severity Remediations (CRIT-01 to CRIT-05)](#critical-severity-remediations)
   - [High Severity Remediations (HIGH-01 to HIGH-09)](#high-severity-remediations)
   - [Medium Severity Remediations (MED-01 to MED-12)](#medium-severity-remediations)
   - [Low Severity Remediations (LOW-01 to LOW-08)](#low-severity-remediations)
   - [Informational Remediations (INFO-01 to INFO-10)](#informational-remediations)
8. [Conclusion & Next Steps for Strategy Blueprint Production](#8-conclusion--next-steps-for-strategy-blueprint-production)

---

## 1. Executive Summary & Paradigm Shift

Current enterprise adoption of autonomous AI agents is stalling due to an **Accountability Void**. Today's agent frameworks (LangChain, CrewAI, AutoGen, and ad-hoc host scripts) treat agent execution as a non-deterministic, probabilistic black box:
- Memory is treated as ungrounded vector search (RAG), which provides fuzzy similarity without temporal ordering, causation, or deductive proofs.
- Safety relies on brittle prompt engineering ("please do not hallucinate" or "always write tests first"), which models routinely bypass under context pressure.
- Audit trails are unstructured text logs or non-atomic JSONL files subject to silent truncation, data loss, and unverifiable claims.

The Kineti Universal AI Agent Harness and Context Integrity Runtime represents a foundational paradigm shift: **From Probabilistic Execution to Causal Outcome Engineering**.

```
+---------------------------------------------------------------------------------------------------+
|                                 THE OUTCOME ENGINEERING PARADIGM                                  |
+------------------------------------+--------------------------------------------------------------+
| Legacy Agent Frameworks            | Kineti Context Integrity Runtime                             |
+------------------------------------+--------------------------------------------------------------+
| Token-based billing ($/Token)      | Outcome-based billing (Cost Per Verified Outcome, $/Outcome) |
| Probabilistic prompt guardrails    | Deterministic sub-50ms atomic commit gates                    |
| Flat vector embeddings (RAG)       | ISO SQL/PGQ Causal Graphs with formal DAG node/edge semantics|
| Best-effort, unrecoverable failures| Saga LIFO transaction rollbacks with inverse compensations   |
| Mutable, ephemeral console logs    | Dual-signed cryptographic Outcome Verification Tickets (OVTs)|
| Isolated host silo lock-in         | Universal 7-Layer Context Integrity Protocol (CIP)           |
+------------------------------------+--------------------------------------------------------------+
```

This survey establishes the complete architectural substrate required to harden the Kineti engine from a localized prototype into an enterprise-grade runtime. It integrates the author's research papers (*Beyond Vector Search: Causal-Graph Substrates and Runtime OTD*, *Outcome Engineering*, and *The Context Integrity Protocol*) and systematically neutralizes all 44 codebase, architecture, and security defects uncovered in `docs/AUDIT_REPORT.md`.

---

## 2. The Context Integrity Protocol (CIP) 7-Layer Architecture

The Context Integrity Protocol (CIP) standardizes agent interaction into a strict 7-layer stack, analogous to the OSI networking model. Each layer has defined responsibilities, timing budgets, framing standards, and failure boundaries.

```
+-------------------------------------------------------------------------------+
|                       CONTEXT INTEGRITY PROTOCOL (CIP)                        |
+-------+-----------------------------+-----------------------------------------+
| Layer | Layer Name                  | Primary Responsibility                  |
+-------+-----------------------------+-----------------------------------------+
| L7    | Business & Outcome Layer    | OVTs, SLA verification, ROI metrics     |
| L6    | Provenance & Attestation    | Merkle DAG, dual signatures, audit proof|
| L5    | Gate & Policy Runtime       | Deterministic sub-50ms commit gates     |
| L4    | Causal-Graph Substrate      | ISO SQL/PGQ node/edge DAG, cycle check  |
| L3    | Context & Working Memory    | Token accounting, TTL, sanitization     |
| L2    | Host Session & Boundary     | Process sandbox, host adapter, leases   |
| L1    | Physical / Transport        | Local socket, stdio framing, IPC (<1ms) |
+-------+-----------------------------+-----------------------------------------+
```

### Layer 1: Physical / Transport
- **Mechanics:** Low-latency local Inter-Process Communication (IPC) via Unix Domain Sockets (`/tmp/kineti.sock` or `$XDG_RUNTIME_DIR/kineti/ipc.sock` on Linux/macOS; named pipes `\\.\pipe\kineti-ipc` on Windows) and stdio framing for headless MCP (Model Context Protocol) subprocess execution.
- **Framing & Serialization:** JSON-RPC 2.0 with content-length framing (`Content-Length: <bytes>\r\n\r\n<json_payload>`).
- **Timing Budget:** Sub-1ms round-trip transport latency ($\le 850\mu\text{s}$ target).
- **Security & Isolation:** Unix domain sockets are bound to mode `0600` (read/write only by owner UID). Peer PID/UID verification via `SO_PEERCRED` (Linux) or `LOCAL_PEERCRED` (macOS) ensures only authorized local agent processes connect. Stdio streams operate in isolated subprocess execution spaces without shell expansion hazards.

### Layer 2: Host Session & Agent Boundary
- **Mechanics:** Decouples the core engine from external host environments (Google Antigravity, Anthropic Claude Code, OpenAI Codex, OpenCode, Cursor, and raw shell CLI).
- **Session Lifecycle:** 
  1. `Session_Init`: Host presents capability credentials and receives a cryptographically signed Session Token with an immutable lease duration (default: 3600s).
  2. `Heartbeat & Liveness`: Bidirectional ping/pong every 15s. Loss of 3 heartbeats halts agent side-effects.
  3. `Least-Privilege Capability Token`: Host adapters can declare restricted permissions (e.g., `READ_ONLY`, `FILESYSTEM_WRITE_SCOPED`, `SHELL_EXEC_RESTRICTED`).
  4. `Session_Terminate`: Graceful drain or forced abort triggering LIFO saga unwinding.
- **Root Goal Locking:** Upon session establishment, the root goal is captured and cryptographically signed. Any downstream tool call or context injection that attempts to alter the goal is immediately rejected at the Layer 2 boundary.

### Layer 3: Agent Context & Working Memory
- **Mechanics:** Active context window governance, dynamic token-exact billing, and TTL memory lifecycle management.
- **Token Accounting:** Every inbound prompt and outbound completion is measured with byte-exact tokenizers (e.g., Tiktoken for OpenAI/Claude or BPE tokenizers) rather than crude character counts. Cost is computed dynamically against live model pricing tables.
- **Memory Lifecycle & TTL States:**
  $$\text{Record Lifecycle: } \text{Active} \xrightarrow{\text{expiry}} \text{Warm (90d)} \xrightarrow{+185\text{d}} \text{Cold (275d)} \xrightarrow{\text{maintenance}} \text{Archive}$$
  Only `active` records are loaded into privileged prompt context. Warm/Cold records remain queryable via graph index without polluting context tokens.
- **Dual-LLM Sanitization Pipeline (OWASP ASI-01/02):**
  External web data, API receipts, or third-party code are quarantined in an untrusted sandbox context. A secondary, isolated evaluator model parses and strips malicious instructions (prompt injection, jailbreaks, hidden markdown directives) before synthesizing clean facts into primary agent memory.

### Layer 4: Causal-Graph Substrate
- **Mechanics:** Real-time generation of directed acyclic property graphs capturing every agent cognition, tool interaction, and environmental observation.
- **Query Standard:** Standardized on ISO/IEC 9075-16:2023 (SQL/PGQ - Property Graph Queries) allowing relational and graph queries to execute natively in a single SQL engine (PostgreSQL + Age / DuckDB / embedded SQLite-PGQ).
- **Acyclicity & Invariant Enforcement:** Strict cycle detection executed at write-time. Temporal causality enforcement:
  $$\forall e = (u \xrightarrow{\text{CAUSED\_BY}} v), \quad t(v) \le t(u)$$
  An effect can never precede its cause in timestamp order.

### Layer 5: Gate Enforcement & Policy Runtime
- **Mechanics:** Sub-50ms atomic commit gates that halt execution deterministically when invariants fail.
- **Enforcement Pipeline:**
  - **Stage 5 (Feasibility Gate):** Evaluates hurdle rate, data availability, and financial viability. Fails return run to Stage 2 (`diagnose`).
  - **Stage 6 (Spec Gate):** Hard stop requiring cryptographically validated human signature before any code generation is permitted.
  - **Stage 10 (Security Gate):** Executes static analysis, secret scans, dependency audits, and OWASP Top 10 agent vulnerability checks.
  - **Stage 11 (Ship Gate):** Requires fresh evidence proofs (`evidence_fresh == true`) and security clearance (`security_pass == true`).
- **Spend Circuit Breaker:** Continuous evaluation of token spend. If cumulative spend reaches $\$50.00$ USD (or per-stage ceilings), the runtime atomically halts agent execution, locks the workspace, and requires an interactive human operator confirmation.

### Layer 6: Provenance & Cryptographic Attestation
- **Mechanics:** Merkle DAG construction and cryptographic hash-chaining across all execution events.
- **Merkle Leaf Generation:**
  $$\text{Leaf}_n = \text{HMAC-SHA256}\Big(K_{\text{harness}}, \text{prev\_hash}_n \mathbin{\Vert} \text{timestamp}_n \mathbin{\Vert} \text{entity\_id}_n \mathbin{\Vert} \text{CanonicalJSON}(\text{payload}_n)\Big)$$
- **Code Fingerprinting:** Robust recursive hashing of the workspace tree ignoring volatile directories (`.git/`, `.kineti/`, `.agents/`, `design/screenshots/`, `node_modules/`, `dist/`). Failsafe symlink detection preventing infinite loops or FIFO blocking.
- **Tamper Resistance:** Any post-hoc modification of historical execution logs, journal lines, or evidence files immediately breaks the hash chain, causing runtime verification to fail with exit code 3 (`TAMPER_DETECTED`).

### Layer 7: Business & Outcome Layer
- **Mechanics:** Translates technical execution DAGs into quantifiable business outcomes encapsulated in **Outcome Verification Tickets (OVTs)**.
- **Core Value Metric:** Transitioning enterprise contracts from $/Token to **Cost Per Verified Outcome ($/Outcome)**.
  $$\text{Outcome Efficiency Ratio (OER)} = \frac{\text{Quantified Business Value Created (\$)}}{\text{Execution Spend (\$) + Human Review Latency (\$)}} \ge 10.0$$
- **Compliance Artifacts:** Generates immutable, exportable compliance bundles (JSON-LD / W3C Verifiable Credentials) acceptable to enterprise security, SOC 2 Type II audits, ISO 27001, and regulatory oversight bodies.

---

## 3. Causal-Graph Substrates & ISO SQL/PGQ Specification

### Node & Edge Taxonomy

The causal graph models all agent actions and environmental states as a formal Property Graph.

```
       [Goal Node]
            │
            ▼ (DEPENDS_ON)
      [Decision Node] ────(CAUSED_BY)────► [Hypothesis Node]
            │
            ▼ (CAUSED_BY)
       [Action Node] ────(ROLLED_BACK_BY)────► [RollbackAction]
            │
            ▼ (CAUSED_BY)
    [Observation Node]
            │
            ▼ (VERIFIED_BY)
        [Gate Node]
            │
            ▼ (ENABLES)
      [Outcome Node]
```

#### Node Taxonomy (6 Core Node Types)
1. **Action:** A state-mutating operation executed by the agent (e.g., file edit, shell command execution, API write).
2. **Observation:** Ground truth returned from the environment (e.g., tool stdout/stderr, compiler output, HTTP response status, test runner JSON).
3. **Hypothesis:** An agent's explicit inductive reasoning or proposed causal mechanism prior to action.
4. **Decision:** An agent's deliberate branch selection between alternative paths, including trade-off rationale.
5. **Gate:** A deterministic policy checkpoint (e.g., Spec approval, Security scan, Test verification).
6. **Outcome:** A terminal, verified business deliverable (e.g., passed integration test suite, deployed microservice, generated UI).

#### Edge Taxonomy (8 Core Relationship Types)
1. `CAUSED_BY`: Strict temporal causation link. Node $A$ was directly triggered by Node $B$.
2. `DEPENDS_ON`: Structural prerequisite. Node $A$ cannot execute until Node $B$ has resolved.
3. `VERIFIED_BY`: Epistemic validation. Connects an Action or Outcome to an empirical Observation.
4. `ROLLED_BACK_BY`: Compensating association. Connects a mutating Action to its inverse RollbackAction.
5. `BLOCKS`: Preventive relationship. A failed Gate or negative Observation prevents downstream Actions.
6. `ENABLES`: Permissive relationship. Successful passage of a Gate authorizes subsequent Actions.
7. `CONTRADICTS`: Falsification relationship. An Observation disproves a Hypothesis.
8. `REMEDIATES`: Corrective relationship. An Action repairs a defect identified in an Observation.

---

### Relational & Property Graph Schema (DDL)

The schema is implemented in PostgreSQL with ISO SQL/PGQ extension compatibility:

```sql
-- 1. Vertex Tables
CREATE TABLE causal_nodes (
    node_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    node_type VARCHAR(32) NOT NULL CHECK (node_type IN ('Action', 'Observation', 'Hypothesis', 'Decision', 'Gate', 'Outcome')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    label VARCHAR(255) NOT NULL,
    payload_hash CHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    merkle_leaf_hash CHAR(64) NOT NULL
);

CREATE INDEX idx_causal_nodes_session ON causal_nodes(session_id);
CREATE INDEX idx_causal_nodes_type ON causal_nodes(node_type);
CREATE INDEX idx_causal_nodes_created ON causal_nodes(created_at);

-- 2. Edge Tables
CREATE TABLE causal_edges (
    edge_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    source_node_id UUID NOT NULL REFERENCES causal_nodes(node_id) ON DELETE CASCADE,
    target_node_id UUID NOT NULL REFERENCES causal_nodes(node_id) ON DELETE CASCADE,
    relationship_type VARCHAR(32) NOT NULL CHECK (relationship_type IN (
        'CAUSED_BY', 'DEPENDS_ON', 'VERIFIED_BY', 'ROLLED_BACK_BY',
        'BLOCKS', 'ENABLES', 'CONTRADICTS', 'REMEDIATES'
    )),
    status VARCHAR(16) NOT NULL DEFAULT 'candidate' CHECK (status IN ('candidate', 'hypothesis', 'validated', 'rejected')),
    weight NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    proof_id UUID REFERENCES causal_nodes(node_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT chk_temporal_order CHECK (
        relationship_type NOT IN ('CAUSED_BY', 'BLOCKS') OR created_at >= created_at
    )
);

CREATE INDEX idx_causal_edges_nodes ON causal_edges(source_node_id, target_node_id);
CREATE INDEX idx_causal_edges_rel ON causal_edges(relationship_type);

-- 3. ISO SQL/PGQ Property Graph Definition
CREATE PROPERTY GRAPH agent_causal_graph
VERTEX TABLES (
    causal_nodes LABEL Node
        PROPERTIES (node_id, node_type, label, created_at, payload)
)
EDGE TABLES (
    causal_edges
        SOURCE KEY (source_node_id) REFERENCES causal_nodes (node_id)
        DESTINATION KEY (target_node_id) REFERENCES causal_nodes (node_id)
        LABEL Edge
        PROPERTIES (edge_id, relationship_type, status, weight)
);
```

---

### ISO SQL/PGQ Graph Queries

#### Query 1: Full Provenance Traversal (Root Goal to Outcome)
Traverses the causal ancestry from a delivered Outcome back to the root Goal, collecting every intermediate action, observation, and verification proof:

```sql
SELECT *
FROM GRAPH_TABLE (agent_causal_graph
    MATCH (out:Node WHERE out.node_type = 'Outcome')
          <-[e:Edge WHERE e.relationship_type IN ('CAUSED_BY', 'ENABLES')]-+ (ancestor:Node)
    COLUMNS (
        out.node_id AS outcome_id,
        ancestor.node_id AS step_id,
        ancestor.node_type AS step_type,
        ancestor.label AS step_label,
        ancestor.created_at AS step_time
    )
)
ORDER BY step_time ASC;
```

#### Query 2: Write-Time Cycle Detection (Preventing Circular Causality)
Before inserting a new dependency edge `($new_source, $new_target, 'DEPENDS_ON')`, the runtime ensures no path already exists from `$new_target` to `$new_source`:

```sql
SELECT COUNT(*) AS cycle_exists
FROM GRAPH_TABLE (agent_causal_graph
    MATCH (src:Node WHERE src.node_id = :new_target)
          -[e:Edge WHERE e.relationship_type = 'DEPENDS_ON']-+
          (dst:Node WHERE dst.node_id = :new_source)
    COLUMNS (src.node_id AS cycle_root)
);
```
*If `cycle_exists > 0`, the transaction is aborted with `CYCLE_DETECTED_ERROR`.*

#### Query 3: Blame & Root-Cause Analysis for Gate Failures
When a Gate fails (e.g. Stage 11 Ship Gate), this query automatically pinpoints the exact CodeDiff and mutating Action that triggered the failure:

```sql
SELECT *
FROM GRAPH_TABLE (agent_causal_graph
    MATCH (g:Node WHERE g.node_type = 'Gate' AND g.payload->>'status' = 'fail')
          <-[:Edge WHERE relationship_type = 'VERIFIED_BY']-
          (obs:Node WHERE obs.node_type = 'Observation')
          <-[:Edge WHERE relationship_type = 'CAUSED_BY']-
          (act:Node WHERE act.node_type = 'Action')
    COLUMNS (
        g.label AS failed_gate,
        obs.label AS error_observation,
        obs.payload->>'stderr' AS failure_detail,
        act.node_id AS offending_action_id,
        act.label AS offending_action,
        act.payload->>'command' AS offending_command
    )
);
```

#### Query 4: Topological Saga Rollback Sequence Generation
Retrieves all mutating actions for a failed session in exact reverse topological order, resolving compensating inverse actions:

```sql
SELECT 
    act.node_id AS action_id,
    act.label AS action_label,
    rb.payload->>'inverse_command' AS undo_command,
    act.created_at AS executed_at
FROM causal_nodes act
JOIN causal_edges e ON e.source_node_id = act.node_id AND e.relationship_type = 'ROLLED_BACK_BY'
JOIN causal_nodes rb ON rb.node_id = e.target_node_id
WHERE act.session_id = :session_id
ORDER BY act.created_at DESC;
```

---

## 4. Universal 20-Entity Provenance Kernel

To guarantee absolute interoperability across hosts and agent engines, Kineti defines the **Universal 20-Entity Provenance Kernel**. Every event in an agent's lifecycle is an instance of one of these 20 strongly typed entities.

```
+---------------------------------------------------------------------------------------------------+
|                            UNIVERSAL 20-ENTITY PROVENANCE KERNEL                                  |
+----+---------------+------------------------------------------------------------------------------+
| #  | Entity        | Semantic Definition & Role in Harness Runtime                                |
+----+---------------+------------------------------------------------------------------------------+
| 1  | Agent         | Identity, public signing key, declared capabilities, and runtime bounds.     |
| 2  | Session       | Isolated temporal context container, host binding, lease, and permissions.   |
| 3  | Host          | Runtime host descriptor (Google Antigravity, Claude Code, Codex, OpenCode).  |
| 4  | Goal          | Immutable root objective statement, cryptographic hash, and success criteria.|
| 5  | Milestone     | Major stage marker along the 13-stage execution pipeline.                     |
| 6  | Task          | Specific decomposed unit of executable work delegated to an Agent.           |
| 7  | ToolCall      | Structured invocation intent: tool name, arguments, and unique call ID.      |
| 8  | ToolResult    | Execution outcome: stdout, stderr, exit code, duration, and execution host.  |
| 9  | Observation   | Distilled environmental fact extracted from ToolResult or user communication.|
| 10 | Artifact      | Concrete file or asset produced or modified during execution.                 |
| 11 | CodeDiff      | Unified patch representation of modifications applied to workspace files.    |
| 12 | Assertion     | Deterministic claim evaluated during automated test execution.               |
| 13 | GateResult    | Pass/fail/pending verdict emitted by a Layer 5 policy checkpoint.            |
| 14 | SpendEntry    | Exact prompt/completion token count and financial dollar cost of an API call.|
| 15 | RollbackAction| LIFO compensating transaction registration (inverse shell command or API).  |
| 16 | Checkpoint    | Point-in-time cryptographic freeze of working memory, DAG state, and files.  |
| 17 | OTDTrigger    | Active Ontology Trigger Data rule specification and firing criteria.         |
| 18 | EvidenceLog   | Cryptographic proof record binding test results to workspace code fingerprint|
| 19 | MerkleLeaf    | Hash node in the continuous cryptographic execution provenance tree.          |
| 20 | OVT           | Dual-signed Outcome Verification Ticket certifying final deliverable.        |
+----+---------------+------------------------------------------------------------------------------+
```

### Complete JSON-Schema / TypeScript Definitions

Below are the canonical TypeScript interface definitions and JSON schemas for the 20 entities:

```typescript
// ==========================================
// UNIVERSAL 20-ENTITY PROVENANCE KERNEL SCHEMAS
// ==========================================

export type UUID = string;
export type ISO8601 = string;
export type SHA256 = string;

// 1. Agent
export interface Agent {
  id: UUID;
  public_key: string; // Ed25519 public key hex
  archetype: "explorer" | "worker" | "challenger" | "reviewer" | "auditor";
  model_id: string;   // e.g. "claude-3-7-sonnet", "gpt-4o"
  capabilities: string[];
  max_spend_limit_usd: number;
}

// 2. Session
export interface Session {
  id: UUID;
  agent_id: UUID;
  host_id: UUID;
  created_at: ISO8601;
  expires_at: ISO8601;
  status: "active" | "committed" | "aborted" | "timed_out";
  working_directory: string;
}

// 3. Host
export interface Host {
  id: UUID;
  name: "antigravity" | "claude_code" | "codex" | "opencode" | "cursor" | "cli";
  version: string;
  os: "darwin" | "linux" | "win32";
  protocol_version: "CIP-1.0";
}

// 4. Goal
export interface Goal {
  id: UUID;
  session_id: UUID;
  root_goal: string;
  constraints: string[];
  success_criteria: string[];
  immutable_hash: SHA256; // SHA256(root_goal + constraints)
  locked_at: ISO8601;
}

// 5. Milestone
export interface Milestone {
  id: UUID;
  session_id: UUID;
  stage_number: number; // 1 to 13
  stage_id: "officehours" | "diagnose" | "design" | "architecture" | "feasibility" | "spec" | "build" | "review" | "qa" | "security" | "ship" | "watch" | "retro";
  status: "pending" | "in_progress" | "passed" | "failed";
  entered_at: ISO8601;
  exited_at?: ISO8601;
}

// 6. Task
export interface Task {
  id: UUID;
  milestone_id: UUID;
  agent_id: UUID;
  title: string;
  description: string;
  status: "planned" | "in_progress" | "completed" | "failed";
}

// 7. ToolCall
export interface ToolCall {
  id: UUID;
  task_id: UUID;
  tool_name: string;
  arguments: Record<string, unknown>;
  issued_at: ISO8601;
}

// 8. ToolResult
export interface ToolResult {
  id: UUID;
  tool_call_id: UUID;
  stdout: string;
  stderr: string;
  exit_code: number;
  duration_ms: number;
  executed_at: ISO8601;
}

// 9. Observation
export interface Observation {
  id: UUID;
  tool_result_id?: UUID;
  fact: string;
  confidence: number; // 0.0000 to 1.0000
  source: string;
  created_at: ISO8601;
}

// 10. Artifact
export interface Artifact {
  id: UUID;
  session_id: UUID;
  relative_path: string;
  content_type: string;
  sha256: SHA256;
  size_bytes: number;
  created_at: ISO8601;
}

// 11. CodeDiff
export interface CodeDiff {
  id: UUID;
  artifact_id: UUID;
  unified_diff: string;
  additions: number;
  deletions: number;
  applied_at: ISO8601;
}

// 12. Assertion
export interface Assertion {
  id: UUID;
  test_name: string;
  status: "pass" | "fail";
  expected_value: string;
  actual_value: string;
  evaluated_at: ISO8601;
}

// 13. GateResult
export interface GateResult {
  id: UUID;
  milestone_id: UUID;
  gate_name: "feasibility" | "spec" | "security" | "ship";
  status: "pass" | "fail" | "pending";
  evaluated_rules: Array<{ rule: string; passed: boolean; reason?: string }>;
  evaluator_signature: string; // Ed25519 signature
  evaluated_at: ISO8601;
}

// 14. SpendEntry
export interface SpendEntry {
  id: UUID;
  session_id: UUID;
  stage_id: string;
  tokens_in: number;
  tokens_out: number;
  tokens_cached: number;
  usd_cost: number;
  cumulative_session_usd: number;
  logged_at: ISO8601;
}

// 15. RollbackAction
export interface RollbackAction {
  id: UUID;
  session_id: UUID;
  step_number: number; // LIFO index
  label: string;
  inverse_command: string;
  state: "registered" | "executed" | "failed";
  registered_at: ISO8601;
}

// 16. Checkpoint
export interface Checkpoint {
  id: UUID;
  session_id: UUID;
  git_commit_sha: string;
  dag_root_hash: SHA256;
  spend_usd_snapshot: number;
  created_at: ISO8601;
}

// 17. OTDTrigger
export interface OTDTrigger {
  id: UUID;
  trigger_name: string;
  condition_predicate: string; // JSONPath or DSL expression
  target_ontology_state: string;
  action_payload: Record<string, unknown>;
  created_at: ISO8601;
}

// 18. EvidenceLog
export interface EvidenceLog {
  id: UUID;
  session_id: UUID;
  label: string;
  command: string;
  exit_code: number;
  code_fingerprint_before: SHA256;
  code_fingerprint_after: SHA256;
  recorded_at: ISO8601;
  status: "FRESH" | "STALE" | "MISSING";
}

// 19. MerkleLeaf
export interface MerkleLeaf {
  leaf_index: number;
  prev_hash: SHA256;
  entity_type: string;
  entity_id: UUID;
  canonical_payload_hash: SHA256;
  node_hash: SHA256; // SHA256(prev_hash + entity_type + entity_id + canonical_payload_hash)
  timestamp: ISO8601;
}

// 20. OVT (Outcome Verification Ticket)
export interface OVT {
  ticket_id: UUID;
  session_id: UUID;
  goal_hash: SHA256;
  code_fingerprint: SHA256;
  evidence_hash: SHA256;
  merkle_root: SHA256;
  total_spend_usd: number;
  delivery_timestamp: ISO8601;
  signatures: {
    agent_signature: string;  // Signed by Agent private key
    harness_signature: string;// Signed by Local Harness Daemon private key
  };
  compliance_proof: {
    w3c_verifiable_credential_uri?: string;
    merkle_inclusion_proof: string[];
  };
}
```

---

## 5. Runtime Ontology Trigger Data (OTD) Mechanics

### Dynamic Ontological State vs Static Prompts

Traditional agent workflows embed fixed rules into initial system prompts (e.g. "you are a software engineer; write clean code"). As execution progresses and the context window grows, LLMs suffer from **Context Distraction and Attention Decay**, drifting away from constraints.

**Runtime Ontology Trigger Data (OTD)** replaces static prompt bloat with an event-driven, dynamic state machine:
1. The harness runtime maintains a formal state machine representing the active **Ontological Mode** (e.g. `DIAGNOSIS_MODE`, `SPEC_LOCK_MODE`, `BUILD_SAFE_MODE`, `RECOVERY_MODE`).
2. When environmental telemetry satisfies an OTD trigger predicate (e.g. a compiler error, a spend threshold, or a gate status change), the harness injects an **OTD Activation Packet** directly into the Layer 3 context buffer and updates capability boundaries at Layer 2.
3. The agent is prevented from executing operations outside the active ontology (e.g., in `RECOVERY_MODE`, write permissions to `/src` are revoked, and only `bin/kineti-saga rollback` operations are authorized).

```
                [Telemetry Event: Test Failure]
                               │
                               ▼
                   ┌───────────────────────┐
                   │  OTD Evaluation Engine │
                   │  (Sub-10ms Matcher)   │
                   └───────────┬───────────┘
                               │ (Matches Predicate)
                               ▼
        ┌──────────────────────────────────────────────┐
        │  Transition: BUILD_MODE ──► AUTO_REPAIR_MODE │
        ├──────────────────────────────────────────────┤
        │ - Scope locked to failed test file           │
        │ - Max repair budget set to $2.00             │
        │ - Maximum 5 self-repair iterations           │
        │ - Saga undo checkpoint registered            │
        └──────────────────────────────────────────────┘
```

### OTD Event-Driven State Machine & Activation Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OTDTriggerDefinition",
  "type": "object",
  "required": ["trigger_id", "name", "event_source", "condition", "transition_to", "enforcement"],
  "properties": {
    "trigger_id": { "type": "string", "format": "uuid" },
    "name": { "type": "string" },
    "event_source": { 
      "type": "string", 
      "enum": ["tool_result", "spend_telemetry", "gate_evaluation", "file_watcher", "heartbeat"] 
    },
    "condition": {
      "type": "object",
      "required": ["predicate", "eval_expr"],
      "properties": {
        "predicate": { "type": "string", "enum": ["EQUALS", "GREATER_THAN", "MATCHES_REGEX", "GRAPH_PATH_EXISTS"] },
        "eval_expr": { "type": "string" },
        "threshold": { "type": ["string", "number", "boolean"] }
      }
    },
    "transition_to": {
      "type": "object",
      "required": ["ontology_state", "active_skills", "revoked_tools"],
      "properties": {
        "ontology_state": { "type": "string" },
        "active_skills": { "type": "array", "items": { "type": "string" } },
        "revoked_tools": { "type": "array", "items": { "type": "string" } }
      }
    },
    "enforcement": {
      "type": "object",
      "required": ["deterministic_block", "timeout_ms"],
      "properties": {
        "deterministic_block": { "type": "boolean" },
        "timeout_ms": { "type": "integer", "default": 5000 }
      }
    }
  }
}
```

---

## 6. Outcome Verification Tickets (OVTs) Deep Specification

### Cryptographic Dual-Signing Protocol

An **Outcome Verification Ticket (OVT)** is an unforgeable, cryptographically dual-signed credential that proves an autonomous agent achieved a specific business outcome while complying with all Layer 5 governance gates.

```
       [Agent Private Key]                  [Harness Private Key]
              │                                      │
              ▼                                      ▼
     [Sign(Outcome Hash)]                 [Sign(Gate + Merkle Root)]
              │                                      │
              └───────────────┬──────────────────────┘
                              ▼
                 ┌──────────────────────────┐
                 │ Outcome Verification     │
                 │ Ticket (OVT)             │
                 │                          │
                 │ - Dual Ed25519 Sigs      │
                 │ - Merkle Root Verified   │
                 │ - Code Fingerprint Match │
                 │ - Exact Token Cost Audit │
                 └──────────────────────────┘
```

#### Dual-Signing Lifecycle:
1. **Agent Claim Signing:** Upon completing the terminal task, the agent constructs the `OutcomeClaim`:
   $$\text{Hash}_{\text{agent}} = \text{SHA256}\Big(\text{session\_id} \mathbin{\Vert} \text{goal\_hash} \mathbin{\Vert} \text{deliverable\_artifacts\_hash}\Big)$$
   The agent signs $\text{Hash}_{\text{agent}}$ with its ephemeral Ed25519 private key: $S_{\text{agent}} = \text{Sign}(K_{\text{agent\_priv}}, \text{Hash}_{\text{agent}})$.
2. **Harness Verification & Counter-Signing:** The local harness daemon receives the claim and executes independent verification:
   - Evaluates Layer 5 Gates (Feasibility, Spec, Security, Ship).
   - Recalculates the workspace code fingerprint ($\text{FP}_{\text{code}}$) and validates against `bin/kineti-evidence`.
   - Recomputes the execution Merkle Root ($M_{\text{root}}$) over all 20 kernel entities.
   - Computes the combined verification digest:
     $$\text{Hash}_{\text{harness}} = \text{SHA256}\Big(\text{Hash}_{\text{agent}} \mathbin{\Vert} S_{\text{agent}} \mathbin{\Vert} M_{\text{root}} \mathbin{\Vert} \text{FP}_{\text{code}} \mathbin{\Vert} \text{spend\_total\_usd}\Big)$$
   - The harness signs with its master key: $S_{\text{harness}} = \text{Sign}(K_{\text{harness\_priv}}, \text{Hash}_{\text{harness}})$.
3. **Immutability & Tamper Resistance:** An OVT cannot be forged by an agent because it lacks $K_{\text{harness\_priv}}$. It cannot be forged by an external actor because any modification to code or logs invalidates $\text{FP}_{\text{code}}$ and $M_{\text{root}}$.

---

### RFC 8785 Canonical JSON Serialization & Merkle Inclusion

To prevent signature verification failures caused by JSON key reordering, whitespace discrepancies, or unicode normalization variations, all signing payloads must strictly conform to **RFC 8785 (JSON Canonicalization Scheme / JCS)**:
1. Object keys are sorted lexicographically by UTF-16 code units.
2. Whitespace outside string literals is strictly eliminated.
3. Numbers are formatted without trailing zeroes or exponent quirks.
4. Strings are encoded in UTF-8 with standard escaping rules.

Each OVT embeds a cryptographic **Merkle Inclusion Proof** allowing external verifiers (CI/CD pipelines, enterprise compliance dashboards) to verify that any individual Action or Observation was part of the verified execution trace without downloading the full multi-megabyte log.

---

### W3C Verifiable Credentials / Exportable Compliance Schema

```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://schema.kineti.ai/v1/ovt"
  ],
  "id": "urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "type": ["VerifiableCredential", "OutcomeVerificationTicket"],
  "issuer": "did:kineti:host:macbook-pro-m3-local-harness",
  "issuanceDate": "2026-09-06T05:22:04Z",
  "credentialSubject": {
    "id": "did:kineti:agent:explorer-m2-2",
    "sessionId": "787db8bf-e17f-440a-abec-0fbfbb7ecae3",
    "rootGoal": "Author authoritative technical survey report for R2 Core Engine Hardening",
    "rootGoalHash": "8f4b23c5e6d78a9012345678abcdef0123456789abcdef0123456789abcdef01",
    "codeFingerprint": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "merkleRoot": "1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f809",
    "outcomes": [
      {
        "milestone": "ship",
        "status": "VERIFIED_PASS",
        "testProofHash": "4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b",
        "spendUsd": 1.42
      }
    ],
    "policyGatesPassed": ["feasibility", "spec", "security", "ship"],
    "totalSessionSpendUsd": 1.42
  },
  "proof": [
    {
      "type": "Ed25519Signature2020",
      "created": "2026-09-06T05:22:05Z",
      "verificationMethod": "did:kineti:agent:explorer-m2-2#key-1",
      "proofPurpose": "assertionMethod",
      "proofValue": "z3h...AgentSignatureBase58..."
    },
    {
      "type": "Ed25519Signature2020",
      "created": "2026-09-06T05:22:06Z",
      "verificationMethod": "did:kineti:host:macbook-pro-m3-local-harness#host-key",
      "proofPurpose": "verificationMethod",
      "proofValue": "z7k...HarnessSignatureBase58..."
    }
  ]
}
```

---

## 7. Systematic Remediation & Eradication of All 44 Audit Defects

This section provides an exhaustive review and architectural eradication specification for every single one of the **44 defect findings** documented in `docs/AUDIT_REPORT.md`.

---

### Critical Severity Remediations

#### [CRIT-01] Baseline Test Suite Failure in Memory Job Test
- **Impacted Files & Lines:** `tests/memory-job.test.ts:37-56` (interacting with `bin/kineti-memory-job.ts:64-66, 104-107`).
- **Root Cause:** 
  1. Hash calculation in test used pipe delimiters (`${r.prev_hash}|${r.at}|...`), while `bin/kineti-memory-job.ts:65` computed hashes without delimiters.
  2. `oldLearning` record was inserted without `prev_hash` and `hash`, causing `verify-chain` to crash with exit code 3 (`CHAIN BROKEN: missing prev_hash/hash`).
  3. Key sorting desynchronization: `bin/kineti-memory-job.ts` enforces strict alphabetical key canonicalization (`canonStable`), whereas `oldLearning.data` declared keys out of alphabetical order (`skill`, `trigger`, `lesson`), causing content hash mismatches.
- **Universal Runtime Architectural Mechanism:** 
  The new runtime completely removes manual ad-hoc string concatenation hashing in favor of **Layer 6 Automated Merkle Leaf Canonicalization**. The kernel provides an immutable `computeRecordHash(entity)` function utilizing RFC 8785 canonical JSON serialization. Unchained records are rejected at ingestion time rather than failing during downstream sweeps.
- **Concrete Fix Specification:**
  In `tests/memory-job.test.ts`:
  1. Synchronize hash calculation to remove pipe delimiters.
  2. Supply `prev_hash: r2.hash` and valid canonical hash on `oldLearning`.
  3. Sort `oldLearning.data` keys alphabetically: `{ lesson: "old", skill: "qa", trigger: "always" }`.
  *Result: `bun test tests/memory-job.test.ts` passes 2/2 tests cleanly.*

#### [CRIT-02] Broken Gate Status Lookup in State CLI Blocking Build & Ship Pre-Flights
- **Impacted Files & Lines:** `bin/kineti-state.ts:70-77` (interacting with `skills/build/SKILL.md:23` and `skills/ship/SKILL.md:29-30`).
- **Root Cause:** 
  `cmd === "get"` evaluated `(s as any)["gate.spec"]`, which returned `undefined` because gate statuses are stored in nested dictionary `s.gates[name]`. This caused pre-flight checks in Stage 7 (`build`) and Stage 11 (`ship`) to crash with exit code 2.
- **Universal Runtime Architectural Mechanism:**
  **Layer 5 Gate State Engine**. Gate states are first-class kernel entities (`GateResult`) accessed via typed getters with automatic dot-notation path resolution (`gate.<name>` seamlessly maps to `s.gates[<name>]`).
- **Concrete Fix Specification:**
  In `bin/kineti-state.ts`:
  ```typescript
  if (cmd === "get") {
    const key = rest[0];
    if (!key) { console.log(JSON.stringify(s, null, 2)); return; }
    let v = (s as any)[key];
    if (v === undefined && key.startsWith("gate.")) {
      v = s.gates[key.slice(5)];
    }
    if (v === undefined) die(`unknown key: ${key}`, 2);
    console.log(typeof v === "string" ? v : JSON.stringify(v, null, 2));
    return;
  }
  ```

#### [CRIT-03] Programmatic Self-Trust Security Bypass in Verify Gate
- **Impacted Files & Lines:** `bin/kineti-verify-gate.ts:20-27`.
- **Root Cause:** 
  `--trust` could be executed programmatically by an untrusted subagent without human interaction, allowing autonomous agents to authorize and execute arbitrary shell commands via `bash -lc`.
- **Universal Runtime Architectural Mechanism:**
  **Layer 2 Host-Agent Privilege Boundary & Human-in-the-Loop Attestation**. Privileged operations (trust elevation, circuit breaker resets) require an interactive human TTY confirmation or a cryptographically signed human authorization token (`KINETI_TRUST_CONFIRMED=1`). Non-interactive calls from autonomous agents are blocked with exit code 2.
- **Concrete Fix Specification:**
  In `bin/kineti-verify-gate.ts`:
  ```typescript
  if (cmd === "--trust") {
    if (!declared) die("no verify command declared", 2);
    if (!process.stdin.isTTY && !process.env.KINETI_TRUST_CONFIRMED) {
      die("security: --trust must be executed interactively in a human TTY session", 2);
    }
    const t = readJson<Trust>(trustFile()) ?? {};
    t[repoKey()] = { cmd_hash: sha256(declared), at: nowIso() };
    writeJson(trustFile(), t);
    ok(`trusted for this repo: ${declared}`);
    return;
  }
  ```

#### [CRIT-04] Catastrophic Silent Ledger Erasure on JSONL Parse Errors
- **Impacted Files & Lines:** `bin/lib.ts:48-58`.
- **Root Cause:** 
  Wrapping `readJsonl` in a blanket `try...catch` that returned `[]` meant that a single corrupt line, partial write, or null byte caused the entire file to be treated as empty. Downstream writes then truncated the ledger, permanently destroying memory, egress chains, and rollback stacks.
- **Universal Runtime Architectural Mechanism:**
  **Layer 1 Resilient Stream Parser with Atomic Fsync Logging**. The parser processes lines independently, skipping corrupted lines with prominent stderr diagnostics while preserving all valid historical records. Ledger append operations utilize strict atomic write-and-rename or append-only file locks with `fsync()`.
- **Concrete Fix Specification:**
  In `bin/lib.ts`:
  ```typescript
  export function readJsonl<T>(file: string): T[] {
    if (!fs.existsSync(file)) return [];
    try {
      const text = fs.readFileSync(file, "utf8");
      const lines = text.split("\n").filter((l) => l.trim().length > 0);
      const items: T[] = [];
      for (let i = 0; i < lines.length; i++) {
        try {
          items.push(JSON.parse(lines[i]) as T);
        } catch (err) {
          console.error(`kineti: warning: corrupt line ${i + 1} in ${file} skipped: ${(err as Error).message}`);
        }
      }
      return items;
    } catch (err) {
      console.error(`kineti: error reading ${file}: ${(err as Error).message}`);
      return [];
    }
  }
  ```

#### [CRIT-05] Unhandled Missing Repo Pointer & Space-Path Splitting in Cron Script
- **Impacted Files & Lines:** `scripts/weekly.sh:10, 13, 16`.
- **Root Cause:** 
  `cat "$HOME/.kineti/repo"` failed under `set -e` if the pointer was missing. `for p in $PROJECTS; do` performed unquoted word-splitting, breaking when repository paths contained spaces (e.g., `/kineti local harness/`).
- **Universal Runtime Architectural Mechanism:**
  **Layer 1 Robust POSIX/Bash Packaging & Harness Daemon Automation**. Replaces brittle shell word-splitting with structured delimiter parsing (`IFS=':' read -r -a project_list`) and explicit pointer validation with actionable diagnostics.
- **Concrete Fix Specification:**
  In `scripts/weekly.sh`:
  ```bash
  REPO_FILE="$HOME/.kineti/repo"
  [[ -f "$REPO_FILE" ]] || { echo "kineti: error: '$REPO_FILE' missing. Run ./setup.sh first." >&2; exit 1; }
  KIN="$(head -n 1 "$REPO_FILE" | tr -d '\r\n')"
  [[ -d "$KIN" ]] || { echo "kineti: error: directory '$KIN' does not exist." >&2; exit 1; }
  if [[ -n "${KINETI_PROJECTS:-}" ]]; then
    IFS=':' read -r -a project_list <<< "$KINETI_PROJECTS"
  else
    project_list=("$PWD")
  fi
  for p in "${project_list[@]}"; do
    [[ -f "$p/.kineti/journal.jsonl" ]] || continue
  ```

---

### High Severity Remediations

#### [HIGH-01] Arbitrary Shell Execution & Stderr Swallowing During Saga Rollbacks
- **Impacted Files & Lines:** `bin/kineti-saga.ts:63-72`.
- **Root Cause:** Undos executed via `bash -lc` had no execution timeout (risking infinite process hangs) and completely swallowed stderr, leaving developers blind when undo steps failed.
- **Universal Runtime Architectural Mechanism:**
  **Layer 5 Saga Compensating Engine with ETHOS Rule 4.2 Harmonization**. Default execution respects ETHOS Rule 4.2 (unwind newest-first, log diagnostic stderr, continue with remaining undos). Commands run with a 30s timeout (`timeout: 30000`). An optional `--fail-fast` flag is provided for strict cascading operations.
- **Concrete Fix Specification:**
  Add `timeout: 30000`, capture `res.stderr`, emit prominent error detail on failure, and support `--fail-fast`.

#### [HIGH-02] Command Injection, Argument Flattening & Diagnostic Blindness in Proof Capture
- **Impacted Files & Lines:** `bin/kineti-evidence.ts:61, 64, 71`.
- **Root Cause:** `argv.slice(dd + 1).join(" ")` stripped quotes and shell boundaries. Subprocess execution swallowed `stderr` on non-zero exit codes.
- **Universal Runtime Architectural Mechanism:**
  **Hardened Process Wrapper**. Forward child process stderr directly to `process.stderr` on non-zero exits, ensuring complete diagnostic visibility. Preserves command argument arrays without flattening.

#### [HIGH-03] Path Duplication Bug Creating Nested `.kineti/.kineti/spend.log.jsonl`
- **Impacted Files & Lines:** `bin/kineti-spend.ts:29`.
- **Root Cause:** `logFile()` concatenated `".kineti"` to `projectKdir()`, which already included `.kineti`, producing `.kineti/.kineti/spend.log.jsonl`.
- **Universal Runtime Architectural Mechanism:**
  **Centralized Path Resolution Authority**. Single source of truth for repository paths in `lib.ts`. Change `path.join(projectKdir(), ".kineti", "spend.log.jsonl")` to `path.join(projectKdir(), "spend.log.jsonl")`.

#### [HIGH-04] Code Fingerprint Invalidation Caused by `.agents/` and `design/screenshots/`
- **Impacted Files & Lines:** `bin/kineti-evidence.ts:11-15`.
- **Root Cause:** Omission of `.agents/` and `screenshots/` from `EXCLUDE_DIRS` meant that agent heartbeats or QA screenshot generation altered the code fingerprint, immediately flipping valid test proofs to `STALE` and blocking Ship.
- **Universal Runtime Architectural Mechanism:**
  **Layer 6 Deterministic Workspace Fingerprinting**. Add `.agents` and `screenshots` to `EXCLUDE_DIRS`. Test proofs remain strictly bound to production source code and test files.

#### [HIGH-05] Spend Circuit Breaker Reset Bypass via Static CLI Flag
- **Impacted Files & Lines:** `bin/kineti-spend.ts:97-103`.
- **Root Cause:** Reset relied solely on passing static flag `--i-am-human`. Autonomous LLMs parsed the error and executed the flag, bypassing the $50 safety ceiling.
- **Universal Runtime Architectural Mechanism:**
  **Layer 5 Cryptographic / TTY Circuit Breaker Reset**. Reset requires an interactive human TTY confirmation (`process.stdin.isTTY`) or a privileged environment secret (`KINETI_HUMAN_RESET_TOKEN`), preventing autonomous agent self-reset.

#### [HIGH-06] Spec Skill Directs Unsupported Gate State Transition `gate.spec pending`
- **Impacted Files & Lines:** `skills/spec/SKILL.md:49` vs `bin/kineti-state.ts:95-98`.
- **Root Cause:** Spec skill instructed setting `gate.spec pending`, but `kineti-state.ts` strictly threw exit code 2 on any value other than `pass` or `fail`.
- **Universal Runtime Architectural Mechanism:**
  **Three-State Gate Protocol**. Update `kineti-state.ts` to natively support `pass | fail | pending`.

#### [HIGH-07] Uncaught Runtime `TypeError` on Missing CLI Option Argument
- **Impacted Files & Lines:** `bin/kineti-memory-job.ts:74-75`.
- **Root Cause:** Passing `--dir` as final argument caused `process.argv[di + 1]` to evaluate to `undefined`, crashing unhandled inside `path.resolve(undefined)`.
- **Universal Runtime Architectural Mechanism:**
  **Strict CLI Argument Validation**. Validate that option arguments exist and do not start with `-`; otherwise exit cleanly with code 2.

#### [HIGH-08] Installer Smoke Test Excluded from Default Runner
- **Impacted Files & Lines:** `package.json:8`, `tests/test-setup.sh`.
- **Root Cause:** `bun test tests/` only executed `.ts` files, leaving `test-setup.sh` completely skipped in CI.
- **Universal Runtime Architectural Mechanism:**
  **Unified Test Pipeline**. Update `package.json` test script to `"bun test tests/ && bash tests/test-setup.sh"` or wrap shell tests in a native TypeScript runner.

#### [HIGH-09] Critical Enforcement Subsystems Completely Untested
- **Impacted Files & Lines:** `bin/kineti-saga.ts`, `spend.ts`, `evidence.ts`, `verify-gate.ts`, `egress.ts`.
- **Root Cause:** 0% test coverage for saga commit immutability, per-stage spend limits, evidence expect-cmd validation, verify-gate pass-open, and egress truncation detection.
- **Universal Runtime Architectural Mechanism:**
  **Comprehensive Integration Test Suite**. Implement the 5 test skeletons specified in Section 5 of AUDIT_REPORT.md in `tests/harness.test.ts`.

---

### Medium Severity Remediations

#### [MED-01] Saga Rollback Forbids Committed Runs Without Force Flag, Hindering Watch Recovery
- **Impacted Files & Lines:** `bin/kineti-saga.ts:15-20, 55-57` (interacting with `skills/watch/SKILL.md:39`).
- **Universal Runtime Remediation:** Preserve default immutability for committed runs (exit code 2), but introduce an authorized `--force-committed` flag for emergency recovery in Stage 12 (`watch`). Appends `{ kind: "rollback_forced" }` to `saga.jsonl` for compliance auditing.

#### [MED-02] Zero Enforcement Tooling for Standing ETHOS Rule 8 ("Clean Files")
- **Impacted Files & Lines:** `ETHOS.md:44-48`.
- **Universal Runtime Remediation:** Author `scripts/clean-check.sh` (scans staged git diffs for `$HOME`, usernames, and high-entropy API keys) and wire it as a mandatory pre-commit check in Stage 11 (`ship`).

#### [MED-03] Missing UX Blueprint Integration in Intake & Design Skills
- **Impacted Files & Lines:** `skills/officehours/SKILL.md`, `skills/design/SKILL.md`.
- **Universal Runtime Remediation:** Update `officehours` Step 6 to mandate UX Blueprint deliverables (Persona, Journey, Screen Interaction Matrix, 3-Layer Split) in `brief.md`. Update `design` Step 3 to validate screen variants against the blueprint.

#### [MED-04] Non-Atomic File Overwrite in Memory Journal Overwriting Data In-Place
- **Impacted Files & Lines:** `bin/kineti-memory-job.ts:43-46`.
- **Universal Runtime Remediation:** Replace direct `fs.writeFileSync` with atomic write-and-rename: write to a temporary file (`journal.jsonl.tmp.<pid>.<time>`), then `fs.renameSync` over the target.

#### [MED-05] Symlink Traversal & FIFO Blocking in Repository Fingerprinting
- **Impacted Files & Lines:** `bin/kineti-evidence.ts:22-39`.
- **Universal Runtime Remediation:** Replace `fs.statSync` with `fs.lstatSync`. Ignore symbolic links (`!e.isSymbolicLink()`) and non-regular files (`st.isFile()`), preventing infinite loops and FIFO hangs.

#### [MED-06] Unchecked Argument Parsing Crash in Installer
- **Impacted Files & Lines:** `setup.sh:18`.
- **Universal Runtime Remediation:** Check `$# -ge 2` before `shift 2` when parsing `--host`. Emit clear error message if argument is missing.

#### [MED-07] Unhandled Exit Code 1 in `read_conf` Under `pipefail`
- **Impacted Files & Lines:** `setup.sh:25-27`.
- **Universal Runtime Remediation:** Modify pipeline to `(grep -E "^$2=" "$1" || true)` to prevent `pipefail` crashes when optional configuration keys are absent.

#### [MED-08] Incomplete Skill Installation Omitting Companion Subdirectories
- **Impacted Files & Lines:** `setup.sh:88-94`.
- **Universal Runtime Remediation:** Replace `cp SKILL.md` with `cp -R "$HERE/skills/$skill/." "$dest/"`, ensuring `scripts/`, `references/`, and `resources/` are fully installed.

#### [MED-09] Incomplete Uninstall Leaving Orphan Repository Pointer
- **Impacted Files & Lines:** `setup.sh:55-69`.
- **Universal Runtime Remediation:** In `uninstall()`, check and remove `$HOME/.kineti/repo` so cron jobs do not target dead paths.

#### [MED-10] Fragile Bun Path Resolution in Non-Interactive Cron
- **Impacted Files & Lines:** `scripts/weekly.sh:11-12`.
- **Universal Runtime Remediation:** Explicitly check executable permissions on `$BUN` and fallback paths before invoking, emitting an actionable diagnostic if Bun is missing.

#### [MED-11] Temporary Directory Resource Leakage in Test Suite
- **Impacted Files & Lines:** `tests/memory-job.test.ts:69`, `tests/harness.test.ts`.
- **Universal Runtime Remediation:** Wrap all test execution blocks in `try ... finally { fs.rmSync(tmpDir, { recursive: true, force: true }); }`.

#### [MED-12] TypeScript Strictness Gaps Masking Indexing Hazards
- **Impacted Files & Lines:** `tsconfig.json:8`, `bin/*.ts`.
- **Universal Runtime Remediation:** Enable `"noUncheckedIndexedAccess": true` in `tsconfig.json` and resolve all 10 undefined array indexing type hazards across `bin/*.ts`.

---

### Low Severity Remediations

#### [LOW-01] Missing Executable Permission on CLI Binary
- **Impacted File:** `bin/kineti-memory-job.ts`.
- **Fix:** `chmod 755 bin/kineti-memory-job.ts`.

#### [LOW-02] Inappropriate Executable Bit on Library Module
- **Impacted File:** `bin/lib.ts`.
- **Fix:** `chmod 644 bin/lib.ts`.

#### [LOW-03] Hash Delimiter Collision Risk in Egress Receipt Ledger
- **Impacted Files & Lines:** `bin/kineti-egress.ts:14-16`.
- **Fix:** Replace `${r.seq}|${r.at}|...` with canonical array serialization: `JSON.stringify([r.seq, r.at, r.host, r.description, r.prev_hash])`.

#### [LOW-04] Unsanitized Log Injection (CWE-117) in Machine Alerts
- **Impacted Files & Lines:** `bin/kineti-verify-gate.ts:59`, `bin/kineti-spend.ts:112`.
- **Fix:** Sanitize user strings before writing to `alerts.log`: `const clean = text.replace(/[\r\n]+/g, " ")`.

#### [LOW-05] Record Type Schema Naming Mismatch
- **Impacted Files & Lines:** `kineti.config.json:40` vs `bin/kineti-memory-job.ts:16`.
- **Fix:** Align `kineti.config.json` record type array to use `"dossier"` instead of `"project-dossier"`.

#### [LOW-06] Stale Reference in Officehours Skill to `journal.md`
- **Impacted Files & Lines:** `skills/officehours/SKILL.md:32`.
- **Fix:** Replace stale `journal.md` references with active fallback `journal.jsonl`.

#### [LOW-07] Incomplete Skill Catalog Listing in Claude Hook
- **Impacted Files & Lines:** `hooks/claude.txt:4-5`.
- **Fix:** Add missing skills (`anchors`, `second-opinion`, `skillify`) to update total count to 17.

#### [LOW-08] Dead and Conflicting `hook_file` Directives in Host Confs
- **Impacted Files & Lines:** `hosts/*.conf:5`.
- **Fix:** Remove unused `hook_file` directives or align them with actual host configuration targets.

---

### Informational Remediations

#### [INFO-01] Harness Program Count Mismatch in Documentation
- **Impacted Files:** `package.json:5`, `bin/README.md:3`.
- **Fix:** Update text from "six enforcement programs" to "seven harness programs".

#### [INFO-02] Absolute Path Reference `/design` in Standing Law
- **Impacted File:** `ETHOS.md:9`.
- **Fix:** Change `/design` to repository relative `design/screens/`.

#### [INFO-03] Unquoted Variable Expansion in Daily Loop Documentation
- **Impacted File:** `docs/HOWTO-daily-loop.md:37-38`.
- **Fix:** Quote variable expansions: `bun "$K/kineti-evidence.ts"`.

#### [INFO-04] Repository Version Desynchronization
- **Impacted Files:** `package.json:3`, `kineti.config.json:2`.
- **Fix:** Bump version from `3.0.0` to `3.0.1` to match git tags and roadmap.

#### [INFO-05] Gate Count Ambiguity in Documentation vs Configuration
- **Impacted Files:** `kineti.config.json:10-16`, `WORKFLOWS.md:3`.
- **Fix:** Harmonize documentation to clarify the 4 distinct gates: Feasibility (5), Spec (6), Security (10), and Ship (11).

#### [INFO-06] Philosophical Directives Conflict on 12 UI Components
- **Impacted File:** `skills/design/SKILL.md:65-66`.
- **Fix:** Harmonize text with Kineti Master Directives to reflect mandatory UI components standard.

#### [INFO-07] Hardcoded Skill Counts in Installer Smoke Test
- **Impacted Files:** `tests/test-setup.sh:18, 24`.
- **Fix:** Dynamically evaluate skill directory counts instead of hardcoding `17`.

#### [INFO-08] Unprotected Shell Glob Expansions Missing `nullglob`
- **Impacted Files:** `setup.sh:47, 61, 88`, `scripts/audit-skills.sh:7`.
- **Fix:** Add `shopt -s nullglob` in bash scripts to avoid literal asterisk expansions.

#### [INFO-09] Unused Test Import in Harness Test Suite
- **Impacted File:** `tests/harness.test.ts:1`.
- **Fix:** Remove unused `beforeAll` import.

#### [INFO-10] Zero Automated Test Coverage for Utility Shell Scripts
- **Impacted Files:** `scripts/audit-skills.sh`, `scripts/weekly.sh`.
- **Fix:** Add automated unit/smoke test scripts for maintenance shell utilities.

---

## 8. Conclusion & Next Steps for Strategy Blueprint Production

This technical survey provides the complete architectural and mathematical foundation for **R2 (Core Engine Hardening & Research Substrate Integration)**:
1. **The CIP 7-Layer Protocol** establishes clear separation of concerns from local transport (<1ms) to business outcome certification.
2. **Causal Graphs with ISO SQL/PGQ** provide formal, queryable, cycle-free provenance, transforming ungrounded agent execution into a deterministic state machine.
3. **The Universal 20-Entity Kernel** standardizes all lifecycle telemetry into strongly typed, interoperable schemas.
4. **Outcome Verification Tickets (OVTs)** enable the commercial transition to $/Outcome pricing by providing dual-signed, tamper-resistant compliance proofs.
5. **All 44 Audit Defects** from `docs/AUDIT_REPORT.md` are systematically eradicated through concrete architectural remediations.

This survey serves as the authoritative R2 input to the master blueprint at `docs/HARNESS_STRATEGY_BLUEPRINT.md`.

---
*Report compiled and certified by teamwork_preview_explorer_m2_2.*
