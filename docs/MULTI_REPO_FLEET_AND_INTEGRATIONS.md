# Multi-Repository Fleet & Integrations Guide

> **v0.4.0 scope:** The Companion is local-only and shows the current checkout. It does not connect GitHub accounts, switch to another repository, monitor repositories in real time, read editor activity, or enforce policy across projects. The workflows below describe supported local behavior unless marked as unavailable.

---

## 1. Overview

The hosted team workflows in this document are not available in v0.4.0. The Companion displays the current local project only; it does not load additional repository entries.

---

## 2. Founder & Engineering Lead Flow

### GitHub account connection

GitHub account connection and repository monitoring are not available.

### Step 2: Set Up Team and Spending Limits
1. Open the companion dashboard at `http://127.0.0.1:8788`.
2. Use the local **Team** or **Settings** view to edit saved owner and budget display values.
3. These values are local preferences. They do not set the project spend limit or enforce a budget on an agent.

### Fleet monitoring

Real-time fleet monitoring, verified-test aggregation, branch monitoring, and editor activity tracking are not available. The Companion displays local state and locally configured entries only.

---

## 3. Solo Developer Flow

Solo engineers can use the local dashboard without a team setup:
1. Work locally in your project folder. Kineti reads your local `.kineti/` state automatically.
2. Run the Companion from the project whose state you want to view.
3. Each project has its own local state directory. Kineti does not sync or enforce policies across those directories.

---

## 4. Agent IDE Auto-Latching

Kineti does not automatically attach to editors or intercept their tool calls. `kineti init` installs instructions and skills for supported hosts; MCP setup is a separate host configuration step.

| Tool | Integration Method | What It Tracks |
|---|---|---|
| **Cursor** | MCP configuration and installed skills | Exposes Kineti tools when the host calls them |
| **Claude Code** | MCP configuration and installed skills | Exposes Kineti tools when the host calls them |
| **Google Antigravity** | Installed skills; MCP setup is host-specific | No automatic sidecar or sync service |
| **OpenAI Codex** | MCP configuration and installed skills | Exposes Kineti tools when the host calls them |

Kineti instructions guide the agent but do not govern every action it can take. Spend entries are reported by the agent; they do not track every provider charge or stop the agent.

---

## 5. Visual Dashboard Features

### Local dashboard
- The home view shows local project state and recorded spend.
- Activity is a view of local records, not a stream of all agent or editor actions.
- Team and Settings views store local labels, contact details, and budget display values.
- The v0.4.0 dashboard displays the current checkout only; it does not support adding fleet entries.

### Settings
The v0.4.0 settings API stores selected local preferences. There is no GitHub connection, auto-latching, webhook status, or service credential storage.

---

## 6. Companion Server API Reference

The local Companion serves HTTP endpoints on `127.0.0.1:8788` by default (`KINETI_COMPANION_PORT` changes the port). The endpoints below expose local state only:

### `GET /api/fleet`
Returns the current local project and local spend/evidence summary. It does not query GitHub or aggregate state across repositories.

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
      "active_task": "Local project",
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
Accepts the current repository ID and returns it unchanged. Other repository IDs return `404`; switching to another repository is not available in v0.4.0. Requires `Authorization: Bearer <token from .kineti/auth_token>`.

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
Returns selected local settings. Connector credentials and a connected GitHub account are not exposed.

### `POST /api/settings`
Updates supported local settings such as repository owner and budget display values. These values do not enforce agent or project spend. Requires `Authorization: Bearer <token>`.

**Request Body:**
```json
{
  "repo_owners": { "kineti": "Local Owner" },
  "repo_budgets": { "kineti": 60.0 },
  "user_name": "Local user"
}
```
