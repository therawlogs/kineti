# First Run Tutorial

This tutorial takes about 10 minutes. It shows you how to install Kineti, connect it to your AI coding agent, run your first verified task, check spending, and use undo safety.

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

# Initialize Kineti for your project and installed editors
kineti init
```

`kineti init` detects your installed editors, copies the Kineti workflow skills into each one, and prints the safety-hook block to paste into your host settings. It does not write MCP server config — do that in Step 2.

---

## Step 2: Connect Kineti to your AI editor (MCP)

Kineti provides a universal Model Context Protocol (MCP) server so your agent can use spend controls, undo safety, and test proofs automatically.

If `kineti init` did not configure your editor automatically, set it up manually:

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

Open your AI coding agent (in Cursor, Claude Code, etc.) and give it a real task:

> "Add a health check test for our API and verify it passes."

Watch how Kineti works with your agent:
1. **Goal anchoring**: The agent locks the goal so it does not drift during multi-step tasks.
2. **Plan approval**: For new features or structural changes, the agent presents a clear plan before modifying code.
3. **Undo recording**: Before modifying files, the agent records an undo command in the SAGA stack.

---

## Step 4: Check spending and limits

The agent records each model call with exact dollar amounts.

Check your spending anytime in your terminal:

```bash
# View current spending and limit
kineti spend check
```

Expected output:
```text
total $0.15 of $50; entries 2; tripped=false
```

- **Recorded-totals spend breaker**: the agent records each model call; when recorded totals reach 95% of the $50 limit ($47.50), the breaker trips on recorded totals and the agent must stop all actions.
- **Human-only reset**: An agent can never reset the spend limit on its own. Only you can reset it:
  ```bash
  kineti spend reset --i-am-human
  ```

---

## Step 5: Run tests and verify evidence

Kineti requires cryptographic proof that tests actually ran against the exact code in your repository:

```bash
# Run tests and record cryptographic proof receipt
kineti test -- bun test

# Verify that the test receipt is fresh and code has not changed
kineti test check --label test
```

If any file in the repository changes after running tests, the proof receipt immediately flips from `FRESH` to `STALE`, ensuring you never deploy unverified code.

---

## Step 6: Test undo safety

If an agent makes unwanted changes or breaks a file, you can roll back changes cleanly:

```bash
# Review and rollback the most recent changes
kineti undo
```

Kineti unwinds changes in reverse order (newest first).

---

## Step 7: Open the Visual Companion

Kineti includes a lightweight local dashboard built to Apple HIG standards:

```bash
# Start the companion dashboard
kineti companion
```

Open `http://127.0.0.1:8788` in your browser to view your real-time spend charts, active tasks, team members, and verification receipts.
