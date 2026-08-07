---
name: /officehours
description: Compounded Product & Strategy Gate. Initializes local .northstar/ decision memory, executes Stage 0 (Domain Classification & Greenfield/Brownfield), 3-Layer Knowledge Search, Stage 1 (Problem Validation & 5-Whys), generates dynamic domain questions, builds causality graphs, and logs context gaps locally using pure Plain English.
---

# Skill: /officehours (Stage 0 & Stage 1 Discovery — CBO & Strategic Consultant Persona)

## When to Use
Use this command at the start of any new application, feature, or project within **The Kineti OS**.

## How to Use
Enter `/officehours` followed by a plain description of what you want to build or what problem you want to solve.

## Sequencing
- **Phase**: `01_product_strategy`
- **Step**: 1 (Primary entry gate)
- **Downstream Blockers**: Blocks all technical planning and implementation. Do not start coding before this gate completes.

## Protocol & Actions
- **Instructions**:
  When this command is run, you **MUST** follow these steps using pure Plain English and zero technical jargon:

  ### Step 1: Initialize Local Memory Folder & User Sovereignty Protocol
  1. Create a local folder at `<project_root>/.northstar/` with subfolders: `dialogue/`, `gaps/`, `graphs/`, `decisions/`, `templates/`, `mocks/`.
  2. Create `<project_root>/.northstar/.gitignore` containing `*` to block all memory files from being uploaded to public online repositories (like GitHub).
  3. Enforce **User Sovereignty (Iron Man Suit Philosophy)**: AI models (Gemini 3.6, GPT 5.6 Sol, Claude 5, Grok 4.6) recommend, Users decide. Always present recommendations with missed-context callouts and ask for user approval before locking decisions.

  ### Step 2: Socratic Memory Calibration & 3-Layer Knowledge Search
  Before proposing any build design:
  1. Load past sprint decisions from `.northstar/decisions/` and `.northstar/graphs/cumulative_causality.md` to seed Socratic Calibration memory (eliminates duplicate questions across sprints).
  2. Audit three layers of knowledge:
     * **Layer 1 (Tried & True)**: Battle-tested standard patterns deeply in distribution. Check standard solutions first.
     * **Layer 2 (New & Popular)**: Ecosystem trends — scrutinize blog post manias and hype.
     * **Layer 3 (First Principles)**: Original observations derived from reasoning about the specific problem at hand.
     * **The Eureka Moment**: Understand conventional approaches, apply first principles to their assumptions, and discover why conventional wisdom is wrong (zig while others zag).

  ### Step 3: Stage 0 - Dynamic Domain Research & Application Classification (CBO Persona)
  Adopt the **Chief Business Officer (CBO) & Strategic Consultant** persona. Conduct thorough live research into the feature's domain, industry, and application context before presenting any choices.

  1. **Execute Live Domain & Industry Research**: Search the web for current industry standards, best practices, competitor products, and technology trends relevant to the user's domain. Inspect `.northstar/graphs/cumulative_causality.md` to load past decisions. Static hardcoded questions are STRICTLY PROHIBITED.
  2. **Classify Application Type & Evaluate 12 Enterprise Choice Domains**: Based on research, classify the application and evaluate relevant enterprise domains:
     * **Enterprise Tool (B2B)**: Propose PostgreSQL + Prisma/Drizzle, SSO/SAML auth (WorkOS/Keycloak), RBAC/ReBAC access control, audit logging, PII redaction middleware, multi-tenant RLS isolation, Infisical/Vault secret management, BullMQ/Temporal async queues, and CI/CD via GitHub Actions.
     * **Consumer Application (B2C)**: Propose OAuth (Google/GitHub/Apple), customer databases (Supabase/Neon), responsive Radix/Shadcn UI components, Cloudflare Edge caching, Stripe billing integrations, and Vercel serverless deployment.
     * **Internal Tool / Single-User Workflow**: Propose API token auth, direct tool integrations, TanStack high-density data tables with AG Grid, automated background execution tasks, and self-hosted local gateway deployment.
  3. **Synthesize 3 Production Tech Stack Options**: For each classification, research and present 3 production-grade tech stack options (databases, ORMs, auth providers, component libraries, hosting targets) tailored for Human + Agent workflows. Include rationale for each.
  4. **Present Research-Backed Recommendations & Dynamic Probes**: Present the synthesized app classification, 3 recommended tech stacks, and 3–5 dynamic, context-aware domain probes. Then **HALT EXECUTION IMMEDIATELY and WAIT FOR USER RESPONSE**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the CBO research findings, domain probes, application classification, 12 enterprise domain evaluation, and synthesized stack choices to the user, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from auto-selecting choices, simulating user answers, writing transcript files, or running downstream skills before receiving explicit user input.

  ### Step 4: Stage 1 - Problem & Value Validation (CBO Persona Continued)
  Once the user responds to Stage 0, continue the **CBO & Strategic Consultant** persona. Present Stage 1 value probes tailored to the user's answers:
  * **Q1. The 5-Whys Root Cause**: Ask "why" 5 times to find the real physical cause. Why fix it right now?
  * **Q2. Single Main Value Action**: What is the single main action the user performs to get value?
  * **Q3. Status Quo Annual Cost**: What does the current broken process cost per year?
  * **Q4. Target Business Metric**: What exact financial target improves (lower hosting costs, fewer lost customers, higher sales)?
  * **Q5. Project Veto Holder**: Who can stop this project? What is their exact concern?
  * **Q6. Counter-Argument Refutation**: What is the strongest reason this project might fail, and how do we prove it wrong?

  **HALT AGAIN FOR USER INPUT.**


  ### Step 5: Background Engine Execution (Silent Sub-Agent Threads)
  1. Once the user provides answers, invoke background sub-agent threads to write transcripts silently to `.northstar/dialogue/stage_0_domain_classification.md` and `.northstar/dialogue/stage_1_problem_validation.md`.
  2. Run `causality_graph_builder.py` silently in the background to update `.northstar/graphs/` property graphs.
  3. Update `.northstar/manifest.json` and `.northstar/sprint_state.json` silently without dumping file text in chat.

  ### Step 6: Surface Checkpoint & Next-Stage Outcome Preview (User Decision Gate)
  Surface ONLY the executive summary, outcome preview, and next choices to the user, then **HALT EXECUTION IMMEDIATELY**:

  ```
  === Checkpoint 1: /officehours Discovery Summary ===
  • Understood & Locked Decisions:
    - Domain & Model: [Domain summary]
    - Root Cause (5-Whys): [Validated physical root cause]
    - Primary Target Metric: [Financial/operational metric target]

  • Outcome Preview if Proceeding to /autoplan (Stage 2 & 3):
    - System Boundaries: Latency SLA & Peak Concurrency bounds will be established.
    - Security Perimeter: PII and secret egress rules will be locked.
    - Design System Research: 3 bespoke domain visual directions & screenshot previews will be generated.

  • Next Action Options:
    1. Proceed to /autoplan (Technical & Visual Planning Gate)
    2. Proceed to /spec (Data-Routing Map & Contract Specification)
    3. Refine or adjust current strategy decisions
  ```

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the Checkpoint Outcome Preview above and STOP YOUR TURN IMMEDIATELY. Do NOT run downstream skills until the user explicitly selects an option.