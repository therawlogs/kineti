---
name: build
description: Produce code in small verified pieces with undo safety. Stage 7 of 13.
stage: build
version: 0.2.0
---

# kineti-build

Write the approved spec into working code, one piece at a time, with an
undo step registered before every change and money checked constantly.

## Harness (every session)

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
bun "$K/kineti-state.ts" get gate.spec | grep -q pass || { echo "spec not approved"; exit 2; }
RUN="build-$(date +%s)"
bun "$K/kineti-saga.ts" begin --run-id "$RUN"
```
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
6. **Progress commits** after each green piece:
   `git commit -m "WIP: <piece>" -m "Decisions: ... Failed approaches: ..."`
   Local-only by default; never push in this stage.
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
- Spec not approved → nothing happens. No scaffolding exceptions.
- Red verify at piece end blocks starting the next piece.

## Memory after

Run-record: pieces completed, decisions, failed approaches, costs.
