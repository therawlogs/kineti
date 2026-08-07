---
name: /freeze
description: Locks core architectural directories and files from AI writes during investigatory, research, or QA phases.
---

# Skill: /freeze (Workspace Write-Lock Gate)

## When to Use
Prior to running diagnostic investigations, code auditing, research subagents, or automated QA verification loops to prevent accidental or unapproved code changes.

## How to Use
Enter `/freeze` pointing to files or directories to freeze.

## Sequencing
- **Phase**: `05_validation_review` (Safety utility)

## Protocol & Actions
- **Mode**: `WORKSPACE_WRITE_LOCK` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, you **MUST NOT** write-lock folders immediately. Halt and prompt the user:

  1. **Select the target scope to freeze:**
     * *Option A*: Core Architecture (Freeze `.agents/`, `config/`, `.github/`, and build configurations like `package.json`/`requirements.txt`).
     * *Option B*: Global Source (Freeze all source files under `/src` or `/lib`; only allow modifications in `/scratch` and test folders).
     * *Option C*: Complete Lock (Block all file write actions across the entire workspace).
     * *Option D*: Write-in.

  2. **Select lock duration:**
     * *Option A*: Single Session (Unfreeze automatically when the current tool call session terminates).
     * *Option B*: Milestone Lock (Keep frozen until `/ship` verification begins or manual thaw command `/unfreeze` is run).
     * *Option C*: Write-in.

  Confirm freeze rules before applying lock markers.

## Expected Output
- **Output type**: `LOCKED_WORKSPACE_STATE_MAP`