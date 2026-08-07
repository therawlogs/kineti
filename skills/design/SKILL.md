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

  ### Step 2: Execute Stage 4 - Visual & Interface Validation (User Sovereignty Gate)
  Prompt the user with Stage 4 plain questions (Skip if CLI/API-only):
  * **Q1. Theme Mark Choice**: Cyber Dark / Glassmorphic Light / Retro Monochrome — which visual theme establishes trust?
  * **Q2. Screen Automation Tags**: `data-agent-action` tags on interactive controls / Unique IDs / None
  * **Q3. Layout Screenshot Validation**: Render 3 mockup options via `generate_image` tool for explicit user approval before writing production code.

  *Dynamic Questions*: Generate 2–3 plain visual validation probes tagged `[DOMAIN-SPECIFIC]`.

  ### Step 3: Execute Stage 5 - Testing Strategy & Autonomic Patching (Grok-Build Pattern)
  Prompt the user with Stage 5 plain questions:
  * **Q1. Testing Strategy**: Unit+Integration CI / Playwright E2E / Manual QA / Fix Forward
  * **Q2. Device Matrix**: Desktop Chrome / Chrome+Safari Desktop+Mobile / Full Cross-Browser / N/A
  * **Q3. Test Data Strategy ($Q = C \times A \times T$)**: Deterministic Fixtures / Scrubbed Prod Data / Synthetic Mocks
  * **Q4. Private Data Removal**: Two-way PII Redaction Middleware / Static Dependency Audit / None

  *Dynamic Questions*: Generate 2–3 plain testing probes tagged `[DOMAIN-SPECIFIC]`.

  ### Step 4: Execute Build & Audit Sub-Skills (Smolagents Code-First Actions)
  1. **Run `/design_html`**: Execute Smolagents Code-First actions to generate Pretext-native HTML/CSS files. Call `generate_image` tool to render 3 screenshot mockups in chat. Compile HTML layout on disk with chosen theme, `data-agent-action` tags, and unique IDs.
  2. **Run `/compress_context`**: Strip implementation bodies (`{ ... }`) for dense AST Skeleton representation.
  3. **Run `/structure_lock`**: Run shadow SQLite in-memory DB tests and lock JSON interface schemas.
  4. **Run `/review`**: Execute static code quality audit.
  5. **Run `/qa` (Grok-Build Autonomic Patching)**: Run tests. If a test fails, catch the error trace, write an autonomic fix patch, apply it, and re-run tests until the build passes cleanly.
  6. **Run `/cso`**: Apply Governance Interception Middleware (PII Egress Redaction) and Financial Spending Circuit Breakers.

  ### Step 5: Causality Graphs, Gap Protocols, & Local Persistence
  1. Build Stage 4 & 5 causality graphs in SQL/PGQ format → `.northstar/graphs/stage_4_causality.md` and `.northstar/graphs/stage_5_causality.md`.
  2. Update `.northstar/graphs/cumulative_causality.md`.
  3. Run Context Gap Detection → `.northstar/gaps/stage_4_gaps.md` and `stage_5_gaps.md`.
  4. Save transcripts → `.northstar/dialogue/stage_4_design_presentation.md` and `stage_5_testing_strategy.md`.
  5. Update `.northstar/manifest.json` and `.northstar/sprint_state.json`.

  Upon completion, print this summary:
  ```
  === Checkpoint 3: /design Complete ===
  - User Sovereignty confirmed via visual screenshot previews (generate_image).
  - Smolagents Code-First Actions executed (zero JSON tool-call drag).
  - Grok-Build Autonomic Patching verified (automated error trace fix-loops active).
  - Stage 4 & Stage 5 complete (Visual Validation & Testing Strategy locked).
  - Cumulative Relational System Map updated (.northstar/graphs/cumulative_causality.md).
  - Next Recommended: Run /ship to verify, deploy, and close sprint loop.
  ```