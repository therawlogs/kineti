# Daily and Weekly Developer Guide

This guide walks you through the daily engineering loop with Kineti: starting tasks, approving plans, running tests with proof receipts, rolling back unwanted changes, and checking pipeline health.

---

## 1. The Daily Loop

When working with an AI coding agent (Cursor, Claude Code, OpenCode, Codex, Antigravity), follow this simple 5-step loop:

### Step 1: Start a task
Give your agent a clear, concrete task:
- Bug fix: *"Fix the login redirect error when session cookies expire."*
- Feature: *"Add an input validation check with unit tests."*
- Cleanup: *"Refactor the database connection pool to use async/await."*

Store the run's goal in `.kineti/state.json` when starting local project state. This file is local workflow state, not an independently tamper-proof policy.

### Step 2: Review and approve the plan
- For simple bug fixes, your agent fixes the issue and runs tests directly.
- For new features or significant changes, Kineti's installed agent instructions ask the agent to present a clear implementation plan before modifying application code in `src/`. This is instruction-level guidance, not an independent code-write interceptor.
- Review the plan, ask for changes if needed, and give approval before the agent proceeds.

### Step 3: Let the agent build with undo safety
As the agent makes code edits:
- The agent should register an inverse before each mutation. Kineti only undoes registered steps.
- You can inspect active progress or roll back unwanted changes anytime with:
  ```bash
  kineti undo
  ```

### Step 4: Run tests and record evidence
Do not rely only on an agent's statement that tests passed. Kineti can save each command's exit code and a workspace fingerprint:

```bash
# Run your test suite and save a local evidence receipt
kineti test -- bun test

# Check receipt freshness
kineti test check --label test
```

If a fingerprinted file changes, the receipt is `STALE` and should be recorded again before shipping. Receipts are local records, not signed certificates. See [FINGERPRINT_LIMITATIONS.md](./FINGERPRINT_LIMITATIONS.md) for what the fingerprint can miss and what exit code 0 does not prove (such as skipped tests or empty suites).

### Step 5: Verify release integrity (CI Gate)
Before opening a pull request or merging code, run the Context Integrity Layer (CIP) verification:

```bash
kineti ci
```

This checks:
1. The recorded spend value and trip flag.
2. That project state contains a root goal.
3. Required test receipts, including freshness and workspace fingerprint, when labels are supplied.
4. Recorded gate values. The report does not independently prove that a person approved a gate or that the agent was contained.

For a fresh CI checkout, initialize state before tests:

```bash
kineti seed
```

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
| **Seed CI State** | `kineti seed` | Create missing state and spend files from project config |
| **CI Verification** | `kineti ci --require-evidence <label>` | Check state and named evidence receipts |
| **Companion UI** | `kineti companion` | Launch local companion dashboard (port 8788) |

---

## 3. Weekly Maintenance

Run the local memory and journal maintenance commands when needed:

```bash
# Expire warm/cold records, verify hash chain, and check timestamps
bun bin/kineti-memory-job.ts sweep
bun bin/kineti-memory-job.ts verify-chain
bun bin/kineti-memory-job.ts time-order
bun bin/kineti-memory-job.ts promote
```

---

## 4. Uninstalling Kineti

To uninstall Kineti from your editor or project:

```bash
./setup.sh --uninstall
```

This removes Kineti skills and host configuration files without touching your project source code.
