---
name: /plan-ceo-review
description: Final business-level alignment check. Audits specifications against corporate ROI hurdle rates and 5-year operational horizons using pure Plain English.
---

# Skill: /plan-ceo-review (Stage 01 - Business Alignment Check)

## When to Use
Use immediately after `/spec` completes.

## Sequencing
- **Phase**: `01_product_strategy`
- **Step**: 3
- **Pre-requisite**: `/spec`

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** prompt the user to configure these business parameters in pure Plain English:

  ### 1. Money Return Threshold Check (Return vs Cost)
  What is the minimum required money return for this project?
  *   **Option A: High Return Requirement (Earn >1.5x Costs)** (RECOMMENDED)
      ├─ Info: Project must earn or save at least $1.50 for every $1.00 spent on building and running it.
      ├─ Why: Stops company money from being wasted on small, low-impact tasks.
      └─ If Fails: Features that save only tiny amounts of money are stopped.
  *   **Option B: Strategic Product Parity Requirement**
      ├─ Info: Built to match competitor capabilities even if direct financial return is lower.
      ├─ Why: Protects existing market position.
      └─ If Fails: Lower profit return per dollar spent.

  ### 2. Project Scope Size Filter
  *   **Option A: Smallest Working Version (Radical Reduction)** (RECOMMENDED)
      ├─ Info: Cut all extra non-essential features, build only the core working job.
      ├─ Why: Lowest build cost, fastest launch time, easiest code to maintain.
      └─ If Fails: Users may miss secondary features.
  *   **Option B: Large Expansion Version**
      ├─ Info: Build extra reusable parts and prepare for high future growth.
      ├─ Why: Prepares software for heavy future usage.
      └─ If Fails: High build cost and risk of building features users never use.

  ### 3. Where Main Data State Lives
  *   **Option A: Central Database** (RECOMMENDED)
      ├─ Info: Application server holds no memory; all facts are saved to a central database.
      ├─ Why: Easy to add more servers as usage grows.
      └─ If Fails: Every action requires a quick database lookup call.
  *   **Option B: Local Device Memory**
      ├─ Info: Facts stored temporarily in the user's browser or device memory.
      ├─ Why: Fast local user feedback, zero central server database load.
      └─ If Fails: Memory is lost when the user closes the app.

  Obtain choices and request approval before finalizing the Business Brief.