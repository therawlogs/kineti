---
name: feasibility
description: Kill losing plans before code exists — money, data, people checks. Stage 5 gate.
stage: gate
version: 0.2.0
triggers:
  - feasibility
  - viability check
  - should we build this
  - gate check
---

# kineti-feasibility

The cheapest place to kill a bad plan is before any code exists. Three
checks; all must pass or the run returns to diagnosis with reasons.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage feasibility`.

## Inputs

`brief.md`, `diagnostics.md`, `architecture.md` (stack math), sample data
or honest data documents, names of the humans involved.

## Procedure

### Check A — Money
1. Cost per user per month from architecture stack math (include the
   expansion ratio: system tokens vs user-visible tokens).
2. Revenue or savings per user per month from diagnose's loss table.
3. Gross margin = (value − cost) / value.
4. Build cost estimate (days × loaded rate) and payback months.
5. Return = first-year value / build cost. Hurdle: config
   `settings.hurdle_rate_min` (default 0.5). Below hurdle → FAIL with the
   gap stated in dollars.
6. If architecture compared API bills vs owned hardware: state breakeven
   month explicitly.

### Check B — Data
For every field the design depends on, from real samples:
- Completeness: share of rows where the field is present and non-fake
  ("N/A", "99999", empty count as fake).
- Accuracy: share passing basic reality rules (ranges, formats).
- Freshness: age versus how fast the fact changes.
Quality = completeness × accuracy × freshness. Threshold: config
`settings.data_quality_threshold` (default 0.8). Any required field below
threshold → FAIL, naming the field and its score.

### Check C — People
List every stakeholder who can stop this: role, power 0–1 (can they kill
budget or access?), agreement −1 to +1 (champion to blocker). Anyone with
power ≥ 0.7 and agreement ≤ 0 becomes a **mitigation task** with a named
owner and an artifact that addresses their specific fear (security chief →
data-flow diagram with redaction points; CFO → this file's money section).

## Verdict

Write `feasibility.md`: each check PASS/FAIL with its numbers, mitigation
tasks, and one-line verdict.

- All pass:
  `bun "$K/kineti-state.ts" set gate.feasibility pass && bun "$K/kineti-state.ts" set stage 6`
- Any fail:
  `bun "$K/kineti-state.ts" set gate.feasibility fail && bun "$K/kineti-state.ts" set stage 2`
  then tell the human exactly which number killed it.

## Hard rules

- No code may be written in this stage, including prototypes.
- A fail is a result, not a shame: record which number failed and why.
- Never round a failing number up to pass.

## Memory after

Store stakeholder matrix and verdict numbers in the project dossier.
