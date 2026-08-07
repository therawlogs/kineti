---
name: /investigate
description: Emergency diagnostic flow triggered on runtime errors, compile failures, or broken tests. Rolls back codebase if 3 consecutive fixes fail.
---

# Skill: /investigate (Emergency Branch Logic)

## When to Use
Immediately upon encountering any runtime error, failed compilation, or broken test.

## How to Use
Enter `/investigate` along with the exact stack trace, error log, and environmental variables.

## Sequencing
- **Phase**: `05_validation_review` (Emergency Branch Logic)
- **Step**: Triggers dynamically on any execution fault.

## Protocol & Actions
- **Mode**: `5_WHYS_DIAGNOSTIC_CHAIN` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/design` or `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** make hot-fixes or edits immediately. Present the user with these diagnostic choices:

  1. **What is the primary suspected cause?**
     * *Option A*: Logic / Code Error (Null references, index out of range, logic loop errors).
     * *Option B*: Environmental / Dependency mismatch (Missing environment variables, package mismatch, port block).
     * *Option C*: Database / Persistence layer issue (Connection failure, constraint violation, transaction timeout).
     * *Option D*: Write-in.

  2. **How should the diagnostic chain isolate execution?**
     * *Option A*: Step-by-Step execution tracing (Add logs/print markers at every function entry point).
     * *Option B*: Sandbox isolation (Run the failing code block in a separate scratch script to test outcomes).
     * *Option C*: Rollback confirmation (Instantly revert the last commit without attempting edits).
     * *Option D*: Write-in.

  Confirm choices before executing fix loops.

## Expected Output
- **Output type**: `ROOT_CAUSE_RESOLUTION_LOG`