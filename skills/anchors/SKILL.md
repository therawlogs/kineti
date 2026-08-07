---
name: /anchors
description: Initializes ETHOS.md and CLAUDE.md files in the project root to enforce Kineti OS low-decay constraints and clear execution directives using pure Plain English.
---

# Skill: /anchors (The Kineti OS Project Root Initializer)

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  1. Copy global `ETHOS.md` from `~/.gemini/config/ETHOS.md` directly into the active project root directory `<project_root>/ETHOS.md`.
  2. Create local `.northstar/` decision memory folders (`dialogue/`, `gaps/`, `graphs/`, `decisions/`, `templates/`, `mocks/`).
  3. Create `.northstar/.gitignore` containing `*` to protect project memory from public online repositories.
  4. Run `python3 ~/.gemini/config/scripts/causality_graph_builder.py [project_root]` to generate initial 4-level causality graphs.

  Upon completion, print this summary:
  ```
  === /anchors Complete ===
  - Kineti OS ETHOS.md initialized in project root.
  - Local .northstar/ decision memory active (.gitignore protected).
  - 4-Level Causality Graphs generated in SQL/PGQ format.
  ```