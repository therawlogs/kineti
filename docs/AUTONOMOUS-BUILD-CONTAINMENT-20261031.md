# Supported descendant termination and native adapter

Date: 2026-10-31. Contract: `AUTONOMOUS-BUILD-CONTRACT.md`, version 1.

## Outcome

A supported Linux mechanism has been verified live: place the whole app-server in an independently owned cgroup v2 before executing task code, then revoke with `cgroup.kill` and acknowledge completion only after `cgroup.events` reports `populated 0`. The task must have a different, non-root identity, no cgroup delegation and no owner control channel.

Both the direct kernel prototype and Kineti's new native adapter passed against a real pinned app-server. Ordinary children, detached-session children and double-fork children were stopped. A no-revocation control performed its delayed write, demonstrating that the canary mechanism was active. Already-open output files remained empty after revocation.

Implementation continued with a small owner-only native lifecycle adapter. This uses the existing Linux kernel interface and the stock Codex Linux sandbox. No custom sandbox, model router, swarm or workflow engine was introduced. Native macOS remains unsupported for this lifecycle mechanism; its earlier detached-child failure is preserved as a dated result.

This removes the demonstrated process-lifecycle blocker for the tested Linux candidate. It does not complete task grants, the automatic repair/evidence loop, accounting eligibility or full supported-host autonomy acceptance. No A01–A14 row is marked complete by this checkpoint.

## Tested environment and provenance

- Local disposable Lima clone, created from the verified-stopped existing VM. No start, stop, guest command or configuration edit targeted the original VM; it remained stopped.
- Lima 2.2.1, Apple Virtualization.framework backend, cached Alpine image, two CPUs, 1 GiB memory, 6 GiB sparse disk. No cloud runner or paid resource.
- Guest: Linux arm64, kernel `6.18.22-0-virt`, Bubblewrap 0.12.0, cgroup v2.
- App-server: official public `codex-cli 0.166.0`, Linux arm64 musl. Archive SHA-256 was checked against official release metadata before extraction or execution.
- App-server executable SHA-256: `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`.
- Test task: existing non-login guest test account, UID/GID 2501; no credentials or login were added. An empty home was created only in the disposable clone.
- Owner controller: guest UID 0, outside the task cgroup. Owner evidence/grant fixture was root-only. No owner revocation operation was exposed to agent MCP tools.
- Native Linux build: Rust/Cargo 1.91.1, lockfile-pinned vendored dependencies, offline build. Matching Alpine Clippy was installed only in the disposable clone, without a Rust upgrade.
- This is a new Linux candidate, not a claim that the previously tested macOS alpha host was fixed.

Official kernel documentation defines `cgroup.kill` as killing the affected group and descendants, including concurrent-fork handling. These semantics are the supported mechanism; live fixtures verify this environment and native adapter, not general reliability. [Linux cgroup v2](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)

## Authority and launch boundary

The controller assigned its child to the cgroup in the pre-exec launch path, before the app-server or any task code could fork. It then switched to the separate unprivileged task account and used `PR_SET_NO_NEW_PRIVS`. Post-hoc PID attachment would leave a launch race and is not accepted by this design.

The task could neither migrate to the root cgroup nor modify its membership control. The stock command sandbox rejected those writes as read-only. The actual app-server client `fs/writeFile` operation also failed to modify the root-owned grant fixture under the separate task UID. No general protected-write or automatic-interception capability is claimed for Kineti.

The owner control channel remained outside the task's writable and process scope. The controller, not a project-local agent file or a confirmation flag, initiated the kill. OS root ownership establishes a role boundary; it does not prove a human identity. A real owner-approved task-grant service and its approval evidence are still required.

No signal-tree polling, PID ancestry enumeration, process-group approximation, modified sandbox policy engine or custom cross-OS sandbox was used to stop descendants. Reparenting and `setsid()` do not change inherited cgroup membership. Kernel confirmation, rather than a successful `turn/interrupt` response, determined completion.

Remote jobs, external requests and effects already committed before revocation are not undone by stopping processes. A timeout or remaining cgroup population must keep authority revoked and report uncertain completion. A separate owner coordinator must reject new launches and must never reuse a revoked task identity for another task.

## Live results

Each table row was exercised once with direct kernel control and once with the final native adapter artifact. An earlier native run before the musl lint fix also passed and is retained separately. These are actual app-server command/API cases, with zero model inference turns in this checkpoint.

| Case | Expected | Direct kernel | Native `kineti` |
| --- | --- | --- | --- |
| No-revocation control, detached child | Delayed write completes | Passed | Passed |
| Ordinary child | App-server/children stop; delayed output stays empty | Passed | Passed |
| New-session detached child | Same, despite `setsid()` | Passed | Passed |
| Double-fork detached child | Same, despite reparenting | Passed | Passed |

For every revocation case the app-server exited with SIGKILL, the group reported `populated 0`, and no delayed bytes appeared during the five-second window after the kill. The child delay was four seconds; files were opened before revocation to exercise an already-open descriptor. All task groups were observed empty before removal.

Additional native role tests passed:

1. The unprivileged task invoking native revoke with valid-looking flags was denied with exit 3.
2. An owner process running inside the target cgroup was denied, avoiding self-termination of the control plane.
3. A group with a writable control directory was denied.
4. Portable regression cases reject root, arbitrary service, nested and traversal paths, UID 0 as the declared task UID and invalid wait limits.

These samples support this lifecycle mechanism. They do not substitute for grant expiry/revocation during a complete authenticated model task, adversarial credential/outbound-data acceptance or broader A02/A03/A08/A10 acceptance.

## Native implementation and interfaces

New module: `core-native/crates/kineti-cli/src/host_control.rs`.

Read-only status, usable without claiming a configured task:

```sh
kineti host status
```

Experimental owner operations, only on a correctly prepared Linux owner control plane:

```sh
kineti host inspect --cgroup /sys/fs/cgroup/kineti-task-<unique-id> --task-uid <non-root-uid>
kineti host revoke --cgroup /sys/fs/cgroup/kineti-task-<unique-id> --task-uid <non-root-uid> --wait-ms 1000
```

The native adapter:

1. Requires the independent owner role, exact task-prefixed group paths and a nonzero declared task UID. The UID argument is informational until authoritative grant/launch binding is implemented; it is not identity proof.
2. Checks real cgroup v2 filesystem type, root ownership, non-writable control directories and owner membership outside the target group.
3. Pins directory descriptors and opens controls relative to them with `O_NOFOLLOW` and `O_CLOEXEC`.
4. Writes the kernel kill control and waits for population zero, with a bounded 1–10,000 ms wait. A timeout fails closed as `inspect-uncertain`.
5. Rejects native macOS rather than falling back to process groups.
6. Returns versioned `kineti-host-control-v1` data, explicitly reporting that task approval, launch membership and grant identity are not established by this primitive.

CLI entrypoint and library wiring use the same native implementation. MCP adds only the read-only `kineti_host_status` tool, yielding nine gate tools in the current source. The privileged revoke operation is deliberately absent from the agent tool catalog. This is the owner-plane/task-plane separation, not full task CLI/MCP parity; those required task controls remain unimplemented.

Existing receipt execution, spending policy and legacy exit meanings remain unchanged. No release version was chosen or bumped. The installed PATH executable was not replaced. The adapter is an unreleased source change, not a published-package acceptance claim.

## Validation and evidence

Local evidence: `.kineti/autonomy-containment-20261031/`.

| Evidence | Meaning |
| --- | --- |
| `guest-probe-4-result.json` | Completed direct-kernel live cases |
| `guest-native-probe-final-result.json` | Completed final native-adapter live cases and native artifact digest |
| `native-role-final-result.json` | Live final native owner-role and control-directory denials |
| `guest-*-probe*.py` | Exact fixture/controller code; includes preserved earlier attempts |
| `guest-evidence.tar` | Guest protocol traces, fixture state and validation logs, collected locally |
| `artifact-verification.json` | Official Codex asset URL, published checksum and observed checksum |
| `native-undo-registration.json` | Native exact-argv source recovery registration |
| `before-native/`, `starting-work.tar`, `pre-edit-inventory.json` | Byte/mode backups and checkpoint before product edits |
| `native-undo-manifest.json`, `linux-cast-fix-undo.json` | Two newest-first source recovery layers; guarded undo refuses concurrent byte, mode or parent/leaf symlink changes |
| `host-account-check.json` | Existing subscription allowance checkpoint; no credentials persisted |
| `source-artifact-digests.json`, `evidence-sha256.json` | Current source/artifact/evidence identities |
| `validation-commands.json`, `linux-validation-results.jsonl` | Exact commands, environments, exits and output references |

On the final source, macOS native tests, formatting and strict Clippy passed under both UTC and Asia/Kolkata (six commands, each exit 0). Linux native tests and strict Clippy passed under both timezones (four commands, each exit 0). Linux-specific Clippy initially caught a redundant musl filesystem-type cast; it was fixed without suppression, with a separate exact undo layer, and all final checks and live native cases were rerun. Exact results and limitations are recorded in `validation-commands.json` and the final evidence index. Full native suites contain 24 library tests and 55 integration tests on each tested OS/timezone, plus the existing isolated child proof appearing separately in the transcript. Formatting and Clippy are separate checks, not signatures or independent execution. The operational receipt for the collected lifecycle observations is not a passing-test authority receipt.

Earlier attempts are preserved and not counted as passes:

1. The existing test account lacked a home, so the app-server could not start.
2. A workspace under the historical guest repository caused a stock Bubblewrap protected-path setup error. The successful fixture used an isolated directory outside it.
3. The observer initially treated expected EOF after killing the entire app-server as an error. Population was zero; the corrected run explicitly observed post-kill state without treating EOF as a failure.
4. An initial offline vendoring attempt lacked a few target-dependent crates. Public lockfile-pinned downloads were checksum-verified; the guest build then stayed offline.
5. An early Linux shell command appended a missing-Clippy lookup to Cargo tests, producing a composite exit 127. Later validation captured each Cargo exit independently and used the matching Clippy package. The composite attempt is not represented as a clean command pass.

## Preservation and approval boundaries

The preservation check detected a concurrent `website/index.html` change before any native source write and stopped that write. It was left intact. A fresh pre-edit checkpoint captured the current workspace, and the four existing native files still matched their specific backups before application. Source recovery is registered before edits; no rollback has been executed.

The locked goal and historical monetary unknowns remain unchanged. Current native `spend check`/`status` still fail on the preserved historical unknown. No amount was guessed, zeroed or forced to $55; no integrity/reservation bypass or accounting-policy change was implemented.

Existing subscription allowance remained available at the checkpoint, with `hasCredits=false`, balance zero and 38 percent account-wide quota used. No model/API paid fallback, purchases, paid runners or provider changes were used. No credentials were copied to the guest. Local SSH transferred only public binaries, synthetic fixtures, native source and pinned public build dependencies to the disposable clone. No project source went to an external provider.

The original VM remains stopped. The disposable clone is stopped after evidence collection and retained under the project for reviewed continuation; its stop command targets only its exact instance name. No system-wide container service or custom sandbox was installed on the Mac.

Merge, publication, deployment, provider settings and credential/scope changes remain separately gated. No such action was taken.

## Contract position and next implementation boundary

The tested Linux lifecycle subcase is feasible and implemented. Remaining work still includes protected one-time owner grants and their launch-time binding; expiry/revocation state and restart behavior; automatic snapshots and conflict-aware recovery; evidence invalidation and bounded repairs; action-aware accounting dimensions and current included-resource eligibility; task CLI/MCP parity; and a full authenticated Linux model task with denial, dirty-workspace, concurrent-edit and interrupted-resume scenarios.

A01–A14 remain pending overall. In particular, this checkpoint does not establish zero-prompt supported-host model autonomy, automatic interception, four-target regression/install/footprint acceptance or release readiness. It establishes the specific kernel lifecycle control that was missing from the macOS candidate and continues the approved native implementation without weakening the contract.

References: [Linux cgroup v2](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html), [Lima usage](https://lima-vm.io/docs/usage/), [Lima stop](https://lima-vm.io/docs/reference/limactl_stop/), [Codex app-server](https://learn.chatgpt.com/docs/app-server).
