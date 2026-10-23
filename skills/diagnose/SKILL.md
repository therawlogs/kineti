---
name: diagnose
description: Gather evidence about project constraints when it helps a decision. Stage 2 of 13.
stage: intake
version: 0.5.0
triggers:
  - diagnose losses
  - friction audit
  - where do we lose money
  - measure the problem
---

# kineti-diagnose

Use diagnosis when project decisions benefit from evidence about delays,
costs, or constraints. It is supporting analysis, not a required business
case for every project.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```

Any web research or external API call in this stage gets recorded first:
`bun "$K/kineti-egress.ts" record --host <host> --description "<what and why>"`
Log model calls with `--stage diagnose`.

## Inputs

`brief.md`, approved `design/storyboard.md` and mock-data shape, approved
data purpose when real inputs are needed, and any systems or evidence the
user chooses to provide.

## Procedure

1. **Check the design and data gates.** Confirm the approved storyboard and
   mock-data shape exist. If absent, do not begin data intake. Real-data use
   requires a separately approved purpose; otherwise use mock data or no data.
2. **Map the process as steps.** Every step is one box: who does it, what
   tool, what triggers the next box. Include manual handoffs as boxes.
3. **Record approved data use.** For every provided dataset, record its owner,
   source, purpose, fields needed, sensitivity, retention/deletion rule, and
   whether it is real or mock in `data-intake.md`. Request only fields needed
   to test the approved storyboard. Get approval for real-data use before
   opening or processing it; otherwise analyze documented metadata only.
4. **Record relevant evidence.** Include wait time, rework, or manual copying
   when those measures apply and are available.
5. **Estimate impact only when appropriate.** State assumptions and show
   calculations for quantitative claims. Leave values unknown when evidence
   is missing; do not force dollar or percentile estimates.
6. **Describe dependencies and possible causes carefully.** Link each claim
   to evidence. Time order alone does not prove cause; label uncertain links
   as hypotheses and include competing explanations when relevant.
7. **Write `diagnostics.md`** with the evidence, constraints, unknowns, and
   data sources that matter to the current decision.
8. Move stage when the analysis is ready: `bun "$K/kineti-state.ts" set stage 3`.

## Approval and acceptance

Human approval of the storyboard/mock-data shape is required before data
intake. Human approval of real-data purpose and scope is required before
processing it. Pass when relevant claims and data sources are inspectable,
unknowns remain visible, and no cause is stated as fact without evidence.

## Outputs

`diagnostics.md`. Relevant findings may be added to project memory with source references.

## Hard rules

- Every quantitative claim shows its evidence and assumptions.
- Do not turn sequence or correlation into a causal claim.

## Memory after

Store: relevant findings, evidence references, and unresolved questions.
