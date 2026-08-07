---
name: /setup-browser-cookies
description: Syncs active session tokens and cookies from user environment into the headless runner to bypass login walls during testing.
---

# Skill: /setup-browser-cookies (Pre-Test Setup Utility)

## When to Use
Prior to running `/qa` or `/browse` on routes requiring authenticated session states.

## How to Use
Enter `/setup-browser-cookies` referencing the target domain.

## Sequencing
- **Phase**: `06_qa_verification`
- **Step**: Pre-test setup utility.

## Protocol & Actions
- **Mode**: `CRYPTOGRAPHIC_SESSION_SYNC` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/ship`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** auto-extract cookies. Present the user with these options:

  1. **From which source should the session state be sync'd?**
     * *Option A*: Local Chrome browser profile (Securely copy active tokens).
     * *Option B*: Local Firefox/Safari browser profile.
     * *Option C*: Manual Paste (Prompt the user to paste the raw JSON cookies array directly).
     * *Option D*: Write-in.

  2. **What is the target domain authentication scope?**
     * *Option A*: Localhost Auth (`http://localhost` session or token store).
     * *Option B*: Production / Staging domain (e.g. `*.my-app.com`).
     * *Option C*: Write-in.

  Confirm source before performing extraction.

## Expected Output
- **Output type**: `ACTIVE_AUTH_SESSION_STATE`