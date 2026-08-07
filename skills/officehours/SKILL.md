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
  Present the following plain questions with clear options or recommendations to the user, then **HALT EXECUTION IMMEDIATELY and WAIT FOR USER RESPONSE**:
  * **Q1. Business Model**: Is this for consumers (B2C), businesses (B2B), partner platforms (B2B2C), government (B2G), an internal company tool, or a hobby project?
  * **Q2. Industry Domain**: Finance, healthcare, developer tools, retail/e-commerce, education, legal, media, enterprise operations, or AI/ML?
  * **Q3. Greenfield vs. Brownfield Execution**:
    * *Greenfield*: Building from scratch. What language, folder layout, and build tools do you prefer?
    * *Brownfield*: Adding to existing software. What is the path to the codebase, what does the current code do, and what rules or parts must NOT be changed?
  * **Q4. Risk Level**: Is this a quick learning test, an internal team tool, a live paid product, a critical infrastructure system, or a safety/legal-critical system?
  * **Q5. Timeline Pressure**: No deadline, soft target date, hard launch deadline, or already behind schedule?

  *Dynamic Questions*: Generate 3–5 plain, domain-specific questions tagged `[DOMAIN-SPECIFIC]` based on Q1–Q5.

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST output the questions above and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from auto-generating answers for the user, simulating choices, writing transcript files, or running downstream skills before receiving the user's explicit response.

  ### Step 4: Stage 1 - Problem & Money Validation (After User Responds to Stage 0)
  Once the user responds to Stage 0, present these Stage 1 questions and **HALT AGAIN**:
  * **Q1. The 5-Whys Root Cause**: Ask "why" 5 times to find the real physical cause of the problem. Why fix it right now?
  * **Q2. Single Main Action**: What is the single main action the user performs to get value?
  * **Q3. Status Quo Cost**: What does the current manual or broken process cost each year in wasted time or money?
  * **Q4. Target Business Metric**: What exact financial target improves (lower hosting costs, fewer lost customers, or higher sales)?
  * **Q5. Project Veto Holder**: Who has the authority to stop or reject this project (security chief, finance head, end user)? What is their exact concern?
  * **Q6. Strongest Counter-Argument Test**: What is the strongest reason why this project might fail, and how do we prove that reason wrong?

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