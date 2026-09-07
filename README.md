# Kineti OS v3

A closed-loop software factory that runs inside your AI coding tools.
You describe an idea once. Kineti walks it through thirteen fixed stages,
enforced by small programs, remembered by a permanent store.

**Skills propose. Programs enforce. Memory remembers.**

## Install (30 seconds)

Requires: git, bash. No other dependencies for install.

```sh
git clone <this-repo> ~/kineti && cd ~/kineti && ./setup.sh
```

The installer finds which of these you use and copies skills into each:

| Host | Skills copied to |
|---|---|
| opencode | `~/.opencode/skills/kineti-*` |
| Claude Code | `~/.claude/skills/kineti-*` |
| Gemini CLI | `~/.gemini/config/skills/kineti-*` |
| ChatGPT Codex | `${CODEX_HOME:-~/.codex}/skills/kineti-*` |

Target one host only: `./setup.sh --host opencode`
Remove everything Kineti installed: `./setup.sh --uninstall` (touches only `kineti-*` files)
Re-running is always safe: it overwrites its own copies and nothing else.

## Use

Open any project in your agent tool and run the first skill:

```
Load kineti. Run /kineti-officehours
```

Then follow the loop in WORKFLOWS.md. Three gates stop for you:
feasibility (stage 5), spec approval (stage 6), ship (stage 11).

## Universal Host Integration & Visual Companion

### 1. Visual Companion Canvas (Port 8788)
Run the live companion dashboard sidecar to observe pipeline stage transitions, real-time spend counters, and 1-click gate approvals:
```sh
bun run companion
```
Visit `http://127.0.0.1:8788` in your browser.

### 2. Universal Model Context Protocol (MCP)
Connect any MCP-compatible AI agent (Cursor, Claude Desktop, Antigravity, OpenCode):
```sh
# Auto-configure Cursor (.cursor/mcp.json)
bun run mcp:init

# Start stdio MCP daemon
bun run mcp
```

### 3. GitHub Actions Causal Verification Gate
Every pull request is automatically verified with cryptographic evidence checks, spend budget monitoring, and status badges via `.github/workflows/kineti-gate.yml`:
```sh
# Run verification check locally
bun run ci
```

## What lives where

- This repository: source of everything (skills, installer, programs, UI components, docs).
- Your computer only: installed copies under each host's skills folder,
  per-project `.kineti/` state, `~/.kineti/` machine data, gbrain data.

Deleting this folder loses nothing permanent. Re-clone and re-run setup.sh.

## Status

- [x] Phase 0 — skeleton + four-host installer
- [x] Phase 1 — harness programs (`bin/`) & cryptographic hardening
- [x] Phase 2 — visual companion canvas (`bin/kineti-companion.ts`) & 12 UI components (`src/components/ui/`)
- [x] Phase 3 — universal MCP server (`bin/kineti-mcp.ts`) & GitHub Actions PR verification gate (`.github/workflows/kineti-gate.yml`)
- [x] Phase 4 — memory wiring (gbrain + causal rules) & v3.0.0 release

Guides: docs/TUTORIAL-first-run.md · docs/HOWTO-daily-loop.md
Strategy Blueprint: docs/HARNESS_STRATEGY_BLUEPRINT.md
Audit Report: docs/BLUEPRINT_AUDIT_REPORT.md
Roadmap: ROADMAP.md
Memory contract: MEMORY.md
Migration notes: MIGRATION.md

