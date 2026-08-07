---
name: /compress-context
description: Strips syntax decoration, implementation function bodies, and comments from target source files using AST Context Shrinking (Alex Verem Pattern) to produce a hyper-dense schema, and triggers the 4-Level Causality Graph Builder (Agent, Loop, Graph, Project levels).
---

# Skill: /compress-context (AST Context Shrinking & 4-Level Causality Builder)

## When to Use
Use during codebase research, dependency mapping, or architectural review to pass dense code context and generate 4-level causality graphs.

## Sequencing
- **Phase**: `02_technical_planning`
- **Step**: Sub-skill

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** execute these two steps using pure Plain English:

  ### Step 1: Execute 4-Level Causality Graph Builder
  1. Run `python3 ~/.gemini/config/scripts/causality_graph_builder.py [project_root]` via `run_command`.
  2. Generate 4 distinct SQL/PGQ Property Graph levels in `.northstar/graphs/`:
     * **Level 1: Agent Level Causality Graph (`agent_causality.md`)**: Subagents, MCP tools, handoffs.
     * **Level 2: Loop Level Causality Graph (`loop_causality.md`)**: Autonomic patch loops, LangGraph Saga undo stack.
     * **Level 3: Graph Level Causality Graph (`graph_causality.md`)**: Code AST dependency DAG & PageRank scores.
     * **Level 4: Project Level Causality Graph (`project_causality.md`)**: 5-Whys, cost ceilings, veto gates.
     * **Master: Cumulative Causality Graph (`cumulative_causality.md`)**: Unified SQL/PGQ property graph.

  ### Step 2: Parse AST Skeleton (Alex Verem Pattern)
  1. Parse target source code files down to their **Abstract Syntax Tree (AST) Skeleton**.
  2. Preserve exported types, interface signatures, structs, and function headers.
  3. Strip all implementation function bodies (`{ ... }`) and comments for research/planning windows ($\ge 85\%$ token reduction).
  4. Full function implementation bodies are loaded **only** when actively editing specific lines.

  Upon completion, print this summary:
  ```
  === /compress-context Complete ===
  - 4-Level Causality Graphs generated in SQL/PGQ format (Agent, Loop, Graph, Project levels).
  - PageRank Criticality Score computed (#1 Keystone File identified).
  - AST Skeletons parsed (85-90% token reduction achieved).
  ```