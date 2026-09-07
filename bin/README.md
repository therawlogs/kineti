# Command-Line Tools

These tools manage state, safety, and testing. Run any tool with `bun bin/<name>.ts`.

| Tool | Purpose |
|---|---|
| `kineti-state` | Tracks current step, task, and goal |
| `kineti-spend` | Tracks token usage and enforces the $50 spend limit |
| `kineti-saga` | Saves undo steps and rolls back changes newest-first |
| `kineti-evidence` | Saves test results with file hashes for proof |
| `kineti-verify-gate` | Prevents closing a session if checks fail |
| `kineti-egress` | Logs outbound network requests with cryptographic hashes |
| `kineti-companion` | Starts the local web dashboard for live monitoring |
| `kineti-ci` | Runs all verification checks and builds reports |
| `kineti-mcp` | Runs the Model Context Protocol (MCP) server |
| `kineti-memory-job` | Runs weekly memory cleanups and integrity checks |
