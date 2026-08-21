---
name: design
description: Find the product look from your references, not presets. Stage 3 of 13.
stage: plan
version: 0.2.0
triggers:
  - design the look
  - mockups
  - style brief
  - make it pretty
---

# kineti-design

Taste-driven design. The human's references decide the look; mockups are
real HTML shown side by side; every pick and rejection is remembered.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage design`. Record egress before fetching any
reference site.

## Procedure

1. **Collect references.** Ask the human for three to five products or
   sites whose look they like, and one they hate. For each, extract
   concrete traits: spacing rhythm, information density, color temperature,
   type pairing, corner language, motion personality. Two lines each.
2. **Write the style brief** (one page): audience, target feeling in three
   adjectives chosen with the human, density, color rules with hex values,
   font pairing, motion rules, and what the design must never do.
   Preset style archetypes may be offered here as starting points only —
   the references override them everywhere they disagree.
   **Get explicit approval of this brief before generating anything.**
3. **Generate four to six variants** of the single most important screen as
   real HTML files (inline CSS, no build step) under `design/screens/`.
   Variants must differ on at least two brief dimensions each, not just
   recolors. Name them `variant-1.html` ... `variant-6.html`.
4. **Show a comparison page**: a simple `design/screens/compare.html` that
   iframes all variants side by side with numbered labels. Open it for the
   human.
5. **Record every verdict** to memory as a taste lesson:
   `{type: learning, topic: design-taste, kept: <variant>, rejected: [...],
   traits credited, expires: +90d}`. Local fallback: append to
   `.kineti/journal.jsonl`. These lessons bias future variant rounds.
6. **Iterate**: regenerate the losers' slots using taste lessons until one
   variant wins or three rounds pass. Three rounds max, then the human
   picks the least-bad and we note why none was loved.
7. **Produce production tokens**: `design/tokens.json` with colors,
   spacing scale, radii, fonts, motion durations taken from the winner.
8. Move stage: `bun "$K/kineti-state.ts" set stage 4`.

## Outputs

Approved style brief, winning `variant-N.html`, `compare.html`,
`design/tokens.json`, taste lessons in memory.

## Hard rules

- No component code before a mockup wins.
- The 12-component UI library is fallback vocabulary for when nothing
  better fits — never a mandatory checklist.
- Every rejected variant teaches: record it, never silently discard.

## Memory after

Taste lessons written; winning traits noted in run-record.
