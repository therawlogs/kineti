# Production owner-service offline acceptance progress

Date: 2026-10-31. Status: production-path offline hardening and three real Linux owner-service fixtures verified. Full production owner-service/model acceptance remains pending.

## Implementation

`task_owner.rs` now selects ephemeral credential storage, disables connected apps/plugins/web search, rejects an ambient authenticated account, and starts an ephemeral task thread before command execution. It uses the supported named permissions profile, on-request host approvals and the existing pinned app-server; no model turn is started. The app-server must start its required native Kineti sidecar. `mcp.rs` validates the actual connected sidecar catalog: exactly the five fixed tools, unchanged scope-free input schemas and unchanged status-only consent annotations. Missing/disconnected servers, pagination, additional tools, scope drift or annotation drift deny startup. A credential-free capability event records the observed connection and catalog.

This closes an app-server-spawned sidecar startup gap; it does not add a production model repair loop or a credential broker. The production runner still uses exact owner-approved check and repair argv. No fresh login, model inference, credential expansion, provider/billing change, new paid spending or release action was performed. The previously approved authenticated synthetic batch remains closed and its seven aggregate observed turns remain unchanged.

## Live, no-model production-path verification

All cases used the actual native `task approve`, `task serve`, `task run/status/pause/revoke/recover` path, fixed protected owner store, separate task UID, pinned official Codex 0.166.0, stock host sandbox and Linux cgroup v2. A scripted root PTY tests the owner-role approval mechanism; it is not proof of a particular human's identity. No custom sandbox fallback was introduced.

| Fixture | Fresh observation |
| --- | --- |
| `prod-bootstrap-01` | A deliberately failing check ran, the exact approved repair ran once, and a fresh check passed. The production host capability event contained the actual connected five-tool sidecar. Unchanged evidence was reused; changed task input triggered a fresh check. Wrong-session and outside-cgroup task callers, a duplicate service and an alternate owner store were denied. Pause/resume kept counters; exhausting the original quota stopped work. Revoke denied recovery. All 57 pre-existing owner-store files retained their bytes. |
| `prod-crash-01` | A detached child became ready. The fixture killed only its own owner-service process and observed the orphan cgroup still populated. Restarting the same native grant drained and removed that group, prevented the delayed write and marked the task uncertain. Counters survived. Blind execution replay and recovery were denied. All 85 pre-existing owner-store files retained their bytes. |
| `prod-missing-sidecar-01` | A new fixture-only executable copy was intentionally not executable by the task UID. Required sidecar startup failed, and the production service denied the task before any check or repair started. The cgroup was drained and task source stayed broken. All 112 pre-existing owner-store files retained their bytes. |

Final guest observations show all three services closed, no active grant markers, no remaining task-UID processes and no persistent auth cache. Original project files were not replaced by copied task output. The disposable VM is stopped with its disk retained.

The crash result has a precise limit: the orphan continues until owner-service restart. There is no verified service-manager watchdog providing immediate descendant termination on an abrupt controller crash. This is restart cleanup and uncertain-replay protection, not crash-proof supervision.

## Regression, provenance and preservation

Fresh native suites contain 25 library tests and 59 integration tests. The added adversarial catalog test rejects an extra executor, changed scope, altered consent hints, a disconnected server and a hidden catalog page. Full macOS arm64 and Linux arm64 tests run separately under UTC and Asia/Kolkata; exact outcomes are in `verification-results.json`. Strict Clippy, formatting and whitespace checks are retained. This does not establish all four release platforms, installed packages or clean-host acceptance.

Evidence root: `.kineti/autonomy-production-service-20261031/`.

- `edit-preview.json`, source backups, `source-undo-manifest.json`, `source-undo-registration.json`, and the separate formatting/test undo layer preserve exact source predecessors with hash/mode guards. No rollback was executed.
- `guest-production-bootstrap.py`, `guest-production-crash.py`, `guest-production-missing-sidecar.py`, and their `production-*.log` files contain exact fixture commands and observations.
- `final-guest-observations.json` contains protected owner event chains and heads. `linux-provenance.log`, `source-digests.json`, `source-artifact-digests.json` and `evidence-digests.json` bind source, executable and environment.
- `mac-tests-utc.log`, `mac-tests-kolkata.log`, `linux-tests-clippy.log`, `linux-tests-kolkata.log`, `mac-clippy.log` and `local-environment-and-native-checks.json` retain fresh checks.
- `runtime-before.json` and `preservation-final.json` verify original accounting, locked goal, signatures and historical files byte-for-byte, allowing only append-only evidence/undo prefixes and the matching undo-guard advance. No historical cost was guessed, cleared or recovered.
- A native kind-`check` receipt verifies saved observations and preservation. Its signature/operational freshness do not establish independent test execution; strict test authority remains separate.

Exact suite commands are `TZ=UTC cargo test --locked --offline --manifest-path core-native/Cargo.toml` and the same command with `TZ=Asia/Kolkata`; strict Clippy adds `--all-targets -- -D warnings`. Linux also supplies the existing vendored-source configuration: `--config 'source.crates-io.replace-with="vendored-sources"' --config 'source.vendored-sources.directory="vendor"'`. All generated build/evidence/cache files are project-contained. Strict spending/status retain exit 3 for historical monetary unknowns; read-only spending assessment retains verified integrity, clear reservations and $0 new paid authority.

## A01–A14 progress and remaining acceptance

No full contract row is declared accepted by these fixtures.

| Row | Current evidence and remaining gap |
| --- | --- |
| A01 | Original runtime/accounting bytes and prior owner-store bytes preserved; broader concurrent/installed-host scenarios remain. |
| A02 | Protected grant, session/UID boundary, resume quotas and revoke tested; complete authenticated production task remains. |
| A03 | Earlier authenticated synthetic terminal/MCP/revocation replay passed. Production model-driven edit/build/fail/repair/retest integration is not implemented or accepted. |
| A04 | Scope/caller/catalog denials tested; complete precise expansion-request deduplication across real model paths remains. |
| A05 | Actual app-server-started sidecar catalog and native CLI lifecycle verified; complete model-driven parity and approved concurrency scenarios remain. |
| A06 | Native snapshot/conflict regressions and actual crash/restart uncertain-replay protection verified; comprehensive crash/recovery injection remains. |
| A07 | Fixed-command stale rerun, bounded repair and evidence reuse verified; automatic command discovery and model repair integration remain. |
| A08 | Existing stock profile and kernel boundaries retained; comprehensive direct-edit/network/credential/escape adversarial acceptance remains. |
| A09 | Preserved valuation unknowns and zero-incremental assessment verified; authenticated resource lifecycle and all relevant failure cases remain. |
| A10 | Aggregate quotas and explicit revoke/restart cleanup verified; immediate fail-stop supervision on controller crash remains unverified. |
| A11 | No release authority added; exact release approvals remain required. |
| A12 | This work uses no credentials or model calls; log hygiene is distinct from a future scoped credential broker's enforcement. |
| A13 | Two native arm64 environments and both timezones checked; all supported release targets and any required installed-package checks remain. |
| A14 | This dated report narrows claims; final installed instructions and independent acceptance review remain. |

## Remaining gates

The next substantive implementation gap is an owner-service model-repair mode with explicit bounded model authority and a reviewed memory-only credential lifecycle. The current owned-local grant accepts no credential handles and must not be silently reused as authenticated model authority. That work must preserve existing grant compatibility and quotas. It can be prepared offline; live validation needs a separately authorized fresh login and model-turn batch. No login or extra model turn is requested or started by this report.

Independent review, broader adversarial acceptance and crash supervision remain necessary before production acceptance. Merging, publication, deployment, accounting recovery, broader credentials and paid resources retain separate exact gates.

Official supported API/configuration references consulted: https://learn.chatgpt.com/docs/app-server and https://learn.chatgpt.com/docs/config-file/config-reference.
