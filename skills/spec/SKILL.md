---
name: spec
description: Remove every ambiguity into typed shapes and pass/fail tests. Stage 6 — hard stop.
stage: contract
version: 0.5.0
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
3. **Acceptance criteria as observable pass/fail checks.** Use measured
   thresholds when the decision needs them and the evidence supports them.
   Do not invent performance targets or force metrics on projects that do
   not need them. Avoid vague acceptance words such as fast, robust,
   scalable, seamless, user-friendly, or intuitive.
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
7. **Product mode.** At new-project activation, ask the human to select
   `harness` (project workflow skills), `gate` (receipt and protected-action
   checks), or `both`, with no preselected default. Record the choice before
   workflow setup. Installing a package does not activate a project. Legacy
   projects remain readable with selection pending.
8. **Host capability contract.** Use a versioned declaration with separate
   claims for installation, workflow guidance, background behavior,
   natural-language routing, and protected actions. Each claim needs its own
   acceptance evidence. Instruction text is guidance, not enforcement.
9. **Natural-language routing contract.** For each host claiming routing,
   define requests, expected skill, and prohibited accidental route. Test
   with an isolated live-model runner and inspect results. Static text
   checks test instructions only. Unknown hosts and untested routes remain
   unsupported; slash commands may remain optional.
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
- Changes requested: revise and re-present the affected contract sections before new feature implementation.
- Rejected: return to the stage the objection belongs to.

Do not begin new feature implementation before the human approves its
contract. Preserve existing source files; direct bug fixes, reviews, and
cleanup may enter at their relevant workflow step.

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
