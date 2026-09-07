# Kineti OS

A safe coding assistant system that works inside your existing AI tools (Claude Code, Cursor, Antigravity, OpenCode, and Codex).

Kineti keeps your AI coding safe by doing four simple things:
1. **Tracks spending**: Pauses work if token costs reach $50.00.
2. **Saves undo steps**: Lets you undo any file changes cleanly.
3. **Runs tests**: Checks that tests actually pass before saving work.
4. **Shows clear progress**: Shows you what is happening in a clean local dashboard.

## Installation (30 seconds)

Requirements: `git` and `bash`.

```sh
git clone <this-repo> ~/kineti && cd ~/kineti && ./setup.sh
```

The setup script detects your tools and adds Kineti skills automatically:
- Claude Code (`~/.claude/skills/kineti-*`)
- OpenCode (`~/.opencode/skills/kineti-*`)
- Gemini / Antigravity (`~/.gemini/config/skills/kineti-*`)
- Codex (`~/.codex/skills/kineti-*`)

To uninstall at any time: `./setup.sh --uninstall`

## How to Use

You do not need to learn special slash commands. Just open your project in your tool (like Claude Code or Cursor) and describe your task in plain English:

```text
"Fix the login redirect bug."
```

The AI agent will:
1. Read `.kineti/state.json` to see the active task.
2. For new features: write a plan and ask for your approval first.
3. For bug fixes: fix the issue directly and run tests.
4. Track spending and save undo commands in the background.

## Companion Dashboard

Open the clean local dashboard in your browser to see the active task, costs, and recent activity:

```sh
bun run companion
```
Open `http://127.0.0.1:8788` in your browser.

## Tool Integration (MCP)

Connect any tool that supports the Model Context Protocol (MCP):

```sh
# Set up Cursor (.cursor/mcp.json)
bun run mcp:init

# Start the MCP server
bun run mcp
```

## Pull Request Check (CI)

Run the automated verification check locally:
```sh
bun run ci
```

## What lives where

- This folder: source code, scripts, UI components, and guides.
- Your project folder: `.kineti/state.json` (current task), `.kineti/spend.json` (costs), `.kineti/evidence.jsonl` (test results).
- Deleting this folder loses nothing permanent. You can re-clone and run setup again anytime.

## Guides and Documentation
- First run tutorial: `docs/TUTORIAL-first-run.md`
- Daily workflows: `docs/HOWTO-daily-loop.md`
- Core rules: `ETHOS.md`
- Workflows: `WORKFLOWS.md`

