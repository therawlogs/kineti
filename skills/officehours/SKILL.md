---
name: /officehours
description: Compounded Product & Strategy Gate. Initializes local .northstar/ decision memory, executes Stage 0 (Domain Classification & Greenfield/Brownfield), 3-Layer Knowledge Search, Stage 1 (Problem Validation & 5-Whys), generates dynamic domain questions, builds causality graphs, and logs context gaps locally using pure Plain English.
---

# Skill: /officehours (Stage 0 & Stage 1 Discovery - The Kineti OS v10)

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

  ### Step 3: Stage 0 - Dynamic Domain Research & Application Classification
  1. **Execute Live Research & Causality Audit**: Conduct research into the industry domain and inspect `.northstar/graphs/cumulative_causality.md` to load past decisions. Static hardcoded questions are STRICTLY PROHIBITED.
  2. **Classify Application Type & Synthesize Production Stack Options**:
     * **Enterprise Tool (B2B)**: Automatically propose production stack with SSO/SAML auth, RBAC user management, audit logging, PII redaction middleware, multi-tenant database isolation, and security password verification.
     * **Consumer Application (B2C)**: Automatically propose customer OAuth onboarding, user databases, responsive component UI, edge caching, and lightweight billing integrations.
     * **Internal Tool / Single-User Workflow**: Automatically propose API token auth, direct tool integrations, high-density data tables, and automated background execution tasks (zero collaboration bloat).
  3. **Present Research-Backed Recommendations & Dynamic Probes**: Present the synthesized app classification, recommended tech stack choices (3 production-grade options tailored for Human + Agent workflows), and 3–5 dynamic, context-aware domain probes. Then **HALT EXECUTION IMMEDIATELY and WAIT FOR USER RESPONSE**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the dynamic domain probes, application classification, and synthesized stack choices to the user, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from auto-selecting choices, simulating user answers, writing transcript files, or running downstream skills before receiving explicit user input.

  ### Step 4: Stage 1 - Problem & Value Validation (After User Responds)
  Once the user responds to Stage 0, present Stage 1 value probes (5-Whys root cause, single main value action, financial target metric, veto holder, and counter-argument refutation) tailored to the user's answers and **HALT AGAIN FOR USER INPUT**.


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