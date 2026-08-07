---
name: /qa
description: Automated functional testing via Playwright browser. Executes user click paths, verifies DOM states, and applies Grok-Build Autonomic Patching on failures using pure Plain English.
---

# Skill: /qa (Stage 05 - Playwright E2E & Grok-Build Autonomic Patching)

## When to Use
Use during pre-ship verification or during `/design` build gate.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using pure Plain English:

  ### Step 1: Multi-Viewport Playwright Test Run
  1. Launch Playwright headless browser test suite.
  2. Run user click paths and DOM assertions across desktop (1280px) and mobile (375px) viewports.
  3. Enforce **"Boil the Ocean" Completeness**: Verify 100% pass rate on all target user flows.

  ### Step 2: Grok-Build Autonomic Patching Loop
  If any test fails or throws an error:
  1. Catch exact error stack trace, failing line number, and Playwright screenshot context.
  2. Write an automated code fix patch directly addressing the root cause.
  3. Apply code patch to disk.
  4. Re-run tests immediately. Repeat patch loop automatically until all tests pass cleanly.

  Upon completion, print this summary:
  ```
  === /qa Complete ===
  - Playwright E2E tests run across desktop and mobile viewports.
  - "Boil the Ocean" 100% test pass rate verified.
  - Grok-Build Autonomic Patching active (0 unresolved test failures).
  ```