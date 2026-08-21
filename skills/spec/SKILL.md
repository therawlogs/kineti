---
name: spec
description: Remove every ambiguity into typed shapes and pass/fail tests. Stage 6 — hard stop.
stage: contract
version: 0.2.0
---

# kineti-spec

The contract stage. Everything buildable gets written as exact shapes and
binary tests. Then the run stops until a human approves.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage spec`.

## Inputs

`brief.md`, `diagnostics.md`, `architecture.md`, `feasibility.md`,
`design/tokens.json`.

## Procedure

1. **Copy contracts verbatim** from architecture for every arrow. Add the
   validation rules each side enforces (required fields, ranges, lengths).
2. **Data shapes**: every stored object as a typed schema (fields, types,
   nullability, indexes). One block per object.
3. **Acceptance criteria as binary tests.** Each criterion must evaluate
   to pass/fail with numbers: "p95 latency < 300 ms at 50 rps", "recall ≥
   0.92 on dataset X". Forbidden words in criteria: fast, robust, scalable,
   seamless, user-friendly, intuitive.
4. **Stage-by-stage build order**: which spec sections build first and what
   proves each done (link to its acceptance test).
5. **Out-of-scope list**: everything explicitly not built now, copied from
   brief and extended. This list is binding.
6. **Declare the verify command** in the project's `kineti.config.json`
   (`settings.verify_command`) — the command that proves the build is
   green (test runner invocation). The verify-gate uses it from now on.
7. **Write `spec.md`.** Then:
   `bun "$K/kineti-state.ts" set gate.spec pending`

## HARD STOP

Present the human with: spec summary, the acceptance tests, the out-of-scope
list, and the verify command. Ask for explicit approval.

- Approved: `bun "$K/kineti-state.ts" set gate.spec pass && bun "$K/kineti-state.ts" set stage 7`
- Changes requested: revise and re-present. No code exists yet, so changes are cheap — say so.
- Rejected: return to the stage the objection belongs to.

No file under `src/` may exist or be created before approval. If one does,
stop and delete it.

## Outputs

Approved `spec.md`, declared verify command, gate.spec = pass.

## Hard rules

- Ambiguity zero: if a sentence can be argued two ways, it is not done.
- The stop is real. Do not "start scaffolding while waiting".
- Out-of-scope items stay out even if they look easy.

## Memory after

Run-record: spec version, approval date, who approved.
