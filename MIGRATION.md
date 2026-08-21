# MIGRATION.md — What replaced the old files

## Inside this repository (v2 → v3)

| Old | New |
|---|---|
| skills/brainstorm.md | skills/officehours + skills/diagnose |
| skills/design.md | skills/design (rebuilt: taste-driven, references first) |
| skills/architecture.md | skills/architecture (rebuilt: failure tables, bounded work, stack math) |
| skills/spec.md | skills/spec (unchanged hard stop) + new skills/feasibility gate before it |
| skills/build.md | skills/build (adds undo registration + progress commits) |
| skills/qa.md | skills/qa (proof-bound) + new skills/review, skills/security |
| skills/ship.md | skills/ship (proof-gated) + new skills/watch, skills/retro, skills/learn |

Old v2 skill files are archived untouched in `legacy/v2-skills/`.

## On your computer

- `~/.opencode/skills/` held the old gstack-derived skill set (35 items:
  officehours, autoplan, plan-*-review, qa, ship, design-*, freeze, ...).
  On user instruction these were **removed** on 2026-08-21 so every job
  appears exactly once, as `kineti-*`. Backup:
  `~/.kineti/backups/pre-v3-cleanup-*.tar.gz`.
- Stale v0.1 docs `~/.opencode/OPENCODE.md` and
  `~/.opencode/guide_to_kinetios.md` removed (same backup).
- `~/.gemini/config/skills/{brainstorm,design,...}.md` and their folders
  (v2 pipeline) were removed at the end of Phase 2; archived in
  `legacy/v2-skills/` here.
- `~/.config/opencode/opencode.jsonc`: the hand-written `command` block
  was removed — skills with triggers are the only interface now, which
  kills the duplicate `/officehours`-style entries. Skills paths point at
  `~/.opencode/skills` only; gemini/Antigravity read
  `~/.gemini/config/skills` natively.
- `~/.gemini/config/{ETHOS,WORKFLOWS,MEMORY}.md` are live v3 copies kept
  in sync for the gemini/Antigravity host.
- `~/.gemini/antigravity/` is Antigravity's own runtime data (binaries,
  MCP servers) — not a Kineti item; untouched.
