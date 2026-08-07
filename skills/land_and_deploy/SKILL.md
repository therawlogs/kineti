---
name: /land-and-deploy
description: Merges pull requests, polls build environments, and runs production release procedures with LangGraph Saga LIFO Rollbacks and Connected Component Fault Isolation in Plain English.
---

# Skill: /land-and-deploy (Stage 07 - GitOps Delivery & Saga LIFO Rollbacks)

## When to Use
Use during final production deployment.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: Register LangGraph Saga LIFO Rollback Stack
  1. For every multi-step deployment action (database migration, container push, DNS routing update), register a corresponding inverse undo command in a Last-In-First-Out (LIFO) stack.
  2. If any deployment step fails, execute the undo stack automatically in reverse order ($Step_n^{-1}, \dots, Step_1^{-1}$) to restore system to last known good state cleanly.

  ### Step 2: Connected Component Fault Isolation
  1. Group deployment services into isolated blast-radius clusters.
  2. Isolate deployment failures to the specific affected cluster without bringing down unrelated operational modules.

  ### Step 3: GitOps Delivery Execution
  1. Execute git merge and push to release branch.
  2. Poll build server and verify production health endpoints.

  Upon completion, print this summary:
  ```
  === /land-and-deploy Complete ===
  - GitOps delivery pipeline executed cleanly.
  - LangGraph Saga LIFO Rollback stack registered and standby active.
  - Connected Component Fault Isolation verified (zero blast-radius leakage).
  ```