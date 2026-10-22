---
name: design
description: Find the product look from your references, not presets. Stage 3 of 13.
stage: plan
version: 0.5.0
triggers:
  - design the look
  - mockups
  - style brief
  - make it pretty
---

# kineti-design

Design from the user's needs and any useful references. Keep the storyboard
and mock-data shape visible before architecture or data intake.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage design`. Record egress before fetching any
reference site.

## Procedure

1. **Collect references when they help.** Ask for examples only when useful
   to resolve a design question. Record the traits that matter; do not
   require a fixed number of references.
2. **Write the style brief** (one page): audience, target feeling in three
   adjectives chosen with the human, density, color rules with hex values,
   font pairing, motion rules, and what the design must never do.
   Preset style archetypes may be offered here as starting points only —
   the references override them everywhere they disagree.
   **Get explicit approval of this brief before generating anything.**
3. **Storyboard the user path.** Create `design/storyboard.md`
   with the entry point, each screen/state in order, user action, visible
   result, error/empty state, and responsible user/client/server/AI actor.
   Mark proposed data as mock data and identify fields; do not collect or
   import real data yet. Get human approval of the storyboard and mock-data
   shape before Workflow 03 intake.
4. **Make proportionate design options** when comparison would help. They
   may be rendered HTML, sketches, or another inspectable form. Record the
   chosen design and alternatives considered; do not force a fixed count or
   choose a "least-bad" option when none is accepted.
5. **Record useful decisions** to the project record, including the selected
   design and reasons. Do not invent user approval.
6. **Produce design tokens** when the project needs them. Keep them tied to
   the approved design.
7. Move stage when the design decision is ready: `bun "$K/kineti-state.ts" set stage 4`.

## Approval and acceptance

The human approves the storyboard and mock-data shape before Workflow 03
intake begins. Pass when the storyboard covers the main journey and relevant
empty/error states, mock fields are explicit, and the selected design is
recorded. Any data intake requested before storyboard/mock approval must
stop and return to this step.

## Outputs

Approved storyboard and mock-data shape, selected design when applicable,
and design tokens when the project needs them.

## Hard rules

- No component code before a mockup wins.
- No real-data intake before the storyboard and mock-data shape are approved.
- The 12-component UI library is fallback vocabulary for when nothing
  better fits — never a mandatory checklist.
- Keep design decisions and unresolved choices inspectable.

## Memory after

Taste lessons written; winning traits noted in run-record.
