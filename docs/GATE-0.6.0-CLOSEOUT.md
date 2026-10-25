# Kineti 0.6.0 release closeout

Date: 2026-10-25. Release publication and required published-package acceptance
are complete. This completion record supplements the contracts; it does not
rewrite their frozen approval-time language or grant new authority.

## Approval and provenance

The owner separately authorized main integration and approved the exact
[publication contract](GATE-0.6.0-PUBLICATION-CONTRACT.md), SHA-256
`a3c26f3b18aa0ef0629acc1337001d71b3fea16a46ba3702eb2642ed4b10e0c4`,
with “1. Approve the full contract”. Its pending header is preserved verbatim.
The original and reconciliation contracts also retain their historical status
text; this record describes subsequent completion.

Release commit and `v0.6.0`: `01c710c8ee3c125d19a0f37988ff95d27e743339`.
Package source: `80cd5d1c27433a402858fd82897db57741271e30`.
These commits have identical Git trees. Documentation/workflow closeout commits
do not replace the release tag or rebuild approved packages.

Published: [Cargo](https://crates.io/crates/kineti-cli/0.6.0),
[npm](https://www.npmjs.com/package/kineti/v/0.6.0),
[GitHub release and audit assets](https://github.com/therawlogs/kineti/releases/tag/v0.6.0).

| Archive | Download bytes | SHA-256 |
| --- | ---: | --- |
| kineti-cli-0.6.0.crate | 55,840 | `5787f90a7ad9204d9f651f3fe2a5f1ad0d82ebf2324223a147dfcadd1bd459a9` |
| kineti-0.6.0.tgz | 3,691,089 | `73ce7ca10e28aa73a5588b62cbb07a89e306e0313273421ca2b0d79c6732ce2e` |

Unpacked package sizes: Cargo 247,462 bytes; npm 7,639,093 bytes.
Registry downloads matched the approved archives. Initial npm authentication
failure was recorded as partial publication; subsequent successful publication
and verification completed it. Frozen release notes and approval bytes remain
unchanged. Final results are separate release assets.

## Acceptance

The [eight-commit disposition](GATE-0.6.0-RECONCILIATION.md) resolves the main-only
changes. [Behavior and compatibility mapping](GATE-0.6.0-BEHAVIOR.md) and
[historical evidence](GATE-0.6.0-EVIDENCE.md) cover the retained gate boundary.

[Exact candidate acceptance](https://github.com/therawlogs/kineti/actions/runs/40184093249)
passed 52 native tests (3 library, 49 integration), formatting and Clippy on four
native targets under UTC and Asia/Kolkata. Eight isolated candidate installations
passed. Original receipt comparison covered 827 records, with 436 accepted by
each verifier and zero differing decisions. The dated 631-test result remains
historical. [Main CI](https://github.com/therawlogs/kineti/actions/runs/40184879280)
also passed. These are saved results, not tests rerun for this documentation edit.

Isolated Codex configuration loading, discovery of eight tools and real native
MCP calls without model turns passed on macOS arm64 for exactly
`0.163.0-alpha.11.2` and stable `0.164.0`. Executable SHA-256 respectively:
`248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad` and
`d8990e3ee7acfbc26f91265b12a1a83f0fee40d0e0f8f1f40a37d4ec21f499cf`.
This establishes host integration only for these tested versions.

[Published-package acceptance](https://github.com/therawlogs/kineti/actions/runs/40186583905)
passed all eight actual registry installations outside source checkouts. All four
self-test outcomes, native MCP discovery/calls, disabled-script errors and runtime
execution with Node/Bun absent from PATH passed.

| Native runner | Target | Cargo installed bytes | npm installed bytes |
| --- | --- | ---: | ---: |
| ubuntu-24.04 | Linux x64 | 2,348,986 | 2,358,784 |
| ubuntu-24.04-arm | Linux arm64 | 1,844,562 | 1,854,356 |
| macos-15-intel | macOS x64 | 1,865,681 | 1,875,420 |
| macos-15 | macOS arm64 | 1,590,075 | 1,599,810 |

All footprints include initial integration assets and are below 10,000,000 bytes;
toolchains, caches and accumulated records are excluded. Runner identities,
commands, binary digests and full reports are in the release asset
`published-acceptance.json`, SHA-256
`cbdd413cbac19010f41831c7fc67c827cf3b8969995f770778a6a1cec5c27a6e`.

The manual [published-package workflow](../.github/workflows/published-060.yml)
preserves the exact workflow used in verification commit
`c73fb72d867abb87027a0f415cfa12bcaf00e49a`. It pins the released source and registry
versions/digests, uses read-only permissions and one-day artifacts, and does not
publish packages. The separate candidate workflow remains available. Dispatching
verification remains subject to the hard $0 resource limit and available quota.

## Accounting and continuing limits

The approved release-specific accounting-policy exception allowed publication;
it did not resolve any historical cost. All 13 unresolved/qualified-zero records,
other allocation gaps, original signatures and recovery records remain intact.
Recorded spending remains $55 against the $75 ceiling; the 95% stop is $71.25,
with $16.25 nominal recorded headroom. New paid spending authorization remains
$0. Native spending checks continue failing closed on unknown costs. No recovery,
reset or invented token-to-dollar valuation was applied. Recovery still requires
an exact evidence-backed proposal and separate approval.

“Record checks, detect changed inputs, and block participating CI commands when
evidence or reported spending fails policy.”

Commands can fabricate passing output; signatures do not prove honest tests.
There is no universal interception, same-account adversary protection,
provider-bill verification, protected-write guarantee or atomic authorization of
later writes. Cursor/Antigravity remain unverified. Website deployment was
excluded and was not performed. These limits and unresolved accounting remain;
no required release installation or publication step is pending.
