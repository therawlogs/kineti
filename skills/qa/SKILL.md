---
name: qa
description: Prove it works for a human at three screen sizes. Stage 9 of 13.
stage: verify
version: 0.2.0
triggers:
  - test the app
  - qa
  - browser test
  - does it work
---

# kineti-qa

Drive the real application in a real browser at phone, tablet, and desktop
widths. Every bug found gets fixed and gains a permanent regression test.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage qa`.

## Setup

1. Start the preview server (project's documented dev command).
2. Confirm the spec's acceptance-test list is the test plan.
3. Screenshots directory: `design/screenshots/` (gitignored artifacts).

## Procedure

1. **Walk every main path** from the spec at three widths:
   phone 390px, tablet 820px, desktop 1440px. A path passes only if it
   passes at all three.
2. **On every failure**: screenshot first (`design/screenshots/<slug>.png`),
   then diagnose root cause — never fix the symptom.
3. **Fix → regression test → re-run.** A bug is not fixed until a
   permanent automated test covers it and passes. The test name mentions
   the bug.
4. **Self-repair limit**: five attempts per bug maximum. Still red after
   five → stop, escalate to the human with the five approaches tried.
5. **Record proof records as you go**:
   `bun "$K/kineti-evidence.ts" run --label qa -- -- <test command>`
6. **Write `qa-report.md`**: paths tested per width, bugs found/fixed,
   escalation list, screenshots index.
7. **Exit gate**: before leaving this stage every label must be fresh:
   `bun "$K/kineti-evidence.ts" check --label qa` → FRESH required.
   Then `bun "$K/kineti-state.ts" set stage 10`.

## Outputs

Green paths at three widths, regression tests, `qa-report.md`,
FRESH qa proof records.

## Hard rules

- Five self-repair attempts max per bug, then escalate. No sixth try.
- No bug closed without a regression test.
- Report-only mode: add `--report-only` to skip fixes entirely and only
  produce the report.

## Memory after

Run-record: bug classes found; recurring classes become build-stage checks.
