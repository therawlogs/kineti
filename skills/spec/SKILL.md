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

  ### Step 3: Smolagents Code-First Specification Output
  Generate clean, high-density specification text describing exact data fields, type constraints, and endpoint signatures without verbose filler.

  Save to `.northstar/spec.md` and print summary.