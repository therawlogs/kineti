---
name: build
description: Produce code in small verified pieces with undo safety. Stage 7 of 13.
stage: build
version: 0.5.0
triggers:
  - build it
  - implement
  - code the spec
  - start building
---

# kineti-build

Implement approved feature work in small pieces, with an undo step registered
before every change and spending checked at checkpoints. Direct fixes,
reviews, and cleanup may enter at the relevant step.

## Harness (every session)

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
RUN="build-$(date +%s)"
bun "$K/kineti-saga.ts" begin --run-id "$RUN"
```
For new feature work, confirm that its outcome contract and required
approvals are current before editing. A direct fix does not need an unrelated
full-project spec gate.
Log every model call: `--stage build`.

## Procedure

1. **Work in spec-section-sized pieces.** One piece = one or more complete
   acceptance tests from the spec. Announce the piece before starting.
2. **Register undo before each change.** Before editing files, running
   migrations, creating services, or any irreversible action:
   `bun "$K/kineti-saga.ts" register --run-id "$RUN" --label "<what>" --inverse "<exact undo command>"`
   Examples: file edit → `git checkout -- <file>`; migration → its down
   script; created dir → `rm -rf <dir>`. No registration, no change.
3. **Freeze scope**: touch only files this piece needs. Never "improve"
   neighboring code — that is a different run.
4. **Check money between pieces**: `bun "$K/kineti-spend.ts" check`.
5. **Verify continuously**: after each piece run the declared verify
   command. Red means fix now, not later.
6. **Checkpoint the work** after each green piece with a clear diff and
   evidence. Commit only when the user or repository workflow asks for it;
   never push in this stage.
7. **Failed approaches go to memory immediately** — approach, why it
   failed, one-line lesson. These are the most valuable records the
   system keeps.
8. **Finish the run**: all spec acceptance tests green →
   `bun "$K/kineti-saga.ts" commit --run-id "$RUN"`
   then `bun "$K/kineti-state.ts" set stage 8`.

If anything unrecoverable happens:
`bun "$K/kineti-saga.ts" rollback --run-id "$RUN"` — undos run newest-first;
a failing undo is logged and the rest still run.

## Outputs

`src/`, tests per acceptance criterion, progress commits, updated memory.

## Hard rules

- A change is allowed only after its undo step is registered.
- Missing approval blocks only the feature work that depends on it.
- Red verify at piece end blocks starting the next piece.

## Memory after

Run-record: pieces completed, decisions, failed approaches, costs.
