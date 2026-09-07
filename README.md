# Kineti OS

A safe coding assistant runtime that works inside your existing AI tools (Claude Code, Cursor, Antigravity, OpenCode, and Codex).

Kineti keeps your AI coding safe and governed by doing five simple things:
1. **Tracks spending**: Pauses work if token costs reach your spending limit ($50.00 default).
2. **Saves undo steps**: Lets you undo any file changes cleanly.
3. **Runs tests**: Checks that tests actually pass before saving work.
4. **Governs multi-repo fleets**: Switch between repositories and monitor team projects from one unified screen.
5. **Coordinates agent swarms**: Assigns cryptographic keys to agents to stop goal drift and verify outcomes.

## Installation (30 seconds)

Choose any method:

```sh
# Method 1: Standalone installer (macOS / Linux)
curl -fsSL https://raw.githubusercontent.com/therawlogs/kineti/main/install.sh | bash

# Method 2: Global npm package
npm install -g kineti

# Method 3: From source
git clone https://github.com/therawlogs/kineti.git ~/kineti && cd ~/kineti && ./setup.sh
```

Then initialize your project:
```sh
kineti init
```

The setup script detects your tools and adds Kineti skills automatically:
- Claude Code (`~/.claude/skills/kineti-*`)
- OpenCode (`~/.opencode/skills/kineti-*`)
- Gemini / Antigravity (`~/.gemini/config/skills/kineti-*`)
- Codex (`~/.codex/skills/kineti-*`)

To uninstall at any time: `kineti init --uninstall` (or `./setup.sh --uninstall`)

## How to Use

You do not need to learn special commands. Just open your project in your tool (like Claude Code or Cursor) and describe your task in plain English:

```text
"Fix the login redirect bug."
```

The AI agent will:
1. Read `.kineti/state.json` to see the active task.
2. For new features: write a plan and ask for your approval first.
3. For bug fixes: fix the issue directly and run tests.
4. Track spending and save undo commands in the background.

## Visual Companion Dashboard

Open the Apple HIG companion dashboard in your browser to view active tasks, costs, multi-repo fleet status, and settings:

```sh
kineti companion
```
Open `http://127.0.0.1:8788` in your browser.

- **Repository Switcher**: Quickly switch between local and remote projects from the top navigation bar.
- **Fleet View**: Monitor all connected repositories, developer owners, active tasks, and spend meters on one screen.
- **Settings Drawer**: Manage GitHub integration, toggle agent tool auto-latching (Cursor, Claude Code, Antigravity, Codex), and assign budget ceilings.

## Tool Integration (MCP)

Connect any tool that supports the Model Context Protocol (MCP):

```sh
# Start the MCP server
kineti mcp
```

## Agent Swarm Coordination

Run multi-agent swarms with cryptographic identity and outcome verification tickets:

```sh
kineti swarm "Build auth service"
```

## Pull Request Check (CI)

Run the automated verification check locally:
```sh
kineti ci
```

## What lives where

- This folder: source code, scripts, UI components, and guides.
- Your project folder: `.kineti/state.json` (current task), `.kineti/spend.json` (costs), `.kineti/evidence.jsonl` (test results).
- Deleting this folder loses nothing permanent. You can re-clone and run setup again anytime.

## Guides and Documentation

- Multi-repo fleet & integrations: `docs/MULTI_REPO_FLEET_AND_INTEGRATIONS.md`
- Apple design standards & materials: `docs/APPLE_DESIGN_GUIDE.md`
- Swarm coordination & cryptographic identity: `docs/SWARM_COORDINATION_AND_IDENTITY.md`
- First run tutorial: `docs/TUTORIAL-first-run.md`
- Daily workflows: `docs/HOWTO-daily-loop.md`
- Core rules: `ETHOS.md`
- Workflows: `WORKFLOWS.md`
- Product roadmap: `ROADMAP.md`
