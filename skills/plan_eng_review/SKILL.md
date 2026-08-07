---
name: /plan-eng-review
description: Technical architecture definition. Maps state flow, persistence engines, VRAM sizing, hexagonal ports/adapters, AST Context Shrinking, and PageRank scoring using pure Plain English.
---

# Skill: /plan-eng-review (Stage 02 - Technical Architecture & AST Skeleton Review)

## When to Use
Use during technical planning (`/autoplan`).

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: AST Context Shrinking & Token-Density Compression
  1. Parse target codebase files down to their **AST Skeleton** (Types, Interfaces, Structs, Signatures only).
  2. Strip implementation bodies (`{ ... }`) and comments for research/planning phases to achieve $\ge 85\%$ token reduction.

  ### Step 2: Spanner PageRank Criticality Indexing
  1. Calculate PageRank centrality scores across all project code files.
  2. Identify and print the **#1 most critical keystone file** (the single file whose modification impacts the highest number of downstream modules).
  3. Require extra validation checks (shadow DB assertions and automated test runs) before modifying that keystone file.

  ### Step 3: Meta AI Local VRAM Sizing Math
  If deploying local quantized LLMs, compute exact hardware limits:
  $$\text{VRAM} = \text{Model Weights} + \text{KV Cache Context} + \text{Activation Buffer}$$
  Verify local GPU VRAM headroom is $\ge 20\%$ to prevent Out-Of-Memory crashes.

  ### Step 4: Low-Decay Hexagonal Architecture ($\lambda < 0.1$)
  1. Separate core business logic from outside vendor libraries using clean Interface Adapters.
  2. Ensure swapping third-party databases, LLM API providers, or web servers requires zero edits to core business logic files.

  Obtain options and update `.sprint_state.json`.