# Migration Guide

This document lists changes between version 2 and version 3.

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

Old version 2 files are preserved in `legacy/v2-skills/`.

## Local Configuration Updates

- **OpenCode Skills**: Old skills in `~/.opencode/skills/` were removed so only current `kineti-*` skills appear. Backups are saved in `~/.kineti/backups/`.
- **OpenCode Config**: Removed duplicate slash commands from `~/.config/opencode/opencode.jsonc`. Agents use skills directly.
- **Gemini / Antigravity Skills**: Kept in sync in `~/.gemini/config/skills/`.
- **Core Directives**: `ETHOS.md`, `WORKFLOWS.md`, and `MEMORY.md` are copied to `~/.gemini/config/` for Antigravity.
- **Antigravity Cache**: Antigravity runtime files in `~/.gemini/antigravity/` remain unchanged.
