# Daily and Weekly Developer Guide

This guide walks you through the daily engineering loop with Kineti: starting tasks, approving plans, running tests with proof receipts, rolling back unwanted changes, and checking pipeline health.

---

## 1. The Daily Loop

When working with an AI coding agent (Cursor, Claude Code, OpenCode, Codex, Antigravity), follow this simple 5-step loop:

### Step 1: Start a task
Give your agent a clear, concrete task:
- Bug fix: *"Fix the login redirect error when session cookies expire."*
- Feature: *"Add an email notification service with unit tests."*
- Cleanup: *"Refactor the database connection pool to use async/await."*

Kineti locks the goal into `.kineti/state.json` to prevent scope creep during autonomous multi-step execution.

### Step 2: Review and approve the plan
- For simple bug fixes, your agent fixes the issue and runs tests directly.
- For new features or significant changes, Kineti requires your agent to present a clear implementation plan before modifying application code in `src/`.
- Review the plan, ask for changes if needed, and give approval before the agent proceeds.

### Step 3: Let the agent build with undo safety
As the agent makes code edits:
- Kineti automatically records an undo command before each mutation in the SAGA undo stack.
- You can inspect active progress or roll back unwanted changes anytime with:
  ```bash
  kineti undo
  ```

### Step 4: Run tests and record evidence
Never rely on unverified agent claims. Kineti binds test results directly to the git tree hash:

```bash
# Run your test suite and save a cryptographic evidence receipt
kineti test -- bun test

# Check receipt freshness
kineti test check --label test
```

If any source file changes, the receipt flips to `STALE`, ensuring that changes must be re-tested before shipping.

### Step 5: Verify release integrity (CI Gate)
Before opening a pull request or merging code, run the Context Integrity Layer (CIP) verification:

```bash
kineti ci
```

This verifies that:
1. The spend circuit breaker is healthy (< $50 ceiling).
2. The root goal is intact and untampered.
3. Test evidence receipts are fresh.
4. Pipeline quality gates (`spec`, `security`, `ship`) are satisfied.

---

## 2. Terminal Quick Reference

| Action | Command | Purpose |
| :--- | :--- | :--- |
| **Check Spend** | `kineti spend check` | Check current spend vs. $50 cap ($10/stage) |
| **Spend Summary** | `kineti spend status` | View detailed spend log and model usage |
| **Reset Breaker** | `kineti spend reset --i-am-human` | Reset tripped spend breaker (human only) |
| **Run Tests** | `kineti test -- <cmd>` | Execute test command and record evidence |
| **Check Tests** | `kineti test check --label <name>` | Verify evidence status (`FRESH` or `STALE`) |
| **Undo Edit** | `kineti undo` | Roll back recent file edits (LIFO stack) |
| **Task State** | `kineti status` | View active project, stage, and locked goal |
| **CI Verification** | `kineti ci` | Run full CIP verification report |
| **Companion UI** | `kineti companion` | Launch Apple HIG local dashboard (port 8788) |

---

## 3. Weekly Maintenance

Run the automated memory and journal maintenance job once a week:

```bash
# Expire warm/cold records, verify hash chain, and check timestamps
bun bin/kineti-memory-job.ts sweep
bun bin/kineti-memory-job.ts verify-chain
bun bin/kineti-memory-job.ts time-order
```

---

## 4. Uninstalling Kineti

To uninstall Kineti from your editor or project:

```bash
./setup.sh --uninstall
```

This removes Kineti skills and host configuration files without touching your project source code.
