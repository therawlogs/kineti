---
name: /causality-diff
description: Runs Causality Differential Analysis before pull requests to compute the exact Blast Radius Risk Index (0.0 to 1.0) and verify PR safety.
---

# Skill: /causality-diff (10/10 Perfect Score Blast Radius Analyzer)

## When to Use
Use right before `/ship` or during code reviews to evaluate pull request impact.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: Execute Causality Engine Differential
  1. Run `python3 ~/.gemini/config/scripts/causality_graph_builder.py [project_root]` via `run_command`.
  2. Read `.northstar/graphs/causality_diff.md`.
  3. Inspect Blast Radius Risk Index ($0.00 \to 1.00$):
     * **LOW RISK ($< 0.25$)**: Safe to merge automatically.
     * **MEDIUM RISK ($0.25 \to 0.60$)**: Requires extra unit test run.
     * **HIGH RISK ($> 0.60$)**: Requires explicit user review confirmation.

  Upon completion, print the Causality Differential summary.
