---
name: /autoplan
description: Compounded Technical Planning & Design Gate. Runs Stage 2 (System Boundaries & Workflow Mechanics) and Stage 3 (Interaction Surface & Experience Mechanics), applies AST Context Shrinking, PageRank File Scoring, and Boil the Ocean completeness inside the Shippable Wedge.
---

# Skill: /autoplan (Checkpoint 2 - Stage 2 & Stage 3 Planning Gate)

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

  ### Step 4: Execute Stage 2 - System Boundaries & Workflow Mechanics
  Prompt the user with Stage 2 plain questions:
  * **Q1. Latency Target**: Under 500ms (interactive) / Under 5s (quick) / Under 60s (async) / Minutes to hours (batch)
  * **Q2. Peak Volume (6 months)**: Under 100/day / 100–10k/day / 10k–1M/day / Unknown
  * **Q3. Output Consuming System**: Human only / Internal API / Third-party API / Agentic loop
  * **Q4. Sensitive Data Rules**: What data must NEVER leave the system (names, cards, health data, trade secrets)?
  * **Q5. Execution Environment**: Cloud managed / Self-hosted GPU / Edge-local / Hybrid
  * **Q6. Error Handling Plan**: What happens when a step fails mid-way (fail loud, fail safe, fail forward, human escalation)?
  * **Q7. Integration Dependencies**: What upstream/downstream systems must connect on day one?

  *Dynamic Questions*: Generate 3–5 plain technical boundary questions tagged `[DOMAIN-SPECIFIC]`.

  ### Step 5: Execute Stage 3 - Interaction Surface & Experience Mechanics
  Prompt the user with Stage 3 plain questions:
  * **Q1. Primary Interaction Surface**: Web browser / Phone app / Command line CLI / API-only / Desktop app / Voice
  * **Q2. User Habit Benchmark**: What existing tool's interaction pattern is the user already trained on (Workspace, Chat, Linear-Notion, IDE, None)?
  * **Q3. Information Density**: High (dashboards) / Medium / Low / Minimal
  * **Q4. Output Accuracy Requirement**: Are outputs advisory (human reviews) or zero-tolerance exact (financial/legal)?
  * **Q5. Non-Text Content Requirements**: None / Charts / Maps / Media / Rich documents

  *Dynamic Questions*: Generate 3–5 plain design probes tagged `[DOMAIN-SPECIFIC]`.

  ### Step 6: Execute Planning Sub-Skills
  1.  **Run `/plan_eng_review`**: VRAM Sizing Math ($VRAM = Weights + KV_{cache}$), Hexagonal Ports/Adapters, In-memory Shadow DB assertions.
  2.  **Run `/plan_design_review`**: User Muscle Memory Benchmarks, Density Constraints.
  3.  **Run `/design_consultation`**: Typography stacks & 8px spacing scales.

  ### Step 7: Causality Graphs, Gap Protocols, & Local Persistence
  1. Build Stage 2 & 3 causality graphs in SQL/PGQ format → `.northstar/graphs/stage_2_causality.md` and `.northstar/graphs/stage_3_causality.md`.
  2. Update `.northstar/graphs/cumulative_causality.md`.
  3. Run Context Gap Detection → `.northstar/gaps/stage_2_gaps.md` and `stage_3_gaps.md`.
  4. Save transcripts → `.northstar/dialogue/stage_2_system_boundaries.md` and `stage_3_design_choices.md`.
  5. Update `.northstar/manifest.json` and `.northstar/sprint_state.json`.

  Upon completion, print this summary:
  ```
  === Checkpoint 2: /autoplan Complete ===
  - AST Context Shrinking applied (85-90% token reduction achieved).
  - PageRank Criticality Score computed (#1 Keystone File identified).
  - "Boil the Ocean" completeness enforced inside the Shippable Wedge (100% test & edge-case coverage).
  - Stage 2 & Stage 3 complete (Boundaries & Experience Mechanics locked).
  - Cumulative Relational System Map updated (.northstar/graphs/cumulative_causality.md).
  - Next Recommended: Run /design to compile layouts & execute code build.
  ```