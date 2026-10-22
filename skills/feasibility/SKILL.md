---
name: feasibility
description: Check project limits and material risks before implementation. Stage 5 of 13.
stage: gate
version: 0.5.0
triggers:
  - feasibility
  - viability check
  - should we build this
  - gate check
---

# kineti-feasibility

Use a feasibility review when cost, data, people, or operational limits could
change the user's decision. It is not a mandatory business case for every
project. Keep evidence and unknowns visible, and route a blocked decision to
the step that can resolve it.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage feasibility`.

## Inputs

The approved brief, design, data decision, architecture, and any limits that
matter to this project. Use synthetic data by default. Real-data approval is
separate and must exist before processing real inputs.

## Procedure

1. Identify the decision this review supports and the factors that could
   change it.
2. Compare relevant costs, resource use, limits, failure behavior, and exit
   costs. Show calculations only where they improve the decision, and state
   their evidence and assumptions. Do not invent revenue, rates, savings,
   or fixed financial hurdles.
3. If real data is needed, verify its approved purpose, scope, provenance,
   sensitivity, retention, and deletion plan. Otherwise use the no-data path
   or synthetic fixtures.
4. Identify people or approvals that actually affect access or outcomes.
   Record roles and unresolved decisions without requiring personal names,
   numeric power scores, or invented agreement.
5. Write `feasibility.md` with supported findings, unknowns, risks, and the
   decision that remains for the user. Mark a factor not applicable when it
   does not affect this project.

## Verdict

Use `PASS`, `FAIL`, or `UNSUPPORTED` for each relevant factor and explain the
evidence. A missing resource or unmeasured claim is unknown, not a pass.
Keep a blocked issue limited to the work that depends on it.

## Hard rules

- Do not process real data without separate purpose approval.
- Do not silently select a paid service or alter the user's approved scope.
- Do not turn time order or dependency into a causal claim.
- This stage does not block direct bug fixes, reviews, or cleanup that do not
  depend on a feasibility decision.

## Memory after

Store relevant findings with their source references and unresolved choices.
