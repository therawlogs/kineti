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

  ### Step 3: Stage 0 - Domain Space & Project Context
  Ask the user these plain questions:
  * **Q1. Business Model**: Is this for consumers (B2C), businesses (B2B), partner platforms (B2B2C), government (B2G), an internal company tool, or a hobby project?
  * **Q2. Industry Domain**: Finance, healthcare, developer tools, retail/e-commerce, education, legal, media, enterprise operations, or AI/ML?
  * **Q3. Greenfield vs. Brownfield Execution**:
    * *Greenfield*: Building from scratch. What language, folder layout, and build tools do you prefer?
    * *Brownfield*: Adding to existing software. What is the path to the codebase, what does the current code do, and what rules or parts must NOT be changed?
  * **Q4. Risk Level**: Is this a quick learning test, an internal team tool, a live paid product, a critical infrastructure system, or a safety/legal-critical system?
  * **Q5. Timeline Pressure**: No deadline, soft target date, hard launch deadline, or already behind schedule?

  *Dynamic Questions*: Generate 3–5 plain, domain-specific questions tagged `[DOMAIN-SPECIFIC]` based on Q1–Q5.

  ### Step 4: Stage 1 - Problem & Money Validation
  Ask the user these plain questions:
  * **Q1. The 5-Whys Root Cause**: Ask "why" 5 times to find the real physical cause of the problem. Why fix it right now?
  * **Q2. Single Main Action**: What is the single main action the user performs to get value?
  * **Q3. Status Quo Cost**: What does the current manual or broken process cost each year in wasted time or money?
  * **Q4. Target Business Metric**: What exact financial target improves (lower hosting costs, fewer lost customers, or higher sales)?
  * **Q5. Project Veto Holder**: Who has the authority to stop or reject this project (security chief, finance head, end user)? What is their exact concern?
  * **Q6. Strongest Counter-Argument Test**: What is the strongest reason why this project might fail, and how do we prove that reason wrong?

  *Dynamic Questions*: Generate 3–5 plain business questions tagged `[DOMAIN-SPECIFIC]`.

  ### Step 5: Execute Causality Engine v2.1
  1. Run `python3 ~/.gemini/config/scripts/causality_graph_builder.py [project_root]` via `run_command`.
  2. Generate 4-level SQL/PGQ Property Graphs in `.northstar/graphs/` (`agent`, `loop`, `graph`, `project`).

  ### Step 6: Check for Missing Information
  Scan all answers for missing facts or unconfirmed details. Prompt the user for each gap:
  * **Option A**: Provide the missing facts inline now.
  * **Option B**: Generate a simple blank template file in `.northstar/templates/`.
  * **Option C**: Generate temporary mock data tagged `[PROVISIONAL]` in `.northstar/mocks/`.

  ### Step 7: Save Conversation & Run Downstream Skills
  1. Save transcripts to `.northstar/dialogue/stage_0_domain_classification.md` and `.northstar/dialogue/stage_1_problem_validation.md`.
  2. Update `.northstar/manifest.json` and `.northstar/sprint_state.json`.
  3. Automatically run `/spec` and `/plan_ceo_review` in sequence.

  Upon completion, print this summary:
  ```
  === Checkpoint 1: /officehours Complete (The Kineti OS v10) ===
  - Local .northstar/ decision memory initialized (.gitignore protected).
  - Socratic Memory Calibrated & 3-Layer Knowledge Search executed.
  - Stage 0 & Stage 1 complete (Domain, Problem, and Business goals locked).
  - 4-Level SQL/PGQ Causality Graphs updated (.northstar/graphs/cumulative_causality.md).
  - Context Gaps checked & resolved.
  - Next Step: Run /autoplan to define system architecture.
  ```