---
name: officehours
description: Turn a rough idea into one clear, testable goal. Stage 1 of 13.
stage: intake
version: 0.5.0
triggers:
  - new idea
  - start a product
  - officehours
  - i want to build
---

# kineti-officehours

Help the user turn a request into a clear goal and a useful project record.
Use this workflow for a new project; direct fixes and reviews can enter at
the relevant step.

## Harness (before anything)

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```

If `.kineti/state.json` does not exist yet, you are starting the run.
Log every model call: `bun "$K/kineti-spend.ts" log --stage officehours --tokens-in N --tokens-out N`

## Memory first

Recall before asking: past runs with similar words, lessons about this
problem area, existing project dossiers. If gbrain MCP is connected use
memory verbs; otherwise read `<project>/.kineti/journal.md` if present.

## Procedure

1. **Select the project mode** at activation and show three numbered choices:
   (1) `harness`, (2) `gate`, or (3) `both`. Do not preselect one. A canceled
   selection leaves the project inactive; legacy projects remain readable
   with mode selection pending.
2. **Understand the request.** Ask for useful examples and evidence when
   available. Record uncertainty instead of inventing people, business
   models, measurements, or commercial goals. Noncommercial goals are valid.
3. **Record the goal and discovery.** Capture personas and their needs,
   initial feature ideas, app model, responsibilities, constraints, and
   unanswered questions. Mark what is known, reported, or assumed.
4. **Compare approaches when it helps the decision.** Include relevant
   tradeoffs and rough effort or cost ranges where evidence supports them.
   Do not force a fixed number of approaches or a one-week delivery target.
5. **Write `brief.md`** with the agreed framing, evidence, assumptions,
   scope, and unresolved questions.
6. **Lock the goal only for a newly activated run.**

```sh
bun "$K/kineti-state.ts" init --project <slug> --mode <harness|gate|both> --goal "<one testable sentence>"
bun "$K/kineti-state.ts" set stage 2
```

The goal is immutable after this point. If the goal changes later, start a
new run instead of editing it.

## Approval and acceptance

Show the goal, discovery, personas or intended users, app model, constraints,
and assumptions. The human approves a new run's goal before it is locked.
Pass when the goal is understandable, important unknowns are visible, and
the app model assigns relevant work to the user, client, server, or AI.

## Outputs

`brief.md`, locked goal in state, stage moved to 2.

## Hard rules

- Keep early feature ideas labeled as ideas until they are evaluated.
- No building in this stage. Design doc only.
- Never edit root_goal after locking.
- Personas and app model are hypotheses unless supported by user evidence.

## Memory after

Write a run-record seed: goal, chosen approach, assumptions still open.
