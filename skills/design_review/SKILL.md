---
name: /design-review
description: Final visual QA pass, correcting sub-pixel shifts and locking visual structures directly prior to merge.
---

# Skill: /design-review (Step 16 - Visual Integrity Audit)

## When to Use
After functional QA passes, ensuring pixel-level integrity before branch merge.

## How to Use
Enter `/design-review` targeting active client views.

## Sequencing
- **Phase**: `06_qa_verification`
- **Step**: 16 (Final UI validation pass).

## Protocol & Actions
- **Mode**: `LAYOUT_STRUCTURAL_AUDIT` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you should output your findings and proceed.

  1. **Select the target view for visual inspection:**
     * *Option A*: Main Feature Dashboard (Check alignment of headers, sidebars, and flex layouts).
     * *Option B*: Form Inputs / Checkout Layout (Check error labels, margins, button hover states).
     * *Option C*: Write-in (Specify path or file name).

  2. **Select the visual audit severity:**
     * *Option A*: Pixel-Perfect Check (Align layout elements to exact margins, check border overlaps, eliminate sub-pixel shifts).
     * *Option B*: General Usability Check (Check text readability, accessibility colors contrast, viewport scaling rules).
     * *Option C*: Write-in.

  Obtain layout context choices before executing structural changes.

## Expected Output
- **Output type**: `LOCKED_PIXEL_PERFECT_LAYOUT_STATE`