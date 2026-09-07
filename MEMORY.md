# Memory System

This document describes how Kineti saves and verifies notes across tasks.

Kineti saves structured records to `.kineti/journal.jsonl` in your project folder. When `gbrain` is connected, it also saves notes to a local database.

## Record Types

Each record is one JSON line:

```json
{
  "at": "ISO timestamp",
  "type": "run-record | learning | dossier",
  "state": "active",
  "project": "project-name",
  "id": "rr-001",
  "data": {},
  "links": [],
  "expires": "ISO date or null"
}
```

### 1. `run-record`
Saved once per task or run.
- `data`: `{ root_goal, stage_outcomes, costs_usd, failed_approaches[] }`
- Includes `prev_hash` and `hash`.
- Does not expire.

### 2. `learning`
A lesson learned from a task.
- `data`: `{ skill, trigger, lesson }`
- Standard expiry: 90 days (180 days for process lessons, 60 days for service-specific lessons).

### 3. `dossier`
Facts about a project, people, and technical stack.
- `data`: `{ stakeholders:[{role, power, agreement}], stack, next_steps[] }`
- Updated during project planning and weekly reviews.

## Rule 1 — Record Expiry

Records follow four stages over time: `active` → `warm` → `cold` → `archive`. No records are deleted.

- **Active**: Used for recall and answering questions.
- **Warm**: Next 90 days after active expires.
- **Cold**: Next 275 days after warm.
- **Archive**: Kept permanently for historical reference.

The weekly job updates these states automatically.

## Rule 2 — Cause and Effect Links

Links between records are saved in `links[]`:

```json
{
  "word": "caused",
  "from_id": "rr-002", "from_at": "...",
  "to_id": "rr-001", "to_at": "...",
  "status": "candidate",
  "proof_id": null
}
```

- Allowed link words: `caused`, `triggers`, `blocks`, `enables`, `requires`, `supports`, `indicates`, `contributes_to`, `remediates`, `contradicts`, `supersedes`, `resolves`, `duplicates`.
- Status sequence: `candidate` → `hypothesis` → `validated` → `rejected`.
- A link reaches `validated` only when `proof_id` points to a record containing evidence.
- Time check: A cause cannot occur after its effect.
- Words used 3 or more times are suggested for permanent addition.

## Rule 3 — Record Hash Chain

Each `run-record` includes a cryptographic hash of the previous record:
- `prev_hash`: Hash of the prior record (or `"GENESIS"` for the first record).
- `hash`: `sha256(prev_hash + at + id + canonical_json(data))`.

The weekly job recalculates hashes. Any change to older records is detected immediately.

## Weekly Maintenance Job

Run these four commands every week:

```sh
KIN="$(cat "$HOME/.kineti/repo")"
bun "$KIN/bin/kineti-memory-job.ts" sweep        --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" verify-chain --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" time-order   --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" promote      --dir <project>
```

These commands expire old records, verify hashes, check timestamps, and suggest new words.
