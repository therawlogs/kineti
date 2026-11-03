# macOS guest feasibility: offline checks and blocked provisioning

Date: 2026-11-02. Status: host preflight verified; image transfer resumed through normal approval; macOS guest combination unrun. No production adapter or platform migration was introduced.

Latest update: the image completed and verified; the installation retry hit the retained storage guard. See `AUTONOMOUS-BUILD-MACOS-RECOVERY-20261102.md`. Transfer/provisioning descriptions below retain earlier checkpoint history.

## Verified offline

On actual macOS 27.2 arm64, stock cached Codex 0.164.0 blocked nineteen meaningful protected-path operations with EPERM/EACCES, while ordinary creation and atomic replacement worked. Cases cover new root/nested environment, key, PEM and private names; rename/link paths; new protected directories; and a read within an existing protected directory. The final corrected fixture also produced all four scheduled child writes when left running: ordinary, new process group, new session and double fork. Those controls establish that the future revocation fixture can write when not stopped.

A separate no-auth app-server probe inside the stock native sandbox denied three filesystem API effects: new excluded-file write, protected-directory creation and existing protected-file read. Ordinary write and read worked. Only initialize/initialized and the five filesystem requests were sent. No account/login or turn request was sent; the server exited normally and its isolated auth cache remained absent. This tests those client filesystem API operations, not a real model's edit tool or complete production integration. Official scope references: [OpenAI permissions](https://learn.chatgpt.com/docs/permissions), [app-server API](https://learn.chatgpt.com/docs/app-server).

The full-directory test found that a root-name glob alone blocked directory creation but allowed a child read inside an existing protected directory. Adding explicit descendant deny globs through the same stock permissions API fixed that case. Protected directories use both `**/.ssh` and `**/.ssh/**`, and corresponding rules for `.git`, `.kineti`, `.aws` and `.codex`. No custom Seatbelt policy or sandbox backend was supplied.

An initial C symlink-target test returned EEXIST because it used O_EXCL on the existing symlink. That was not enforcement evidence. The corrected v2 fixture follows the symlink and requires a real permission error. Initial failures and the older raw results remain retained; only the final nineteen-case v2 results are counted here. A checker also initially compared a child's parent PID across reparenting; it now compares stable PID/group/session/result fields and preserves the parent observations. Earlier shell-based symlink tests are separate evidence.

## Guest prerequisites and preparation

Installed Lima 2.2.1 provides an experimental macOS-guest route. Its requirements match the available Apple-silicon host; host RAM is 16 GiB. The prepared configuration selects macOS 26.6.2, VZ, two CPUs, 4 GiB RAM, a 64 GiB logical guest disk, plain mode, no shared host mounts and no import of personal SSH public keys. The configuration validates. Existing project-local Lima management credentials would control the disposable guest; the scripts do not read its generated GUI password or attempt Apple ID/OpenAI sign-in. Source: [Lima macOS guests](https://lima-vm.io/docs/usage/guests/macos/).

The pinned public Apple image is 19,772,231,540 bytes, SHA-256 `885503b7f4b06609e9a512f2befd40f59730640a3f1233e3892d60affdd51c95`. A metadata-only request confirmed availability, size and the matching advertised digest. Bounded range support was checked before parallel transfer. No cached restore image was found in the project or the targeted local cache/index checks; that is not a claim that every disk location was exhaustively searched.

The first bounded transfer batch stopped after thirty minutes. Its retained manifest records 114 complete ranges, totaling 7,650,410,496 bytes (38.7% of the image), plus incomplete data. Complete ranges mean validated response range/length, not cryptographically verified image contents. The full image is absent and no full-image digest acceptance has occurred. An earlier single-stream partial is also retained. All downloads, configurations, source and principal test artifacts remain in the project. A 15 GiB host free-space threshold prevents continuing these operations at low space; it is not a continuous production disk quota.

Prepared scripts resume only a pure batch-timeout case with the same artifact, deny redirects, require the final pinned SHA-256, recheck accounting and space, and then provision only the new guest. The guest harness is prepared to rerun the nineteen-path policy, five filesystem API cases, a short unrevoked child control, and an active four-child VM force-stop followed by a cold-boot preservation/late-write check. These guest steps have not executed and their scripts are not accepted production controls. No existing guest is overwritten or reused as the macOS guest.

## Earlier review block and subsequent resumption

Starting the continuation/provisioning command was rejected because automatic approval review could not complete after reaching its usage limit. The rejection explicitly was not an unsafe-action determination. Its message offered trying again on November 4 at 12:07 PM, without establishing a timezone. No credit purchase, review bypass, alternate executor or weaker sandbox was attempted. A subsequent user instruction to continue permitted further independent offline checks and documentation, but did not resolve the review system's unavailable capacity.

The first continuation attempt was not executed. On the owner's subsequent request to pick up the work, the normal approval retry succeeded. The prepared continuation command is now running and resuming the same image from its retained ranges. No macOS guest has been created at this checkpoint; guest acceptance remains unrun. Original VMs were not started during this pass. Original VM disks/configuration/EFI and management-file hashes are recorded for preservation checking. Prior work and accounting are preserved.

## Acceptance and next step

The active continuation must complete and verify the pinned image, then run the prepared guest cases. Do not treat the Linux-guest whole-VM result, host filename checks or host filesystem API checks as macOS-guest combination acceptance. If provisioning requires fresh account authentication, model inference, a purchase or changed credential authority, stop at its separate gate.

Production owner/VM binding, immediate owner-crash supervision, admission/revocation ordering, stop/error acknowledgment, safe recovery, quotas, real model/direct-edit acceptance, installed-platform checks and release approval remain pending. New paid spending, fresh subscription/account logins, acceptance model turns, reviewer agents and release actions were zero. No custom sandbox or production Rust change was introduced. No full native regression run was needed for this research-only pass. Accounting integrity remains verified, monetary reservations clear, all thirteen historical unknowns and reported amounts unchanged; strict spending/release restrictions remain in force.

Evidence: `.kineti/autonomy-macos-guest-20261102/`, including corrected v2 sources/results, initial diagnostics, filesystem RPC transcript, schema/provenance, native operational preflight receipt, image transfer manifest, blocked checkpoint and prepared continuation scripts. Operational signatures do not supply independent test or release authority. All A01–A14 rows remain partial or pending.
