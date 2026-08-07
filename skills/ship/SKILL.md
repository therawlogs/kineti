---
name: /ship
description: Compounded Verification, QA & Release Gate. Runs Stage 6 (Cost Architecture) and Stage 7 (Deployment & Ops), enforces 100% Boil-the-Ocean test completeness, LangGraph Saga LIFO rollbacks, Meta AI spend circuit breakers, and exports MCP tools.json.
---

# Skill: /ship (Checkpoint 4 — CSO & Infrastructure SRE Lead Persona)

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

  ### Step 2: CSO & SRE Infrastructure, Security & Deployment Target Synthesis
  Adopt the **Chief Security Officer (CSO) & Infrastructure SRE Lead** persona. Conduct live research into hosting, security, and observability options specific to the project context.

  1. **Audit Causality Map, Live Research & Enterprise Domain Evaluation**: Read `.northstar/graphs/cumulative_causality.md` and perform live web research to evaluate hosting infrastructure, security compliance, and observability options against the 12 Enterprise Choice Domains.
  2. **Synthesize 3 Infrastructure Options**:
     * **Option A: Multi-Cloud Vercel / Cloudflare Edge**: Zero-config edge deployment, global CDN, automated SSL, serverless function scaling. Best for consumer B2C apps and marketing sites.
     * **Option B: Dedicated GPU Server (RunPod / Lambda / Modal)**: Custom container runtime for local LLM / VRAM inference and heavy compute tasks under 50ms SLA. Best for AI/ML-heavy enterprise tools.
     * **Option C: Self-Hosted / Local Gateway**: Complete data privacy, local network deployment, zero third-party egress cost. Best for regulated industries (healthcare, finance, government).
  3. **Evaluate CI/CD Pipeline & Quality Gates**: Configure GitHub Actions / GitLab CI with automated linting, TypeScript type checking, unit/integration tests, Playwright E2E verification, Snyk/SonarQube vulnerability scans, and Blue-Green/Canary deployment patterns.
  4. **Configure Secret Vault & Environment Security**: Evaluate Infisical, HashiCorp Vault, or AWS Secrets Manager. Enforce Zod boot validation and GitLeaks secret leak prevention.
  5. **Execute OWASP Security Audit**: Run OWASP Top 10 for Web and OWASP Top 10 for AI Agents (ASI-01 to ASI-05) compliance checks. Verify PII egress redaction middleware and API rate limiting.
  6. **Configure OpenTelemetry Agent Observability (Directive 26)**: Ensure all agent runs and LLM calls emit OpenTelemetry traces with `feature_id`, `prompt_tokens`, `completion_tokens`, `cost_usd` metadata.
  7. **Tailor Financial Spend Caps**: Configure unit cost ceilings, spend circuit breakers ($50.00 USD default), and Saga LIFO rollback triggers based on chosen hosting target.
  8. **Verify Cryptographic Approval Token (Directive 25)**: For production deployments and spend cap overrides, verify that a signed `approval_token` exists before unblocking.
  9. **Evaluate Agent Safety & Guardrails (If Persona B Active)**: Verify content filtering, hallucination bounds, tool-use sandboxing, and adversarial input defense (prompt injection) via `/agent_safety`.
  10. **Token Unit Economics & Model Routing (If Persona B Active)**: Evaluate model routing (cheap vs expensive models), prompt caching, and cost-per-query margin analysis.
  11. **Agent Evaluation & Quality Benchmark Gate (If Persona B Active)**: Run systematic agent evaluation (accuracy benchmarks, tool-use correctness, regression detection) via `/agent_eval` before unblocking deployment.
  12. **AI Regulatory Compliance (If Persona B Active)**: Verify EU AI Act risk classification, SOC 2, and domain-specific regulations.
  13. **Model Integration & Serving Infrastructure (If Persona D Active)**: Configure vLLM, TensorRT-LLM, continuous batching, and inference latency optimization.

  ### Step 3: Present Deployment Options & HARD PAUSE
  Present the CSO/SRE infrastructure research, security audit results, CI/CD pipeline configuration, secret vault options, OpenTelemetry observability setup, spend caps, and Saga LIFO rollback plans to the user, then **STOP EXECUTION IMMEDIATELY**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the CSO/SRE infrastructure research, security compliance audit, CI/CD quality gates, secret vault configuration, observability setup, spend caps, and deployment plans to the user, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from triggering GitOps deployment, running test suites, or marking the sprint complete before the user explicitly approves the release configuration.


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
  === Checkpoint 4: /ship Complete (CSO & SRE Verified) ===
  - Sprint state successfully marked COMPLETED.
  - "Boil the Ocean" 100% test pass rate verified across desktop and mobile viewports.
  - LangGraph Saga LIFO Rollback stack registered with idempotency keys for auto-undo on deploy failures.
  - Cryptographic approval_token verified for production deployment.
  - OWASP AI Agent Security (ASI-01 to ASI-05) compliance audit passed.
  - AST Tenant Leak Analyzer: Zero cross-tenant query leak vectors detected.
  - OpenTelemetry traces emitting feature_id + token cost attribution.
  - Meta AI Financial Spend Circuit Breakers active.
  - MCP Tool Schema (tools.json / OpenAPI) compiled & exported.
  
  === Financial & Performance Return Summary ===
  - Execution Time Spent: [elapsed_time]
  - Total AI Tokens Consumed: [total_tokens]
  - Total Sprint Cost: $[calculated_token_cost]
  - Estimated Gross Profit Margin: [gross_margin_pct]%
  - Final Pass Criteria Verified: [Stage 7 success criteria verified]
  ```