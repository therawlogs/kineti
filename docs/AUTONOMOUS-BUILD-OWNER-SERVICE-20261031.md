# Autonomous building: owner grant service and repair loop

Date: 2026-10-31. Status: experimental source candidate; not release acceptance.

The owner approved implementation and section 6 of the autonomous building
contract, with $0 new paid spending. This work uses local resources and existing
tooling. It makes no model turns, buys no credits and changes no provider or
billing settings. Historical monetary unknowns remain unresolved. Merge,
publication, deployment and installed-package acceptance remain separate gates.
The locked project goal is unchanged.

## Implemented scope

1. `task_grant.rs` parses a strict, versioned request. It binds project and locked
   goal, session, exact verification/repair argv, pinned host, separate task UID,
   expiry, wall/command time, aggregate attempts, memory/PID bounds, concurrency,
   exclusions and recovery policy. This initial mode accepts owned local resources,
   one worker, zero new paid spending, no task network and no credential handles.
   Parsing a request or displaying its digest grants nothing.
2. `task_owner.rs` provides a Linux owner control plane outside the task cgroup.
   Approval requires UID 0 and an interactive owner console with a readable
   summary and numbered choices. There is no approval flag or MCP approval tool.
   Kernel UID is an owner-role boundary, not proof of a particular human identity.
   Owner-protected approval, service executable digest, active marker, chained
   events and persistent counters live outside task writes under a dedicated
   `/var/lib/kineti-owner-native-v1` store. This initial adapter accepts only
   that one store per host so per-UID leases cannot be multiplied across stores. Task IDs cannot be replaced or renewed.
3. Local Unix IPC checks kernel peer credentials and the fixed grant/session.
   Effectful task callers must have the approved UID, workspace and active cgroup
   membership. Only the owner can pause or revoke through the owner channel.
   Task cancellation can stop authority, not expand it. Per-task and per-UID
   leases prevent multiplying the same task account's running authority.
4. A pinned, root-protected Codex 0.166.0 app-server runs as the task account.
   Its supported named permissions profile restricts commands to the copied
   workspace and public runtime paths, denies original/owner data, and disables
   task network. Stock host sandboxing and Linux cgroup v2 enforce this mode;
   there is no custom sandbox or macOS/process-group fallback. Memory and PID
   controllers are required. Glob expansion and starting snapshots are bounded
   to 32 path levels; custom exclusions currently support top-level roots only.
5. `task_snapshot.rs` copies starting dirty and untracked work without changing
   the original project. It preserves ordinary file bytes/modes and symlinks,
   does not follow links for file reads, and omits Git, gate state and named
   credential/configuration paths. Exclusion roots get empty owner-owned mount
   placeholders; excluded source bytes are not copied. This is not a universal
   detector of secrets in arbitrary files. Root-protected baseline blobs and
   output inventories support preauthorized recovery. Conflicts stop recovery;
   displaced files are quarantined and original project files are never restored
   over. Interrupted or uncertain execution is not blindly replayed or undone.
6. The owner runner performs configured checks, runs the exact approved repair
   after definite failure, and retests within the same persistent quotas. It
   conservatively fingerprints all included workspace files, declared inputs,
   public runtime, host, environment and grant. It freezes the task cgroup while
   recording a completed checkpoint and drains the whole group before success.
   Unchanged evidence can be reused only under a still-valid grant. Expiry,
   revocation, accounting failures, incomplete output or exhausted limits stop
   dependent work. Clean resume keeps expiry, elapsed-time origin and counters.
7. The bound MCP mode exposes only five scope-free tools: task status, run,
   check, recover and cancel. It cannot select argv, change scope, approve,
   renew, revoke as owner, or invoke the legacy arbitrary-execution tools.
   The CLI also provides owner-only approval/serve/pause/revoke operations.
8. `spend assessment` adds read-only, action-aware dimensions. It requires
   accounting integrity and clear monetary reservations before reporting
   zero-incremental local eligibility. It preserves historical valuation unknowns
   and does not grant resources, task authority or release authority. The ordinary
   `spend check`, status, verify and CI policies retain their historical-cost block.

## Fresh verification

Local evidence is in `.kineti/autonomy-service-20261031/`. It is ignored runtime
material, not a publication artifact. Commands, output, source/artifact hashes,
starting-work snapshots and guarded undo registrations are retained there.

| Check | Observed result |
| --- | --- |
| macOS native full suite | 24 library and 59 integration tests passed; default environment and Asia/Kolkata runs retained |
| Linux native full suite | 24 library and 59 integration tests passed in UTC; integrated Asia/Kolkata result retained separately |
| Strict Clippy and formatting | Checked for both native platforms; exact logs identify the source generation |
| Real app-server repair, fixture v6 | Failed check, one repair, passing fresh check, unchanged-evidence reuse, owner revoke; original dirty/untracked project unchanged |
| Real app-server lifecycle, fixture v9 | Stale inputs rerun checks; original aggregate quota retained; wrong session and outside-group task peer denied; clean pause/resume reuses valid evidence without resetting counters; later quota exhaustion stops; recovery after revoke denied |
| Real restricted MCP, fixture v9 | Initialize, five-tool discovery and bound task status succeeded; legacy execution tool rejected |
| Real detached descendant, fixture v8 | Owner revoke during an active setsid descendant removed its cgroup and prevented its delayed write; active marker removed; original project unchanged |
| Accounting behavior on this project | Ordinary spending check and status still exit 3 on `rr-contract-repair-1792163379549`; read-only assessment permits zero-incremental local eligibility without resolving that record |
| New regression tests | Paid/network/credential/identity/scope rejection, bound-catalog escape denial, valid historical unknown versus integrity/reservation blockers, dirty/untracked/mode/link preservation and recovery conflict preservation |

Earlier fixture failures are retained. They exposed a missing host-executable read
rule, nonexistent exclusion destinations rejected by stock Bubblewrap, and a
clean-resume phase error. The accepted fixture runs above followed those repairs.
A diagnostic-only guest build temporarily printed synthetic command errors; it
was kept out of product source and replaced by the normal candidate build.

Observed test summaries and chained/signed records do not prove an honest test
program, independent execution, human identity or release acceptance.

## Operation and remaining acceptance

This is an explicit Linux runner for preapproved commands and configured repairs.
It does not yet constitute the whole autonomous model-driven building contract.
The owner must provision a separate task account, protected executable/store and
supported cgroup controllers; the source does not install a privileged service.
The tested guest account had no copied credentials and no authenticated model
session. The owner control plane remains trusted.

1. Inspect a request with `kineti task plan --request <request.json>`.
2. At the separate owner console, use a root-protected executable for
   `kineti task approve --owner-store <store> --request <request.json>` and choose
   the numbered decision. `serve` must use the same approved executable bytes.
3. Run `kineti task serve --owner-store <store> --id <id>` outside task cgroups.
   The owner can initiate `task run` using the store's socket, ID and session;
   the task MCP sidecar receives a fixed binding. Status is read-only. Pause,
   revoke and cancel do not merge or publish copied task output.
4. On abrupt controller interruption, preserved running/uncertain state requires
   inspection; the next owner startup stops a recorded orphan group and refuses
   blind replay. This is not a separately verified crash-proof service manager.

Full A01–A14 acceptance remains pending. In particular, an authenticated Linux
model-driven task, automatic command discovery, app-server-spawned sidecar
lifecycle, all host/edit/escape and crash/recovery adversarial cases, independent
review and installed-package/clean-host acceptance are not established here.
The real MCP fixture called the bound native server directly; it was not a model
turn. Cursor and Antigravity remain unverified. No automatic interception or
universal protected writes are claimed.

No merge, publication, deployment, external data mutation, accounting recovery,
new paid spending or credential expansion was performed or approved by this work.
