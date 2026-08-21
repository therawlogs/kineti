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

- `~/.gemini/config/skills/{brainstorm,design,architecture,spec,build,qa,ship}.md`
  are the old v2 pipeline. A DEPRECATED notice file now sits beside them.
  They get replaced by installed `kineti-*` skills at the end of Phase 2,
  not before, so nothing stops working mid-build.
- `~/.opencode/skills/*` (the gstack-derived set) is an independent toolkit.
  Kineti does not remove or modify it. Where both offer the same job
  (review, qa, ship), prefer the `kineti-*` version inside Kineti runs.
- `~/.gemini/config/ETHOS.md` and `WORKFLOWS.md` still hold v2 text.
  They get updated to point at this repository during Phase 2 cutover.
