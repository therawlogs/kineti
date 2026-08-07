---
name: /autoplan
description: Compounded Technical Planning & Design Gate. Runs Stage 2 (System Boundaries & Workflow Mechanics) and Stage 3 (Interaction Surface & Experience Mechanics), applies AST Context Shrinking, PageRank File Scoring, and Boil the Ocean completeness inside the Shippable Wedge.
---

# Skill: /autoplan (Checkpoint 2 — CTO & Principal Architect Persona)

## When to Use
Use immediately after Stage 1 strategy validation completes.

## How to Use
Enter `/autoplan` in the active workspace.

## Sequencing
- **Phase**: `02_technical_planning`
- **Step**: 0 (Orchestrator Gate)
- **Pre-requisite**: Approved Stage 0/1 discovery from `/officehours` and `/spec`.
- **Downstream Blockers**: Blocks `/design` build gate.

## Protocol & Actions
- **Instructions**:
  When this command is run, you **MUST** follow this sequence using pure Plain English:

  ### Step 1: Assert Sequence & Load Decision Memory
  1. Verify `.sprint_state.json` exists in `.northstar/`.
  2. Verify `last_completed_gate` equals `"officehours"`.
  3. Load cumulative causality graph from `.northstar/graphs/cumulative_causality.md`.

  ### Step 2: AST Context Shrinking & PageRank File Scoring
  1. Parse target codebase files down to their **AST Skeleton** (Types, Interfaces, Structs, Signatures only) — strip function bodies (`{ ... }`) to save 85–90% context tokens and eliminate attention decay.
  2. Run **PageRank Criticality Indexing** on the file dependency graph. Automatically identify and print the **#1 most critical keystone file**.

  ### Step 3: Enforce "Boil the Ocean" Completeness Inside the Wedge
  Set the Minimal Shippable Wedge for rollout scope. Inside that wedge, enforce 100% completeness:
  * 100% test coverage for target modules.
  * Full feature implementation with zero skipped shortcuts.
  * Complete error handling paths and edge case checks.

  ### Step 4: Execute Stage 2 & Stage 3 - CTO & Architect Deep Dive (User Selection Gate)
  Adopt the **Chief Technology Officer (CTO) & Principal Architect** persona. Conduct live research into architecture patterns, database strategies, and infrastructure options specific to the project's domain before presenting choices.

  **Stage 2 Boundaries, Enterprise Domains & Edge Case Matrix**:
  * **Q1. Database & Persistence Strategy**: Based on domain research, present 3 database architecture options evaluating relational (PostgreSQL + Prisma/Drizzle), document (MongoDB/DynamoDB), vector (pgvector/Qdrant), and caching (Redis/Upstash) layers. Include connection pooling, read replicas, and automated migration strategies.
  * **Q2. Multi-Tenancy & Data Scoping**: Evaluate tenant isolation models (PostgreSQL Row-Level Security, schema-per-tenant, DB-per-tenant) with automatic middleware `tenant_id` injection. Mandate AST Tenant Leak Analyzer integration in CI/CD.
  * **Q3. Authentication & Access Control Architecture**: Evaluate enterprise SSO/SAML (WorkOS/Keycloak/Auth0), consumer OAuth, session security (Argon2id hashing, JWT rotation, refresh token revocation), and RBAC/ABAC/ReBAC access control systems.
  * **Q4. Data Pipeline & Async Processing Strategy**: Evaluate job queues (BullMQ/Temporal/Kafka), ETL pipelines (Airbyte/dbt/Debezium CDC), and reliability patterns (exponential backoff, Dead Letter Queues, idempotent handlers).
  * **Q5. Data Integrity & Transactional Warranties**: Define Saga Pattern ACID transactions with LIFO undo handlers, idempotency keys (`root_goal_id + step_hash`) for external side-effects, strict foreign keys, soft-delete flags, and immutable audit event ledgers.
  * **Q6. Secret Management & Environment Configuration**: Evaluate secret vaults (Infisical/HashiCorp Vault/AWS Secrets Manager), Zod boot validation preventing startup on missing env vars, and GitLeaks/TruffleHog CI secret leak prevention.
  * **Q7. Latency Target & SLA**: Under 500ms (interactive) / Under 5s (quick) / Under 60s (async) / Batch
  * **Q8. Peak Concurrency & Volume**: Under 100/day / 100–10k/day / 10k–1M/day / High Concurrency
  * **Q9. Failover & Recovery Plan**: What happens when a dependency drops (fail-safe LIFO undo, circuit breaker, retry loop)?
  * **Q10. API Contract Strategy**: Define strict OpenAPI / TypeSafe RPC contracts (tRPC or Zod schema) that frontend agents must compile against. Consumer-driven contract testing (Pact / OpenAPI Spec Guard) is mandatory.
  * **Q11. Multi-Agent Orchestration (If Persona B Active)**: Evaluate supervisor → worker hierarchies, peer-to-peer agent collaboration, swarm execution, and agent handoff protocols.
  * **Q12. Production Agent Memory (If Persona B Active)**: Evaluate episodic memory (past conversations), procedural memory (learned tool patterns), working memory context management, and memory privacy boundaries via `/agent_memory`.
  * **Q13. Context Optimization & Data Flywheels (If Persona D Active)**: Evaluate RAG pipeline tuning, vector retrieval, and RLHF preference capture.

  **Stage 3 Experience & Design System Requirements**:
  * **Q1. Primary Interaction Surface**: Web browser / Phone app / CLI / API-only / Desktop
  * **Q2. User Habit Benchmark**: What existing tool pattern should this match (Stripe, Linear, Notion, Vercel)?
  * **Q3. Information Density**: Micro-density dashboard / Medium / Low / Minimal
  * **Q4. Non-Text Content Requirements**: Charts / Maps / Real-time feeds / Documents

  **Frontier AI Architecture (When Persona B or A+B is active)**:
  * **F1. Agent Orchestration Pattern**: Based on the multi-agent requirements, evaluate Supervisor → Worker, Peer-to-Peer, or Swarm patterns. Run `/agent-orchestration` as a sub-skill.
  * **F2. Agent Memory Architecture**: Based on memory requirements, evaluate episodic, procedural, and working memory systems. Run `/agent-memory` as a sub-skill.
  * **F3. Model Routing Strategy**: Define which models handle which agent tasks (cheap/fast for classification, expensive/capable for reasoning).
  * **F4. Agent-to-Agent API Contracts**: Define strict typed interfaces between agents in the orchestration graph (Zod schemas for handoff payloads).

  **AI/ML Optimization (When Persona D is active)**:
  * **M1. Context Window Strategy**: Evaluate AST Shrinking, document chunking, and summarization strategies for the project's data volume.
  * **M2. Vector Store Architecture**: Evaluate pgvector, Qdrant, Pinecone, or Weaviate for the project's retrieval requirements.
  * **M3. Embedding Model Selection**: Evaluate embedding models (OpenAI text-embedding-3, Cohere embed-v3, local models) for cost/quality tradeoff.

  *Dynamic Probes*: Generate 3–5 CTO-level technical deep-dive questions tagged `[DOMAIN-SPECIFIC]` based on the live research conducted.

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the CTO & Architect deep-dive questions, enterprise domain evaluations, and synthesized infrastructure options above and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from auto-generating answers for the user, simulating technical choices, writing transcript files, or running downstream skills before receiving the user's explicit response.

  ### Step 5: Background Engine Execution (Silent Sub-Agent Threads)
  1. Once the user responds, sub-agent threads execute `/plan_eng_review` (Hexagonal Ports, AST Skeletons, VRAM sizing) and `/plan_design_review` in the background.
  2. Generate the Edge Case & Failure Mode Matrix and data schema contracts silently in `.northstar/spec.md`.
  3. Run `causality_graph_builder.py` silently in background threads to generate `.northstar/graphs/stage_2_causality.md`, `stage_3_causality.md`, and update `cumulative_causality.md`.
  4. Save transcripts and update `.northstar/manifest.json` silently without dumping file content in chat.

  ### Step 6: Surface Checkpoint 2 & Build Outcome Preview (User Decision Gate)
  Surface ONLY the executive technical summary, build outcome preview, and next choices to the user, then **HALT EXECUTION IMMEDIATELY**:

  ```
  === Checkpoint 2: /autoplan Architecture Summary ===
  • Understood & Locked Technical Parameters:
    - Latency SLA & Concurrency: [Target SLA & concurrency limits]
    - Data Security & Egress Rules: [Scrubbed data perimeter]
    - Keystone Code File (PageRank): [#1 critical file identified]

  • Outcome Preview if Proceeding to /design (Stage 4 & 5 Build):
    - Domain Visual Research: 3 bespoke visual design systems & screenshot previews (`generate_image`) will be compiled.
    - Code-First Execution: Smolagents Python code blocks will construct components with zero JSON tool drag.
    - Autonomic Patching: Grok-Build automated error-fixing loops will run on test failures.

  • Next Action Options:
    1. Proceed to /design (Domain Design Research & Component Build)
    2. Adjust technical boundaries or concurrency rules
  ```

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the Checkpoint 2 Outcome Preview above and STOP YOUR TURN IMMEDIATELY. Do NOT run downstream skills until the user explicitly selects an option.