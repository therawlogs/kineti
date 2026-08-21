---
name: architecture
description: Draw services, contracts, failures, and limits before code. Stage 4 of 13.
stage: plan
version: 0.2.0
---

# kineti-architecture

Produce the drawing builders follow: what talks to what, in what shape,
what happens when each piece fails, and why this stack beats the others
on numbers.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage architecture`.

## Inputs

`brief.md`, `diagnostics.md` (loss table + constraint), `design/tokens.json`,
and any ceilings already known (latency, budget).

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
7. **Stack choice by math.** Managed blocks (Supabase, Resend, Vercel) are
   candidates, never defaults. For each candidate vs alternative (including
   self-hosting using diagnose's volume numbers): monthly cost at expected
   load, latency added, control/exit cost. Pick winners in writing.
8. **Test matrix**: rows = failure-table rows and state transitions;
   columns = which test proves it. This matrix feeds qa directly.
9. Show the human a summary of choices. On approval:
   `bun "$K/kineti-state.ts" set stage 5`.

## Outputs

`architecture.md` (drawing, contracts, machines, error paths, failure
table, limits, stack math, test matrix).

## Hard rules

- No unbounded work anywhere.
- Stack choices must show their comparison math; "default" is not a reason.
- Every arrow has an error path before this stage ends.

## Memory after

Store chosen stack + reasons; they anchor future run-records.
