# Kineti — enforced production release path contract

Date: 2026-10-25. Version: 1. **Status: proposed; implementation not authorized.**

## 1. Purpose, scope and authority

The owner selected **Kineti's next Cargo/npm and GitHub release** as the first
production boundary. The objective is to prevent the coding agent or an untrusted
contributor from publishing unapproved artifacts or bypassing independently
controlled acceptance checks. Kineti remains one native gate; this work does not
restore a harness, evaluator service, protected-write product or model routing.

This document records requirements and an initial inspection, not implemented
security, an independent security audit or a claim that the boundary works.
Creating this contract authorizes no code/workflow changes, GitHub or registry
settings changes, credentials changes, CI dispatches, merge, push, publication,
deployment, recovery or new paid spending. Approval of a later implementation
plan will not itself authorize release or recovery.

The published 0.6.0 packages, tag, frozen approval documents, historical signatures,
locked goal and unresolved accounting remain unchanged. This contract supplements
[GATE-0.6.0-CONTRACT.md](GATE-0.6.0-CONTRACT.md); it neither retroactively enlarges
0.6.0's guarantees nor extends its exact-publication exception. No next package
version is assigned here. An exact future version and availability preflight are
required before artifact approval; never auto-bump an occupied version.

Website deployment, social posting, media uploads, provider-billing integration
and universal command interception are outside this milestone. Existing website
auto-deployment is a discovery/approval hazard, not permission to deploy it.

## 2. What exists: inspected baseline, not inferred protection

Local source baseline: `9f9fc1dd15d1aa7f6fdfe76984947fbae8d2bf56`.
Last-fetched `origin/main`: `5857984a96774ea1f478158502305dd8207855bb`;
this cached ref is not a fresh remote-head assertion. The local website commit
is deliberately not pushed. Untracked `design/` remains preserved.

| Existing behavior or control | Evidence inspected | Reuse and gap |
| --- | --- | --- |
| Native evidence and policy checks | `core-native/crates/kineti-cli/src/gate.rs` and `receipt.rs` | Reuse exact argv/input/environment/runtime binding, freshness and issuer checks. They do not bind registry promotion to an independent CI identity or make same-account keys authoritative. |
| Accounting and recovery | `spend.rs`, `journal.rs`, `undo.rs` | Reuse locked integer accounting, unknown/reservation stops, chain/checkpoint checks and interrupted recovery/undo rules. No unknown is cleared by this proposal. |
| Regression inventory | `tests/native_gate.rs`, compatibility fixtures and release closeout | Existing tests cover altered/legacy receipts, argv, staleness, corrupt/concurrent accounting, reservations/reset and interrupted undo. Self-test deliberately accepts fabricated summaries. Saved 52-test/four-platform results are historical evidence, not new tests run for this contract. |
| Main CI | `.github/workflows/kineti-gate.yml` | Runs fmt, Clippy and tests in two timezones with read-only contents permission. Source existence does not make its result mandatory or independent of candidate edits. |
| Composite Action | `action.yml` | Calls a supplied executable with `ci --require-evidence`. Does not itself pin/verify that executable, protect the caller or enforce complete command maps and publisher permissions. |
| Packaging and installation | `release.yml`, `published-060.yml`, saved release reports | Reuse four native targets and isolated installation checks. The published workflow pins 0.6.0; it is not the next release's promotion workflow. No production publishing jobs were found in tracked workflows. |
| Main rules: live GitHub GET inspection | Ruleset `23894511`, effective rules for main | Active rules only prevent deletion and non-fast-forward updates. No required status check or review rule was returned. Classic branch-protection GET returned “Branch not protected”; this does not negate the active ruleset. |
| Environments: live inspection | `Preview`, `Production`, `registry-release` | Preview/Production have no protection rules and allow admin bypass. Registry-release has only a custom `v*` tag policy, no reviewer rule, and permits admin bypass. |
| Actions settings: live inspection | Repository Actions/permissions APIs | Default workflow token is read-only and cannot approve PR reviews. Allowed actions is `all`; global SHA-pinning enforcement is off, although inspected workflow action references are pinned. |
| Agent's present GitHub authority | Repository API permissions | Current credential reports admin/maintain/push. This is incompatible with the proposed untrusted coding-agent role; no claim of credential separation can pass while it remains accessible. |
| Automatic deployment evidence | Deployment `7312404791` and status | GitHub reports `vercel[bot]` created a Production deployment of `5857984…`, with success at `2026-10-25T11:25:21Z`. This indicates an external automatic deployment route; it is not a live website-content audit. No Vercel settings were inspected or changed. |
| Registry publisher configuration | Not inspected in this audit | Crates.io/npm owner roles, trusted publishers, legacy tokens, local sessions and alternate publish paths remain **unverified**. They are blockers, not assumed safe. |

Raw read-only API responses, failures, timestamps from returned records, source
inspection notes and request destinations are private local evidence under
`.kineti/production-release-contract/`. Do not publish credentials, personal paths
or raw account evidence. The live findings must be rechecked before implementation
and before each production approval; unobservable settings remain unverified.

## 3. Threat model and trust boundaries

### Assets and actors

Protect package bytes/digests, release manifests/tags, registry and GitHub
publication authority, controlled tests/policies, signing identity, approval
records, accounting history and the public release audit.

The adversary may be a mistaken or malicious coding agent, an untrusted PR author,
a hostile test/build script or dependency, or a person holding only the permitted
agent credential. They may edit candidate source/tests/workflows, invent passing
stdout, replace workspace executables/keys, supply old receipts or malicious
artifacts, race/retry requests, cancel jobs and attempt direct publication.
Agent identity must be isolated from trusted administrator/publisher identities;
a confirmation flag or chat phrase is not cryptographic proof of a human.

Trusted components are the separately controlled policy/test definitions,
reviewed release controller, host isolation, pinned tools, GitHub/registry identity
services and authorized operator/reviewer accounts. Trust is conditional and must
be demonstrated. GitHub itself issuing an OIDC token does not prove the allowed
workflow or environment was correctly restricted.

Out of scope: compromise of GitHub/registry infrastructure, kernel/hypervisor or
the trusted administrator/reviewer's account; an authorized owner intentionally
changing policy; proving arbitrary test completeness or honest program behavior;
provider-bill correctness; preventing every local write/command. Compromise of
these trusted components invalidates the assurance and requires incident response.
This milestone does not claim native same-account adversary protection.

### Required boundaries

1. **Agent to repository:** the coding principal can propose changes but cannot
   administer rules, approve protected policy changes, push main directly, mint
   release identities or access publication credentials. Use a distinct OS/runner
   identity with no inherited owner sessions, home/config, keychain, tokens or
   browser credentials. Existing broad credentials must be removed from that
   execution environment by an explicitly authorized operator, not silently by
   implementation. A separate workflow under the same unrestricted admin session
   does not satisfy this requirement.
2. **Candidate code to trusted tests:** acceptance policy and release-control
   workflow have independent ownership/review and pinned versions. Candidate
   code/tests/build scripts execute without publisher/signing credentials or
   writable access to trusted tests, verifier or host control files. A candidate
   change cannot approve its own new acceptance baseline. Gate/workflow/test-owner
   changes require independent approval and new negative tests before activation.
3. **Test output to authorization:** a trusted harness runs independently
   maintained assertions and records actual execution status, inventory and
   artifact identities. A candidate's printed summary or self-signed local
   receipt alone cannot authorize promotion. Native receipt compatibility stays
   separate; legacy receipts acquire no extra authority. Host isolation and
   independent assertions reduce fabrication risk; they do not prove all tests
   meaningful or all software safe.
4. **Artifact to promotion:** use digest-identified, read-only staged archives.
   A separate controller validates exact origin/run/attempt/commit/workflow and
   policy identities before issuing an authorization. Untrusted build processes
   cannot alter the staged promotion set after checking. Publish these exact
   bytes; no source checkout, build, install hook or candidate script executes in
   the privileged publishing job.
5. **Approval to publication:** require an independently authenticated reviewer
   decision tied to the exact manifest digest and protected environment. No
   self-approval, automatic approval, admin bypass by the coding principal or
   reusable generic approval. Approval never substitutes for failed technical or
   accounting checks.

## 4. Required release design

### Repository, CI and independent checks

- Require reviewed PRs, current successful checks and protected changes to release
  workflows, policy, test ownership and dependencies. Bind required checks to the
  intended issuer; do not trust a job name alone. Prevent the agent from editing
  protections, bypassing review or introducing a second publisher. Revalidate
  effective settings, not just checked-in YAML or CODEOWNERS.
- The release controller must require exact `success` for all allowlisted jobs on
  the selected commit/run attempt. Missing, skipped, neutral, cancelled, timed-out
  and stale results deny promotion. GitHub's ordinary required-status behavior
  alone is insufficient for this rule. Fail closed on API/auth/service outages.
- Use fresh disposable runners, least privilege and full-commit action pins.
  Untrusted PR/build execution has no secrets, signing keys, OIDC mint permission
  or publication token. Do not execute untrusted content through privileged
  `pull_request_target`, `workflow_run`, package hooks, caches, artifacts or
  unchecked workflow outputs. Treat artifact metadata as untrusted input.
- Maintain controlled acceptance assertions outside candidate write authority.
  Fresh four-platform packaging/install checks must identify actual runners and
  prove both installation routes work without Node/Bun in runtime PATH. Preserve
  the below-10,000,000-byte footprint requirement, compatibility tests and existing
  exit meanings. Run fmt/Clippy and retained suites in UTC and Asia/Kolkata.

### Manifest and promotion interface

A new **versioned release manifest and append-only decision audit** are the
release-control interfaces; they are not upgraded native receipt formats. At
minimum bind repository immutable identity, candidate commit/tree, exact version,
workflow/controller/policy/test/toolchain identities, run ID/attempt, required job
results and native runner identities; archive names, byte sizes and SHA-256;
installation reports; evidence identity/freshness; accounting snapshot digest and
its decision; owner approval identity/reference and expiry; target registries,
package names/tag and a unique promotion ID.

The default evidence maximum age is **240 minutes**. The manifest approval expires
after **24 hours**, and cannot outlive its evidence. Changed source, archives,
accounting decision/snapshot, policy, reviewer authorization or effective security
settings invalidate approval. Reject missing, unknown-version, malformed,
ambiguous or duplicate fields, unsafe paths, unexpected artifacts, mismatched
identities and unsupported algorithms. Preserve exact argv arrays. CLI/MCP/native
receipt changes, if necessary, require an explicit interface amendment before
implementation; do not silently add public commands or alter legacy exit codes.

Re-read effective policy and rehash immutable artifacts at the publication
boundary. Serialize promotions for the package/version, allow no cancellation of
an in-flight publication, and persist intent **before** the first irreversible
operation. Use `created -> validated -> approved -> publishing -> complete`, with
explicit `denied`, `partial` and `uncertain` outcomes. Retries reconcile registry
state by exact package/version/digest before any write. Network ambiguity never
means “not published”. Missing audit persistence blocks the next write.

### Registry authority and release sequence

- Default to short-lived OIDC trusted publishing bound to the reviewed repository,
  workflow and protected environment. Audit actual claims and provider rules for
  both registries. Disable or isolate alternate long-lived tokens, manual owner
  sessions and other publishing workflows from the agent. Record administrator
  emergency authority as outside the adversary's role; do not pretend it is gone.
- If either registry cannot enforce the required identity restrictions, stop for
  an exact amended design. No broad token or local-session fallback is implicit.
  Registry owner changes and revocation require separate settings approval.
- Before publication, require the exact owner-approved version/artifact packet,
  resolved accounting or a separately approved exact new-release exception,
  destination allowlist and availability of both package versions and tag.
- Publish Cargo, verify downloaded bytes, then npm and verify downloaded bytes;
  then create the approved tag/GitHub release and attach only approved assets.
  All irreversible steps have separately auditable outcomes. Any tool repackaging
  the approved archives blocks publication until new bytes are reviewed.
- Partial publication is possible across services. Record it, stop and present
  exact outcomes; never auto-bump, unpublish, overwrite, relabel provenance, guess
  outcomes or conceal failure. Completing the untouched approved remainder needs
  valid unexpired authorization; changed bytes/scope require renewed approval.
- After separately authorized publication, repeat exact registry installations and
  self-test on all four actual targets, with native MCP calls, runtime PATH and
  footprint checks. Release completion requires these results, not only upload.

## 5. Accounting, resources and preserved history

**New paid spending is $0.** Existing included subscription usage and verified
no-additional-charge resources only. Preserve the hard $0 Actions limit and do
not change payment settings. Confirm included allowance, runner/storage capacity
and feature eligibility before use. Standard public native runners and at most
one-day temporary artifacts remain the default. Quota exhaustion, uncertain
charges or unavailable required controls block work; no paid upgrade, API-key
fallback, credits, external verifier service or paid runner is authorized.

The ledger is still $55, not a value to force; the historical ceiling is $75 and
95% stop $71.25, with $16.25 nominal recorded headroom that is not spend authority.
Original ledger/log/journal hashes match the frozen review in this inspection.
The native spend check freshly returns exit 3 for the first historical unknown.
Its journal reader verifies chain/checkpoint before that stop, but the early
unknown means this command is not proof that every later reservation/ledger path
was evaluated. A complete checkpoint inspection is required before future work.

Preserve all **13 unresolved/qualified-zero records** listed in the frozen
0.6.0 publication contract, all reservations/recovery records, unattributed token
balances, other-host and ongoing allocation gaps. Never infer money from tokens,
mark an unknown zero, select a clean snapshot to hide history, or reset a flag to
manufacture headroom. Synthetic acceptance ledgers must remain explicitly marked
and physically separate from the real release accounting; they cannot authorize
Kineti's real release. A missing ledger must not be replaced with a fresh empty
one for promotion.

The earlier subscription-only implementation approval is not an unrestricted
budget or a new-publication approval. Before implementation, request exact approval
for this milestone's included-usage/no-additional-charge work with these unknowns
preserved. Before publication, independently resolve uncertainty through the
reviewed recovery process or obtain a new exact artifact-scoped accounting-policy
exception. The 0.6.0 exception cannot be reused. Such an exception must appear in
the release audit, retain the failed native check and never silently weaken its
behavior. Corruption, uncertain accounting writes and unresolved monetary
reservations are never exemptible.

Recovery remains separate: inspect only, preserve original bytes/checksums,
propose exact evidence-backed adjustments, obtain exact review, append resolution
and verify before resumption. No recovery or guessed cost is authorized here.

## 6. Mandatory acceptance matrix

Every row starts **not run for this milestone**. Old tests are reusable regression
coverage, not evidence of a working publication boundary. Expected denials must
show no publication authority was granted and no unauthorized write occurred,
not merely a CLI error. Store commands, actual outcomes, environment identities,
policy/manifest/artifact digests and redacted audit references for every case.

| ID | Scenario | Required result/evidence |
| --- | --- | --- |
| P01 | Agent directly pushes main, edits rules/owners or self-approves policy | Denied by the actual protected repository using the restricted principal; owner settings remain unchanged. |
| P02 | Agent invokes Cargo/npm/GitHub publication outside the approved controller | No usable publication credential/identity; registry/GitHub boundary rejects the restricted principal. Local token absence alone is not enough. |
| P03 | Agent reads owner home/config/keychain, CI secrets or signer material | Denied by isolation; nonsecret canary assertions prove separation without exposing credentials. |
| P04 | Delete/rename/skip gate, forge a same-name success, submit neutral/skipped/cancelled/old-run status | Controller denies; exact expected issuer, job IDs, commit and attempt must all succeed. |
| P05 | PR modifies workflow, trusted tests, command map or verifier | Candidate cannot activate the altered policy or suppress independent tests without independent approval. |
| P06 | Replace gate binary or issuer key; re-sign a receipt locally | Independent controller rejects unknown executable/issuer identity; no extra trust from a valid local signature. |
| P07 | Replace tests with `printf` passing output or an exit-zero stub | Native fabricated-output limitation remains demonstrable; independent protected assertions fail or are missing and promotion denies. |
| P08 | Delete tests, run no assertions, use skipped/truncated/failed test output | Controlled inventory/assertions fail; no success solely from a printed count or exit zero. |
| P09 | Replay receipt from another commit/repository/session/run attempt or change argv including empty arguments | Reject identity/argv mismatch; compatibility formats remain readable without new authority. |
| P10 | Change input/dependency/environment/runtime after checking; expire evidence or use future timestamps | Deny; exact-source fresh rerun required. |
| P11 | Swap archive, rebuild after approval, mutate staged bytes or replace a same-name artifact | Digest/identity check denies at promotion; only the tested approved bytes are publishable. |
| P12 | Poison cache/artifact, inject path traversal/symlink or run package hooks in publisher | Reject unsafe material; no untrusted execution with publisher privileges. |
| P13 | Use wrong OIDC audience/repository/workflow/ref/environment/package | Provider denies token exchange/publication. Unsupported claim enforcement is a blocker, not presumed present. |
| P14 | Use fork PR or privileged trigger to gain secrets/OIDC/release rights | Denied; untrusted execution never shares the privileged context. |
| P15 | Change effective protections, revoke reviewer/publisher or use expired approval after approval | Final policy/identity revalidation denies without registry writes. |
| P16 | Use unknown costs, unresolved monetary reservations, corrupt/missing history, interrupted write/recovery or manipulated snapshot | Deny before promotion; preserve original bytes. A fresh empty ledger is not a repair. |
| P17 | Concurrent accounting updates, stage/total 95% stops and reset without headroom | No lost integer amounts; stopped accounting still denies. Tokens never become guessed dollars. |
| P18 | Simulate approved exception versus omitted, stale or wrong-artifact exception | Only an exact separately approved exception may permit its stated historical unknowns; reservations/integrity failures still deny, and native failures remain visible. |
| P19 | Race two promotions, cancel a publishing job, timeout or lose network after an upload | One durable intent; no duplicate/changed artifact, no inferred nonpublication; uncertain outcome blocks replay pending inspection. |
| P20 | Fail Cargo, npm, tag/release or audit persistence at each boundary | Accurate per-service partial/uncertain state; no subsequent unauthorized write or automatic version substitution. |
| P21 | Prioritize wrong/occupied version or moved tag; use artifact signed by a valid but disallowed workflow | Deny; signature validity is insufficient. |
| P22 | Clean approved canary promotion through the protected boundary | Only exact approved bytes arrive at the test sink; denied cases produce zero sink writes and no credential release. |
| P23 | Four native target Cargo/npm candidate installation, self-test, MCP and footprint acceptance | All eight isolated installs pass; Node/Bun absent from runtime PATH; every footprint below 10,000,000 bytes. |
| P24 | Retained native/legacy regression suites under UTC and Asia/Kolkata | fmt/Clippy and actual inventory pass; original signatures, freshness, exit semantics, unknowns and interrupted undo remain preserved. |
| P25 | GitHub/registry API outage, exhausted included quota or possible additional charge | Fail closed, preserve audit, stop; no paid fallback. |
| P26 | Incident drill: revoke publisher, pause promotion and resume a known partial state | New publication denied after revocation; exact inspected state required for resumption; no automatic unpublish or history erasure. |
| P27 | Separately authorized real publication and four-platform published-package check | Registry-downloaded bytes match approval; all real install/self-test checks pass before release is called complete. |

Acceptance proceeds from local adversarial fixtures to an owner-approved,
zero-paid disposable GitHub/canary environment. Mock sinks do not establish real
host/provider enforcement. Do not test writes against real main, production
registries, 0.6.0 or the website. Demonstrate provider-side denial through safe
non-mutating authorization checks where available. Where proving a boundary
requires a real canary write, obtain exact separate approval for its account,
namespace, version, actions and $0 resource bound first; otherwise mark the row
blocked. No fake-cost fixture or test publication can become a real-release
approval. Failure or missing external access leaves the milestone incomplete.

## 7. Review packet and approval gates

1. **Design review:** exact contract digest, threat model, baseline evidence,
   named independent policy/reviewer/operator identities, settings-change preview,
   complete credential/publisher inventory, proposed pinned tools/controller and
   registry claim mapping; exact included-resource plan and accounting checkpoint.
   Owner approval is required before implementation or settings mutation.
2. **Implementation acceptance:** all P01–P26 evidence applicable before real
   publication, unresolved blockers, redacted effective-settings exports,
   compatibility results and incident drill. Independent review must be performed
   by a person or service outside the coding principal's authority. No self-review
   or same-session helper is described as independent security review.
3. **Production enablement and release approval:** separate exact permission to
   change live protections/registry trust, followed by exact artifact/version and
   accounting decision approval. Recheck remote heads and automatic integrations
   before any push; main pushes may deploy the website. If deployment side effects
   cannot be ruled out, hold the push for separate approval or an approved isolated
   branch. Never silently disconnect Vercel or change its settings.
4. **Published-release completion:** P27, per-registry/tag/release outcomes and
   exact published installation evidence. Do not label the release path
   production-enforced until the real configuration and protected publication
   boundary have been demonstrated. A canary-only pass is candidate readiness.

Every evidence row identifies exact source/configuration and artifact digests,
commands, actors, timestamps, actual results and scope. Changed source, policy,
artifacts or settings invalidate affected evidence and approvals. A new release
must retain a local immutable audit and a reviewed redacted public packet without
secrets, personal paths or raw private account/ledger data. If remote evidence
cannot be retained within the $0 constraint, stop rather than silently discard
required audit or incur storage charges.

Until these gates pass, the truthful claim is: **the 0.6.0 native gate is published;
an enforced production publication path is proposed, not implemented or proven.**

## 8. Primary documentation consulted

Inspected 2026-10-25; provider capabilities must be rechecked at implementation.
These documents describe capabilities, not this account's configured guarantees.

- [GitHub protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches): reviews/status requirements and administrator bypass scope.
- [GitHub secure use](https://docs.github.com/en/actions/reference/security/secure-use): least privilege, untrusted code and privileged workflow risks.
- [GitHub environments](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments): protection applies before jobs access environment secrets; availability varies by repository/plan.
- [GitHub artifact attestations](https://docs.github.com/en/actions/concepts/security/artifact-attestations): provenance must be verified; it does not prove tests meaningful or software secure.
- [npm trusted publishing](https://docs.npmjs.com/trusted-publishers/): identity-bound registry publication; exact configuration remains unverified here.
- [crates.io development update](https://blog.rust-lang.org/2026/01/21/crates-io-development-update/): GitHub trusted publishing and registry enforcement capability; this crate's current settings remain unverified.
