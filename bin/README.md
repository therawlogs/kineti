# Command-Line Tools

These tools manage state, safety, and testing. Run any tool with `bun bin/<name>.ts`.

| Tool | Purpose |
|---|---|
| `kineti-state` | Tracks current step, task, and goal |
| `kineti-seed` | Creates missing CI state and spend files from project config |
| `kineti-spend` | Tracks agent-reported costs and checks the local project ceiling ($50 default) |
| `kineti-saga` | Saves undo steps and rolls back changes newest-first |
| `kineti-evidence` | Saves local test results with workspace fingerprints |
| `kineti-audit` | Appends and checks a local hash-linked audit log |
| `kineti-router` | Keyword-based plain-talk response helper |
| `kineti-sync` | Encrypts a manual local export/import file; no cloud sync service |
| `kineti-pairing` | Local budget settings and pairing prototype; no cloud server |
| `kineti-verify-gate` | Runs a locally trusted verification command |
| `kineti-egress` | Records outbound requests when explicitly called; not a network filter |
| `kineti-companion` | Starts the local dashboard; external integrations are not connected by default |
| `kineti-ci` | Checks state and required local evidence; does not issue a certificate |
| `kineti-mcp` | Runs the Model Context Protocol (MCP) server |
| `kineti-memory-job` | Runs weekly memory cleanups and integrity checks |
| `kineti-epistemic` | Queries the Epistemic Engine and persona beliefs |
| `kineti-invite` | Local invite-record prototype; no redemption service |
| `kineti-stripe` | Local card-record demo; does not create or charge real cards |
| `kineti-swarm` | In-memory identity and signing demonstration |
| `kineti-models` | Static task-to-host suggestion table; does not switch models |
| `kineti-schedule` | Local job storage and trigger callbacks; external watchers are not connected |
| `kineti-schema` | Simple text truncation and token estimate; not a multimodal standardizer |
