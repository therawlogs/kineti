---
name: /design
description: Compounded Build & Audit Gate. Runs Stage 4 (Visual Validation) and Stage 5 (Testing Strategy), triggers image mocks, applies Smolagents Code-First Actions and Grok-Build Autonomic Patching, and executes PII/Spend security checks.
---

# Skill: /design (Checkpoint 3 - Stage 4 & Stage 5 Build Gate)

## When to Use
Use immediately after technical and design planning (`/autoplan`) completes.

## How to Use
Enter `/design` in the active workspace.

## Sequencing
- **Phase**: `03_build_and_audit`
- **Step**: 0 (Orchestrator Gate)
- **Pre-requisite**: Approved Stage 2/3 planning from `/autoplan`.
- **Downstream Blockers**: Blocks `/ship` release gate.

## Protocol & Actions
- **Instructions**:
  When this command is run, you **MUST** follow this sequence using pure Plain English:

  ### Step 1: Assert Sequence & Load State
  1. Verify `.sprint_state.json` exists in `.northstar/`.
  2. Verify `last_completed_gate` equals `"autoplan"`.
  3. Load cumulative causality graph from `.northstar/graphs/cumulative_causality.md`.

  ### Step 2: Execute Stage 4 - Domain Visual Research & Bespoke Design System Selection (User Sovereignty Gate)
  1. **Perform Domain Visual Research**: Research top production-grade applications and industry standards for the project's specific domain (e.g., Stripe/Linear/Vercel for Dev Tools & SaaS, Bloomberg for Finance, Apple/Airbnb for Consumer). Generic canned presets (glassmorphic, dark mode, analog) are STRICTLY PROHIBITED.
  2. **Formulate 3 Bespoke Visual Directions**: Develop 3 domain-tailored visual design systems specifying typography pairings (Google Fonts like Outfit, Inter, JetBrains Mono), spatial density, custom color palettes (HSL), component structures, and micro-interactions.
  3. **Render Screenshot Previews**: Call `generate_image` to generate high-fidelity UI previews for all 3 visual directions.
  4. **Present Directions & HARD PAUSE**: Present the research findings, visual system options, and generated screenshots to the user, then **STOP EXECUTION IMMEDIATELY**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the 3 bespoke domain visual directions and screenshot previews to the user, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from auto-selecting a design theme or writing HTML/CSS code before the user explicitly selects a design system.

  ### Step 3: Execute Stage 5 - Testing Strategy & Autonomic Patching (After User Selects Design)
  Once the user selects their preferred design direction, prompt the user with Stage 5 testing questions and **PAUSE FOR INPUT**:
  * **Q1. Testing Strategy**: Unit+Integration CI / Playwright E2E / Manual QA / Fix Forward
  * **Q2. Device Matrix**: Desktop Chrome / Chrome+Safari Desktop+Mobile / Full Cross-Browser / N/A
  * **Q3. Test Data Strategy ($Q = C \times A \times T$)**: Deterministic Fixtures / Scrubbed Prod Data / Synthetic Mocks
  * **Q4. Private Data Removal**: Two-way PII Redaction Middleware / Static Dependency Audit / None


  ### Step 4: Execute Build & Audit Sub-Skills (Smolagents Code-First Actions)
  1. **Run `/design_html`**: Execute Smolagents Code-First actions to generate Pretext-native HTML/CSS files. Call `generate_image` tool to render 3 screenshot mockups in chat. Compile HTML layout on disk with chosen theme, `data-agent-action` tags, and unique IDs.
  2. **Run `/compress_context`**: Strip implementation bodies (`{ ... }`) for dense AST Skeleton representation.
  3. **Run `/structure_lock`**: Run shadow SQLite in-memory DB tests and lock JSON interface schemas.
  4. **Run `/review`**: Execute static code quality audit.
  5. **Run `/qa` (Grok-Build Autonomic Patching)**: Run tests. If a test fails, catch the error trace, write an autonomic fix patch, apply it, and re-run tests until the build passes cleanly.
  6. **Run `/cso`**: Apply Governance Interception Middleware (PII Egress Redaction) and Financial Spending Circuit Breakers.

  ### Step 5: Background Engine Execution (Silent Sub-Agent Threads)
  1. Sub-agent background threads generate `.northstar/graphs/stage_4_causality.md` and `stage_5_causality.md`.
  2. Update `.northstar/graphs/cumulative_causality.md` silently.
  3. Save transcripts and update `.northstar/manifest.json` silently without dumping raw file text in chat.

  ### Step 6: Surface Checkpoint 3 & Release Outcome Preview (User Decision Gate)
  Surface ONLY the executive build summary, release outcome preview, and next choices to the user, then **HALT EXECUTION IMMEDIATELY**:

  ```
  === Checkpoint 3: /design Build Summary ===
  • Understood & Built Components:
    - Selected Visual System: [Bespoke domain design system]
    - Code Components: Production HTML/CSS and Smolagents actions compiled.
    - Autonomic Test Validation: Grok-Build error patch loops verified (0 open failures).

  • Outcome Preview if Proceeding to /ship (Stage 6 & 7 Release):
    - Financial Spend Cap: Meta AI spend circuit breaker ($50.00 USD cap) will be locked.
    - GitOps Deployment: Automated deploy script will run with registered Saga LIFO Rollback stack.
    - Production Telemetry: Canary OpenTelemetry monitoring & MCP tools.json will be exported.

  • Next Action Options:
    1. Proceed to /ship (Release Verification & Production Deploy)
    2. Request visual component or layout refinements
  ```

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the Checkpoint 3 Outcome Preview above and STOP YOUR TURN IMMEDIATELY. Do NOT run downstream skills until the user explicitly selects an option.