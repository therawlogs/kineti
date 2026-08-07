---
name: /design
description: Compounded Build & Audit Gate. Runs Stage 4 (Visual Validation) and Stage 5 (Testing Strategy), triggers image mocks, applies Smolagents Code-First Actions and Grok-Build Autonomic Patching, and executes PII/Spend security checks.
---

# Skill: /design (Checkpoint 3 — CPO & Head of UX Design Persona)

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

  ### Step 2: Execute Stage 4 - CPO Domain Visual Research & Bespoke Design System Selection (User Sovereignty Gate)
  Adopt the **Chief Product Officer (CPO) & Head of UX Design** persona. Conduct thorough live research into industry-leading applications and UI patterns specific to the project's domain.

  1. **Execute Deep Domain Visual Research**: Search the web for production-grade applications and industry UI standards for the project's specific domain (e.g., Stripe/Linear/Vercel for DevTools & SaaS, Bloomberg/Robinhood for Finance, Apple/Airbnb for Consumer, Epic/Cerner for Healthcare). Evaluate from 4 perspectives:
     * **User POV**: Onboarding friction, click depth, cognitive load, accessibility (WCAG 2.1 AA).
     * **Data Density POV**: Information density requirements (high-density TanStack Table/AG Grid for enterprise dashboards vs minimal cards for consumer apps).
     * **Business POV**: Conversion optimization, feature discoverability, premium perceived quality.
     * **Workflow Velocity POV**: Keyboard shortcuts (Command+K palette), bulk actions, real-time collaboration signals.
  2. **Evaluate Component Library Fit**: Research and present component library recommendations (Radix Primitives + Tailwind/Shadcn UI, Mantine, Ant Design, or custom) with rationale specific to the project type.
  3. **Verify API Contract Compliance (Directive 28)**: Before generating any frontend code, verify that the backend spec has produced a strict OpenAPI / TypeSafe RPC contract (tRPC or Zod schema). The frontend agent must compile against these exact exported types.
  4. **Formulate 3 Bespoke Visual Directions**: Develop 3 domain-tailored visual design systems specifying typography pairings (Google Fonts like Outfit, Inter, JetBrains Mono), spatial density, custom color palettes (HSL), component structures, and micro-interactions. Generic canned presets (glassmorphic, dark mode, analog) are STRICTLY PROHIBITED.
  5. **Render Screenshot Previews**: Call `generate_image` to generate high-fidelity UI previews for all 3 visual directions.
  6. **Present Directions**: Present the CPO research findings, component library rationale, visual system options, and generated screenshots to the user, make a logical default choice if needed, and proceed to the next step.

  ### Step 3: Execute Stage 5 - Testing Strategy & Autonomic Patching
  Make a logical default choice for the preferred design direction, output the Stage 5 testing choices, and proceed using defaults:
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

  ### Step 6: Surface Checkpoint 3 & Release Outcome Preview
  Surface ONLY the executive build summary, release outcome preview, and next choices to the user, make a logical default choice if needed, and proceed to the next step or finish.

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