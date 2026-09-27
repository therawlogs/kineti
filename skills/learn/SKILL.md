---
name: learn
description: Search, prune, and export what Kineti learned.
stage: reflect
version: 0.2.0
triggers:
  - search memory
  - what do you remember
  - prune lessons
---

# kineti-learn

Maintenance window for the memory store. Not a pipeline stage; run any time.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
```

## Commands

- `learn search <words>` — find matching lessons, dossiers, run-records;
  show each with its expiry date and source run.
- `learn stale` — list lessons expired or expiring within 14 days.
- `learn prune` — move expired lessons to cold storage after confirming
  they no longer fire on recent work.
- `learn export <file>` — dump all active lessons as markdown for sharing
  or backup.
- `learn stats` — counts by type, age histogram, projects covered.

## With gbrain connected

Use memory verbs (recall/remember/forget) against the brain; the same
commands map onto queries over `type: learning` records.

## Without gbrain

Operate on `.kineti/journal.jsonl` files across known projects plus
`~/.kineti/alerts.log`.

## Hard rules

- Never delete history. Expire it to cold storage.
- Every listing shows provenance: which run produced this record.

## Memory after

Prune and export operations themselves get a one-line journal entry.
