---
name: /spec
description: Translates validated intent into a high-density, zero-ambiguity functional specification using Smolagents Code-First Actions & LlamaIndex Sub-Query Data Breakdown in Plain English.
---

# Skill: /spec (Stage 01 - Scope Elaboration & Data Quality Specification)

## When to Use
Use immediately after Stage 1 validation in `/officehours`.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: LlamaIndex Sub-Query Data Breakdown
  Break down feature requests into explicit, atomic sub-queries. Audit data quality ($Q = C \times A \times T$):
  *   **Completeness ($C$)**: Are all required inputs and error paths defined?
  *   **Accuracy ($A$)**: Is data validated against strict schemas before processing?
  *   **Timeliness ($T$)**: Is data fetched with low latency targets?

  ### Step 2: Data-Routing Map Construction
  Draw an explicit data map connecting every component:
  ```
  User Input -> Validation Schema -> Business Logic Handler -> Storage DB -> API Response -> User Interface
  ```

  ### Step 3: Present Data Contract & HARD PAUSE
  Present the atomic sub-queries, data-routing map, data validation rules ($Q = C \times A \times T$), and input/output contracts to the user, then **STOP EXECUTION IMMEDIATELY**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the data-routing map, field schemas, and failure handling rules, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from saving `spec.md` or triggering downstream planning until the user explicitly reviews and approves the specification.

  ### Step 4: Save Public Functional Specification (After User Approves)
  Once approved by the user, save the complete production functional specification to `<project_root>/spec.md` as a public repository asset (NOT inside `.northstar/`). Update `.northstar/manifest.json` silently in the background, and present the spec completion checkpoint.