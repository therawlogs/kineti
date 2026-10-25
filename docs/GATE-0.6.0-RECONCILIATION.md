# Main reconciliation disposition — Kineti 0.6.0

This records eight independent read-only commit reviews, followed by sequential
native integration. It supplements the [reconciliation contract](GATE-0.6.0-RECONCILIATION-CONTRACT.md).
The baseline is candidate `6ebe8b5` and main `307331a`. No merge or publication is
approved. Final acceptance belongs to the exact candidate identified in the
private publication packet, not this baseline or a previous Actions run.

## Dispositions

### 39da025 — evaluator safety and gate validation

- **Port:** `bin/kineti-evidence.ts:unsignedRecord/verifyReceiptSignature` added
  JS-signed receipt v3 with session/parent references. Native v3 now dispatches
  by explicit native encoding; unmarked originals use original JS bytes. Unknown
  encodings fail. Legacy signatures confer no native authority (exit 6).
- **Port:** `bin/strict-options.ts` equals-value normalization, without modifying
  command tails; malformed values and duplicates fail before writes.
- **Port:** exact configured CI mappings and regression coverage for partial,
  substituted and malformed mappings. New explicit maps precede legacy settings.
- **Already covered:** strict history, independent total/stage stops after reset,
  integer bounds, shared spending lock, pre-write undo/init validation, explicit
  removed-command errors and audited confirmation without human identity claims.
- **Exclude:** evaluator socket/service/helper/Docker/export/revocation policy;
  removed audit/egress/memory/pairing/swarm commands; assistant interfaces, rates
  and model estimates; Stripe/swarm/scheduler implementation and old Bun pipeline.
  Native input/runtime signatures, durable undo/recovery intent and guarded
  completion retain the reusable safeguards. No protected-export claim returns.

### 73ebb09 — bounded routing

- **Port:** `bin/spending-state.ts:readSpending/spendingStop` and
  `src/routing/index.ts:reserve/settle` historical reservation, settlement and
  cancellation formats. Only realized settlements add to totals/entry counts.
  Open, malformed or inconsistent obligations remain blocking; no routing or
  settlement command is restored.
- **Already covered:** root-scoped signing; optional signed metadata preservation;
  CI signatures/freshness, exact integer tokens and honest reported-cost claims.
  Rust maps do not need JavaScript prototype-mutation implementation, while
  original reservation-stage validation is preserved for historical records.
- **Exclude:** model classification/access catalogs, routing execution/probes,
  Companion/schema promotion, evaluator isolation/runtime deployment, Stripe and
  Bun workflow. Native receipts retain exact argv, environment, input and runtime
  binding without claiming transitive runtime attestation or independent tests.

### 071fed9 — merge resolutions

Reviewed against both parents `73ebb09` and `88b518d` (merge base `dac373f`).
Parent-2-only baseline work is not counted again as newly ported work.

- **Port:** merge-specific reservation/log/snapshot reconciliation in
  `bin/spending-state.ts`, deduplicated with the routing reviews.
- **Port:** `bin/kineti-evidence.ts:configuredCommand` compatibility precedence:
  current/explicit gate mapping, legacy per-label `verify_commands`, then test-only
  `verify_command_argv` and parsed `verify_command`. Operational checks do not
  inherit a default test command. Dedicated verify declarations remain independent
  from unrelated mandatory CI labels. Empty arguments stay supported.
- **Already covered:** strict history, locked CI accounting, exact receipt argv,
  initialized accounting markers and reset headroom.
- **Exclude:** evaluator retained-artifact copying and its Docker assertion,
  memory/routing imports, old package payloads and old workflow combinations.
  Native recovery already preserves original bytes and signatures before repair.

### 937fdfc — operational receipts

- **Port:** `bin/kineti-evidence.ts:configuredCommand/validateOperationalReceipt`
  behavior. `evidence check` and its MCP counterpart validate successful native
  check/package records with signature, freshness, input/runtime/environment and
  exact-command checks; results explicitly report `gate_authorized:false`.
- **Already covered:** test check and CI require test evidence. Verify retains its separately trusted
  declared-command execution and reported-spend check. A printed
  passing summary in an operational receipt cannot grant test authority.
- **Exclude:** updating current claims to historical 608-test counts. Dated
  history, including the 631-test result, remains unchanged.

### dcacfe2 — completion-recording failure

- **Already covered:** original `evaluator.rs:failure_response` invariant: errors
  after intent/execution do not prove denial. Native undo and accounting/recovery
  guards already block uncertain replay and interrupted writes.
- **Port regression:** execute an actual undo side effect, then sabotage completion
  guard persistence. Assert exit 3, preserved interrupted bytes, and no repeated
  side effect on retry.
- **Exclude:** evaluator protocol/export reconciliation, service failpoint and
  Docker implementation. No native authorization/export service is introduced.

### 1e20360 — revocations and partial costs

- **Port:** `src/routing/index.ts:runHost` accounting invariant: observed partial
  cost is not final cost. Original unknown reservations remain blocking and
  byte-identical. A clean recovery snapshot cannot hide them.
- **Already covered/port compatibility:** legacy v3 signature dispatch and
  reservation history above; exact main-reader goldens include settlement rounding.
- **Exclude:** access-manifest signed-date equality, probe revision CAS and
  revoked/deleted model protection as routing interfaces. Native signatures bind
  timestamps; init ownership and recovery source-digest checks preserve relevant
  settings/history without recreating access manifests.

### a71638c — validation before locks

- **Port:** `bin/kineti-spend.ts` blank model/stage and malformed-value rejection
  before locks; equals options; explicit positive reset extension versus omitted
  extension; delimiter presence even with an empty tail. Positional undo inverses
  containing equals signs remain supported.
- **Already covered:** unknown subcommands/options, safe integer tokens, explicit
  reported USD, confirmation and locks. Native MCP/library entrypoints reject NUL
  stage/model values where supported. Existing headroom rules remain stricter.
- **Port regression:** exact recursive path/file-byte comparisons before first
  initialization and after a tripped accounting entry, not just file counts.
- **Exclude:** route-model declaration, pricing/estimation branches, new spend
  execution/session flags and obsolete inventory claims.

### 307331a — bounded Codex controller

- **Port:** historical embedded reservations above; extend rejection inventory
  with top-level `run` and preflight/capabilities/status/cancel subcommands.
- **Port regression:** independent spending-checker cases for unknown options,
  unchanged bytes and tripped reset; genuine JS-signed controller-style receipt
  remains signature-readable without native test authority.
- **Already covered:** single accounting lock, integer thresholds, uncertain
  completion guards, exact command/runtime/input binding and fabricated-output
  limitation. Native reset without headroom returns 3 and remains blocking.
- **Exclude:** controller/broker/workers/capability probes/transports/task lifecycle,
  model reservation creation/cancellation APIs, Docker checker runtime, dynamic-tool
  quotas, HARNESS.md promotion and harness package payloads. No inference execution
  is retained; historical obligations remain visible and blocking.

## Regression and provenance evidence

`tests/native_gate.rs` contains named regressions for original v3 signatures and
limited authority, original accounting decisions without rewrites, snapshot-only
reservations, recovery hiding, operational/test separation, exact CI declarations,
verify independence, equals/delimiter/tail parsing, exact malformed-spend snapshots,
native encoding/session alteration and real undo completion-persistence failure.

`scripts/generate-gate-060-main-legacy.py` extracts exact original reader and
signer functions from pinned main, records source digests and writes public-key
fixtures only. Two genuine signatures and 19 accounting cases are checked against
original TypeScript decisions. Bun is a fixture-development dependency only.
No historical ledger or receipt is migrated, rewritten or newly trusted.

Private evidence under `.kineti/reconciliation-060/` preserves pre-fix failing
regressions, original state/source digests, undo registrations and fresh results.
Old receipt verification, accounting, freshness and all other retained tests remain
in the new inventory. Actual target results and artifacts must be read from the
replacement publication packet; previous candidate results do not certify this work.
