---
name: /browse
description: Drives headless browser directly to inspect rendering states and UI layouts, feeding DOM snapshots to workspace context.
---

# Skill: /browse (Contextual Utility)

## When to Use
When the AI agent requires immediate visualization of rendering behaviors or layout states.

## How to Use
Enter `/browse` followed by the local development target URL.

## Sequencing
- **Phase**: `06_qa_verification`
- **Step**: Contextual execution utility.

## Protocol & Actions
- **Mode**: `PROGRAMMATIC_HEADLESS_ENGINE_DRIVE` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** immediately load the target URL. Stop and confirm the navigation parameters:

  1. **Select the target URL category:**
     * *Option A*: Local Dev Server (Typically `http://localhost:3000` or `http://localhost:5173`).
     * *Option B*: Staging / Deployment Environment (External URL).
     * *Option C*: Live Documentation / API portal (For external schema inspection).
     * *Option D*: Write-in.

  2. **Select the primary action to perform upon load:**
     * *Option A*: Pure Render Check (Capture screenshot and DOM snapshot on initial paint).
     * *Option B*: Form Interaction (Fill inputs, trigger submission, wait for redirect).
     * *Option C*: Click & Inspect (Navigate to a specific CSS selector link).
     * *Option D*: Write-in.

  Confirm parameters before driving the browser.

## Expected Output
- **Output type**: `RUNTIME_RENDERING_VERIFICATION_STATE`