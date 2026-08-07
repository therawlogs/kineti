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

  ### Step 2: Infrastructure & Deployment Target Synthesis
  1. **Audit Causality Map & Live Research**: Read `.northstar/graphs/cumulative_causality.md` and perform live research to fetch optimal hosting infrastructure options.
  2. **Synthesize 3 Infrastructure Options**:
     * **Option A: Multi-Cloud Vercel / Cloudflare Edge**: Zero-config edge deployment, global CDN, automated SSL, serverless function scaling.
     * **Option B: Dedicated GPU Server (RunPod / Lambda / Modal)**: Custom container runtime for local LLM / VRAM inference and heavy compute tasks under 50ms SLA.
     * **Option C: Self-Hosted / Local Gateway**: Complete data privacy, local network deployment, zero third-party egress cost.
  3. **Tailor Financial Spend Caps**: Configure unit cost ceilings, spend circuit breakers ($50.00 USD default), and Saga LIFO rollback triggers based on chosen hosting target.

  ### Step 3: Present Deployment Options & HARD PAUSE
  Present the synthesized infrastructure options, spend caps, and Saga LIFO rollback plans to the user, then **STOP EXECUTION IMMEDIATELY**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the infrastructure options, spend caps, and deployment plans to the user, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from triggering GitOps deployment, running test suites, or marking the sprint complete before the user explicitly selects a hosting target.


  ### Step 4: Execute Release Sub-Skills & Verification (After User Approves)
  Once approved by the user, execute:

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