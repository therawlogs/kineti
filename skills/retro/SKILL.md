---
name: /retro
description: Analyzes repository throughput metrics (LOC, commits, test failure slopes) to trace and optimize development lifecycle bottlenecks.
---

# Skill: /retro (Pipeline Optimization Feedback Pass)

## When to Use
At standard engineering phase completions or post-project milestones to optimize velocity.

## How to Use
Enter `/retro` targeting development logs.

## Sequencing
- **Phase**: `08_release_ops`
- **Step**: Pipeline optimization feedback pass (Step 23).

## Protocol & Actions
- **Mode**: `DATA_DRIVEN_VELOCITY_COACHING` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** automatically output the retro analysis. Stop and present these options to the user:

  1. **Select the target scope of the retrospective:**
     * *Option A*: Active Feature Branch (Analyze the coding cycles, commit cadence, and errors for the current task).
     * *Option B*: Iteration Cycle (Analyze past 1-2 weeks of global repo activities and commits).
     * *Option C*: Write-in.

  2. **Select the primary metrics focus:**
     * *Option A*: Friction & Wait-time (Analyze command failure slopes, test failures, and blockages).
     * *Option B*: Throughput & Code Churn (Analyze lines of code generated, commits count, and file modifications frequency).
     * *Option C*: Write-in.

  Obtain choices before compiling the optimization report.

## Expected Output
- **Output type**: `SYSTEM_VELOCITY_OPTIMIZATION_REPORT`