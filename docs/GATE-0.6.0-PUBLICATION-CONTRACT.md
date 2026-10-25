# Kineti 0.6.0 publication contract

Date: 2026-10-25. Version: 1. Status: **PENDING OWNER APPROVAL**.

This is the exact approval request, not an approval or completion report. It
supplements G01/G20 of GATE-0.6.0-CONTRACT.md and section 5 of
GATE-0.6.0-RECONCILIATION-CONTRACT.md. The locked goal is unchanged.

## 1. Scope of the requested approval

Approval of this contract grants two distinct permissions for this release only:

1. A narrow publication-policy exception for the preserved historical monetary
   uncertainty identified below. This does not resolve, value or clear it.
2. Publication of the exact Cargo/npm archives below, creation of the stated
   GitHub tag/release with the frozen release notes, and subsequent published-
   package acceptance on all four actual native targets.

There is no approval until the owner explicitly approves this contract. Approval
must be recorded with its exact document digest and the owner's reply. It is
an audited acknowledgment, not proof of human identity or invoice verification.

## 2. Exact source and release artifacts

- Release tag: `v0.6.0` at main `01c710c8ee3c125d19a0f37988ff95d27e743339`.
- Package source: `80cd5d1c27433a402858fd82897db57741271e30`.
- These commits have identical Git trees. Preserve original archive provenance;
  do not relabel metadata, rebuild packages or silently change the tag target.
- Cargo: `kineti-cli` version `0.6.0`, providing the `kineti` executable.
- npm: `kineti` version `0.6.0`.

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `kineti-0.6.0.tgz` | 3,691,089 | `73ce7ca10e28aa73a5588b62cbb07a89e306e0313273421ca2b0d79c6732ce2e` |
| `kineti-cli-0.6.0.crate` | 55,840 | `5787f90a7ad9204d9f651f3fe2a5f1ad0d82ebf2324223a147dfcadd1bd459a9` |

Frozen evidence is inside `.kineti/publication-contract-060/`:

- `PUBLICATION-PACKET.json` — SHA-256 `db2373b4505f673c87e2b7fbd0dd9d115633e0a72973a1cad299f203bfbfa6ea`.
- `ACCOUNTING-REVIEW.json` — SHA-256 `7abf6a238607c33b4d9cc4751e5ac8d79b77edc98a43e532509fb830b6842785`.
- `RELEASE-NOTES.md` — SHA-256 `2448b4d0d45193119c3dff47cd35fca057f74233a862cd8f706833da19408ed3`.

## 3. Accounting exception being requested

Recorded spending stays $55. Keep the $75 ceiling, configured stage limits and
95% stop ($71.25 globally). Nominal recorded headroom is $16.25; it is not approval
to spend it. Authorized **new paid spending remains $0**.

The exception permits only the publications and acceptance work in this contract
using existing included subscription usage and verified no-additional-charge
resources. Existing subscription overhead remains unallocated. Preserve every
original record, signature, recovery record, null and qualified zero. Neither token
counts nor this approval establish historical dollar amounts or zero cost.

The following 13 records remain unresolved, with exact source-record digests in
the frozen ACCOUNTING-REVIEW.json:

- `rr-contract-repair-1792163379549`
- `rr-release-v050-dac373f`
- `rr-contract-remaining-20261016-1`
- `rr-remaining-implementation-20261016-1`
- `rr-resume-20261017-1`
- `rr-audit-20261017-comprehensive`
- `rr-spend-recovery-20261021`
- `rr-completion-progress-1792602902405`
- `rr-completion-resumed-1792669452309`
- `rr-branch-consolidation-20261022`
- `rr-gate-060-feasibility-20261025`
- `rr-gate-060-native-candidate-20261025`
- `rr-gate-060-main-reconciliation-20261025`

Also preserve the unattributed 312,239-token review starting balance, three reviews
without usage, other-host uncertainty and current-session/helper allocation gaps.
Included usage for this narrowly scoped publication and acceptance work remains
unallocated rather than receiving invented dollar entries.

This amends the publication restriction only for these acknowledged monetary
uncertainties and this exact release. It does not authorize accounting recovery,
ledger adjustments or weakening native accounting checks. `kineti spend check`
continues reporting unresolved records; do not reset it or manufacture a pass.
Record the publication-policy exception openly in the release audit trail.

Never exempt corruption, uncertain accounting writes, unresolved monetary
reservations, exhausted allowance, possible additional charges or failed native
acceptance. If any occurs, stop. Preserve the hard $0 Actions limit. No upgrades,
credits, API-key fallback, paid runners or paid services are authorized.

## 4. Required checks and publication sequence

1. Immediately before publication, recheck remote main, exact artifact and frozen
   evidence digests, accounting integrity/reservations, ordinary subscription
   availability, the hard $0 resource constraint and registry/tag availability.
   Changed scope or artifact bytes requires renewed approval. An occupied Cargo
   version, npm version, `v0.6.0` or `0.6.0` tag blocks publication; no auto-bump.
2. Publish the exact reviewed Cargo archive, then the exact reviewed npm archive.
   Stop if tooling repackages an archive or changes its provenance, credentials
   are unavailable, a registry fails or potential additional cost appears.
3. Record each registry outcome. If only one succeeds, report partial publication;
   do not erase history, unpublish, substitute artifacts or hide the failure.
4. After both registries are verified, create `v0.6.0` at the exact main commit
   above and publish the frozen GitHub release notes and reviewed archives.
   A GitHub failure is separately recorded; do not claim a complete release.
5. On actual Linux x64/arm64 and macOS x64/arm64 runners, install the exact
   published Cargo/npm packages outside the source checkout and run self-test.
   Verify execution with Node/Bun absent from runtime PATH, native MCP discovery
   and a real call, disabled-script failure, artifact provenance and final
   installed footprints below 10,000,000 bytes. No cross-compilation substitute.
6. Save commands, results, runner identities, digests and sizes. Candidate tests
   do not count as published-package acceptance. Publication is complete only
   when both registry results, tag/release and all published-package checks pass.

## 5. Evidence already available and limits

Saved acceptance: 52 native tests (3 library, 49 integration), UTC and
Asia/Kolkata on all four targets; formatting and Clippy; eight isolated Cargo/npm
candidate installations; largest installed footprint 2,358,754 bytes. Main CI
also passed both timezones. Original 827 receipt records had zero verification
differences. Both exact Codex versions 0.163.0-alpha.11.2 and 0.164.0 passed isolated
native MCP loading/discovery/calls without model turns on macOS arm64.

Keep the approved gate-only headline, platform restrictions, older assistant
lineage announcement and documented exit meanings. Cursor/Antigravity remain
unverified. Signatures do not prove honest tests; commands can fabricate passing
summaries. No universal interception, same-account adversary protection, provider-
bill verification, protected writes or atomic authorization of later writes.

Website deployment, unrelated versions, new product features, deletion of user
settings/data, accounting recovery and new paid spending are excluded.

## 6. Owner decision

1. **Approve this contract:** grant both the narrowly scoped publication-policy
   exception and the exact Cargo/npm/tag/GitHub-release publication above, plus
   the required zero-new-paid-spend post-publication checks.
2. **Keep publication blocked:** supply billing/coverage evidence and review a
   separate exact accounting recovery proposal instead. No recovery is automatic.

Until an explicit decision is received, this contract remains pending and no
publication, recovery, tag or release is authorized by this document.
