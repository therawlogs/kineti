# Protected guest worker and stop/export checkpoint

Date: 2026-11-03. Status: offline checkpoint component verified; production macOS owner service incomplete and disabled. No A01–A14 row is accepted. This report does not authorize a login, model turn, credential broker, installation, merge, publication or deployment.

## Approved mechanism and implementation

The owner approved “stop/export checkpoints” after the retained macOS guest denied the public APFS snapshot syscall's additional entitlement requirement. The implementation adds a root-controlled guest worker and fixed owner transport using the existing guest identity and SSH channel. It grants no arbitrary privileged command interface. Root launchd installation and exact guarded updates used stock Lima system provisioning; no administrator authentication, provider login, entitlement, security-setting change, custom sandbox or new VM was used.

The installed worker and host test executable have SHA-256 `641e9389c5b8fc17a077362bc4efcc780ba29b6ca4f231c77b9c11d5a26eb959`. The protected official guest SDK remains version 0.164.0, SHA-256 `d8990e3ee7acfbc26f91265b12a1a83f0fee40d0e0f8f1f40a37d4ec21f499cf`. Worker configuration, executable and SDK paths must be canonical, independently root-controlled and hash-matched. The native worker refuses a physical Mac or a task caller. Its Unix socket checks kernel peer identity before parsing a request; the bridge authenticates the root peer. Live task-UID impersonation and alternate-broker acceptance remain unverified; source checks are not a substitute.

The worker exposes only probe, initial copied-volume preparation, shutdown and cold export. It explicitly reports `task_execution_admitted: false`. The existing shared owner engine still refuses macOS task admission. Kernel readbacks are 512 total processes and 128 per UID; the VM has 4 GiB memory and the finite APFS image reports 268,394,496 bytes. These are capability observations, not completed resource exhaustion or broker acceptance.

An initial worker attempt rejected the official SDK because its 260,293,200-byte size exceeded a decimal 256-million-byte bound. The corrected 256 MiB bound preserves the exact SDK pin. The first installed-worker checkpoint replay then lost its shutdown response. The corrected transport durably arms shutdown, sends its acknowledgment, and asks launchd to halt. Acknowledgment explicitly says stopped state is unverified. The host joins its native lifetime driver and independently verifies stopped state before booting the export worker. A timeout, missing acknowledgment or uncertain stop remains a failure requiring inspection; no action is replayed automatically.

## Verified offline evidence

The production acceptance entry point now has an explicit `--component macos-cold-checkpoint` lane. It delegates to `scripts/accept-macos-guest-checkpoint.py` and never reports full service acceptance. The Linux production lane retains its existing behavior. The component uses the installed root worker, fixed authenticated transport, stock APFS image tools and native host-held VM lease; it does not install per-case privileged fixture code or run a separate repair engine.

The live installed-worker replay passed all of these checks:

1. Pinned worker/SDK identity, actual SDK version, guest memory and kernel process-limit readbacks.
2. Arbitrary root command and probe command-argument rejection.
3. Initial bounded-volume copy; duplicate preparation refusal preserving prior state.
4. Same-boot live-export refusal; wrong-grant and expired-request refusal.
5. Acknowledged shutdown intent followed by native driver exit without forced stop, with independently verified stopped state.
6. A fresh worker-only boot, read-only APFS attachment, byte-identical codec export and repeated-export refusal.
7. Native stop verification after the export session, with the original VM configuration preserved.

Private evidence is in `.kineti/owner-completion-20261103/worker-cold-order-acceptance/`. The earlier direct root component and failed installed-worker attempts remain in their own directories. Their results are not promoted into a passing production-service result. The acceptance harness subsequently received an output-path canonicalization guard and waits for native validated readiness before touching SSH configuration; offline traversal and malformed-binding denial checks passed before any SSH call. The passing live replay precedes that harness-only hardening.

Fresh macOS arm64 native suites passed in UTC and Asia/Kolkata: 37 library and 62 integration tests per timezone. Formatting and Clippy with warnings denied passed. Exact command freshness was checked with an explicit expected argv: the initial check against the configured default command refused the mismatch, and that failure is preserved. Configured policy was not changed. Signed receipts describe local observations, not independent execution or full acceptance. Source/artifact digests, environments, commands and logs are retained privately beside this report's evidence.

Current-source Linux arm64/musl checks also passed in the retained Alpine guest: 37 library and 62 integration tests in each timezone, the release build, and the full existing offline owner-service acceptance entry point. Its native artifact SHA-256 is `bf7c16ae9f91fd69969758584dd201975e8f81c0faa7cadfb8fb37417434ce94`. The production service exercised copied failure/repair/retest, five-tool MCP, durable quotas, freshness, recovery/concurrent-edit refusal, active descendants and owner-crash stop, interrupted replay refusal, and read-only inspection. All 5,301 prior owner-store files were preserved in its first case, and later cases checked their respective prior-store inventories. Scripted console fixtures do not prove human identity. No login or model turn ran; full contract acceptance remains false. Linux Clippy with warnings denied also passed. Native Linux formatting could not run because the retained toolchain has no `cargo fmt`; no component was downloaded. The full source formatting check passed on macOS. Linux's tested new-filename exclusion limitation remains unchanged. Private commands, logs, artifact and acceptance result are in `linux-current/` under the same evidence root.

Three unused, identical legacy Bun compiled cache files were removed from the project root after confirming they were untracked, hash-identical and not open. They represented 179,447,376 logical bytes; no allocated-space claim is made. Exact bytes already existed in the verified global-package archive, so no duplicate backup was made. The native guarded undo proposal preserves original modes and refuses later changed paths. Source, offline dependency caches, current verification tools and work/accounting records were retained.

## Acceptance rows

| Row | Current checkpoint contribution | Still required before acceptance |
| --- | --- | --- |
| A01 | Original records, goal and configuration preserved; copied bytes survived read-only cold export. | Production setup/work/failure/cleanup with concurrent edits and installed artifacts. |
| A02 | Host-held VM lease and acknowledged stop/export use the native stop/reap boundary. | Full service setup cancellation, expiry/revoke/crash/restart, grant/VM/worker binding and current model replay. |
| A03 | No new build/repair outcome is claimed by the checkpoint fixture. | Shared-engine edit → failure → repair → fresh owner retest → delivered diff on macOS. |
| A04 | Fixed worker operation and argument denials passed. | Precise deduplicated scope reporting through every production tool path. |
| A05 | Existing Linux lane retained; protected guest transport is available. | Integrated macOS CLI/MCP parity, task-UID denial, concurrent quotas and file ownership. |
| A06 | Duplicate and live-export refusal; fresh read-only export and replay denial passed. | Production conflict-safe delivery/recovery and complete interrupted-operation grid. |
| A07 | Worker/SDK pins and boot identity are observed in the component replay. | Shared-engine guest runtime fingerprints and complete stale evidence/input/environment invalidation. |
| A08 | Checkpoint storage/lifecycle uses supported OS mechanisms. | Combined actual task exclusions, direct patch, terminal, descendants and alternate brokers; Linux new-filename exclusion remains unsupported in its tested stock profile. |
| A09 | Thirteen historical monetary unknowns preserved; no new spending. | Separate historical accounting resolution and live eligibility-loss acceptance. |
| A10 | Finite APFS capacity, VM memory and kernel process caps read back. | Real exhaustion, control-plane survival, broker coverage, setup/expiry/revoke limits and durable counters in the production adapter. |
| A11 | Release authority remains absent and automatic deployments remain disabled. | Separately approved exact release scope/artifacts/destinations and interrupted external-operation reconciliation. |
| A12 | No login, model turn, token export or credential expansion used. | Full task credential/network boundaries and supported authenticated continuation across cold checkpoints. |
| A13 | Current macOS native checks and exact component artifact are recorded. | Fresh native checks and isolated approved website-package installation on all four actual targets; no x64 resource is presumed free. |
| A14 | Uninstall, mechanism, evidence limits and gates are documented. | Final user instructions/defaults verified against accepted installed artifacts. |

## Remaining implementation and later gates

Connect the macOS controller and worker to the existing shared grant/audit/repair/recovery engine. Extend the grant format explicitly for guest SDK, VM identity, worker configuration and enforceable resource class while preserving Linux v1. Complete bounded workspace/home/runtime placement, actual SDK execution and MCP callbacks, host-held authority during every setup/checkpoint phase, guest runtime fingerprints, conflict-safe projections, limits and failure recovery. No component success supplies those missing implementations.

Cold shutdown destroys guest memory-only model authentication. No token may be copied to disk, exported to the host, restored or brokered under this approval. Further authenticated acceptance needs a concrete reviewed continuation strategy plus the separately gated fresh logins and bounded turns. A single passing checkpoint does not establish unattended authenticated repair across multiple cold epochs.

For independent review, prepare the immutable source/artifact manifest plus the grant parser, shared owner engine, VM lifecycle code, guest control/checkpoint implementation and complete evidence index. A new reviewer is a later explicit gate; no additional reviewer was launched.

For website installation acceptance, prepare isolated roots for `darwin-arm64`, `darwin-x64`, `linux-arm64` and `linux-x64`, with exact independently approved archive and executable digests, actual native OS/architecture checks, empty PATH/home/config fixtures, offline installation, version/self-test, absence of automatic MCP registration and guarded cleanup. Do not use the historical registry-download acceptance mode or reinstall the old global product. The retained arm64 machines supply local resources; x64 availability and no-additional-charge eligibility remain unverified. No runner, installer download or package installation is authorized by this preparation.

The old global CLI and Cursor integration remain removed. Exact restoration material stays private and requires separate reinstallation approval. Source-built development tools and the protected guest worker are the approved implementation/test tools. All thirteen monetary unknowns, original work/signatures and accounting history remain intact. New paid-spending authority remains zero.

Final verification for this batch: only `kmac3` and `kineti-autonomy` remain, both stopped. The original macOS VM configuration hash is unchanged. All 38 original work/accounting files or append-only historical prefixes, the undo guard, locked goal and main Codex configuration verified. Accounting integrity is verified, reservations are clear, and thirteen historical monetary unknowns remain unresolved; ordinary spend/status gates still refuse.
