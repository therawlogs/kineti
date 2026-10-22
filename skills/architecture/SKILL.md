---
name: architecture
description: Draw services, contracts, failures, and limits before code. Stage 4 of 13.
stage: plan
version: 0.5.0
triggers:
  - architecture
  - system design
  - contracts
  - how should this be built
---

# kineti-architecture

Produce the drawing builders follow: what talks to what, in what shape,
what happens when each piece fails, and why selected tools fit the project.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage architecture`.

## Inputs

`brief.md`, approved design and data decisions, relevant diagnostics when
available, and known project limits.

## Feature discovery and tool selection

Before drawing services, refine feature ideas into accepted scope linked to
user needs, outcomes, and tests. Compare suitable tools and approaches when
the choice matters, including local or open-source options where relevant.
Record relevant costs, limits, data access, failure behavior, and exit cost.
Keep the user's choices visible; do not silently select paid services.

## Procedure

1. **Data-flow drawing** (mermaid in `architecture.md`). Every box is one
   deployable service or store. Every arrow is one contract. Number every
   arrow — later steps refer to arrows by number.
2. **Contracts per arrow**: exact JSON request/response shapes with types.
   Store them as named blocks; the spec stage copies them verbatim.
3. **State machines** for anything with status (order, task, approval,
   subscription): list every allowed transition, its trigger, and who may
   fire it. Transitions not listed are forbidden.
4. **Error paths per arrow**: timeout value, retry policy with backoff,
   what the caller shows the user on failure. An arrow without an error
   path is incomplete.
5. **Failure table**: one row per component — how it fails | how we detect
   it | what the user sees | recovery step. Include the money tracker and
   undo stack as components.
6. **Bounded execution**: every loop, external call, queue, and agent run
   gets explicit time and size limits. Write them next to the component.
   Nothing unbounded anywhere.
7. **Tool choice.** Record why selected tools fit and relevant alternatives,
   costs, limits, failure behavior, and exit cost. Use measured calculations
   when making quantitative claims; do not require financial math where it
   does not help the decision.
8. **Test matrix**: rows = failure-table rows and state transitions;
   columns = which test proves it. This matrix feeds qa directly.
9. Show the human a summary of choices. On approval:
   `bun "$K/kineti-state.ts" set stage 5`.

## Outputs

`architecture.md` (drawing, contracts, machines, error paths, failure
table, limits, relevant tool comparisons, test matrix, and feature-to-outcome
map).

## Approval and acceptance

Pass when accepted outcomes map to features and tests, scope choices are
recorded, selected tools have a reason and relevant comparison, and the
architecture test matrix covers important failure paths and state
transitions. The human approves the summary before implementation.

## Hard rules

- No unbounded work anywhere.
- Quantitative claims must show their evidence and assumptions.
- Every arrow has an error path before this stage ends.

## Memory after

Store chosen stack + reasons; they anchor future run-records.
