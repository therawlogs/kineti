---
name: /checkpoint
description: Enables a state-saving system for your work. In "continuous" mode, it can auto-commit your work-in-progress (WIP) status to maintain a snapshot.
---

# Skill: /checkpoint (WIP State Protection Gate)

## When to Use
Prior to running experimental edits, launching multi-file changes, or executing subagents that might introduce breaking changes.

## How to Use
Enter `/checkpoint` targeting the active branch or codebase status.

## Sequencing
- **Phase**: `05_validation_review` (Safety utility)

## Protocol & Actions
- **Mode**: `WIP_STATE_SAVER` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, you **MUST NOT** auto-commit immediately. Stop and present the user with these checkpoint options:

  1. **Select the target scope for the checkpoint:**
     * *Option A*: Global Workspace (Commit all uncommitted modified files and untracked assets).
     * *Option B*: Selected File Buffers (Commit only the files modified in the active feature phase).
     * *Option C*: Staging Area (Commit only what has been explicitly added to the git staging area).
     * *Option D*: Write-in.

  2. **Select the commit message tagging format:**
     * *Option A*: WIP Auto-Generated (e.g. `wip: checkpoint before subagent execution`).
     * *Option B*: Task Specific (e.g. `wip: [task-id] current implementation state`).
     * *Option C*: Manual (Prompt user to input message).

  3. **Select checkpoint mode:**
     * *Option A*: Manual Trigger (Save state once and stop).
     * *Option B*: Continuous Mode (Automatically commit a snapshot every time a file write operation is completed during implementation).
     * *Option C*: Write-in.

  Obtain selections before committing the active git index.

## Expected Output
- **Output type**: `VERIFIED_CODEBASE_SNAPSHOT_STATE`