# Migration Guide (v2 → v3.1.0)

This document lists changes between version 2 and version 3. Current version: `3.1.0` (`package.json`, `kineti.config.json`). Skill packs are versioned separately (`0.2.x`).

## Skill Changes

| Old Skill | New Skill | Description |
|---|---|---|
| `skills/brainstorm.md` | `skills/officehours` + `skills/diagnose` | Clarify problem and measure financial impact |
| `skills/design.md` | `skills/design` | Reference-based visual design |
| `skills/architecture.md` | `skills/architecture` | Service diagrams and failure handling |
| `skills/spec.md` | `skills/spec` + `skills/feasibility` | Feasibility check followed by specification approval |
| `skills/build.md` | `skills/build` | Small verified edits with undo steps |
| `skills/qa.md` | `skills/qa` + `skills/review` + `skills/security` | Multi-screen testing, code review, and security checks |
| `skills/ship.md` | `skills/ship` + `skills/watch` + `skills/retro` + `skills/learn` | Verified commits, monitoring, and weekly reviews |

Old version 2 files were preserved in `legacy/v2-skills/` at migration time (not shipped in this repo).

## Local Configuration Updates (what setup.sh actually does)

- **Skills install**: Copies `skills/*/SKILL.md` to each detected host as `kineti-*`. No backups are made — back up `~/.claude/skills/kineti-*` etc by hand if you customized them.
- **OpenCode Config**: `setup.sh` does not edit `~/.config/opencode/opencode.jsonc`. Remove duplicates by hand if needed.
- **Gemini / Antigravity Skills**: Kept in sync in `~/.gemini/config/skills/` when that folder exists.
- **Core Directives**: `ETHOS.md`, `WORKFLOWS.md`, `MEMORY.md` stay in the repo. Copy them to `~/.gemini/config/` by hand if you want them global.
- **Antigravity Cache**: Antigravity runtime files in `~/.gemini/antigravity/` remain unchanged.
- **Cursor**: New in 3.1. Skills go to `~/.cursor/skills/kineti-*`, project hook to `.cursor/rules/kineti.mdc`. MCP example in `.cursor/mcp.example.json`.
