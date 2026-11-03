# Enforced production release path: design-review packet

Date: 2026-10-25. Status: prepared for review; implementation not authorized.

## 1. Authority and exact baseline

The governing [contract](ENFORCED-PRODUCTION-RELEASE-CONTRACT.md), version 1,
SHA-256 `fdee210105e06e9653751173e124e664bec3591391c2a91fec166228209fafc1`,
is byte-identical to the owner's second attachment. This packet proposes a design;
it does not change that contract or authorize implementation, live settings,
canary writes, dispatches, credentials, push, release, recovery or paid spending.
The companion [implementation plan](ENFORCED-PRODUCTION-RELEASE-PLAN.md) gives
ordered work and acceptance dependencies. No next version is selected.

Fresh local inspection: HEAD `9f9fc1dd15d1aa7f6fdfe76984947fbae8d2bf56`,
tree `48ff3843d774a32769417db580d35584a06c781c`.
Cached origin/main `5857984a96774ea1f478158502305dd8207855bb` was not fetched.
Existing untracked `design/` and the governing contract are preserved. Locked
state goal remains unchanged. No outbound requests were made for this packet.
Live security findings below are saved inspection evidence, not refreshed claims.

Private evidence is in `.kineti/production-release-design/`: preview, guarded undo,
source digests, native command transcripts, accounting inspection, preservation
hashes and document validation. Existing `.kineti/production-release-contract/`
contains saved GitHub settings responses and outbound audit. Keep both private.
The present documentation is reviewable locally; public redaction still needs review.

## 2. What already exists and what to reuse

| Source inspected | Existing behavior | Missing production boundary |
| --- | --- | --- |
| `core-native/crates/kineti-cli/src/gate.rs`, `receipt.rs`, `compat.rs` | Exact command/input/environment/runtime binding, issuer and freshness checks, legacy verification | Independent execution identity and release authorization |
| `spend.rs`, `journal.rs`, `undo.rs` | Locked accounting, chain/checkpoint verification, reservation and recovery stops, durable undo | Complete release-specific accounting decision and durable multi-service promotion audit |
| `tests/native_gate.rs`, compatibility fixtures | Tests for forged/changed receipts, exact argv, concurrent/corrupt spending, unknowns and interrupted undo | Independent protected assertions and P01–P27 execution evidence |
| `self_test.rs` | Demonstrates fabricated-output limitation | Printed success cannot authorize publishing |
| `.github/workflows/kineti-gate.yml` | Read-only token, pinned action revisions, fmt/Clippy, UTC and Asia/Kolkata tests | Mandatory issuer-bound exact-success policy outside candidate control |
| `.github/workflows/release.yml` | Four actual target matrices, candidate packaging, eight isolated installs, one-day artifacts | Version hardcoded to 0.6.0; candidate-controlled checks, no publisher or independent controller |
| `.github/workflows/published-060.yml` | Frozen 0.6.0 registry install checks and exact source/digests | Reuse assertions only; do not edit the dated 0.6.0 workflow into a next-release workflow |
| `scripts/build-native-package.py`, `accept-native-install.py`, `native-runner-identity.py` | Packaging, footprint/runtime/MCP acceptance and runner identification | Controlled ownership, extraction hardening and next-version provenance |
| `action.yml` | Runs supplied executable with required evidence | Does not verify executable identity or independently require a complete command map |
| `docs/GATE-0.6.0-CLOSEOUT.md` | Saved 52-test, four-target, eight-install results and published digests | Historical results; no fresh test pass or next-boundary acceptance established here |

No production publication job exists in inspected tracked workflows. Do not restore
the old harness, evaluator service, model routing or protected-write product.

## 3. Threat model and proposed architecture

Protect approved bytes, tags, evidence, policy, reviewer authorization, accounting
and publication authority against candidate code, malicious dependencies, fabricated
stdout, workspace binary/key replacement, stale evidence, artifact swaps and races.
The trusted operator, independent reviewer, policy owner, isolated controller host
and provider identities form the trusted base; compromise invalidates assurance.
Same-account protection and arbitrary test completeness are not claimed.

Propose a separately controlled private or public policy repository, subject to
verified no-additional-charge eligibility, owned outside the coding principal.
Its immutable reviewed revision contains acceptance assertions, a command inventory,
manifest validator and promotion controller. Exact repository and owners are
unassigned blockers. If this cannot be provided within $0, stop for design review;
do not replace it with a same-session helper or an external paid verifier service.

1. Candidate build jobs run on disposable native runners with no publishing,
   signing or OIDC authority. They produce archives and reports as untrusted data.
2. Independently maintained acceptance jobs consume exact candidate identities
   in isolation. Candidate scripts cannot change assertions, verifier or host
   control files. Assertions inspect actual installations and native calls.
3. The reviewed controller reads exact GitHub run/attempt/job/issuer identities,
   validates inventory, policy, accounting and evidence, and stages read-only
   digest-addressed archives outside candidate write authority.
4. An independent reviewer approves the exact immutable manifest digest through
   a protected environment and a durable digest-specific decision record. A job
   approval alone is insufficient unless its exact manifest binding is proven.
5. A minimal privileged publisher runs reviewed pinned tools only, revalidates
   settings and bytes, persists intent, exchanges narrowly scoped OIDC identity,
   and uploads the approved archives. It performs no candidate checkout, build,
   install hook or candidate script execution.
6. A separate unprivileged install phase verifies exact downloaded bytes and
   installations on all four native targets before declaring completion.

No self-hosted persistent runner is proposed. Read-only permissions alone are not
an isolation demonstration. Untrusted dependencies execute only before privilege
is available. Avoid privileged `pull_request_target`/`workflow_run` data execution.

## 4. Identity and credential inventory: unresolved assignments

| Role or authority surface | Present evidence | Required assignment / decision |
| --- | --- | --- |
| Coding GitHub principal | Saved repository response reports admin/maintain/push | Operator must name a restricted principal and isolated execution environment; remove inherited owner authority only under separate approval |
| Policy owner | Unassigned | Name account/team and immutable policy repository; prove coding principal cannot change its rules or approval |
| Independent reviewer | Unassigned | Name authenticated account distinct from candidate principal; no self-review or coding-principal bypass |
| Settings operator / emergency administrator | Unassigned | Name account and record emergency authority outside adversary role |
| Controller identity and repository | Unassigned | Name immutable repository ID, workflow path/revision, environment and allowed issuer |
| crates.io owners, teams, trusted publishers, tokens | Not inventoried | Operator-reviewed metadata inventory, revocation/isolation plan and provider claim restrictions |
| npm owners, organizations, automation/granular tokens, trusted publishers | Not inventoried | Same; no token/session fallback |
| GitHub PATs, Apps, deploy keys, environment/repository/organization secrets, alternate workflows | Incomplete | Enumerate metadata, permissions, holders, expiry, reachability and alternate release paths; never export secret values |
| Local gh/git/Cargo/npm sessions, environment, OS keychain, browser sessions and other hosts | Incomplete | Operator inventories credential reachability; prove separate identity with harmless canaries, not reads of secret contents |
| Vercel integration | Saved production deployment from vercel[bot] | Recheck branch/PR deployment effects before any remote write; no disconnect or settings change implied |

This is an explicit incomplete inventory, not a credential separation claim.
The packet is not ready for unconditional design approval until named identities,
complete metadata inventory, exact pins and resource eligibility are supplied.

## 5. Settings-change preview: proposed only

| Surface | Saved baseline | Proposed desired state and acceptance |
| --- | --- | --- |
| Main ruleset 23894511 | Deletion and non-fast-forward prevention only | Reviewed PRs, independent policy-owner review, current issuer-bound checks, no coding-principal bypass; export effective rules after operator approval |
| Release-policy/workflow ownership | No independent ownership demonstrated | Protect workflow, policy, test inventory, toolchain and dependency changes; approval cannot activate itself |
| registry-release environment | v* tag policy, no reviewer, admin bypass allowed | Named reviewer, prevent self-review and coding-principal bypass, narrowly allowed controller ref; prove feature eligibility first |
| Actions | Default read token; all actions allowed; global SHA enforcement off | Minimum job permissions, reviewed full-commit action allowlist, no untrusted OIDC/secrets, disabled checkout credential persistence |
| Registries | Unknown | Exact trusted-publisher binding, isolate alternate tokens/sessions and workflows; unsupported restriction blocks enablement |
| Website integration | Saved automatic production route | No change proposed; hold pushes unless side effects are ruled out or separately approved |

Required-check names alone are insufficient. The controller must reject every
non-success conclusion, including skipped/neutral/cancelled/missing/stale results.
GitHub concurrency with cancel-in-progress false only avoids automatic replacement;
it does not prevent a person/API cancellation. The implementation must prove an
external durable lock and interruption/reconciliation behavior, and review whether
literal no-cancellation enforcement is feasible. Do not claim YAML satisfies it.

## 6. Interfaces and exact-byte publication feasibility

Use an internal version-1 release manifest and append-only decision audit, separate
from native receipt/CLI/MCP formats. Suggested files after implementation approval:
`release-control/` for internal controller/schema/policy examples, controlled tests
in the independent policy repository, and new next-release workflows only after
review. This is a location proposal, not a new public command authorization.

Manifest groups: immutable repository ID; commit/tree; exact version and packages;
controller/workflow/policy/assertion/toolchain digests; run ID/attempt; allowlisted
job IDs, issuer IDs, conclusions and runner identities; ordered archive names,
lengths and SHA-256; install report identities; receipt identities/timestamps and
exact argv arrays; accounting snapshot and decision; effective-settings digest;
approval identity/reference/times; destination allowlist; unique promotion ID.

Reject duplicate JSON keys, unknown schema/fields/algorithms, missing or ambiguous
identities, extra artifacts, traversal, symlinks, future evidence and unsupported
encodings. Hash the canonical immutable proposal. To avoid approval-digest
self-reference, the proposal contains approval policy and requested expiry; the
separate signed/authenticated approval record binds its digest. A final envelope
references both immutable objects without changing approved proposal bytes.
Any interface detail inconsistent with the contract requires an explicit amendment.
Evidence lasts at most 240 minutes. Effective authorization expires at the earlier
of approval time +24 hours and the earliest evidence expiry; a 24-hour approval
does not extend four-hour evidence. Refreshing evidence changes the approved packet.

States: created -> validated -> approved -> publishing -> complete, with denied,
partial and uncertain outcomes. Record durable per-service intent before each write,
then actual outcome and downloaded digest. Persist audit before proceeding. Hold a
package/version lock across retries; a lost lock, missing audit or network ambiguity
requires exact-state reconciliation and prevents another write. Administrative
cancellation or runner death cannot erase intent or imply nonpublication.

Publication feasibility is an early blocker: normal Cargo publishing may package
from source. Prove that a reviewed pinned client can upload the exact staged .crate
with trusted-publishing identity and no candidate execution or repackaging. Do the
same for npm tarball publication, including provider tooling/version requirements.
Do not invent an archive-upload command or use source rebuilding as a substitute.
If exact-byte trusted publishing is unavailable, request a precise contract amendment
before implementing a fallback. After Cargo download/hash confirmation, publish npm
and verify, then approved tag/GitHub release/assets. Confirm version/tag availability
before approval and immediately before publication. Occupied versions deny, never bump.

## 7. Proposed pins and provider-claim mapping

Existing action pins, reuse subject to independent review:
checkout `3d3c42e5aac5ba805825da76410c181273ba90b1`;
Rust toolchain action `6bed0761d98439e5a578e2877258200ad565ba87`;
upload-artifact `cf430e030ddbb5b0abf93d22962f4752f3646cd9`;
download-artifact `9000827ccba6bdab643e8b6fd33ac0654aef8333`.
An action pin does not pin the downloaded Rust toolchain. Exact Rust, Cargo, npm,
Node used for packaging/publishing, OS image, controller build and dependency-lock
digests must be selected and reviewed before implementation acceptance. No claim
of current provider support is made; official capability and account-setting
checks are pending. Existing compatibility claims remain limited to saved versions.

| Claim / restriction | GitHub controller must verify | crates.io and npm exchange must prove |
| --- | --- | --- |
| Repository | Immutable ID plus expected owner/name | Actual configured repo restriction and rename/transfer behavior |
| Workflow | Reviewed path/ref and immutable controller revision, event/ref | Exact accepted workflow binding; determine reusable-workflow claim behavior |
| Environment | Exact protected environment and digest-bound reviewer decision | Provider enforcement of configured environment, not merely token presence |
| Audience/subject | Expected issuer, audience, subject, run/attempt and expiry | Wrong audience/repo/workflow/ref/environment/package denial |
| Package/version | Approved crate/npm names, exact version and archive hashes | Scope cannot publish another package; no generic session fallback |

Record actual claim values and provider restriction coverage without retaining
usable tokens. Unsupported coverage stops the design; a local comparison is not
provider-side enforcement. Current account capabilities and exact publisher
configuration remain unverified; this mapping is a verification specification.

## 8. Accounting checkpoint and $0 resource plan

Fresh native `status` and `spend check` both returned exit 3 for historical unknown
`rr-contract-repair-1792163379549`. Commands, environment and executable digest are
saved privately. The journal checkpoint byte count and SHA-256 match; the native
journal reader completes chain checking before the early unknown stop. Scanning
all journal rows finds 13 unresolved/qualified-zero records, unchanged from the
frozen publication contract. Full later native accounting validation is not claimed.

The ledger records 55,000,000 microcents ($55), one legacy charge and no ledger
routing reservations. Root reservation/unknown/native-log/recovery-view/intent files
listed in the inspection report are absent; absence is not a reconstructed clean
ledger. Existing historical recovery and other-host/allocation records remain
preserved, including unresolved starting balance and reviews lacking usage. The
complete checkpoint must examine every recovery-guard source and reservation path
under native semantics before implementation is cleared; any uncertainty blocks.

Keep the $75 ceiling, configured stage limits, 95% stop $71.25 and nominal $16.25
headroom without treating headroom as spending authority. No cost is valued from
tokens, reset, resolved or converted to zero. The old implementation/publication
exceptions do not authorize this milestone. A new exact implementation-only
subscription exception is required, preserving all unknowns; integrity failures,
uncertain writes and monetary reservations remain absolute stops.

Current work uses local inspection/document generation only; no paid resource,
remote runner, storage, API-key fallback or new service was requested. Proposed
post-approval first candidate batch: four build/test jobs and four install jobs,
30 minutes maximum each, one packaging job limited to 15 minutes; maximum 255
runner-minutes per batch, not a billing allowance. Use standard public native
runners only after current $0 eligibility, remaining quota/storage and hard limit
are confirmed. No automatic reruns. Measure artifact sizes before upload, set
one-day remote retention, download and hash required evidence locally before expiry.
If storage/quota/feature eligibility cannot be verified, do not dispatch. Local disk
capacity and evidence export must be checked too. Approving this plan buys nothing.

## 9. Design-review completion and decisions

1. Accept the architecture direction subject to the explicit open blockers; this
   does not authorize implementation or settings changes.
2. Supply named policy owner, independent reviewer, operator, restricted principal
   and controller repository; complete the credential metadata inventory and $0
   eligibility checks before final design approval.
3. Review exact controller/toolchain pins, exact-byte OIDC upload feasibility,
   digest-bound approval and cancellation/lock design. Amend the contract explicitly
   if a provider cannot meet it.
4. Only then approve exact local implementation scope plus its milestone-specific
   included-subscription exception. Separate canary/settings/production approvals
   remain required as described in the plan.

No tests were rerun for this documentation packet; no P row passed. The truthful
release-boundary claim remains proposed, not implemented or proven.
