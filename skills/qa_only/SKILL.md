---
name: /qa-only
description: Executes same E2E runs as /qa, but locks the workspace to forbid file writes, returning a diagnostic replication report.
---

# Skill: /qa-only (Alternative Diagnostic Loop)

## When to Use
For diagnostic testing on active branches where workspace isolation must be maintained.

## How to Use
Enter `/qa-only`.

## Sequencing
- **Phase**: `06_qa_verification`
- **Step**: Alternative diagnostics execution loop. Excludes workspace write actions.

## Protocol & Actions
- **Mode**: `ISOLATED_DIAGNOSTIC_REPORTER` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** execute testing immediately. Confirm diagnostic limits with the user:

  1. **What is the focus of this isolated diagnostic run?**
     * *Option A*: Bug Replication (Try to reproduce a reported crash log; do not write regression tests).
     * *Option B*: Performance Profile (Measure memory/timing profiles on the active branch without modifications).
     * *Option C*: Write-in.

  2. **Select the diagnostic logging granularity:**
     * *Option A*: Asymmetric Logs (Capture network traces, console warnings, and stack traces on error).
     * *Option B*: Visual trace (Extract page screenshots at 500ms intervals, output to console).
     * *Option C*: Write-in.

  Verify the read-only constraints are acknowledged before starting the run.

## Expected Output
- **Output type**: `ASYMMETRIC_DIAGNOSTIC_DATA_LOG`