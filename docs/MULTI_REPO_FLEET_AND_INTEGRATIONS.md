# Multi-Repository Fleet & Integrations Guide

This guide explains how Kineti manages multiple repositories, connects to GitHub, and automatically attaches to developer coding tools.

---

## 1. Overview

Kineti supports two main ways of working:
1. **Founder and Engineering Lead Flow**: Connect your GitHub account once. Kineti automatically protects all connected repositories, tracks team member activity, enforces spending limits, and provides a single fleet dashboard.
2. **Solo Developer Flow**: Work independently across multiple personal or client projects. Switch between repositories instantly without setting up a team or configuring complex permissions.

---

## 2. Founder & Engineering Lead Flow

### Step 1: Connect GitHub (optional, local-first)
1. Open the companion dashboard at `http://127.0.0.1:8788` (`KINETI_COMPANION_PORT` overrides the port).
2. Click **Settings** → **GitHub** tab. Paste your account name to track it locally.
3. No Marketplace login is required for the local harness. Team data stays in `.kineti/fleet.local.json` (gitignored) — never committed.

### Step 2: Set Up Team and Spending Limits
1. Open the companion dashboard at `http://127.0.0.1:8788`.
2. Click **Settings** in the top navigation bar.
3. In the **Team & Budgets** tab:
   - View your engineering team members.
   - Assign owners to specific repositories using the dropdown menu.
   - Set a spending ceiling in dollars (for example, $30.00 or $50.00) for each repository.

### Step 3: Monitor Fleet Activity
1. Click the **Fleet View** tab in the top navigation bar.
2. View real-time status cards for all connected repositories:
   - Current Git branch
   - Assigned developer
   - Active coding task
   - Connected agent tool (Cursor, Claude Code, Antigravity, or Codex)
   - Live spending meter vs. budget ceiling
   - Verified test count
3. Click **Open Dashboard →** on any card to view that repository's 5 W's analytics.

---

## 3. Solo Developer Flow

Solo engineers can use the exact same dashboard without needing a team setup:
1. Work locally in your project folder. Kineti reads your local `.kineti/` state automatically.
2. Use the **Repository Switcher** in the top bar to move between different local or remote projects.
3. Each project maintains its own isolated:
   - Goal and task state
   - Spending limit ($50.00 default)
   - Undo history
   - Test proofs table

---

## 4. Agent IDE Auto-Latching

Kineti attaches to your existing coding tools without requiring you to switch editors. You can enable or disable connections inside the **Settings** drawer under the **Agent IDEs** tab:

| Tool | Integration Method | What It Tracks |
|---|---|---|
| **Cursor** | `.cursor/mcp.json` | Watches rules, file edits, and terminal commands |
| **Anthropic Claude Code** | CLI hook & skills | Intercepts tool calls and tracks token spending |
| **Google Antigravity** | MCP Sidecar | Syncs state changes and task goals |
| **OpenAI Codex / Operator** | Local MCP socket | Listens on local loop proxy |

When auto-latching is enabled, any developer on your team who starts a session in one of these tools is immediately governed by Kineti rules and spending limits.

---

## 5. Visual Dashboard Features

### Pinned Top Bar
- **Repository Switcher (`[ 􀈕 repo-name ▾ ]`)**: Frosted capsule dropdown with live search to switch between projects.
- **Three-Tab Switcher**:
  - `Dashboard`: The 5 W's single-repository view (Why, What, How, When, Where).
  - `Fleet View`: The multi-repository grid showing all projects.
  - `Logs & Proofs`: The audit table of test results and file mutations.
- **Settings Button (`􀍟 Settings`)**: Slides out the configuration panel.
- **Spend Gauge**: Displays total money spent versus the safety limit.

### Fleet View Grid
- **Summary Cards**: Displays total repositories, total fleet spend, active tasks, and total verified tests across all projects.
- **Repository Cards**: Clean frosted glass cards showing status tags (`active`, `idle`, `action_needed`, or `limit_reached`), spend bars, test counts, and quick inspection buttons.
- **Local-first defaults**: Fresh installs show only the local repo with owner `Local Owner`. Add your own repos in `.kineti/fleet.local.json` (see `bin/kineti-companion.ts:loadLocalFleetOverrides`). Example:
```json
{
  "repos": [{ "id": "my-api", "name": "my-api", "path": "/path/to/my-api", "owner": "Team Member", "branch": "main", "status": "active", "active_task": "Task", "spend_usd": 0, "ceiling_usd": 50, "tests_passing": 0, "ide": "Cursor", "is_local": false }]
}
```

### Settings Slide-Out Sheet
- **GitHub Tab**: View connected GitHub account, toggle automatic repository latching, and check webhook delivery health.
- **Agent IDEs Tab**: Toggle auto-latching on or off for Cursor, Claude Code, Antigravity, and Codex.
- **Team & Budgets Tab**: Change repository owners and update dollar spending limits with instant auto-save.

---

## 6. Companion Server API Reference

The local companion daemon provides HTTP JSON endpoints on port `8788`:

### `GET /api/fleet`
Returns the list of all repositories in the fleet, total fleet spending, and settings.

**Response Example (local-only defaults):**
```json
{
  "active_repo_id": "kineti",
  "repos": [
    {
      "id": "kineti",
      "name": "kineti",
      "path": "/path/to/kineti",
      "owner": "Local Owner",
      "branch": "main",
      "status": "active",
      "active_task": "Local tasks governed by Kineti OS",
      "spend_usd": 0.0,
      "ceiling_usd": 50.0,
      "tests_passing": 0,
      "ide": "Local IDE",
      "is_local": true
    }
  ],
  "total_fleet_spend": 0.0,
  "total_fleet_budget": 50.0,
  "settings": { ... }
}
```

### `POST /api/fleet/select`
Switches the active repository displayed on the main dashboard. Requires `Authorization: Bearer <token from .kineti/auth_token>`.

**Request Body:**
```json
{
  "repo_id": "kineti"
}
```

**Response Example:**
```json
{
  "success": true,
  "active_repo_id": "kineti"
}
```

### `GET /api/settings`
Returns current GitHub, IDE, team, and budget configurations.

### `POST /api/settings`
Updates settings, repository owners, or budget ceilings. Requires `Authorization: Bearer <token>`.

**Request Body:**
```json
{
  "ides": { "cursor": true, "claude_code": true },
  "repo_budgets": { "kineti": 60.0 }
}
```
