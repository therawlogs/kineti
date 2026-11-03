# macOS descendant stop: supported whole-VM boundary

Date: 2026-11-02 Asia/Kolkata. Status: whole-VM termination verified offline with a Linux guest on macOS; macOS-guest and production owner-adapter acceptance remain pending. No production platform was switched.

## Supported route and scope

A task can be placed entirely inside a dedicated VM, with its owner controller outside. Stopping the whole VM ends guest execution independently of guest process groups, sessions or reparenting. Apple documents destructive `VZVirtualMachine.stop(completionHandler:)` with completion/error reporting: [Apple stop API](https://developer.apple.com/documentation/virtualization/vzvirtualmachine/stop(completionhandler:)). The installed public SDK header confirms those semantics and warns that guest shutdown is not graceful. A stop acknowledgment must inspect errors and final state, not just assume success from initiating a request.

For an existing supported macOS-hosted implementation, Lima's `vz` backend uses Virtualization.framework and provides `limactl stop --force`: [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/), [Lima stop](https://lima-vm.io/docs/reference/limactl_stop/). This exact command was tested. Pinned Lima 2.2.1 force-stop kills its VM driver/host agent; it is not a direct invocation of Apple's stop completion method. That distinction is preserved: [Lima 2.2.1 implementation](https://raw.githubusercontent.com/lima-vm/lima/v2.2.1/pkg/instance/stop.go). The test establishes supported whole-VM termination, not acceptance of a new native VZ controller.

This is a broader isolation unit, not a discovered API for killing every descendant of an arbitrary native-host PID. Existing launchd process-group failures remain valid. No private coalition call, custom Seatbelt policy, Endpoint Security policy, polling-based sandbox, process-tree revoker or filesystem fallback was introduced.

## Fresh offline verification

Host: macOS 27.2 arm64, existing Lima 2.2.1 with `vz`. Guest: cached Alpine Linux arm64. Both original VMs were stopped before testing. The disposable guest was cloned into a new project-contained instance with no shared host mounts; its writable disk had a different inode from the original. Only the clone was started or stopped. No image, binary, dependency or tool was downloaded.

The guest fixture used the existing dedicated synthetic task UID 31001. It created an ordinary child, a child with a new process group, a new-session child, and a double-forked new-session child. Readiness records bound PID, parent, process group, session, UID, boot ID, process start ticks and scheduled write time. All four forms were ready, including confirmed distinct group/session identities, before host stop. Records and prior work were synced before the destructive test.

The host then called the supported force-stop command on that clone. All VM instances reported stopped and the clone's local SSH endpoint became unreachable. The recorded VZ backing-process PID was later independently confirmed absent. The stop returned before the scheduled late writes. The harness waited beyond that deadline, cold-started only the clone, and checked:

1. The boot ID changed; this was a cold boot rather than resuming the old execution state.
2. All original readiness records matched their saved contents.
3. No child left a late-write file, and no process belonging to the synthetic task UID remained.
4. All 38 pinned protected owner-record files still matched their earlier hashes, and no persistent model-auth cache existed.
5. The clone was stopped again. All three instances are now stopped, with disks retained.

The native stop's observed duration is recorded as a sample, not an immediate-stop latency guarantee. Guest PID numbers alone were not used to claim process identity across reboot; boot ID, task-UID absence, preserved readiness records, VM state, unreachable endpoint and the backing-process check supply separate observations. No in-guest cgroup kill or process-group cleanup was used as the stop mechanism in this test.

A separately compiled, read-only framework probe confirmed actual host support, the public stop selector and macOS bootloader/restore-image class availability. It created no VM. Inside the enclosing managed sandbox, the same capability query returned unsupported; that restricted result is retained separately rather than reported as a host hardware limitation. No entitlement or system setting was added. Existing Lima's entitlement and SDK metadata were inspected read-only.

Existing project-local SSH credentials were used only to operate the local disposable guest. There was no fresh subscription/account login, credential transfer into a model, token creation or scope expansion. The synthetic payload performed local process and file operations; this is not acceptance of guest network isolation or all outbound-data policy.

## Native-host and coalition limits

No compatible documented native-host all-descendant barrier was established in this pass. Source visibility is not a supported API guarantee. Apple's XNU coalition-termination implementation explicitly allows existing members to continue forking after a termination request, so the function name does not establish an all-member kill barrier: [Apple XNU source](https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/bsd/kern/sys_coalition.c). The tested public SDK lacked `sys/coalition.h`. No unsupported syscall or private entitlement was used to bypass that gap. These observations are not proof that every future macOS control is impossible.

## What this enables, and what remains pending

Apple supports macOS guests on Apple silicon using a compatible restore image, macOS bootloader and platform configuration: [Installing macOS on a VM](https://developer.apple.com/documentation/virtualization/installing-macos-on-a-virtual-machine). A macOS guest could combine the previously tested stock Codex native filename policy inside that guest with a host-owned whole-VM lifecycle boundary. This is a feasible documented architecture to investigate, not a verified integrated product. No macOS guest image was installed or booted in this pass.

The Linux sample does not close dynamic filename exclusions: applying a macOS policy around the VM host process does not automatically enforce guest filenames. A native-host macOS owner adapter remains blocked. A macOS-guest implementation still needs image/resource feasibility, protected host ownership, exact VM/task binding, admission and revocation ordering, fail-closed stop/error acknowledgment, owner-crash supervision, no automatic resume after revocation, snapshots/recovery after abrupt stop, accounting/audit integration and model/tool acceptance. Published supported controls must enforce the boundary; no custom sandbox fallback is authorized.

All A01–A14 rows remain partial or pending. No fresh subscription login, additional acceptance model turn, credential expansion, merge, publication or deployment is authorized by this report. Whole-VM force-stop is destructive to the test guest's execution state; production work preservation and recovery still require explicit acceptance evidence. This probe used a synced disposable clone and preserved both original disks byte-for-byte.

## Records and preservation

Evidence is in `.kineti/autonomy-macos-stop-20261102/`: baseline work/accounting inventories, original VM disk/config/EFI hashes, clone/start logs, exact host/guest source and argv, fork/session records, force-stop and cold-start observations, final backing-process check, framework probe/source, public SDK extract, artifact digests, verification script and native operational receipt. The clone's disk is retained under the project's Lima directory. Earlier reports and acceptance failures remain unchanged.

Production Rust code and existing user work did not change. Only this report and the living acceptance matrix were edited, after guarded native undo registration. Original ledger/signature and evidence/undo prefixes remain preserved; new evidence appends. Fresh accounting assessment matches the baseline: integrity verified, reservations clear, thirteen monetary unknowns and reported amounts unchanged. Strict spend/status remain blocked on unresolved history, and operational evidence does not become independent test or release authority.

New paid spending, downloads, subscription logins, acceptance model turns, reviewer agents and release actions were zero. Full native regression suites were not rerun for this research-only pass. No owner-service installation or production migration was performed.
