# Kineti OS — Skill Workflow Guide

This document defines the exact sequential workflows for using Kineti OS skills across three primary development modes: **Greenfield Projects**, **Brownfield Integration**, and **Adding New Features**.

---

## The 11 Core Skills Overview

| Skill | Category | Primary Purpose | Gate Behavior |
| :--- | :--- | :--- | :--- |
| `/officehours` | Pipeline | Discovery & Strategy Gate (5-Whys, business metrics, veto holder) | Interactive Prompts |
| `/spec` | Pipeline | Data contracts & functional specification (`spec.md`) | **HARD PAUSE (Schema Gate)** |
| `/autoplan` | Pipeline | Architecture & infrastructure planning (DB, Auth, Tenancy, UI alignment) | Interactive Prompts |
| `/design` | Pipeline | Domain visual research, 3 Visual Archetypes, 12 UI components & code build | Interactive Prompts |
| `/ship` | Pipeline | Cost architecture, infrastructure targets, deployment & release | **HARD PAUSE (Deploy Gate)** |
| `/design_consultation` | Support | Standalone visual design token system establishment | Interactive Prompts |
| `/browse` | Support | Headless UI & layout verification with screenshots | Automated Utility |
| `/qa` | Support | Multi-viewport test suite execution with 5-attempt automated fix loop | Automated Utility |
| `/review` | Support | Static code quality, type safety, modularity & accessibility audit | Automated Utility |
| `/cso` | Support | Security audit (Spend bounds, PII redaction, OWASP Top 10) | Automated Utility |
| `/investigate` | Support | Emergency 5-Whys debugging loop for runtime errors or broken tests | Automated Utility |

---

## Workflow 1: Greenfield Projects (Building from Scratch)

Use this workflow when starting a brand new product or service.

```
/officehours  ──>  /spec  ──>  /autoplan  ──>  /design  ──>  /ship
    (Discover)     (Contract)   (Architect)    (Build UI)    (Release)
```

### Step-by-Step Execution:

1. **Step 1: Discovery & Validation (`/officehours`)**
   * Run `/officehours` in your empty project directory.
   * Answer Stage 0 questions (Business Model, Domain, Greenfield selection, Risk level, Timeline).
   * Answer Stage 1 questions (5-Whys, Single Main Value Action, Status Quo Cost, Target Metric, Veto Holder, Counter-Arguments).
   * *Output:* Initializes `.northstar/` private memory (`dialogue/`, `gaps/`, `decisions/`, `templates/`, `mocks/`).

2. **Step 2: Functional Specification & Schema Gate (`/spec`)**
   * Run `/spec` to execute the Data Quality Audit ($Q = C \times A \times T$) and construct the Data-Routing Map.
   * Generates field schemas, API endpoint signatures, typed request/response contracts, and error handling rules.
   * *Output:* Writes public `<project_root>/spec.md`.
   * **HARD PAUSE:** Review `spec.md` and give explicit user approval before continuing.

3. **Step 3: Architecture & UI Stack Selection (`/autoplan`)**
   * Run `/autoplan` to configure system boundaries (Latency targets, Peak concurrency, Failover plans).
   * Evaluate the 7 Enterprise Domains: Database (relational/vector), Multi-tenancy (`tenant_id`), Auth (SSO/OAuth), Data Pipelines, Data Integrity, Secret Management, and API Contract Strategy.
   * Align with the **Master Design System Standard** (`shadcn/ui` + `Radix UI` + `Lucide` + `Motion`).
   * *Output:* Saves architecture plan to `.northstar/decisions/architecture.md`.

4. **Step 4: Design Audit & Code Build (`/design`)**
   * Run `/design` to execute domain visual research and map the project to one of the **3 Master Visual Archetypes**:
     * *Archetype A:* Modern Technical SaaS (Linear/Vercel style — Dark Zinc, Inter + JetBrains Mono).
     * *Archetype B:* High-Trust Enterprise & Fintech (Stripe/Mercury style — Slate Pearl, Navy, Plus Jakarta Sans + Fira Code).
     * *Archetype C:* Premium Editorial & Consumer (Notion/Arc style — Warm Alabaster, Space Grotesk + Source Sans 3).
   * Renders screenshot previews via `generate_image` directly to `<project_root>/design/screens/`.
   * Select a direction and build UI components in `<project_root>/src/components/ui/` using the **12 Repeat-and-Rinse Components**.
   * Automatically invokes sub-skills: `/review` (code audit), `/qa` (viewport testing), `/cso` (security audit).

5. **Step 5: Cost Architecture & Release (`/ship`)**
   * Run `/ship` to define Unit Cost Ceilings, Spend Circuit Breakers ($50 cap), and target hosting infrastructure (Edge, Dedicated Compute, or Self-Hosted).
   * **HARD PAUSE:** Confirm deployment target and rollback plan.
   * Once approved, executes release pipeline: `/qa` → `/cso` → Deployment script → `/browse` (live URL verification).
   * Marks sprint COMPLETED in `.northstar/sprint_state.json`.

---

## Workflow 2: Brownfield Integration (Existing Codebase)

Use this workflow when integrating Kineti OS into an existing codebase, performing refactoring, or modernizing an enterprise application.

```
/officehours (Brownfield) ──> /investigate (If Buggy) ──> /spec ──> /autoplan ──> /design_consultation ──> /design ──> /ship
```

### Step-by-Step Execution:

1. **Step 1: Codebase Scoping & Constraints (`/officehours`)**
   * Run `/officehours` and explicitly select **Brownfield Execution** in Stage 0.
   * Specify:
     * Path to the existing codebase.
     * What the current code does.
     * Architectural boundaries and files that **MUST NOT BE CHANGED**.
   * Define the Veto Holder and safety constraints.

2. **Step 2: Emergency Audit / Triage (Optional: `/investigate`)**
   * If the brownfield codebase has active bugs or failing build scripts, run `/investigate`.
   * Captures exact error traces, performs 5-Whys root cause analysis, and applies minimal isolated patches without altering unbroken modules.

3. **Step 3: Interface & Data Contract Lock (`/spec`)**
   * Run `/spec` to define how new or modified features interface with the legacy system.
   * Define strict input/output data schemas that wrap around existing database models or external enterprise APIs.
   * **HARD PAUSE:** Verify that `spec.md` does not break legacy data schemas.

4. **Step 4: Architectural Integration (`/autoplan`)**
   * Evaluate multi-tenancy (`tenant_id` scoping), secret management, and database migrations.
   * Ensure new API routes compile against existing backend layers.

5. **Step 5: Visual System Alignment (`/design_consultation` or `/design`)**
   * If modernizing the visual layer, run `/design_consultation` first to audit the existing UI, establish HSL design tokens, and write them to `<project_root>/index.css`.
   * Run `/design` to build modern `shadcn/ui` + `Radix` components that safely wrap or replace legacy UI screens. Screen mockups are saved to `<project_root>/design/screens/`.

6. **Step 6: Release & Verification (`/ship`)**
   * Run `/ship` to execute `/qa` (multi-viewport testing) and `/cso` (PII and OWASP security audit).
   * Deploy the updated build and run `/browse` to verify live integration.

---

## Workflow 3: Adding New Features (Iterating on an Existing App)

Use this workflow when extending an established project that already has `ETHOS.md`, `spec.md`, and an active Kineti OS environment.

```
/officehours (Feature Scope) ──> /spec (Update) ──> /design (Build Feature) ──> /ship (Deploy Feature)
```

### Step-by-Step Execution:

1. **Step 1: Feature Intent Discovery (`/officehours`)**
   * Run `/officehours "feature description"`.
   * Focus discovery on Stage 1: Single Main Value Action of the new feature, business impact, and veto concerns.

2. **Step 2: Update Data Contract (`/spec`)**
   * Run `/spec` to add new fields, endpoints, or contracts to the existing `<project_root>/spec.md`.
   * **HARD PAUSE:** Approve the updated specification.

3. **Step 3: Build & Test Feature (`/design`)**
   * Run `/design` to generate feature screen previews into `<project_root>/design/screens/`.
   * Build new UI components or backend endpoints adhering to the project's established Archetype and design tokens.
   * Runs automated verification: `/review` → `/qa` → `/cso`.

4. **Step 4: Release Feature (`/ship`)**
   * Run `/ship` to run end-to-end verification, execute deployment, and close the feature sprint.

---

## Quick Reference: Utility & Emergency Skills

* **Need to establish design tokens without building code?** → Run `/design_consultation`.
* **Did a test fail or code throw a runtime error?** → Run `/investigate`. It will perform 5-Whys diagnosis and apply a targeted fix.
* **Want to visually verify a live local/staging URL?** → Run `/browse`. Saves screenshots to `<project_root>/design/screenshots/`.
* **Need a full multi-viewport QA pass with auto-fix?** → Run `/qa`. Executes up to 5 automated fix loops on failing tests.
* **Need a security audit before release?** → Run `/cso`. Audits spend bounds, PII egress, and OWASP Top 10 vulnerabilities.
