---
name: /ship
description: Compounded Verification, QA & Release Gate. Runs Stage 6 (Cost Architecture) and Stage 7 (Deployment & Ops), enforces 100% Boil-the-Ocean test completeness, LangGraph Saga LIFO rollbacks, Meta AI spend circuit breakers, and exports MCP tools.json.
---

# Skill: /ship (Checkpoint 4 - Plain English Release Gate)

## When to Use
Use immediately after the Build & Audit Gate (`/design`) completes.

## How to Use
Enter `/ship` in the active workspace.

## Sequencing
- **Phase**: `04_verification_release`
- **Step**: 4 (Release Gate - Final Step)
- **Pre-requisite**: Completed Build & Audit checks from `/design`.
- **Downstream Blockers**: None. This skill execution finalizes the active sprint loop.

## Protocol & Actions
- **Instructions**:
  When this command is run, you **MUST** follow this sequence using pure Plain English:

  ### Step 1: Assert Sequence & Load State
  1. Verify `.sprint_state.json` exists in `.northstar/`.
  2. Verify `last_completed_gate` equals `"design"`.
  3. Load cumulative causality graph from `.northstar/graphs/cumulative_causality.md`.

  ### Step 2: Execute Stage 6 - Cost Architecture & Financial Guardrails (Meta AI Pattern)
  Prompt the user with Stage 6 plain questions:
  * **Q1. Unit Cost Ceiling**: What is the maximum allowed cost per user action to maintain >80% Gross Margin?
  * **Q2. Auto Spend Circuit Breaker**: What API cost cap triggers an immediate auto-stop to prevent runaway bills?
  * **Q3. Outage Cost per Hour**: $0 / Under $100 / $100–$10k / Over $10k

  *Dynamic Questions*: Generate 2–3 plain cost probes tagged `[DOMAIN-SPECIFIC]`.

  ### Step 3: Execute Stage 7 - Deployment & Operations (LangGraph Saga Rollbacks)
  Prompt the user with Stage 7 plain questions:
  * **Q1. Target Server**: Cloud PaaS / Cloud IaaS / Kubernetes / Static Hosting / Bare Metal / Local
  * **Q2. Auto Undo Plan (Saga LIFO Stack)**: How do we automatically undo multi-step actions in reverse order if a deploy step fails midway?
  * **Q3. Outage Detection**: Error rate tracking (>1%) / Metric anomaly / User-reported / None
  * **Q4. Alert Contact**: Builder directly / On-call team / None / Automated self-healing
  * **Q5. Secret Keys Storage**: Environment variables at runtime / Secrets Manager / Local .env excluded / Hardcoded prototype
  * **Q6. Pass/Fail Criteria**: What binary, non-subjective test proves this sprint is 100% complete?

  *Dynamic Questions*: Generate 2–3 plain deployment probes tagged `[DOMAIN-SPECIFIC]`.

  ### Step 4: Execute Release Sub-Skills
  1. **Run `/qa`**: Playwright E2E multi-viewport tests across desktop and mobile screen sizes. Enforce 100% test pass rate ("Boil the Ocean" completeness check).
  2. **Run `/land_and_deploy`**: Trigger GitOps deployment script with registered LangGraph Saga LIFO Rollback stack.
  3. **Run `/canary`**: Setup latency tracking with OpenTelemetry log tracing headers.
  4. **Run `/benchmark`**: Core Web Vitals and response time benchmarks.
  5. **Run `/document_release`**: Update documentation files and sync sitemaps.
  6. **Run `/document_generate`**: Compile user guides, system diagrams, and export valid **MCP Tool Schema** (`tools.json` / OpenAPI).

  ### Step 5: Causality Graphs, Gap Protocols, & Sprint Closure
  1. Build Stage 6 & 7 causality graphs in SQL/PGQ format → `.northstar/graphs/stage_6_causality.md` and `.northstar/graphs/stage_7_causality.md`.
  2. Update `.northstar/graphs/cumulative_causality.md`.
  3. Run Context Gap Detection → `.northstar/gaps/stage_6_gaps.md` and `stage_7_gaps.md`.
  4. Save transcripts → `.northstar/dialogue/stage_6_cost_architecture.md` and `stage_7_deployment_ops.md`.
  5. Update `.northstar/manifest.json`.
  6. Update `.northstar/sprint_state.json`: set `last_completed_gate` to `"completed"`, `current_stage` to `"completed"`.

  Upon completion, print this plain summary:
  ```
  === Checkpoint 4: /ship Complete ===
  - Sprint state successfully marked COMPLETED.
  - "Boil the Ocean" 100% test pass rate verified across desktop and mobile viewports.
  - LangGraph Saga LIFO Rollback stack registered for auto-undo on deploy failures.
  - Meta AI Financial Spend Circuit Breakers active.
  - Simple AI Tool List (tools.json) compiled & exported.
  
  === Financial & Performance Return Summary ===
  - Execution Time Spent: [elapsed_time]
  - Total AI Tokens Consumed: [total_tokens]
  - Total Sprint Cost: $[calculated_token_cost]
  - Estimated Gross Profit Margin: [gross_margin_pct]%
  - Final Pass Criteria Verified: [Stage 7 success criteria verified]
  ```