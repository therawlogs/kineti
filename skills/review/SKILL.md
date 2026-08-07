---
name: /review
description: Zero-tolerance structural audit of modified files within The Kineti OS. Scores code quality on a 0-100 scale using pure Plain English.
---

# Skill: /review (The Kineti OS Code Review & Structural Audit Gate)

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** audit target code changes using pure Plain English:

  ### Step 1: Low-Decay & AST Quality Audit ($\lambda < 0.1$)
  1. Verify zero intermediate assumptions and zero hardcoded static offsets.
  2. Verify interface separation between business logic and third-party vendors (Hexagonal Architecture).
  3. Verify `data-agent-action` tags and unique IDs exist on interactive components.

  ### Step 2: Causality Graph Consistency Check
  1. Verify modified files match dependencies in `.northstar/graphs/graph_causality.md`.
  2. If modifying a PageRank Keystone File, require clean test runs before approval.

  Upon completion, print score (0-100) and Plain English feedback.