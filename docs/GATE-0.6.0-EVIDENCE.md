> Historical candidate checkpoints follow. For final approval, publication and
> published-package acceptance, see [0.6.0 closeout](GATE-0.6.0-CLOSEOUT.md).
> Supplemental [fresh npm/Cargo and Codex smoke results](GATE-0.6.0-CLOSEOUT.md#fresh-local-registry-smoke-test--2026-10-25)
> cover local macOS arm64 only; they are not a new four-platform run or security audit.
> Earlier publication-blocking statements describe their original checkpoint;
> the exact later owner-approved exception did not resolve historical costs.

# Kineti 0.6.0 candidate evidence

The [contract](GATE-0.6.0-CONTRACT.md) controls acceptance. This document records
implementation evidence; it does not approve publication.

## Accounting boundary

The owner approved subscription-only implementation with **$0 new paid spending**
and confirmed a hard $0 GitHub Actions spending limit. Standard public runners
and one-day candidate artifacts are used. No registry publication is authorized.
The original spending ledger still records $55 against the approved $75 ceiling:
$71.25 is the 95% stop, leaving $16.25 nominal recorded headroom. This is not
permission to spend that amount. Historical unknown costs remain unresolved and
publication-blocking. No costs were guessed, cleared, or inserted to match $55.
Recovery requires separate review of exact evidence and proposal digests.

## Compatibility before source removal

Commit `7c2b459` added the native foundation while the former implementation was
still present. Six genuine TypeScript-generated Ed25519 receipt fixtures cover
property ordering, numeric keys, Unicode, escapes, numeric boundaries and omitted
optional fields. Native tests reproduce original JavaScript signing bytes,
accept genuine signatures, and reject altered receipts and issuer substitution.
Five separate canonical fixtures prove native encoding parity. Five valid and five damaged
legacy spending goldens were checked against the preserved original TypeScript
reader during the final audit. They cover exponent-form numbers, four-decimal
USD displays, IEEE-754 rounding boundaries and integer-less records. Native
validation preserves original integer totals, display precision and required
fields; small discrepancies, missing model/reason and missing integer stage
entries fail closed. Four journal fixtures validated by the original reader
cover both old and delimited hashes, each with fixed-decimal and raw-number
canonicalization. The native reader also verifies the original journal checkpoint. Legacy signing
and native canonical encoding are separate paths.

The original local history comparison examined **827 records**. The former
TypeScript verifier and native verifier each accepted **436**, with **zero
mismatched decisions**. Exact private evidence is retained locally in
`.kineti/gate-060-port/history-compat.json`. The saved before-removal native run
passed 29 tests (3 library, 26 integration), and its evidence check was fresh
before source deletion. Deletion happened in the later commit `1d69914`.
Original source is preserved by Git history and a private local byte backup.

This proves signature compatibility, not native authority for every old runtime.
Unsupported historical parser/runtime evidence remains readable and
signature-checkable, but cannot authorize a native check. Run fresh native
checks after migration. The historical 631-test result is preserved separately.

## Actual native environments and candidate installs

Initial access run:
https://github.com/therawlogs/kineti/actions/runs/40124627870

| Target | Actual runner | Observed architecture | Initial OS |
| --- | --- | --- | --- |
| Linux x64 | ubuntu-24.04 | x86_64 | Ubuntu 24.04.5 LTS |
| Linux arm64 | ubuntu-24.04-arm | aarch64 | Ubuntu 24.04.5 LTS |
| macOS x64 | macos-15-intel | x86_64 | macOS 15.7.9, 24G830 |
| macOS arm64 | macos-15 | arm64 | macOS 15.7.9, 24G830 |

Candidate run https://github.com/therawlogs/kineti/actions/runs/40126379788
passed all native build/test and installation jobs at commit `f2d138b`.
Both Cargo and npm were installed from packaged candidates outside the source
checkout on each actual target. Execution and self-test passed with Node and Bun
absent from the runtime PATH. Disabled npm scripts produced the explicit error.
Every measured installation was below 10,000,000 bytes; the largest preliminary
measurement was Cargo Linux x64 at 4,272,773 bytes. These are preliminary sizes,
not the final candidate packet. Any subsequent source change requires a new
exact-commit run. Final reports include per-route footprints, download sizes,
unpacked sizes, binary and artifact digests, runner identities and test logs.

## Codex host integration

Only these exact versions are in scope: **0.163.0-alpha.11.2** and stable
**0.164.0**. The alpha executable SHA-256 is
`248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
Both versions loaded isolated generated configuration, discovered eight native
gate MCP tools, and successfully called native status through app-server.
The allowlist contains initialize/initialized, config/read, ephemeral
thread/start, mcpServerStatus/list and mcpServer/tool/call; no model turn starts.
The generated configuration points the model provider to an unused local port.
HOME, CODEX_HOME, XDG and temporary directories stay inside disposable project
fixtures. Real user configuration and credentials are not loaded or modified.

Probe: `scripts/check-gate-060-codex.py --gate <candidate-executable>`; use
`--codex`, `--expected-version` and `--state-root` to pin each host and output.
Earlier native proofs are retained under `.kineti/gate-060-port/alpha-native`
and `stable-native`. Final candidate installation must repeat both proofs and
record executable/configuration digests. These checks establish host integration,
not autonomous model behavior. Cursor and Antigravity remain unverified; existing
configuration is preserved. No intervening Codex version is implied supported.

## New acceptance inventory

At candidate `6ebe8b5`, the native suite had **37 tests: 3 library and 34 integration**.
Main reconciliation adds regressions; its exact new inventory and results belong
to the replacement publication packet. See [reconciliation dispositions](GATE-0.6.0-RECONCILIATION.md).
Its named tests live in the crate and map to the
[retained behavior and exit-code table](GATE-0.6.0-BEHAVIOR.md).
Six additional mocked installer policy cases test all supported target selections,
unsupported platforms and checksum mismatch without replacing the sentinel.
They supplement actual native installations, not replace them.

Coverage includes exact/empty argv, environment/dependency/runtime changes,
signature alteration/issuer substitution, stale/future evidence, missing checks,
corrupted history, concurrent accounting, thresholds/reset, reservations,
unknown journal costs, reviewed recovery/source preservation, interrupted undo and accounting recovery,
undo guard inspection, preview/backup/repetition/uninstall, modified user
configuration, removed command/tool inventory, MCP discovery/calls and self-test.
Formatting, Clippy and the suite must pass in UTC and Asia/Kolkata for the final
commit. Final artifacts and sizes must be reviewed before publication.

## Claims audit

| Current surface | Allowed claim and evidence |
| --- | --- |
| README and packaged crate README | Gate headline; installation prerequisites; explicit platform restriction; older assistant lineage change; version-pinned Codex integration; limitations and exit table |
| GitHub Action (`action.yml`) | Runs the native participating CI gate; no Bun/evaluator/protected-write claim |
| MCP registry (`server.json`) | Native gate candidate, exact 0.6.0; no universal interception claim |
| Website (`website/index.html`) | Gate-only candidate, platform limits, reported spending and evidence limitations; no deployment claimed |
| Cargo/npm descriptions | One native implementation; Node needed only for npm installation; no other Kineti crate dependencies |
| Release workflow | Four native builds, one crate and one bundled npm package, artifact-only acceptance; no publication job or automatic version bump |

Current claims must be checked against final reports. Dated historical documents
are labeled historical and preserved, not asserted as current behavior. Native
receipts cannot prove honest tests: self-test deliberately accepts fabricated
passing output. Local keys do not defend against a same-account adversary.
Reported costs do not verify provider bills. No protected-write guarantee or
atomic authorization of later writes is offered.

## Still gated

Final exact-commit rerun, final installed-candidate Codex probes and packet review
remain required. Registry/tag occupancy must be checked again immediately before
publication. Historical monetary uncertainty remains publication-blocking.
Separate publication approval is mandatory; published-package installation
checks can run only after authorized publication, and partial publication must
be reported explicitly.
