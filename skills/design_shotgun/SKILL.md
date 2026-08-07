---
name: /design-shotgun
description: Rapid parallel generation of multiple layout variations, validating performance metrics in headless browser sessions.
---

# Skill: /design-shotgun (Step 12 - Parallel UI Experimentation)

## When to Use
When rapid visual experimentation or layout exploration is requested.

## How to Use
Enter `/design-shotgun` with a structural view component description.

## Sequencing
- **Phase**: `04_implementation`
- **Step**: 12 (UI variation execution loop).

## Protocol & Actions
- **Mode**: `VARIANT_MATRIX_EXPLORATION` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/design`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** immediately spawn design alternatives. Present the user with these variation choices:

  1. **How many layout variants should be generated?**
     * *Option A*: 2 Extreme Alternatives (e.g. Side-by-side comparison of a glassmorphic dashboard vs a minimal monochrome terminal layout).
     * *Option B*: 3 Micro-Adjustments (Same basic layout, but testing variations in spacing, line-heights, and buttons alignment).
     * *Option C*: Write-in.

  2. **Which target viewport variations should the headless check validate?**
     * *Option A*: Compact Mobile Viewport (375x812; checking wrap constraints).
     * *Option B*: Desktop HD Viewport (1920x1080; checking horizontal scaling layout).
     * *Option C*: Multi-breakpoint Matrix (Simulate all widths from 320px to 1440px).
     * *Option D*: Write-in.

  3. **What is the primary optimization variable?**
     * *Option A*: DOM Node Minimization (Reduce deep tag nesting; target low weight structure).
     * *Option B*: Interaction Velocity (Prioritize layouts that expose crucial options instantly without sub-menus).
     * *Option C*: Write-in.

  Acquire choice selections before running variant loops.

## Expected Output
- **Output type**: `COMPATATIVE_UI_VARIANT_SETS`