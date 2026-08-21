---
name: ship
description: Clean commits and a pull request, proof-gated. Stage 11 of 13.
stage: ship
version: 0.2.0
---

# kineti-ship

The release engineer. Ship refuses to run on stale proofs, failed
security, or an unapproved spec — the checks are the point.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage ship`.

## Pre-flight (all must pass, in order)

```sh
bun "$K/kineti-state.ts" get gate.spec      # must be pass
bun "$K/kineti-state.ts" get gate.security  # must be pass
bun "$K/kineti-evidence.ts" check --label qa          # FRESH or stop
bun "$K/kineti-evidence.ts" check --label review      # FRESH or stop
```
Any STALE/MISSING → go back to the stage that owns it. Never re-stamp a
proof without re-running its command:
`bun "$K/kineti-evidence.ts" run --label <lane> -- -- <command>`

Final gate check:
`bun "$K/kineti-verify-gate.ts"` → must exit 0.

## Procedure

1. **Sync base branch**: fetch, rebase or merge main as project convention.
2. **Full suite once more** through evidence:
   `bun "$K/kineti-evidence.ts" run --label ship -- -- <verify command>`
3. **Fold progress commits**: squash `WIP:` commits into clean commits per
   spec section. Keep non-WIP commits intact so history stays bisectable.
   Message style: imperative subject, body with what and why.
4. **Coverage delta**: report test coverage change vs main; new code needs
   tests or a written reason.
5. **Open the pull request**: summary from brief + spec, test results,
   coverage delta, security-report link, screenshots index from qa.
6. **Never push straight to main.** The PR is the deliverable. If this
   repo has no review process and the human says merge, merge — but only
   after they say it.
7. Record deploy target if the project auto-deploys on merge, then:
   `bun "$K/kineti-state.ts" set stage 12`.

## Outputs

Clean commits, pull request URL, FRESH ship proof record, stage 12.

## Hard rules

- Refuses STALE or MISSING proofs. Re-running the command is the only cure.
- Never pushes directly to main.
- No "small extra fixes" smuggled in during squashing.

## Memory after

Run-record: PR url, commit range, coverage delta, time taken.
