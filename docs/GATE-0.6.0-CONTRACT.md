# Kineti 0.6.0 gate-only acceptance contract

Date: 2026-10-25. Version: 2.

Status: product requirements approved in the owner's implementation request;
implementation and acceptance are not complete. The accounting-policy amendment
below was explicitly approved by the owner on 2026-10-25 for subscription-only
implementation with $0 new paid spending. It grants no publication, deployment
or accounting-recovery authorization.

This is the release acceptance contract, not a roadmap or completion report.
For 0.6.0 it replaces conflicting harness, delegation and OpenCode milestones.
It does not alter the locked project goal or historical contracts and records.
Requirement changes require an explicit owner-approved amendment. Unrun,
unsupported, failed or missing evidence cannot count as accepted.

## Acceptance requirements

| ID | Binding requirement | Required acceptance evidence |
| --- | --- | --- |
| G01 | Reconcile spending, reservations, unknown costs and prior recovery before implementation. Preserve the expected recorded $55 without forcing totals. Enforce the authorized $75 ceiling, stage limits and 95% stop. Integrity failure or unresolved accounting blocks implementation and publication under the current policy. | Source digests, reconciled entries, outstanding items and headroom; no cleared or guessed costs. |
| G02 | Separate reviewed accounting recovery under spend. Inspection proposes changes without changing accounting. Back up damaged originals byte-for-byte. Apply only the exact reviewed proposal, append resolution and evidence references, preserve history, and verify afterward. | Corruption, interruption, unresolved-cost and recovery tests; reviewed proposal and resolution trail. |
| G03 | Establish actual GitHub-hosted Linux x64/arm64 and macOS x64/arm64 runner access before the full port. No cross-compilation or emulation substitute. | Successful native jobs identifying runner labels, OS versions and actual architecture. Missing target blocks implementation pending review. |
| G04 | Early Codex feasibility on 0.163.0-alpha.11.2, isolated home/configuration inside the project, no real configuration or credentials. Through supported app-server MCP, load generated configuration, discover tools and make a real native MCP call without a model turn. | Exact executable digest and protocol evidence. Failure blocks full port. Before release also test and pin an exact stable version. Claim only tested versions; Cursor and Antigravity become unverified with configuration preserved. |
| G05 | Map every retained behavior, exit meaning and compatibility format to native tests before deleting the old implementation. Preserve the dated 631-test result without imposing that count on the new suite. | Traceable behavior/fixture matrix and actual new test inventory. |
| G06 | Generate genuine legacy signature golden fixtures with the TypeScript signer before deleting it. Verify original JSON.stringify bytes separately from new canonical receipts. No migration may increase legacy authority. | Genuine acceptance and alteration/issuer rejection; ordering, Unicode, escapes, numbers and optional-field cases. |
| G07 | One published Rust crate, kineti-cli, provides kineti without depending on other Kineti published crates. Keep their past releases untouched. | Dependency/package inspection and isolated Cargo installation. |
| G08 | Provide init, test run/check, verify, ci, spend, undo, status, mcp and add self-test. Preserve gate-related evidence and saga aliases and gate exit meanings. Publish exit-code table. Preserve argv including empty arguments; validate malformed options before writes; provide versioned JSON. | CLI/JSON/exit/argv tests. Inventory every former command, alias, subcommand and MCP tool; test explicit rejection of removed behavior, including old native assistant and protected-write/service commands. |
| G09 | Init creates required missing gate state and usable configuration, requiring an explicit new-project goal and preserving existing locked goals. Missing state gives a repair instruction; damaged records require separate recovery. | Fresh/missing/damaged-state tests; no invented goal or rewritten historical record. |
| G10 | Init previews exact configuration diff, destination and backup before applying native MCP registration and one gate instruction skill. Preserve unrelated settings/user edits; support repeat installation and ownership-aware uninstall, with restart guidance. | Isolated Codex preview, backup, preservation, repeat, uninstall and real discovery/call tests. |
| G11 | Preserve existing records byte-for-byte and original signature/freshness rules. New explicitly versioned receipts use existing cross-language canonical encoding and bind command argv, declared inputs, dependencies, relevant environment, runtime identity, execution/session references and observed results. | Alteration, issuer, staleness, environment, arguments, missing-check and corrupt-history fixtures. Distinguish signature validity from trustworthy results and demonstrate fabricated output. |
| G12 | One locked accounting path uses integer amounts and total/stage thresholds. Corruption and unresolved reservations fail closed. Reset cannot pass without headroom; existing confirmation interface is audited acknowledgment, not proof of a human. | Concurrent accounting, thresholds, reset, corruption, reservation and recovery tests. |
| G13 | Registered undo runs newest first with durable intent/completion. Continue after definite failures; uncertain interruption blocks replay until inspection and explicit resolution. | Ordering, failure and interruption/recovery tests, including exact argv. |
| G14 | Remove active harness, model routing, delegation, swarm, dashboard, memory, workflow generation, benchmarks, evaluation services and unrelated integrations through new commits. Preserve reusable integrity/signing/accounting/recovery safeguards, Git history, dated releases, signatures, data and unresolved records. | Removal inventory and diff audit; compatibility evidence exists before deletion. |
| G15 | Cargo and npm deliver the same native implementation on Linux musl x64/arm64 and macOS x64/arm64. Cargo installation needs Rust; runtime needs neither Node nor Bun. One npm package contains four binaries/checksums; installer verifies/selects native executable and removes unused payloads. Disabled scripts and unsupported platforms fail explicitly. | Both candidate routes installed outside checkout on all four actual targets; Node/Bun absent from runtime PATH; no source dependencies. |
| G16 | Self-test uses disposable fixtures for stale evidence blocking simulated release, fresh evidence passing, invalid-signature rejection and fabricated-output limitation. | Installed self-test transcripts on all four targets. |
| G17 | Run native formatting, Clippy and retained suites under UTC and Asia/Kolkata; save and check fresh evidence. Installed footprint for both routes is below 10,000,000 bytes on every platform including binary, assets and initial integration files, excluding toolchains/caches/accumulated records. | Exact test inventory, fresh receipts, file inventory and sizes. Report download and unpacked package sizes separately. Any target at/over limit blocks release. |
| G18 | Replace eight-crate/Bun release pipeline with single-crate/native-package pipeline. Target exactly 0.6.0; occupied tag/registry version blocks publication. | Reviewed workflow, candidate provenance and version preflight. No automatic version bump. |
| G19 | Audit README, GitHub Action, MCP registry, website and package descriptions. Announce platform restrictions, unverified hosts and older kineti-cli assistant lineage change. | Claim-to-evidence audit; historical release descriptions remain dated history. |
| G20 | Present exact commit, artifacts/digests, runner identities, sizes, tests, Codex versions, accounting reconciliation/recovery/outstanding items and limitations for separate publication approval. Repeat installation/self-test against exact published packages afterward. | Publication packet, explicit approval, per-registry outcomes and published-package evidence. Partial publication is recorded, not hidden. |

## Required claim and limits

“Record checks, detect changed inputs, and block participating CI commands when
evidence or reported spending fails policy.”

No universal interception, same-account adversary protection, provider-bill
verification, genuine execution established solely by printed summaries, atomic
authorization of later writes or protected-write guarantees. Codex checks prove
host integration, not autonomous model behavior. Do not advertise unverified
automatic native-tool hooks. An executed command can fabricate passing output.

## Approved accounting amendment: subscription-only continuation

Approved by the owner on 2026-10-25: “Approve the subscription-only implementation
exception with $0 new paid spending. Preserve all unresolved records, update the
0.6.0 contract with this approval, and continue implementation through the
required feasibility checks.” This narrowly amends G01 for implementation only.

1. Preserve historical monetary nulls, qualified zeros, recorded $55 and all
   original records. Token usage and money are distinct accounting dimensions.
2. Classify the matched historical Codex logs as subscription-associated,
   supported by 2,457 snapshots across 23 files showing plan_type=prolite,
   has_credits=false and balance=0. This supports included usage but is not a
   historical billing statement or proof of zero charge.
3. Keep 2,426 response IDs / 407,261,937 total tokens as locally observed usage,
   not a dollar amount. Keep the review starting balance of 312,239 tokens
   unattributed and the three reviews lacking usage unresolved. Preserve other
   host and current-session accounting gaps. Do not manufacture prices.
4. Approved budget: retain the $75 historical reported-spend ceiling and 95%
   stop; authorize $0 NEW incremental paid spending for continuation. Existing
   subscription payments are shared overhead, unallocated to this project,
   not historical zero-cost entries. No upgrades, credit purchases, API-key
   fallback, paid runners or paid services without separate exact approval.
5. The approved exception to G01 allows local implementation using
   included subscription allowance and verified no-additional-charge resources
   while the ten historical records listed below remain unresolved. The reason
   is subscription-associated usage with no inferred monetary valuation. It must
   never bypass corruption, uncertain accounting writes, unresolved monetary
   reservations or a detected possible additional charge.
6. At each checkpoint verify ordinary subscription usage remains allowed and
   extra credits remain absent. Stop on allowance exhaustion or ambiguity;
   wait for renewal rather than purchase. Usage percentages are account-wide,
   not a promise that the allowance will finish this implementation.
7. Four-platform runner availability and billing must be verified separately.
   If a required runner costs extra, stop and present its exact rate and a
   bounded proposed job budget before using it. No paid resource is assumed.
8. Publication remains blocked by G01 until historical monetary questions are
   resolved or a separate exact publication-policy exception is reviewed and
   explicitly approved. An implementation-only exception never authorizes ship.

This approval does not claim that historical subscription coverage has been
invoice-verified. The affected journal records remain unresolved and unchanged:

- rr-contract-repair-1792163379549
- rr-release-v050-dac373f
- rr-contract-remaining-20261016-1
- rr-remaining-implementation-20261016-1
- rr-resume-20261017-1
- rr-audit-20261017-comprehensive
- rr-spend-recovery-20261021
- rr-completion-progress-1792602902405
- rr-completion-resumed-1792669452309
- rr-branch-consolidation-20261022

The review starting balance, reviews without usage, other-host costs and
current-session valuation gaps also remain unresolved. Monetary reservations
and integrity failures are never exempted.

## Acceptance record

All G01–G20 rows are pending acceptance. No test result is established by this
document. Maintain an evidence matrix keyed by these IDs; each entry must identify
the exact source/artifact digests, command, environment, result and remaining gaps.
No release is accepted until every requirement is satisfied, or an explicit
owner amendment changes it. Publication always needs separate approval.

## Zero-paid-spend runner storage confirmation

On 2026-10-25 the owner confirmed: “Hard $0 Actions spending limit confirmed”.
Candidate jobs may use standard public-repository runners and available included
artifact storage with one-day retention. The account limit must stay unchanged.
Quota exhaustion blocks acceptance; it never authorizes paid usage.
