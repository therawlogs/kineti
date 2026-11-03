# Autonomous build feasibility audit

Date: 2026-10-31. Contract: `docs/AUTONOMOUS-BUILD-CONTRACT.md`, version 1.

## Decision

**HOLD at the host feasibility gate. No autonomy implementation or accounting-policy code changed.**
The existing MCP-only integration cannot enforce this contract across native host tools.
A pinned Codex sandbox has passed seven command-boundary fixture probes, so this is
not a finding that Codex is intrinsically infeasible. A dedicated app-server adapter
is a candidate, not yet a supported or accepted host. No host has passed the complete
boundary needed to advance to contract delivery step 2. Do not build an unenforced
grant prototype and describe it as autonomy. No custom OS sandbox is proposed.

The owner explicitly approved implementation and section 6 with $0 new paid spending
on 2026-10-31. This audit does not request that approval again. Merge, publication,
deployment, provider settings, credentials and billing remain separately gated.
The original proposal is preserved without rewriting its dated approval-time status.

## Evidence location and limits

Exact command results, source/artifact SHA-256 values, generated host schemas,
platform identity, starting-work inventory and recovery preview are under
`.kineti/autonomy-audit-20261031/`. These are local observations, not signatures,
independent review, clean-machine acceptance or a new release packet.

The first offline build failed because the default Cargo cache lacked `shell-words`.
Reusing the existing project cache succeeded, with no dependency download:

```sh
CARGO_HOME="$PWD/.kineti/cargo-home" CARGO_TARGET_DIR="$PWD/.kineti/autonomy-audit-20261031/target" cargo build --locked --offline --manifest-path core-native/Cargo.toml
```

The PATH executable reports v0.3.9 and warns about `verify_command_argv`.
The existing release executable reports 0.6.0. Neither is proof of the current source.
The newly built debug executable reports 0.6.1 and its digest is recorded separately.
Its fresh `spend check` and `status` both exit 3 on historical unknown
`rr-contract-repair-1792163379549`. No unknown cost was valued, cleared or overwritten.
A first-error result does not establish that every later integrity or reservation
check has run successfully. No accounting-health pass is claimed.

A read-only app-server checkpoint (no model turn) reports ChatGPT `prolite`, ordinary
usage allowed, 36 percent quota used, `hasCredits=false`, balance `0`, and no available
rate-limit reset credits. This is a point-in-time resource-eligibility observation,
not a historical billing statement or proof of total project costs. Refresh before
new dependent work. No purchase, API-key fallback, reset redemption or paid runner
was used. Account identity and credential values are excluded from the saved projection.

## Candidate host and capability map

Pinned candidate: local Codex CLI/app-server `0.166.0-alpha.5.1`, macOS arm64.
The actual executable SHA-256 and environment are in `digests.json` and
`environment.json`. Only this executable was probed; no version interval is supported.

| Surface | Existing native gate | Host evidence | Remaining condition |
| --- | --- | --- | --- |
| Exact argv execution | `receipt.rs::execute` uses ordinary child processes and process groups | Codex Seatbelt fixture enforced command restrictions | Route every claimed execution through the pinned host; prohibit broad fallback |
| Direct model file edits | MCP cannot intercept host edits | Not exercised with a real model turn | Prove built-in edit tools apply the same effective task policy |
| Filesystem exclusions | A cwd is not containment | Outside write, protected write, secret read, traversal and symlink fixture denials | Verify canonical roots, missing paths, rename/link races, nested scripts and protected authority store |
| Descendants | Cleanup is bounded; escaped descendants are not contained by Kineti | A shell child inherited the sandbox write denial | Test detached/session children, cancellation and surviving descendants |
| Network | Gate has no destination isolation | Direct test-net connection denied with EPERM | Approved destinations/data policy, proxy bypass/DNS/raw sockets still untested |
| Credentials | Environment inherited by ordinary child spawn | Dummy protected-file read denied | Broker-scoped credential operations, inherited environment and real host credential separation untested |
| Owner grant | Exact argv trust is project-local, not owner identity | App-server approval messages exist in local schemas | Owner-controlled grant authority outside agent write scope, expiry/revocation and session binding absent |
| Revocation | No task grant or live revoke controller | Interrupt message exists; not executed | Revoke immediately, stop running effects, prove escaped child handling |
| Evidence | Signed receipts and conservative input checks exist | Source inspection only for autonomous scheduling | Automatic reruns, bounded repair, durable task linkage and prompt capture absent |
| Recovery | Durable intent/completion and uncertain-replay guards exist | Source inspection; no historical rollback attempted | Automatic snapshots, ownership/conflict checks and crash injection absent |
| Spending | Integer locked fail-closed legacy policy exists | Fresh source binary still blocks unknown historical valuation | Separate valuation/integrity/reservations/eligibility/incremental spend without weakening hard failures |
| Other hosts | Cursor/Antigravity unverified | No probes | Unsupported |

The public app-server documentation currently describes more controls than its
nonexperimental generated schema exposes. The pinned local experimental schema adds
`ThreadStartParams.permissions` and `CommandExecParams.permissionProfile`.
Do not deserialize the online examples blindly: use the pinned schema and explicit
experimental negotiation. Schema generation is not a running authority service.

## Boundary probes actually run

`codex sandbox /usr/bin/true` failed inside the enclosing sandbox with
`sandbox_apply: Operation not permitted`, then succeeded when launched outside that
enclosing sandbox with platform approval. This distinguishes nesting limits from
host capability. No safety prompt suppression or auto-approval bypass flag was used.

A disposable named permission profile allowed reads, granted writes only to the
fixture work directory, denied its private directory and disabled network. One
allowed write succeeded. These seven assertions passed in one local invocation:

1. Outside write denied (EPERM).
2. Dummy credential file read denied (EPERM).
3. Protected policy write denied (EPERM).
4. Symlink read into denied directory denied (EPERM).
5. `..` traversal write denied (EPERM).
6. Shell child outside write denied; no target created.
7. Direct IPv4 connection to documentation-only test-net address denied (EPERM).

The initial dotted CLI config form failed parsing before execution. Retrying with
one inline profile table succeeded. No target action was performed on that failure.
Platform Python emitted cache-directory warnings under the intentionally narrow
profile; these did not count as successful sandbox escapes.

These are command-only, fixture-only probes with broad read enabled outside the
explicit dummy-secret exclusion. This is not the final secret-read policy. They do
not satisfy A03, A08 or A12. They did not test a real agent, built-in edits, approved
network requests, secret brokering, process revocation or every escape path.
Host-launch platform approval reviews in this audit are not excluded from a future
A03 count. A03 sample size is zero; no zero-prompt claim is made.

## Architecture required before implementation

1. An owner-controlled launcher resolves the grant before starting the task agent.
   It owns approval evidence, expiry/revocation, quotas and exact irreversible-action
   approvals. The agent receives an informational grant copy and structured decisions.
   A project file or agent-supplied flag can never mint authority. Project snapshots
   remain local; authoritative host approval storage must be outside agent write scope.
2. A small pinned Codex app-server adapter translates allowed effects to actual host
   permissions, validates effective restrictions after restart and refuses unsupported
   capabilities. It supplies no unfiltered host filesystem/configuration API to the
   agent. The owner control channel is not an agent-accessible tool. Losing either
   policy enforcement or resource evidence stops dependent execution.
3. A native task coordinator owns snapshot inventory, file ownership, bounded repairs,
   evidence invalidation and durable intent/completion. Commands run through the host,
   not ordinary `receipt::execute` as an alleged sandbox. Host events bind observed
   actions/prompts/results to the task; unrelated native tools cannot silently escape.
4. A separate eligibility assessment reads preserved accounting records. Unknown
   historical valuation is displayed independently. Integrity damage or uncertain
   monetary writes/reservations still deny accounting-dependent work. Included-resource
   eligibility requires current host evidence; no guessed zero or paid fallback.
5. CLI and MCP call the same policy/recovery functions. Adapter loss is an explicit
   unsupported/deny result, never a local-only allow decision. Cancellation and quota
   checks apply to concurrent work in aggregate and stop new actions immediately.

Stable decision envelope (design only):

```json
{"schema":"kineti-task-decision-v1","decision":"deny","grant_id":null,"action":"execute","reason":"host-enforcement-unverified","required_change":"verified owner-controlled host adapter","execution_started":false}
```

Decision values: `allow`, `deny`, `needs-scope`, `needs-action-approval`,
`retry-within-grant`, `inspect-uncertain`. Error information must distinguish
unsupported host capability, expired authority, stale evidence and accounting damage.

Proposed grant shape (design only; not an approved grant or selected public API):

```json
{
  "schema":"kineti-task-grant-v1",
  "id":"owner-issued-id", "revision":1,
  "project":{"identity":"canonical-project-id","roots":["canonical-root"]},
  "binding":{"host_digest":"sha256","session_id":"owner-session","task_id":"task"},
  "purpose":"reviewed task purpose",
  "effects":{"read":[],"write":[],"exclude":[],"operations":[],"destinations":[],"outbound_data":[],"credential_handles":[]},
  "resources":{"class":"verified-included","incremental_usd_micros":0,"concurrency":1,"wall_seconds":1800,"repair_attempts":3,"snapshot_bytes":100000000},
  "verification":{"argv":[],"inputs":[],"environment":[],"completion":[]},
  "recovery":{"owned_paths":[],"automatic_known_local":true,"conflict_policy":"preserve-and-stop"},
  "authority":{"issuer":"owner-controlled-host","approval_reference":"external-host-event","expires_at":"RFC3339","revocation_reference":"owner-controlled-channel"}
}
```

The adapter must reject unknown fields, noncanonical/escaping paths, stale approvals,
invalid/missing limits, unapproved credentials/destinations and inconsistent identities.
Limits above are illustrative and confer no authority. A host digest or signature
alone does not prove owner approval, honest tests or effective OS enforcement.

## Compatibility and denial test plan

Keep existing CLI/MCP names, aliases, receipt signature encodings, trust behavior,
accounting history and exit meanings. Add explicitly versioned task decisions rather
than reinterpreting legacy `spend check` as permission to proceed. Legacy verify/CI
remain fail-closed unless an explicit versioned action selects approved eligibility.
`status` should gain a versioned partial-health view without hiding preserved unknowns;
legacy exit compatibility must be specified and tested before changing it.

New task commands and MCP names are not selected yet. Both interfaces must expose
effective grant, status, check, recovery and explicit capability limitations. CLI-only
options must not silently disappear from MCP. Existing argv fidelity must survive.

Required additional denials before selecting a supported host:

1. Agent-created, expired, revoked and cross-session/project grants rejected.
2. Revocation during a long-running command stops affected running work and children.
3. Built-in direct edits, shell wrappers and scripts cannot modify policy or authority.
4. Missing-path/symlink/rename/hard-link traversal cannot bypass resolved boundaries.
5. Forbidden network destinations, raw sockets, proxy bypass and credential operations fail before effects.
6. Repeated out-of-scope attempts emit one deduplicated precise scope request.
7. Host/client restart cannot broaden rights or revive revoked grants.
8. Concurrent tasks share quotas and cannot restore each other's or the user's edits.
9. Dirty and untracked files/modes survive task, crash, rollback and resume.
10. Unknown valuation alone can allow verified included work; damaged accounting,
    uncertain reservations, possible charges and exhausted eligibility still stop it.
11. Fixture secrets never enter argv, grants, logs, receipts or snapshots.
12. Exact artifact/destination changes invalidate release approval; partial remote effects reconcile before retry.

## Acceptance matrix

| ID | Current status | Evidence or missing work |
| --- | --- | --- |
| A01 | Pending | Audit checks starting paths; setup/failure/resume lifecycle not tested |
| A02 | Pending | No protected task-grant service or live revocation acceptance |
| A03 | Pending | Zero real agent autonomy cases; no baseline/proposed prompt comparison |
| A04 | Pending | Fixture denials only; no structured expansion/deduplication implementation |
| A05 | Pending | No task CLI/MCP parity, resume or concurrency implementation |
| A06 | Pending | Audit snapshot only; no automatic conflict-aware recovery acceptance |
| A07 | Pending | No automatic rerun/repair scheduler acceptance |
| A08 | Pending | Seven command probes pass; direct edits and other paths untested |
| A09 | Pending | Unknown remains blocked; included eligibility observed separately, amendment not implemented |
| A10 | Pending | No task limit/revocation lifecycle acceptance |
| A11 | Pending | Approval boundary retained; no new exact-action controller acceptance |
| A12 | Pending | Dummy read denied; broker/outbound/log/snapshot acceptance missing |
| A13 | Pending | Local offline build passed; full multi-target/two-timezone checks not established |
| A14 | Pending | Audit documents limits; installed autonomy instructions not implemented |

## Resume condition

Continue under the existing approval after a supported owner-controlled host path can
be established and verified for the required effects, including direct edits and
revocation. Current MCP registration cannot supply it. The command-sandbox result
supports evaluating a dedicated app-server adapter; it does not justify silently
relaxing the contract to command-only enforcement. If that adapter requires provider
settings, credential/scopes or external authority setup, present the exact required
change separately before making it. No new implementation approval is needed.

Official references checked without transmitting project source:
[Codex permissions](https://learn.chatgpt.com/docs/permissions),
[Codex sandboxing](https://learn.chatgpt.com/docs/sandboxing),
[Codex app-server](https://learn.chatgpt.com/docs/app-server).
Local executable schemas and actual probes take precedence over untested online examples.
