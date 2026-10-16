---
name: spec
description: Remove every ambiguity into typed shapes and pass/fail tests. Stage 6 — hard stop.
stage: contract
version: 0.4.0
triggers:
  - write the spec
  - specification
  - acceptance criteria
  - lock the contract
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
6. **Continuity probe (Workflow 07).** Compare the request, proposed
   changes, and acceptance criteria with the locked `root_goal`, approved
   `brief.md`, and current spec. If scope or outcomes differ, write
   `contract-probe.md` with the exact matching and differing statements,
   consequences for cost/data/approval, and numbered options: continue
   within contract, revise the spec within the same goal, or stop and start
   a new run for a genuinely different goal. Explain the options in plain
   language and wait for the human's explicit choice. Never edit the locked
   goal or silently reinterpret it. Record the decision in the spec/run
   record.
7. **Product mode (Workflow 11).** Before packaging or setup, ask the human
   to select one: `harness` (project workflow skills), `gate` (receipt and
   protected-action checks), or `both`. Record the selection and included
   capabilities in `spec.md`; do not infer a mode from host/editor choice.
8. **Adapter support contract (Workflow 12).** List adapters as
   `supported`, `stub`, or `unsupported`, with a tested capability and
   acceptance evidence for every supported claim. An adapter runs in the
   background and adds its declared capability; it does not replace the
   user's foreground editor/agent. Until tested, GitHub Action, Codex,
   Claude Code, and other adapters are stubs/unsupported.
9. **Natural-language routing contract (Workflow 13).** For each supported
   host, define natural-language example requests, expected skill, and
   prohibited accidental route. Test routing without slash commands. A
   host passes only with inspectable results; unknown hosts and untested
   routes remain unsupported.
10. **Declare the verify command** in the project's `kineti.config.json`
   (`settings.verify_command`) — the command that proves the build is
   green (test runner invocation). The verify-gate uses it from now on.
11. **Write `spec.md` and project-local planning docs.** Keep
   `docs/` inside the project; include a concise architecture/spec pointer,
   an owned roadmap, and a prioritized list with acceptance tests and
   dependencies. Preserve the approved out-of-scope list. Then:
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

Approved `spec.md`, `contract-probe.md` when a mismatch was found,
`docs/` architecture/spec pointers and prioritized roadmap, selected
product mode, adapter support table, natural-language routing test matrix,
declared verify command, gate.spec = pass.

## Approval and acceptance

Pass only when the human has selected a product mode, every contract
mismatch has an explicit decision, each priority has an owner or a recorded
unassigned status plus an acceptance test, and adapter/routing claims have
evidence or are labeled unsupported. A missing selection or unresolved
mismatch keeps the spec gate pending.

## Hard rules

- Ambiguity zero: if a sentence can be argued two ways, it is not done.
- The stop is real. Do not "start scaffolding while waiting".
- Out-of-scope items stay out even if they look easy.

## Memory after

Run-record: spec version, approval date, who approved.
