---
name: /plan-design-review
description: Frontend structural validation gate. Audits layouts to eliminate click friction and layout shifts using pure Plain English.
---

# Skill: /plan-design-review (Stage 03 - Interaction Surface & Density Check)

## When to Use
Use during technical and design planning (`/autoplan`).

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** prompt the user using pure Plain English:

  ### 1. User Habit Match Benchmark
  *   **Option A: Match Existing Product Habit (Linear / Notion / Stripe / VS Code)** (RECOMMENDED)
      ├─ Info: Match interaction patterns, keyboard shortcuts, and screen layouts of software the target user already uses daily.
      ├─ Why: Zero learning curve; users feel instantly comfortable.
      └─ If Fails: Non-standard interaction patterns increase friction and user hesitation.
  *   **Option B: Custom Unique Screen Flow**
      ├─ Info: Design novel screen layouts and custom controls.
      ├─ Why: Differentiates your product visually.
      └─ If Fails: Users must learn new habits, increasing initial onboarding friction.

  ### 2. Screen Information Density
  *   **Option A: High Density Dashboard (Multi-Panel Operator View)** (RECOMMENDED for power users)
      ├─ Info: Show key metrics, data tables, and action panels on a single screen without scrolling.
      ├─ Why: Enables decision-making without context switching or multi-tab jumps.
      └─ If Fails: Screen looks busy to casual consumers.
  *   **Option B: Focused Single-Action View (Low Density / Mobile Focus)**
      ├─ Info: One clear task per screen with large touch targets.
      ├─ Why: Simple, clean, and mobile-friendly.
      └─ If Fails: Power users must click through multiple screens to complete complex tasks.

  ### 3. Screen Tags for AI Agents & Testing
  *   **Option A: Add Plain Action Tags (`data-agent-action`)** (RECOMMENDED)
      ├─ Info: Inject plain action tags on all buttons, inputs, and state panels.
      ├─ Why: Allows automated testing tools and AI helper agents to use the app cleanly.
      └─ If Fails: Slightly adds hidden attribute tags to HTML elements.
  *   **Option B: Standard HTML IDs Only**
      ├─ Info: Use standard element ID tags.
      ├─ Why: Standard basic HTML setup.
      └─ If Fails: AI testing tools must guess element actions using fragile selectors.

  Obtain choices and update `.sprint_state.json`.