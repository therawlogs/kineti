---
name: /document-release
description: Syncs internal documentation, diagrams, and manuals with git diff states to prevent documentation drift and decay.
---

# Skill: /document-release (The Kineti OS Documentation Sync & Drift Prevention)

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: Git Diff vs. Documentation Check
  1. Inspect git diff state for exported function signatures or API endpoints.
  2. If exported signatures changed, update Diátaxis documentation manuals and Mermaid diagrams automatically.

  ### Step 2: Update Local Memory Manifest
  1. Sync `.northstar/manifest.json` with updated file hashes.
  2. Transition active working frames to cold graphs and compressed archives.

  Upon completion, print summary of synced documentation files.