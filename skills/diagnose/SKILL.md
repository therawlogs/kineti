---
name: diagnose
description: Prove in dollars where the business loses time or money today. Stage 2 of 13.
stage: intake
version: 0.4.0
triggers:
  - diagnose losses
  - friction audit
  - where do we lose money
  - measure the problem
---

# kineti-diagnose

Before designing anything, measure the wound. This stage converts messy
complaints into a loss table a CFO would accept, and finds the bottleneck.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```

Any web research or external API call in this stage gets recorded first:
`bun "$K/kineti-egress.ts" record --host <host> --description "<what and why>"`
Log model calls with `--stage diagnose`.

## Inputs

`brief.md`, approved `design/storyboard.md` and mock-data shape, names of
systems to inspect, logs/exports the human provides. Intake is downstream of
the design approval; before that, use only explicitly labeled mock examples.

## Procedure

1. **Check the intake gate.** Confirm the human-approved storyboard and
   mock-data shape exist. If either is absent, stop and route back to
   `kineti-design`; do not request, inspect, or import real data.
2. **Map the process as steps.** Every step is one box: who does it, what
   tool, what triggers the next box. Include manual handoffs as boxes.
3. **Record data intake.** For every provided dataset, record its owner,
   source, purpose, fields needed, sensitivity, retention/deletion rule, and
   whether it is real or mock in `data-intake.md`. Request only fields needed
   to test the approved storyboard. Get approval for real-data use before
   opening or processing it; otherwise analyze documented metadata only.
4. **Mark the waste.** On each box record: wait time before it starts,
   repeats/rework loops, manual copying between tools.
5. **Convert to dollars (P90, not worst case).** For each waste item:
   `cost = frequency × hours_blocked × loaded_hourly_rate`. Use the 90th
   percentile of observed durations, never the single worst event. Show
   every input number next to every result.
6. **Find the constraint.** The slowest required step caps everything
   downstream. Name it explicitly; improving non-constraints wastes money.
7. **Write cause chains for the top three losses.** Each link needs
   evidence (a log line, a ticket, a measurement). A link supported only by
   guessing scores half confidence. Stop at five links or at a hard
   physical limit, whichever comes first.
8. **Write `diagnostics.md`:** loss table sorted by cost, the constraint,
   three cause chains with confidence per link, data sources used.
9. Move stage: `bun "$K/kineti-state.ts" set stage 3`.

## Approval and acceptance

Human approval of the storyboard/mock-data shape is a prerequisite. Human
approval of real-data purpose and scope is required before processing real
data. Pass when every intake item has provenance, purpose, sensitivity,
retention, and real/mock status, and `diagnostics.md` identifies its data
sources. Missing approval or metadata means stop, not infer consent.

## Outputs

`diagnostics.md`. Loss table also stored to memory (dossier).

## Hard rules

- Every dollar figure shows its math inline.
- No solutions proposed in this stage. Measurement only.
- P90 bounds, not maximums — hyperbole kills credibility at the gate.

## Memory after

Store: loss table, constraint name, cause chains. These feed the
feasibility gate's money math later.
