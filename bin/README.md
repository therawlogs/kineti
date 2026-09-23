# Command-Line Tools

These tools manage state, safety, and testing. Run any tool with `bun bin/<name>.ts`.

| Tool | Purpose |
|---|---|
| `kineti-state` | Tracks current step, task, and goal |
| `kineti-spend` | Tracks token usage and enforces the per-project ceiling ($50 default, set at mirror time) |
| `kineti-saga` | Saves undo steps and rolls back changes newest-first |
| `kineti-evidence` | Saves test results with file hashes for proof |
| `kineti-audit` | Write-only hash-chained audit log for price changes and tamper tries |
| `kineti-router` | Plain-talk intent router. No skill or command names needed |
| `kineti-sync` | Encrypted multi-device notes sync. Off by default |
| `kineti-pairing` | Cloud dashboard pairing codes. 10 min life, one use |
| `kineti-verify-gate` | Prevents closing a session if checks fail |
| `kineti-egress` | Logs outbound network requests with cryptographic hashes |
| `kineti-companion` | Starts the local web dashboard for live monitoring |
| `kineti-ci` | Runs all verification checks and builds reports |
| `kineti-mcp` | Runs the Model Context Protocol (MCP) server |
| `kineti-memory-job` | Runs weekly memory cleanups and integrity checks |
| `kineti-epistemic` | Queries the Epistemic Engine and persona beliefs |
| `kineti-invite` | Manages viral peer invites and vanity referral handles |
| `kineti-privacy` | Manages privacy opt-outs and data deletion |
| `kineti-stripe` | Manages virtual payment card issuance and checkout |
| `kineti-swarm` | Coordinates multi-agent peer mesh connections |
| `kineti-models` | Task-based model table (code, plan, chat, fix). Ask-first, auto-switch off by default |
| `kineti-schedule` | Persistent scheduler and watcher jobs (reminders, price and package watchers) |
| `kineti-schema` | Two-pass input standardizer. Fixed fields, memory for repeats, token proof |
