---
name: watch
description: Watch the live system for errors and speed regressions. Stage 12 of 13.
stage: ship
version: 0.2.0
triggers:
  - watch production
  - monitor the deploy
  - any regressions
---

# kineti-watch

The site reliability pass. Baseline before deploy, compare after, surface
alerts on the next session.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage watch`.

## Procedure

1. **Baseline (before deploy)**: record page load time, core web vitals,
   and error rate of the current production or staging URL into
   `.kineti/baseline.json`. Without a baseline, comparison is opinion.
2. **After deploy**, poll the live URL:
   - HTTP status and error rate over a sample window.
   - Page load and vitals versus baseline; flag regressions over 20%.
   - Console errors on the main paths.
3. **Write alerts** to `~/.kineti/alerts.log` — one line each:
   `<date> <project> <severity> <what regressed> <numbers>`.
   Alerts surface automatically at the start of the next session.
4. **Severity**: red = users blocked (errors, broken path); yellow =
   degradation within tolerance; info = noteworthy trend.
5. **Recovery check**: if red, propose rollback using the saga undo steps
   from the build run, or the platform's rollback command. The human
   approves rollbacks of production.
6. When stable: `bun "$K/kineti-state.ts" set stage 13`.

## Outputs

`baseline.json`, alert lines, stability verdict.

## Hard rules

- No baseline → take one before judging anything.
- Production rollbacks need human approval.
- Alerts are numbers with dates, not adjectives.

## Memory after

Append incident/regression notes to the project dossier.
