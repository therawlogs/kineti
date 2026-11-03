# macOS owner adapter: offline lifecycle feasibility

Recorded 2026-11-01 UTC (local checks crossed into 2026-11-02 Asia/Kolkata). Status: blocked on supported all-descendant revocation. No production adapter, platform migration or custom sandbox was introduced.

## Finding

The previously verified stock Codex macOS Seatbelt profile supplies dynamic filename exclusions, but launchd does not supply the required descendant stop boundary in the tested configuration. An ordinary child stopped; children that changed process group, created a session or double-forked survived and wrote after native bootout or service-leader kill. The stock sandbox allowed those detach operations. These are failed revocation cases, not acceptance successes.

The installed `launchd.plist(5)` documents remaining-process cleanup by the job's process group. Apple's published manual describes the same scope: [Apple launchd manual](https://github.com/apple-oss-distributions/launchd/blob/main/man/launchd.plist.5). Apple's daemon guide recommends launchd for service lifecycle, which does not establish an adversarial descendant boundary: [Creating Launch Daemons and Agents](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html). The observed limit is consistent with that documented scope; it is not a claim that every possible macOS architecture is impossible.

## Offline probes

Environment: actual macOS 27.2 arm64. Synthetic C fixtures were compiled locally with the installed Clang, using `-Wall -Wextra -Werror`; compiler temporary files stayed in the project. Stock cached Codex 0.164.0, SHA-256 `d8990e3ee7acfbc26f91265b12a1a83f0fee40d0e0f8f1f40a37d4ec21f499cf`, was used without authentication or inference. No new binary, SDK, image or dependency was downloaded.

Each transient GUI-domain launchd job had `AbandonProcessGroup=false`, `KeepAlive=false`, `ExitTimeOut=1`, a unique label and a project-local plist. Four child forms wrote readiness records with their PID, parent, process group, session and syscall result, waited five seconds, then attempted a synthetic late write. Host `ps` observations corroborated their identity and survival. All readiness records existed and no late file existed before the stop action.

| Case | Ordinary child | New process group | New session | Double fork after new session |
| --- | --- | --- | --- | --- |
| launchd, bootout | Stopped; no late write | Survived; late write | Survived; late write | Survived; late write |
| launchd, leader SIGKILL | Stopped; no late write | Survived; late write | Survived; late write | Survived; late write |
| Stock Codex sandbox under launchd, bootout | Stopped; no late write | Survived; late write | Survived; late write | Survived; late write |
| Stock Codex sandbox under launchd, leader SIGKILL | Stopped; no late write | Survived; late write | Survived; late write | Survived; late write |

The sandbox cases used a supported named permission profile: minimal read access, read access to the synthetic helper, a writable synthetic work directory, deny globs for `.env*`/`*.key`, and network disabled. No custom Seatbelt profile or policy backend was supplied. These lifecycle probes do not repeat the previous filename acceptance or establish direct-model tool acceptance.

All twelve detached children across the four cases were observed alive with late files before test cleanup. Cleanup separately checked the recorded fixture executable and unique work argument before signaling only those surviving test PIDs. Every final fixture PID was absent, every transient service was removed, and a subsequent host process check found no helper processes. This cleanup is test hygiene, not a proposed production revoker. No blanket PID scan/kill or per-UID kill was adopted.

An initial user-domain bootstrap returned EIO without starting the fixture. That diagnostic is retained separately; the four successful launches used the current GUI domain. No persistent LaunchAgent/LaunchDaemon location was written. Earlier enclosing-sandbox socket/process inspection restrictions and the first peer-PID probe failure are retained as diagnostics, not passes.

## Owner control and remaining feasibility

A separate, bounded Unix-socket probe verified public macOS `getpeereid` and `LOCAL_PEERPID` against a connected synthetic child. Its false owner claim did not change kernel UID 501 or the observed child PID; the probe rejected owner authority. The final probe kept the connection open through acknowledgment. This proves that primitive on this host, not a protected owner role, unforgeable task membership, resistance to socket delegation or a specific human's identity. See [Apple getpeereid documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/getpeereid.3.html).

Current production owner code remains Linux-only and depends on root-protected state, Linux peer credentials/proc membership and cgroup lifecycle. A macOS adapter still needs a protected owner/task identity split, process-instance and membership binding, supported all-descendant stop with acknowledgment, immediate owner-crash supervision, setup/recovery synchronization, quotas, accounting and audit integration. The GUI test domain shares the current user and does not establish those boundaries. No root service or new task account was installed.

Process-tree polling or remembered PID lists cannot be promoted to an atomic stop barrier; process groups have the demonstrated escape. A launchd coalition shown in diagnostic output does not itself establish a supported public coalition-kill interface. No private coalition API was called. Endpoint Security would require an Apple entitlement and additional policy implementation ([Apple system extensions](https://developer.apple.com/system-extensions/)); it was not requested, installed, treated as a free entitlement or used as a custom sandbox fallback. Wrapping the existing Linux VM with a macOS policy was not assumed to constrain its guest execution.

Implementation remains paused at this platform feasibility boundary. A compatible supported lifecycle mechanism or an explicitly reviewed contract amendment is needed before a production macOS owner adapter can satisfy the contract. No weaker revocation semantics or unsupported fallback was silently adopted. No new login or acceptance model turn can resolve this offline lifecycle failure by itself.

## Evidence and preservation

Evidence is in `.kineti/autonomy-macos-owner-20261101/`: C source/helpers, `probe.py`, exact job plists and command/result transcripts for each case, peer diagnostics/final result, installed manual extracts, environment/source/artifact digests, final process/VM checks, work inventory and the native operational verifier/receipt. The four cases were freshly executed, not inferred from old evidence. Production Rust code did not change; its full regression suites were not rerun for this research-only pass. The operational receipt is not independent test or release authority.

The locked goal, existing source and user work, original signatures, historical accounting/evidence/undo prefixes and prior dated reports remain preserved. Read-only accounting assessment matches the previous checkpoint: integrity verified, monetary reservations clear, thirteen valuation unknowns and reported amounts unchanged. Strict spend/status checks continue to reject unresolved history. A verifier initially confused strict spend-check output with the assessment object; that diagnostic was corrected without changing any accounting record. New native undo/evidence entries append to history.

Both existing VMs remain stopped with disks retained. New paid spending, fresh logins, acceptance model turns, reviewer agents, credential expansion, merges, publication and deployment were zero. All release and credential/inference approval gates remain separate. Full contract acceptance is still pending.
