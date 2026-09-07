# MEMORY.md — How Kineti Remembers

Engine: **gbrain** (local database, installed from GitHub). When gbrain MCP
is connected, skills use its memory verbs. Everywhere else — and always as
the durable fallback — Kineti writes structured records to a project's
`.kineti/journal.jsonl`. This file defines the record formats and the three
rules that make memory trustworthy.

## Record types

Every record is one JSON line with these top-level fields:

```json
{
  "at": "ISO timestamp",
  "type": "run-record | learning | dossier",
  "state": "active",
  "project": "slug",
  "id": "rr-001",
  "data": {},
  "links": [],
  "expires": "ISO date or null"
}
```

**run-record** — one per pipeline run.
`data`: `{ root_goal, stage_outcomes, costs_usd, failed_approaches[] }`.
Also carries `prev_hash` and `hash` (see Rule 3). Expires: never.

**learning** — one insight. `data`: `{ skill, trigger, lesson }`.
Default expiry 90 days (process lessons 180, vendor-specific 60).

**dossier** — living facts about a project.
`data`: `{ stakeholders:[{role, power, agreement}], stack, next_steps[] }`.
Refreshed by diagnose/feasibility/retro; expires only if abandoned.

## Rule 1 — Expiry states

Records move: `active → warm → cold → archive`. Nothing is deleted.
Ages (since `at`, using type-specific expiry for active): active until
expiry, then warm 90 days, cold 275 days, then archive. Only `active`
records answer recall. The weekly job performs all moves.

## Rule 2 — Open causality

Links live in `links[]`:

```json
{
  "word": "caused",
  "from_id": "rr-002", "from_at": "...",
  "to_id": "rr-001", "to_at": "...",
  "status": "candidate",
  "proof_id": null
}
```

- Any relationship word is allowed at write time. Core words come from the
  Kineti causal schema: `caused triggers blocks enables requires supports indicates
  contributes_to remediates contradicts supersedes resolves duplicates`.
- Status flow: `candidate → hypothesis → validated → rejected`.
- A link reaches `validated` only when `proof_id` points at an existing
  record holding evidence. Unproven links stay candidate and expire.
- Time order: for `caused`, `triggers`, `blocks`, the effect's `at` must
  not precede the cause's `at`. The weekly job flags violations; nothing
  is silently dropped or rewritten.
- The weekly job reports non-core words used 3+ times as promotion
  candidates. Vocabulary grows by use, never capped.

## Rule 3 — Project hash chain

Each run-record sets `prev_hash` to the previous run-record's `hash` for
the same project (`GENESIS` for the first), and
`hash = sha256(prev_hash + at + id + canonical_json(data))`.
The weekly job recomputes the chain; any break proves history was edited.

## Weekly job

```sh
KIN="$(cat "$HOME/.kineti/repo")"
bun "$KIN/bin/kineti-memory-job.ts" sweep        --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" verify-chain --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" time-order   --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" promote      --dir <project>
```
Schedule weekly (launchd/cron). `retro` runs the same four commands at the
end of every retro regardless.

## With gbrain connected

The same records are remembered via memory verbs with their settings block
as page metadata; `journal.jsonl` remains the export/import format. Search
today is keyword-based because no embedding key is configured — exact
recall works, semantic ("similar meaning") search activates when an
embedding provider key is added to gbrain config.
