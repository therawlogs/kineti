# macOS checkpoint decision

Date: 2026-11-03. Status: the owner approved stop/export checkpoints in this chat. Implementation and acceptance remain incomplete. This document grants no credential, signing, model, spending or release authority.

## Verified boundary

The global npm Kineti 0.3.9 package and command, this project's legacy Cursor MCP registration and its always-on Cursor rule were removed with verified exact backups. Main Codex configuration and locked project state are unchanged. No affected Cursor or old Kineti runner process was running. Source-built verification tools remain; this is not a product reinstallation.

Native guest capability checks ran through stock Lima root system provisioning in the retained no-mount macOS 26.6.2 guest. Kernel limits accepted and read back 512 total processes and 128 per UID. The bounded APFS image reported 268,394,496 bytes, below its 268,435,456-byte class. Both attempts restored the prior kernel values, 2,000 total and 1,333 per UID. These are capability readbacks, not full alternate-broker or process-exhaustion acceptance.

The first attempt failed because `diskutil apfs snapshot` is not a supported verb on that guest. The corrected direct Darwin `fs_snapshot_create` call ran as guest root and returned `Operation not permitted (os error 1)`. Apple's installed SDK manual and [published syscall manual](https://raw.githubusercontent.com/apple-oss-distributions/xnu/main/bsd/man/man2/fs_snapshot_create.2) require both superuser privileges and an additional entitlement. No entitlement was added; no signing, SIP, authenticated-root or other security setting was changed. No custom sandbox was substituted. The guest backend remains unadmitted.

Exact commands, source/artifact hashes, kernel state and failed admission records are in the private `.kineti/owner-completion-20261103/guest-admission/` and `guest-admission-native-api/` directories. Both native owner sessions stopped their VM and the original VM configuration was restored. No provider login, administrator authentication, real model turn or new paid spending occurred.

## Approved stop/export alternative

Replace live APFS filesystem snapshots with checkpoints taken across a controlled whole-VM shutdown and fresh read-only export boot. This is the owner-approved replacement for the checkpoint mechanism. It adds reboot latency and remains unadmitted until runtime and production integration checks pass.

1. Keep grant authority, counters, wall-clock accounting, journal and delivered projections on the host. Guest startup runs only the protected worker; no task, SDK or persisted task job may start until the host authorizes its phase.
2. After a command, durably record checkpoint intent and stop admitting task calls. Flush the bounded task volume, request an orderly guest shutdown, then join the native owned driver and verify stopped state. If orderly shutdown fails, force-stop for safety, mark the checkpoint uncertain and preserve raw guest state plus the previous verified host projection. Never claim clean completion or replay the action.
3. Start a fresh worker-only boot. All prior task descendants are gone. Attach the root-protected task image with stock `hdiutil` read-only mode, verify APFS identity/capacity and `MNT_RDONLY`, then export through the existing bounded checkpoint codec. No task-writable or concurrently changing source is accepted as a sealed checkpoint.
4. Bind the exported generation, original grant, VM identity, actual runtime and boot identity in the host audit. Reuse or resume requires fresh validation; counters and elapsed time never reset. A later writable execution phase starts a fresh SDK session. Revocation remains available during shutdown, startup and export and must reap every owned launcher before acknowledging stop.
5. Prove read-only export, crash/partial-write refusal, stale-generation rejection, detached-descendant shutdown, callback/CLI/MCP parity and delivered-diff preservation through the production acceptance entry point. Existing filename exclusions and sandbox remain stock SDK controls. Supported volume/process controls still need exhaustion and alternate-broker tests.

This proposal requests no new credentials, paid service, sandbox implementation, provider login or acceptance model turn. It reuses the two retained VMs and installed public SDK/toolchain artifacts. It does not request an Apple entitlement or permission to weaken security settings. Its performance and complete owner integration are unverified until implemented and tested.

If the original APFS snapshot requirement is retained, the macOS backend stays fail-closed; an independently supported, explicitly approved snapshot capability is required before completion can be claimed. No further guessed snapshot utility or security-setting experiments are planned.


The owner replied “Approve stop/export checkpoints” to the exact proposal. This authorizes the changed checkpoint mechanism only. Cold shutdown also destroys memory-only guest model authentication. No automatic reauthentication, token export, credential broker or new login is authorized by this decision. Uninterrupted model continuation across such checkpoints is not established and must remain separately gated.
