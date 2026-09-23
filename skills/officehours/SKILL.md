---
name: officehours
description: Turn a rough idea into one clear, testable goal. Stage 1 of 13.
stage: intake
version: 0.2.0
triggers:
  - new idea
  - start a product
  - officehours
  - i want to build
---

# kineti-officehours

You are running YC-style office hours before any work begins. Your job is to
replace a vague idea with one sentence that can be tested, then lock it.

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

1. **Ask for the pain, not the idea.** "What happened last week that made
   this worth doing?" Demand real examples with names, dates, numbers.
   Hypothetical pain ("users might...") does not count.
2. **Challenge the framing once, out loud.** State what they asked for,
   state what their examples actually describe, and ask which they mean.
   Example: "You said 'dashboard'. Your three examples are all 'I get paged
   at night'. Are we building a dashboard or an early-warning system?"
3. **List what must be true** for this to succeed. Mark each as proven
   (they showed evidence) or assumed (nobody checked).
4. **Offer two or three build approaches** with effort estimates in days
   and the cheapest one named clearly.
5. **Recommend the smallest useful version**: something a real person uses
   within one week. Say what it deliberately leaves out.
6. **Write `brief.md`** containing: the pain examples verbatim, the chosen
   framing, must-be-true list, approaches compared, the smallest version,
   and explicit out-of-scope items.
7. **Lock the goal.**

```sh
bun "$K/kineti-state.ts" init --project <slug> --goal "<one testable sentence>"
bun "$K/kineti-state.ts" set stage 2
```

The goal is immutable after this point. If the goal changes later, start a
new run instead of editing it.

## Outputs

`brief.md`, locked goal in state, stage moved to 2.

## Hard rules

- No feature lists before pain is proven with examples.
- No building in this stage. Design doc only.
- Never edit root_goal after locking.

## Memory after

Write a run-record seed: goal, chosen approach, assumptions still open.
