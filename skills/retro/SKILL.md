---
name: retro
description: Weekly review; write lessons with expiry dates. Stage 13 of 13.
stage: reflect
version: 0.2.0
---

# kineti-retro

The loop-closer. Once a week (or after any run), turn what happened into
expiring lessons that stages 1–5 will recall automatically.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage retro`.

## Procedure

1. **Collect the period's runs**: state files, progress commits, qa
   reports, security reports, alerts since the last retro.
2. **Ask three questions per run**:
   - What failed, and which earlier stage should have caught it?
   - What was slower or costlier than estimated, and why?
   - What surprised anyone? Surprises are unpriced risk.
3. **Write lessons** (one per insight, max five per retro). Each lesson:
   `{type: learning, skill: <where it applies>, trigger: <when to apply>,
   lesson: <one sentence>, expires: <date>}`. Default expiry 90 days;
   process lessons 180; anything about specific vendors 60.
4. **Update dossiers**: stakeholder agreement changes, new constraints,
   stack facts learned the hard way.
5. **Prune**: list expired lessons, confirm they no longer fire, move them
   to cold storage (never delete).
6. **Extend the project hash chain**: append this retro as a run-record
   carrying the fingerprint of the previous record.
7. **Feed forward**: name the top three lessons and the exact stages they
   change. Next officehours/diagnose/design must quote them.

## Outputs

Lessons in memory (or `.kineti/journal.md` fallback), updated dossiers,
pruned expiries, chained run-record.

## Hard rules

- Every lesson carries an expiry date. Undated lessons are not written.
- Lessons are one sentence, plain words, with a trigger condition.
- Max five per retro — forcing rank forces honesty.

## Memory after

This skill is the memory write path. It always runs last.
