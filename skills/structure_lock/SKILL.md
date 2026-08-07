---
name: /structure-lock
description: Configures ingestion/egress schemas for LLM interaction layers, locking inputs and outputs into strict schemas without intermediate regex parsing in Plain English.
---

# Skill: /structure-lock (The Kineti OS Schema Locking & Shadow DB Assertions)

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: In-Memory Shadow DB Test Run
  1. Initialize SQLite in-memory database (`:memory:`).
  2. Run migration schema files against in-memory DB and verify 0 schema errors.

  ### Step 2: Lock JSON Input/Output Contracts
  1. Lock Pydantic/JSON Schema contracts for all subagent interaction envelopes.
  2. Verify all API responses return validated typed schemas.

  Upon completion, print summary of locked schemas.