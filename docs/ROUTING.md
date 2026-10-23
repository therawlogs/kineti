# Single-agent routing

`kineti route` exposes JSON operations for OpenCode and Codex. Read-only planning does not launch either host or inspect credentials. Every live probe or execution requires explicit `--consent`, a positive owner-set per-run cap, a permitted model, an estimate with its source, and budget headroom. There are no automatic retries.

The cap controls admission and further work using recorded or estimated usage. It cannot guarantee the provider's invoice or stop an in-flight request from overrunning its estimate. The CLI displays this limitation before a launch. Unknown final cost retains a reservation and blocks further launches and CI until explicitly reconciled. Recorded costs and host-calculated prices are not independently verified invoices.

## Read-only operations and configuration

```sh
kineti route --help
kineti route classify --task-file task.txt
kineti route manifest list --host codex
kineti route manifest declare --host codex --model OWNER_SELECTED_MODEL
kineti route manifest refresh --host codex --consent
kineti route manifest validate --host codex --file manifest.json
kineti route policy validate --host codex --file policy.json
kineti route policy import --host codex --file policy.json
kineti route plan --host codex --task-file task.txt
```

Declaration/import writes configuration only. A catalog listing is recorded as `host-listed`, never confirmed access. Codex uses the supported app-server `model/list` interface; OpenCode uses `models`. A listing failure remains blocked. Configure the host's existing authentication through that host's normal secure settings; never put credentials in a Kineti manifest, policy or task file.

Each host has separate `.kineti/routing/HOST.manifest.json` and `HOST.policy.json`. Both schemas have version 1 and a UUID revision. Unknown top-level fields are rejected. Manifest fields are `host`, `ttl_hours` (default 24 on creation), and `models`. Model rows contain `id`, `provenance`, optional `listed_at`, `confirmed_at`, `expires_at`, `confirmation_run_id`, and `revoked`. Live confirmations must reference one signed successful host execution for that host and requested model. They establish that the host accepted a live requested-model call; actual resolved identity is a separate requirement.

A policy contains:

- `host`, `host_version`, and optionally `executable` for the installed binary.
- `per_run_cap_usd` and `max_wall_seconds` (at most 600).
- `class_models`: ordered model ID arrays for `code`, `plan`, `chat` and `fix`.
- `cost_estimates`: model ID keys with explicit positive `usd` estimates and nonempty `source` descriptions. Unknown cost cannot be entered as zero.
- `required_check`: one required test `label` and exact `command` argument array for v1 outcome verification.
- `destinations`: explicit network destination hostnames declared before execution. This declaration does not mediate every request made internally by a host.
- Optional `default_class`: the explicitly chosen treatment of ambiguous tasks.

No host, model, provider or price is silently selected. The deterministic classifier records its rule/version and matching classes. Mixed matches block unless the policy supplies a default. The selected model is the first permitted, suitable model in the owner's order. Expired, revoked or unconfirmed access blocks ordinary routing.

## Bounded calls and outcome checks

A deliberate probe can establish live requested-model access before ordinary routing. It must name a model already declared/listed and permitted by the policy, and satisfy the same cap, estimate and headroom checks:

```sh
kineti route probe --host codex --model OWNER_SELECTED_MODEL --task-file task.txt --consent
kineti route execute --host codex --task-file task.txt --consent
```

The adapters use `codex exec --model MODEL --json --sandbox read-only --ephemeral -` and `opencode run --model MODEL --format json`, with the task on stdin. Requested arguments, executable digest/version, host session, exit status, termination reason, event digest and separate Kineti run/session identities are signed into the existing receipt ledger. Raw event files remain private under `.kineti/routing/output/`. A timeout, interruption, malformed/truncated event stream or nonzero exit cannot complete execution.

Current adapters **do not establish a verified resolved-model channel**. Model flags, configuration echoes and agent-written text are insufficient. Response-level observations are retained separately; conflicting observations block the run. Missing proof remains `unverified`, so execution alone cannot produce an accepted routing loop. Fixture tests do not upgrade this limitation or establish live host acceptance.

After a real run, record its required tests through the existing runner using the returned identities:

```sh
kineti test run --label OWNER_CHECK_LABEL --session-id SESSION_UUID --parent-run-id ROUTE_RUN_UUID --required-test tests -- bun test tests/
kineti route settle --run-id ROUTE_RUN_UUID --usd REPORTED_AMOUNT --source "owner-provided usage report"
kineti route verify --run-id ROUTE_RUN_UUID --task-file task.txt --check-run-id CHECK_RUN_UUID
```

The test label and exact command must match the captured owner policy. Verification checks signed identities, task/classification binding, retained event integrity, test freshness/signature/inputs, model identity and the spending join. A foreign, missing or ambiguous join cannot pass. Each execution remains distinct, even within one session. `settle` reports a known amount; never invent zero to clear unknown cost. Interrupted reservations require investigation before settlement or cancellation; do not delete accounting locks or reservations simply to retry.

Execution returns separate execution, model identity, checks and budget fields. An executed task with missing checks or unverified model identity is `not-accepted`, even when the host exits zero. A probe may record successful requested-model access while full outcome verification remains blocked.

## Current delivery boundary

The local test suite exercises classification, catalogs, policy ordering, exact arguments, reservations, malformed/failed/timed-out runs and cross-run joins using explicitly fake host binaries. Live OpenCode and Codex acceptance are separate, outstanding gates. Installation and authentication status do not establish model access. No paid host run or billing claim follows from fixture tests.

The owner must explicitly supply the per-run cap, permitted models and pricing estimates; these are not inferred from installation or a catalog. Candidate package acceptance, the 0.6.0 version update, publication and published-package acceptance remain gated on both live loops. Stranger outreach and live delegation require their separate approvals.
