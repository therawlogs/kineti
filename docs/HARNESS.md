# Owned execution controller (experimental)

`kineti run` implements a bounded Codex app-server loop. Kineti creates the session, exposes four tools, applies scoped edits, runs an independent checker, and signs a result. A fixture passing this loop is a controller test. It is not evidence of a successful live model task.

The first supported task is deliberately fixed: repair unknown options in `kineti spend` from Kineti main commit `88b518d951a30614450c6fec22b9ed5195776a17`. This is not yet a general coding runner. Delegation and OpenCode execution acceptance follow a successful live Codex coding proof; they are not exposed by this controller.

## Commands

From a Kineti checkout containing the pinned baseline commit:

```sh
# Requires Bun, Docker, Linux x86-64, cc, and Codex exactly 0.163.0.
bash scripts/harness/build.sh
bun bin/kineti.ts run preflight --host codex
# Real host configuration and sandbox tests; does not start inference.
bun bin/kineti.ts run capabilities --host codex
bun bin/kineti.ts run --host codex --task-file task.json
bun bin/kineti.ts run status RUN_ID
bun bin/kineti.ts run cancel RUN_ID
```

The task file contains only:

```json
{
  "version": 1,
  "task_id": "spend-cli-options-v1",
  "instruction": "Fix kineti spend so unknown CLI options exit 2 before creating or changing accounting files, while preserving supported commands."
}
```

All results use JSON. Invalid input exits 2. Blocked readiness or failed execution exits 1. A status query exits 0 when it can read the record; inspect the separate verdict fields.

A project must explicitly select `harness` or `both` in its existing state. `gate` refuses execution; a legacy project with no mode remains pending. `both` also refuses failed project gates. The controller always applies its fixed task checks. It does not approve workflow stages or release gates.

## Admission and costs

The existing routing manifest must declare owner permission for `gpt-6.1-sol` and contain a fresh host listing or live confirmation. Freshness uses the manifest TTL (normally 24 hours); revocation blocks execution. A catalog listing does not establish live access. A successful genuine coding run provides access evidence in its signed receipt; this controller does not automatically promote the routing manifest or H04.

The existing Codex policy must pin 0.163.0, permit the model for fixes, set a positive cap no greater than $3, and supply a supported model-specific cost estimate with its source. A made-up rate or an unrelated model's price is insufficient. Read-only preflight needs no credentials or model calls. Capability inspection starts the installed host and may use its ordinary runtime metadata, but never requests a model turn.

One accounting lock and ledger cover admission and continuation. The controller reserves the full owner cap. Another run's unresolved cost blocks launch, and all outstanding reservations count toward total and stage headroom. A definite failure before any `turn/start` attempt cancels only this run's reservation. Once inference is attempted, uncertain cost remains unknown and the reservation stays. Token observations are retained separately; they do not establish a USD invoice.

The cap controls admission and continuation. It does not guarantee the provider invoice or prevent an in-flight overrun. There are no automatic transport retries. One recorded check-feedback correction is a continuation of the same run within the same cap and limits.

## Tool and worker boundary

The model works in a fresh directory separate from the candidate. Kineti archives the exact baseline commit, without Git history, local runtime state, existing local fixes, or prepared patches. Model reads/searches are limited to `bin/kineti-spend.ts`, `bin/lib.ts`, and `bin/ledger-integrity.ts`. Only `bin/kineti-spend.ts` may change. Tests, checker expectations, dependencies, policy, and file inventory are immutable.

The dynamic tools are `search_files` (literal search), `read_file`, `apply_patch` (one exact replacement with a current SHA-256), and `run_check` (only `spend-cli-options-v1`). There is no arbitrary command tool. Paths, symlinks, tool arguments, session/turn identities, duplicate calls, byte limits, and cancellation are checked before effects. Limits are 180 seconds, 12 tool calls, 64 KiB per tool request/response or patch, and one correction turn. Native approvals and unexpected native tool activity terminate the run. Model/host/interface fallback is disabled.

Each check uses the pinned Docker image ID, a read-only candidate snapshot, no network, no credentials or Docker socket, 512 MiB memory, 64 processes, and a 30-second outer timeout. The trusted root driver runs candidate programs as UID 10002 with no new privileges and smaller command limits. Candidate output is captured as bounded test data. Fixed assertions, rather than candidate summaries, determine the verdict. The image contains the driver and Bun, not the source checkout or credentials.

The eleven black-box cases cover unknown options in log/check/status/reset, valid log/check/status/economics, threshold/reset behavior, reset with extension, supported pricing options, help, corrupt accounting, and an unknown option attempting to clear a stop. Acceptance also requires a reproduced baseline failure, a model file read, a substantive broker patch, unchanged immutable files, and final passing checks. A canned answer, unchanged checkout, or printed success summary cannot satisfy these requirements. The handwritten acceptance control only validates the checker.

Real capability checks inspect disabled native/external features and exercise allowed reads, protected outside reads/writes, and network denial. An outside write may succeed within a private sandbox overlay; the test requires the real host sentinel to remain unchanged. The trusted Docker driver, controller owner, host administrator, installed host binary, and Docker daemon are outside the candidate threat model. This suite demonstrates the named boundary on the tested machine, not arbitrary deployments or every future host behavior.

## Recovery and receipts

Each run retains `.kineti/runs/RUN_ID/` with a checkpointed hash-chain journal, state, baseline source, candidate bytes, host diagnostics, tool intent/completion, partial messages, usage observations, and checks. The signed local receipt joins task, policy, baseline manifest, checker and worker identity, host executable/command, requested and reported model, sessions, tool history, patch/artifact digests, checks, and budget status. Keep this private directory and the existing spending history during recovery.

`status` reports an interrupted controller as unknown. `cancel` requests termination of the host process group and disposable checks. Inspect incomplete tool intents and retained bytes; there is no automatic restart or replay of an uncertain patch or command. Never delete a reservation or create another ledger to escape unknown costs. Reconcile against defensible host/provider records before another launch.

Functional execution, fixed-test acceptance, provider identity, and spending are separate verdicts. A host's requested/reported model is not provider-resolved identity. Missing provider evidence keeps identity unverified and H04 unaccepted. Harness receipts use a separate unsupported gate-verification status, so they cannot silently authorize an existing test/release gate.

## Current evidence and blockers (2026-10-23)

Local controller/transport fixtures and real Linux checker controls have passed. No genuine live coding run has passed. Real Codex non-inference controls passed the feature, external-tool, profile, filesystem, and network cases, but zero provider retries could not be verified: 0.163.0 rejects overrides of its reserved built-in `openai` provider. Readiness therefore remains blocked. Renaming or replacing the provider would require a separately reviewed change; the controller does not do so.

The prior failed OpenCode probe retains an unknown $0.01 reservation. There is no defensible zero-cost reconciliation or supported `gpt-6.1-sol` estimate in this environment, and the checkout's legacy product mode remains pending. No new inference was launched to bypass these prerequisites. Single-agent live acceptance remains blocked; delegation and repeated OpenCode behavioral acceptance remain deferred. Versions remain 0.5.0; release preparation and publication remain gated.

## Reproduce local validation

```sh
bun run typecheck
bun run check:router
bun test tests/harness-runtime.test.ts tests/harness-transport.test.ts
bash scripts/harness/build.sh
bun bin/kineti-evidence.ts run --label owned-worker --required-test scripts/harness-acceptance.test.ts -- bun test scripts/harness-acceptance.test.ts
bun bin/kineti-evidence.ts check --label owned-worker
```

The fixture clients and handwritten patches in these tests cannot upgrade a run to live proof. Repeat the same boundary and genuine task acceptance on later installations before claiming them accepted.
