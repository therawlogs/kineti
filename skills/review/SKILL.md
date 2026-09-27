---
name: review
description: Hunt the bugs that tests miss. Stage 8 of 13.
stage: verify
version: 0.2.0
triggers:
  - review the code
  - bug hunt
  - find bugs
---

# kineti-review

A staff-engineer pass over every change since the last review. Tests prove
what was thought about; this finds what was not.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage review`.

## Scope

`git diff` from the last review tag (or spec approval) to now. If the diff
exceeds ~1500 lines, review in chunks and say so.

## Procedure

1. **Read the diff hunk by hunk.** For each, ask: what does the author
   believe about the world here? Then ask what happens when that belief is
   false — null, empty, timeout, duplicate, reordered, huge input.
2. **Hunt the recurring killers**, in order:
   - Race conditions: shared state written by two paths without a lock.
   - Missing error paths: calls whose failure is never handled.
   - Contract mismatches: shapes here vs shapes declared in architecture.
   - Unbounded work: loops, queries, or agent runs without limits.
   - Silent catches: `catch {}` that swallow evidence.
3. **Auto-fix the trivial**: typos, missing null checks, obvious off-by-one.
   Each fix is its own commit labeled `review-fix:`. Never bundle fixes.
4. **Flag the serious** in `review.md`: file:line, what breaks, smallest
   fix suggested. Do not fix large items silently — the human chooses.
5. **Second opinion** (if another model CLI is available): send the diff
   with "find bugs only, no style comments". Merge findings; mark which
   reviewer found what. Cross-model agreement raises severity.
6. **No new features.** Anything that smells like scope creep goes in the
   out-of-scope list, not the code.
7. Refresh proofs for touched areas:
   `bun "$K/kineti-evidence.ts" run --label review -- -- <verify command>`
8. All serious items resolved → `bun "$K/kineti-state.ts" set stage 9`.

## Outputs

`review.md`, review-fix commits, refreshed proof records.

## Hard rules

- No features during review. Not even small ones.
- Every flagged item has file and line numbers.
- Trivial fixes commit separately from everything else.

## Memory after

Record which bug classes appeared; they bias future build-stage checks.
