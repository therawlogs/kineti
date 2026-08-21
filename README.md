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

## What lives where

- This repository: source of everything (skills, installer, programs, docs).
- Your computer only: installed copies under each host's skills folder,
  per-project `.kineti/` state, `~/.kineti/` machine data, gbrain data.

Deleting this folder loses nothing permanent. Re-clone and re-run setup.sh.

## Status

- [x] Phase 0 — skeleton + four-host installer
- [ ] Phase 1 — harness programs (`bin/`)
- [ ] Phase 2 — full pipeline skills
- [ ] Phase 3 — memory wiring (gbrain + rules)
- [ ] Phase 4 — polish, v3.0.0 tag

See MIGRATION.md for what replaced the old v2 files.
