# macOS native owner lifecycle implementation

Date: 2026-11-02. Offline implementation evidence under the approved autonomous-building contract. No full A01-A14 acceptance, installed-package acceptance or release approval is claimed.

## Implemented boundary

`vm_control.rs` adds host-only `kineti host vm-inspect`, `vm-revoke`, `vm-supervise` and `vm-run`, each requiring an exact private `--binding` file. This is VM lifecycle control, not a task grant or proof of human identity. The agent MCP catalog does not expose these commands.

Bindings pin the host owner UID, exact canonical Lima home, executable SHA-256, instance directory, configuration SHA-256 and VZ machine identifier/hardware-model hashes. Unknown authority fields, shared or redirected paths, public binding permissions and changed artifacts fail closed. Inventory must identify exactly the bound plain Darwin/aarch64 VZ guest with no host mounts. A stopped acknowledgment requires stopped state and no live host-agent or driver IDs. Command output and wait times are bounded. There is no process-group or custom sandbox fallback.

`vm-supervise` revokes on private owner-pipe loss. `vm-run` additionally owns startup: it starts pinned Lima with the supported `start --foreground --tty=false` mode and retains its direct child handle. The pinned Unix implementation uses `syscall.Exec` to replace the launcher with the host agent, preserving the PID. Admission requires both inventory driver IDs to match this owned child. Cancellation kills and reaps that child before stop verification, preventing a surviving launcher from activating after acknowledgment. Driver readiness is not SSH readiness, task approval or resource-limit acceptance.

An exclusive, private per-instance flock lease prevents two native supervisors from owning the same VM. The lease stays open until cleanup and is not inherited by the VM child. The lock file is retained rather than unlinked; inspection and explicit revocation remain available without taking the lease. The live concurrent-owner test refused both a second startup and a second supervisor without changing the first driver, then verified lease reuse after stop.

The binding and control pipe remain outside the guest. Host UID numbers and guest UID numbers belong to different kernels. This does not defend against malicious programs already running as the host owner, or prove survival of a killed supervisor itself. External lifecycle changes by another host owner process are outside this primitive's ownership guarantee.

## Fresh checks

Evidence root: `.kineti/autonomy-macos-owner-integration-20261102/` (uncommitted runtime data).

1. Release build with the project Cargo cache, `--locked --offline`: passed. Native binary is 1,651,856 bytes, SHA-256 `f0f94de107f0f18ab5ad5ae7443f91a1e0d6f8927318da2bf33cdfad6020125a`. Exact source/runtime hashes and host environment are in `digests-environment-lease.json`.
2. Final UTC and Asia/Kolkata suites: each 28 library and 62 integration tests passed. `cargo fmt --check` and Clippy `--all-targets -- -D warnings` passed. Commands and output are retained in `tests-lease-utc.log`, `tests-lease-kolkata.log`, `fmt-lease.log` and `clippy-lease.log`. These are macOS arm64 source checks, not installed-artifact or four-platform acceptance.
3. The final lease-enabled candidate replayed all three startup-loss cases (`live-start-lease`), the full descendant/cold-boot case (`live-owned-lease`) and all seven negative bindings (`negative-bindings-lease`). Earlier no-lease results below remain historical and the corresponding binary/source bytes are preserved in `kineti-before-lease` and `preserved-before-lease.json`.
4. Seven real negative binding invocations refused stale executable/configuration/identifier hashes, unknown authority, traversal names, wrong owner and public permissions. Original binding bytes were unchanged.
5. `live-eof2`: synthetic owner crash closed the native supervisor's pipe; four sandboxed descendant types were stopped and no deadline writes appeared after cold boot. This case used independently started VM activation.
6. `live-start`: pipe loss before launch, during launch and while running returned verified stop in approximately 1.43, 0.61 and 0.40 seconds, respectively. The VM remained stopped after a further five-second check in each case.
7. `live-owned2`: native-owned foreground startup plus ordinary, changed-process-group, new-session and double-fork descendants. Killing only the synthetic owner yielded native stop acknowledgment about 0.48 seconds later, before every future write deadline. After the deadlines and cold boot, no late files or auth cache appeared. Final state was stopped.
8. `resource-stock2`: the guest's `RLIMIT_NPROC` hard ceiling permitted three children before `fork` returned EAGAIN. Each child detached into a new session, reported inherited hard-limit checks through a pipe, and failed to raise the hard limit. All probe children were reaped. This is per-UID primitive evidence; production requires a dedicated task UID and complete alternate-launch/tool-path testing.
9. A supported fixed-size 64 MiB UDIF/HFS+ guest disk image mounted without account or sudo authentication. A write through the stock Codex sandbox stopped with ENOSPC at 65,011,712 bytes; the image backing file was explicitly excluded, and runtime/home/temp writes were scoped inside the mounted volume. The volume was detached and the VM stopped. This proves this fixture's finite writable volume, not a general host disk quota or complete task integration.

## Preserved failures and corrections

`live-eof` is incomplete: its harness checked the log immediately after VM stop without waiting for the supervisor acknowledgment. The independent stopped-VM EOF check passed; the corrected live run is separate. The cause of the first orphan supervisor's disappearance was not established.

`live-owned` stopped safely after SSH connection refusal because VM driver readiness preceded SSH boot readiness. `live-owned2` adds a bounded transport-readiness wait and is the accepted replay. The first resource-image attempt used an invalid `-format` argument for a blank image; subsequent runs use documented `-type UDIF`. All failed attempts remain preserved and are excluded from successful acceptance evidence. Managed-sandbox VM inspection was rejected as uncertain; native inspection was run outside that enclosing sandbox.

## Preservation and authority

Native guarded undo was registered before each source/report edit; stage manifests and original bytes remain under the evidence root. Historical signature bytes and journal prefixes verify. Current spend assessment exactly matches the preceding assessment: integrity verified, reservations clear and thirteen historical valuation unknowns unresolved. No source reset, cleanup, merge, publication or deployment occurred. New paid spending, subscription logins and model turns were all zero.

The old protected Linux binaries and stores remain unchanged. Linux filename-glob enforcement for newly created matches remains unsupported by the tested candidate; these macOS results do not qualify it. No grant is issued by this report.

## Work still required

The full macOS task owner service is not implemented: its grant authority, dedicated guest identity, bounded-volume lifecycle, host-controlled transport, wall/repair limits, copied-workspace snapshots, conflict-safe delivery/recovery, expiry/resume and every CLI/MCP/tool route must be connected and tested. The new primitives must not be represented as that completed adapter. Pinned public Lima source also confirms that plain mode continues to execute system provisioning scripts. A dedicated guest task account and owner bootstrap have not yet been installed or tested; no new credential or account access was introduced in these tests.

Fresh model acceptance, independent review of the resulting candidate, installed website artifacts across the required native platforms, final documentation and historical accounting clearance remain pending. Fresh logins/model turns and all release actions retain their separate approval gates.

Pinned supported implementation reference: Lima v2.2.1 [`start.go`](https://github.com/lima-vm/lima/blob/v2.2.1/pkg/instance/start.go) and [`start_unix.go`](https://github.com/lima-vm/lima/blob/v2.2.1/pkg/instance/start_unix.go). Exact fetched copies and local hdiutil help are retained with the evidence.
