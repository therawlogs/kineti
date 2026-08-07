---
name: /skillify
description: Post-execution utility. Parses shell histories and execution logs to package verified developer workflows into reusable macros.
---

# Skill: /skillify (Post-Execution Automation Codifier)

## When to Use
When successful workflow pathways or automation sequences need codification.

## How to Use
Enter `/skillify` referencing recent execution logs or shell sessions.

## Sequencing
- **Phase**: `07_data_tools`
- **Step**: Post-execution utility.

## Protocol & Actions
- **Mode**: `AUTOMATION_CODIFIER` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** auto-generate scripts immediately. Halt and prompt the user:

  1. **What is the primary log source for codification?**
     * *Option A*: Local Terminal History (Analyze shell executions, compile scripts, and test runs).
     * *Option B*: Agent Tool Call Logs (Analyze steps from `transcript.jsonl` in the conversation workspace).
     * *Option C*: Write-in.

  2. **Select the target automation asset format:**
     * *Option A*: Reusable Bash Shell Script (Create executable `.sh` file inside `/scratch`).
     * *Option B*: Local Agent Custom Skill (Create `skills/<name>/SKILL.md` file to load globally or in workspace).
     * *Option C*: Write-in.

  Acquire selections before compiling logs into automation assets.

## Expected Output
- **Output type**: `REUSABLE_AUTOMATION_ASSET`