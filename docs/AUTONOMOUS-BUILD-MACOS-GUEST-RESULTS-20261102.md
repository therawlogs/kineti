# macOS guest: combined offline boundary result

Date: 2026-11-02. Status: combined stock filename-policy and host-controlled VM-revocation fixture passed offline. Production owner-service and model-driven acceptance remain pending.

After the owner-authorized cleanup of failed disposable instances and incremental build cache, installation of the new `kmac3` guest completed within the existing 25-minute allowance and 15 GiB free-space guard. The verified local Apple image was reused without another download. Actual guest: macOS 26.6.2 (25G83), arm64; host: macOS 27.2 arm64; Lima 2.2.1 VZ. Configuration retained two CPUs, 4 GiB RAM, 64 GiB logical disk, plain mode and no shared host mounts. Earlier failures, metadata archives and cleanup manifests remain preserved.

## Observed results

1. Stock Codex 0.164.0 denied nineteen protected-path attempts with real permission errors. These include new root/nested environment/key/PEM/private filenames, rename and link paths, new protected directories and an existing protected-directory child read. Ordinary file creation and atomic replacement worked.
2. The full protected-directory policy uses both root-name and descendant deny globs, such as `**/.ssh` and `**/.ssh/**`. No custom Seatbelt policy/backend was supplied. The corrected symlink-target probe follows the link rather than returning EEXIST from O_EXCL.
3. The app server itself, inside the stock sandbox, denied three filesystem API effects: new excluded-file write, protected-directory creation and protected-file read. Ordinary write/read succeeded. Its transport exited normally. Only initialize/initialized and filesystem requests were sent; no login, account, thread or model turn was requested. This is client filesystem API evidence, not acceptance of a real model edit tool.
4. A short unrevoked control produced all four expected late writes under the same policy: ordinary child, new process group, new session and double-forked child. The main case then observed five live fixture PIDs, including confirmed detached group/session forms, before stopping the VM through supported Lima force-stop.
5. Revocation returned before the planned child-write deadline. The controller waited past that deadline and cold-booted the guest. Its boot ID changed; readiness records matched, no late-write files appeared and no probe process remained. No persistent Codex auth cache was present. The stop call's roughly 61 ms sample is an observation, not a guaranteed latency bound.
6. The controller stopped the guest afterward; final inventory reports all four retained VMs stopped. About 26 GiB host free space remained. The successful installed guest is retained for future work; the failed instances deleted with the owner's cleanup authorization were not restored.

The guest stock executable SHA-256 matched `d8990e3ee7acfbc26f91265b12a1a83f0fee40d0e0f8f1f40a37d4ec21f499cf`; both compiled probe artifacts also matched their host hashes before execution. Full raw argv, profiles, kernel PID/group/session observations, RPC responses, boot IDs, artifact identities, stop and restart results are retained in the evidence directory.

## Scope and remaining work

This verifies the combination in a macOS guest, extending earlier separate host-policy and Linux-guest VM-stop results. It does not establish an arbitrary native-host all-descendant kill primitive. The tested stop mechanism is Lima 2.2.1 VZ force-stop, not a direct test of Apple's stop completion callback. No custom sandbox fallback, process-tree security revoker, private coalition API or relaxed permission mode was introduced.

The production grant service remains the Linux implementation. A production macOS-guest owner adapter still needs protected host ownership and exact task/VM binding, admission/revocation ordering, immediate owner-crash supervision, fail-closed stop/error acknowledgment, no automatic resume of revoked grants, snapshots/recovery, quotas and accounting/audit integration. Broader escape/tool paths, direct model edits, real model repair, concurrent work and all installed platform/footprint requirements remain pending. No full A01–A14 row is declared accepted.

Existing local guest-management SSH access was used. No Apple ID, OpenAI/subscription account login, model turn, provider credential expansion, paid resource, merge, publication or deployment was performed. The historical ten real model-turn attempts remain unchanged. Production Rust code did not change; full Cargo suites were not rerun for this fixture-only pass. Operational evidence is not independent test or release authority.

Prior VM and management files were byte-hash checked. Accounting is unchanged: integrity verified, reservations clear, thirteen historical monetary unknowns and reported amounts preserved. Original signed history remains intact; new operational evidence appends. Existing source/user work and the locked project goal were not changed.

Evidence: `.kineti/autonomy-macos-recovery-20261102/attempt3/`, especially `combined-guest-result.json`, `recovery-events.json`, exact configuration/source, installation/boot logs, final inventory, preservation assessment and native operational verification. Earlier failure evidence remains under the preceding attempt directories and the guest-feasibility evidence directory.
