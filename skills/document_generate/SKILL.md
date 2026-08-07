---
name: /document-generate
description: Interprets system logic to generate multi-dimensional developer documentation structured around Diátaxis framework, Mermaid diagrams, and MCP tools.json export in Plain English.
---

# Skill: /document-generate (Stage 07 - Diátaxis Docs & MCP Tool Schema Export)

## When to Use
Use during release documentation generation.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: Diátaxis 4-Quadrant Manual Generation (LlamaIndex Pattern)
  Apply LlamaIndex Hierarchical Indexing to generate four distinct documentation sections:
  1. **Tutorial**: Step-by-step onboarding guide for new users.
  2. **How-To Guide**: Action-oriented recipes for common tasks.
  3. **Reference Manual**: Complete schema specs, API parameters, and data types.
  4. **Explanation / ADR**: Plain English rationale explaining key architecture decisions.

  ### Step 2: System Diagrams
  Generate Mermaid visual diagrams:
  * **System ERD Graph**: Relational schema layout.
  * **Sequence Diagram**: Input-to-output data transaction flow.

  ### Step 3: Auto-Compile MCP Tool Schema (`tools.json`)
  1. Parse backend API routes and action parameters.
  2. Compile and export a valid **MCP Tool Schema** (`tools.json` / OpenAPI specification).
  3. Verify AI helper agents can consume your application as a native tool package instantly.

  Upon completion, print this summary:
  ```
  === /document-generate Complete ===
  - Diátaxis 4-Quadrant manuals written in Plain English.
  - Mermaid ERD and Sequence diagrams generated.
  - Simple AI Tool List (tools.json) compiled & exported.
  ```