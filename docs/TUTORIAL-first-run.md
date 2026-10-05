# First Run Tutorial

This tutorial takes about 10 minutes. It shows you how to install Kineti, connect it to an AI coding agent, record a local test receipt, check reported spending, and use registered undo steps.

## Prerequisites
- **Node.js**: version 18 or higher
- **Bun**: version 1.1 or higher (`curl -fsSL https://bun.sh/install | bash`)
- **AI Coding Agent or Editor**: Claude Code, Cursor, Windsurf, OpenCode, Codex, or Antigravity

---

## Step 1: Install Kineti in your project

Open your terminal in your project directory and run:

```bash
# Install Kineti globally
npm install -g kineti

# Install Kineti instructions and skills for your installed editors
kineti init

# Create local task state and lock this run's goal
kineti state init --project my-project --goal "Describe the outcome for this run"
```

`kineti init` detects your installed editors, copies the Kineti workflow skills into each one, and prints the safety-hook block to paste into your host settings. It does not write MCP server config — do that in Step 2.

`kineti state init` creates local project state. It is separate from editor setup.

---

## Step 2: Connect Kineti to your AI editor (MCP)

Kineti provides an MCP server exposing local tools for state, reported spend, registered undo, and test receipts. Your editor agent decides when to call these tools; they do not intercept every action the agent can take.

If `kineti init` did not configure your editor automatically, set it up manually. Kineti instructions do not intercept every action your agent can take.

- **Claude Code (CLI)**:
  ```bash
  claude mcp add kineti npx -y kineti mcp
  ```

- **Cursor**:
  Add to `.cursor/mcp.json` or your Cursor MCP settings (Windsurf uses `~/.codeium/windsurf/mcp_config.json`):
  ```json
  {
    "mcpServers": {
      "kineti": {
        "command": "npx",
        "args": ["-y", "kineti", "mcp"]
      }
    }
  }
  ```

- **OpenCode**:
  ```bash
  opencode mcp add kineti -- npx -y kineti mcp
  ```
  Or add to `~/.config/opencode/opencode.jsonc`:
  ```json
  {
    "mcp": {
      "kineti": {
        "type": "local",
        "command": ["npx", "-y", "kineti", "mcp"]
      }
    }
  }
  ```
  Confirm with `opencode mcp list`.

- **Codex (CLI)**:
  Add to `${CODEX_HOME:-$HOME/.codex}/config.toml`:
  ```toml
  [mcp_servers.kineti]
  command = "npx"
  args = ["-y", "kineti", "mcp"]
  ```
  Confirm Codex sees Kineti:
  ```bash
  codex mcp list
  ```
  Kineti should appear in the list.

---

## Step 3: Run your first task with your agent

Open your AI coding agent (in Cursor, Claude Code, etc.) and give it a task:

> "Add a health check test for our API and verify it passes."

Kineti provides local records and instructions for your agent; it does not independently verify completion of the whole task:
1. **Goal record**: The agent can store this run's goal in local Kineti state.
2. **Plan approval**: For new features or structural changes, the agent presents a clear plan before modifying code.
3. **Undo recording**: Before modifying files, the agent should register an inverse with Kineti. Kineti only undoes registered steps.

---

## Step 4: Check spending and limits

The agent reports each model call and its cost. Kineti adds up what is reported, so the total is only as accurate as those reports. At 95% of $50 ($47.50) the log command exits with code 3. Your agent or hooks need to act on that code. Kineti does not stop the agent itself yet.

Check your spending anytime in your terminal:

```bash
# View current spending and limit
kineti spend check
```

Expected output:
```text
total $0.15 of $50; entries 2; tripped=false
```

- **Recorded-totals spend breaker**: the agent reports each model call and its cost. When reported totals reach 95% of the $50 limit ($47.50), `kineti spend log` exits with code 3. Your agent or hooks need to act on that code.
- **Human-only reset**: An agent can never reset the spend limit on its own. Only you can reset it:
  ```bash
  kineti spend reset --i-am-human
  ```

---

## Step 5: Run tests and verify the receipt

Kineti can record that a test command finished and bind its result to a workspace fingerprint. The receipt is local; it is not a signed certificate.

```bash
# Run tests and save a local receipt with the current workspace fingerprint
kineti test -- bun test

# Verify that the test receipt is fresh and code has not changed
kineti test check --label test
```

If a file in the fingerprinted workspace changes after the test run, the receipt is `STALE` and should be recorded again before release.

---

## Step 6: Test undo safety

If an agent makes unwanted changes or breaks a file, you can roll back registered inverses:

```bash
# Review and rollback the most recent registered changes
kineti undo
```

Kineti unwinds registered steps in reverse order (newest first). A failed inverse prints `rollback incomplete`, exits 1, and that step stays pending.

---

## Step 7: Open the local Companion

Kineti includes a local dashboard for reviewing project state and recorded information:

```bash
# Start the companion dashboard
kineti companion
```

Open `http://127.0.0.1:8788` in your browser. The Companion is local. It does not connect external services, send messages, or provide cloud pairing in v0.4.0.
