# Enforced production release path: implementation status

Date: 2026-10-25. Status: local prototype implemented; production boundary disabled.

## Authorization and preserved scope

The owner requested “implement it” after preparing the plan under the explicit $0
new paid spending constraint. This pass implements local validation, synthetic
transport and adversarial fixtures only. It does not treat that instruction as
permission for credential changes, remote settings, canary writes, CI dispatch,
push, deployment, publication, recovery or an actual-release accounting exception.
The governing contract and earlier frozen documents remain unchanged. The design
and plan retain their original approval-time status; this record describes later work.

The complete design-review gate remains open. In particular, independently owned
identities/policy, registry inventory, exact production pins and provider enforcement
have not been established. Local fixtures do not certify that gate as passed.

## Implemented work

1. `release-control/controller.py`: strict versioned manifest parsing with duplicate
   and unknown-field rejection; exact repository/source/run/attempt/job/issuer/argv
   and binary/policy/toolchain bindings; exact success and assertion inventory;
   eight installation-report bindings; expiry, settings and resource revalidation.
2. Manifest-digest approval binding, independent-reviewer policy checks and scoped
   synthetic accounting exceptions. Integrity problems, uncertain writes, monetary
   reservations and the 95% stage/total spending stop cannot be exempted.
3. Exact staged archive inventory, no-follow reads, size/hash checks, traversal,
   noncanonical path, symlink/hardlink/device and expansion-limit rejection. Bytes
   are held unchanged in memory; no extraction, package hooks or source builds.
4. Registry-coordinate/version locks, independent initialization markers, append-only
   hash-chain/checkpoint audit with timestamp/actor and exact approval/decision
   references, durable per-service intent and outcome, explicit partial/uncertain
   states and manual reconciliation of synthetic sink state before resuming.
5. Structural manifest/approval JSON schemas, executable relational validation,
   a real unittest runner, local adversarial tests and an operator-facing README.

`ProductionAuthority` always denies; a second guard requires the exact synthetic
adapter, synthetic mode and synthetic destinations before any write. There is no
real network transport, OIDC exchange, credential discovery or public CLI change.
The native gate, version 0.6.0, package layout, old signatures and accounting
behavior are untouched. No new crate, harness, routing or protected-write product.

## Evidence and accounting

Private evidence and isolated verification workspaces are under
`.kineti/production-release-implementation/`. They include preview, native undo
registration, source/artifact digests, timestamps, environment, exact commands,
receipts, check outcomes, accounting checkpoint and failed live GET attempts.
Earlier verification logs are retained separately rather than replaced as history.
The guarded undo removes only newly created implementation files if hashes match;
it never deletes accounting or publication history.

Fresh native status/spend checks return exit 3 for the first historical unknown.
A read-only recovery proposal retains that error and proposes no adjustments.
An initial unsupported `spend recover --inspect` attempt returned exit 2; the
correct supported read-only `spend recover` command and result are saved separately.
No recovery was applied.

A full current-file inspection covers journal checkpoint/chain entry, the single
legacy charge and integer/stage totals, all native log/anchor/initialization paths,
reservation/unknown paths and accounting-recovery log/guard/marker/view paths.
The current tail paths are absent, the legacy charge matches $55, no root monetary
reservation is present, and all six inventoried historical recovery evidence files
retain their original digests. All 13 unresolved/qualified-zero journal records
and other-host/allocation gaps remain. This is an inspection, not a replacement
for the native early-stop decision or proof of historical provider bills.

The account checkpoint reported ordinary subscription usage allowed and no credits.
Only included subscription work and local offline build/test resources were used.
No Actions run or remote storage was requested, no payment setting changed and
new paid spending remained $0. Full accounting data and account identifiers stay
private. Synthetic ledgers cannot authorize Kineti's actual release.

## Provider feasibility and current access

Read-only GitHub GETs were attempted for repository, main head, effective rules,
registry-release environment and Actions permissions. All failed to connect to
api.github.com. Their failures and intended destinations are retained; saved earlier
settings are not represented as refreshed. The connected GitHub tool inventory
exposes repository/PR operations but no equivalent ruleset/environment inspection.
No protected-setting claim is inferred from those tools.

Official [Cargo publish documentation](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)
describes packaging before upload; an exact staged .crate/OIDC adapter is still
required. [npm trusted-publisher documentation](https://docs.npmjs.com/trusted-publishers/)
describes repository/workflow binding and an optional environment field; actual
package configuration and required environment binding remain unverified.
[GitHub concurrency documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)
controls queued/running jobs; it does not establish the contract's complete
no-cancellation boundary. These capability references do not prove account settings.

## What you need to arrange before protected-host work

1. Keep your administrator/publisher account in an environment the coding agent
   cannot access. An operator must inventory and remove/isolate the agent's existing
   broad credentials only under separately approved changes.
2. Choose a restricted coding account or App. It may propose changes but must not
   administer protection settings, approve its own releases or publish packages.
3. Choose who owns and reviews release rules. You may use your separately controlled
   owner account; another reviewer is also possible. Name the accounts before setup.
4. Choose a separate repository controlled by that owner for the release policy and
   assertions. Verify features/resources are available with no additional charge.
   The owner then reviews an exact settings/canary proposal before applying it.

No action is required from you to run the local tests. These choices are required
before claiming the real release boundary works, not proof supplied by a chat flag.

## Remaining implementation and acceptance

The production part of the plan remains unfinished: authenticated observations and
approvals, independently protected assertions, isolated identity setup, registry
OIDC claim enforcement and exact-byte upload adapters, cross-host durable
serialization/audit storage, actual cancellation/revocation/partial-state handling,
safe canary demonstrations, exact next version, fresh four-platform candidate
installations and independent acceptance. Production settings, actual-release
accounting decision, artifact approval, publication and P27 are separate later gates.

All P01–P27 retain pending final acceptance. Local tests exercise validation patterns
for P04/P06–P12/P15–P21/P25, synthetic promotion/revocation portions of P22/P26 and
retained local regression for P17/P24. This is not a claim those rows passed against
a protected host/provider. P01–P03/P05/P13/P14 require actual isolated authority;
P23/P27 require eight actual installations on four native targets. No such new
external acceptance was run in this pass. The local filesystem lock is not a
cross-host lock, the synthetic sink is not durable registry state, and its approval
objects are not independent authenticated reviewer decisions.

## Fresh local verification results

On local macOS arm64, the final isolated source copies passed:

| Check | UTC | Asia/Kolkata |
| --- | --- | --- |
| Controller adversarial assertions | 44 passed, 0 failed, 0 skipped | 44 passed, 0 failed, 0 skipped |
| Retained native suite | 52 passed, 0 failed, 0 skipped | 52 passed, 0 failed, 0 skipped |

Cargo formatting and Clippy with warnings denied passed. Native `kineti test run`
and exact-argv `kineti test check` passed for each suite/timezone. Tests ran on
isolated copies inside this project; final input hashes were compared back to
current source. No remote runner or four-platform acceptance is implied.

Exact child argv:

```sh
python3 release-control/run_tests.py
cargo test --offline --locked --manifest-path core-native/Cargo.toml
cargo fmt --manifest-path core-native/Cargo.toml --check
cargo clippy --offline --locked --manifest-path core-native/Cargo.toml --all-targets -- -D warnings
```

Both test commands were recorded through native evidence with their required test
files declared and then checked using exact argv. Command transcripts and timezone
values are in `verification-commands.json`; result counts, receipt identities and
log hashes are in `verification-summary.json` under the private evidence directory.
Python syntax compilation also passed with a project-local bytecode cache; an
initial attempt at the system cache location was denied and retained as an
environmental failure, not a passing check.

All 41 protected original files stayed byte-identical. The only expected original
runtime-file changes are the native undo registration and its guard. No unresolved
accounting entry, locked goal, frozen contract or user design file changed.
