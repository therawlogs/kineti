---
name: /design-consultation
description: Sets foundational design tokens and deterministic layout rules before interface coding using pure Plain English.
---

# Skill: /design-consultation (Stage 03 - Design Rules & Token System)

## When to Use
Use before frontend layout building.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** prompt the user using pure Plain English:

  ### 1. Spacing Ratio System
  *   **Option A: 8-Pixel Spacing Grid (8px / 16px / 24px / 32px)** (RECOMMENDED)
      ├─ Info: All padding, margins, gaps, and component heights align strictly to multiples of 8 pixels.
      ├─ Why: Prevents visual misalignment and ensures clean layout symmetry across all screen sizes.
      └─ If Fails: Random pixel spacing causes visual clutter and uneven alignment.
  *   **Option B: 4-Pixel Micro Grid (High-Density Data Views)**
      ├─ Info: Align spacing to 4-pixel steps for compact data tables and code tools.
      ├─ Why: Fits maximum data on dense screens.
      └─ If Fails: Tight spacing can feel cramped if used on simple consumer screens.

  ### 2. Typography Pairings
  *   **Option A: System Font Stack (Native Fast Load)** (RECOMMENDED)
      ├─ Info: Use native operating system fonts (`Inter`, system UI fonts).
      ├─ Why: Zero font file download delay; instant page rendering speed.
      └─ If Fails: Relies on OS default typography rendering.
  *   **Option B: Brand Web Font (`Outfit` / `Roboto`)**
      ├─ Info: Load curated Google Fonts for distinctive brand presentation.
      ├─ Why: Consistent visual typography across every device and browser.
      └─ If Fails: Requires loading small external font files.

  Obtain choices and write token values to `index.css`.