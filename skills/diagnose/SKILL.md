---
name: diagnose
description: Prove in dollars where the business loses time or money today. Stage 2 of 13.
stage: intake
version: 0.2.0
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

`brief.md`, names of systems to inspect, logs/exports the human provides.

## Procedure

1. **Map the process as steps.** Every step is one box: who does it, what
   tool, what triggers the next box. Include manual handoffs as boxes.
2. **Mark the waste.** On each box record: wait time before it starts,
   repeats/rework loops, manual copying between tools.
3. **Convert to dollars (P90, not worst case).** For each waste item:
   `cost = frequency × hours_blocked × loaded_hourly_rate`. Use the 90th
   percentile of observed durations, never the single worst event. Show
   every input number next to every result.
4. **Find the constraint.** The slowest required step caps everything
   downstream. Name it explicitly; improving non-constraints wastes money.
5. **Write cause chains for the top three losses.** Each link needs
   evidence (a log line, a ticket, a measurement). A link supported only by
   guessing scores half confidence. Stop at five links or at a hard
   physical limit, whichever comes first.
6. **Write `diagnostics.md`:** loss table sorted by cost, the constraint,
   three cause chains with confidence per link, data sources used.
7. Move stage: `bun "$K/kineti-state.ts" set stage 3`.

## Outputs

`diagnostics.md`. Loss table also stored to memory (dossier).

## Hard rules

- Every dollar figure shows its math inline.
- No solutions proposed in this stage. Measurement only.
- P90 bounds, not maximums — hyperbole kills credibility at the gate.

## Memory after

Store: loss table, constraint name, cause chains. These feed the
feasibility gate's money math later.
