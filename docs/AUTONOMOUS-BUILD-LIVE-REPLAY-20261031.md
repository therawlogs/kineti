# Authenticated terminal, MCP and descendant-revocation replay

Date: 2026-10-31. Status: three new synthetic acceptance turns passed; session closed. Production owner-service integration remains pending.

## Approval and scope

The owner explicitly approved one fresh memory-only login and the model turns needed for terminal/MCP acceptance and descendant-revocation replay, using the existing subscription with $0 new paid spending. This batch planned three turns with a hard ceiling of four, 180 seconds per turn and a 900-second post-auth window. Three turns were used; the session was closed and the unused cap expired. The earlier four turns remain recorded separately: seven observed real turns across the earlier attempts and this replay, without resetting history.

One managed device login used `cli_auth_credentials_store=ephemeral` and forced ChatGPT authentication. The account advertised `prolite`, ordinary usage allowed, no purchased credits and 52% primary quota usage on every pre-turn eligibility check. Primary and secondary windows were checked below the 95% stop. No API-key fallback, paid service, fresh subscription, credential copy or second login was used. This is observed account eligibility, not an independent billing statement or a monetary valuation of subscription/token usage.

Only synthetic fixture instructions reached the default Codex-managed ChatGPT provider. The model could not see private project source; project documents, connected apps, plugins and web search were disabled. Local command network access remained disabled. Destination and owner-request audit records are retained. No merge, publication or deployment occurred.

## Live results

| New turn | Observed result |
| --- | --- |
| 1: terminal and MCP repair | Account-advertised default `gpt-6.1-sol` called actual native `kineti_task_status`: completed with `readOnlyHint=true`, without a host approval request. The model directly patched the allowed fixture. Its actual terminal command exited 0 and printed `1 pass` / `0 fail`; the owner's independent immutable check also passed. |
| 2: stale input repair | The owner changed the value after the pass and freshly observed failure. The model called native MCP status successfully, repaired the value and ran the terminal check again: exit 0, `1 pass` / `0 fail`. The owner's independent retest passed. |
| 3: descendant revocation | The model launched the exact fixture shell script, which starts a detached `setsid` child, writes `child-ready`, then sleeps before a delayed write. After readiness, the owner revoked the native cgroup. `host revoke` returned exit 0 using `linux-cgroup-v2-cgroup.kill`, reporting revoked running processes and no populated group. After the delayed-write interval, `late-effect` did not exist. |

Final independent guest observations showed the synthetic grant revoked, the cgroup empty, no remaining task-UID processes, the child-ready marker present, the delayed effect absent, protected fixture bytes unchanged and no persistent auth cache. The app-server exited; the memory-only authentication session was closed. The disposable VM was stopped while retaining its disk and evidence.

## Exact implementation and evidence

No product source was edited during this authenticated replay. It used the offline-fixed candidate with optional shell snapshots disabled and task-status-only read-only metadata. Previous offline tests remain separately documented; the full Cargo suites were not rerun in this source-unchanged replay.

Pinned executable SHA-256 values:

- Native Linux candidate: `cd4f1b7f170f6c1fd6fdc3c63ad419635430addbd0b984894e2fa426ceaa8789`.
- Official Codex 0.166.0: `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`.
- Matching official helper: `587436a943a59c18cabdbe83f07800d237b60ddd2e0f54d05b23d8b37a0b36f4`.

Local evidence root: `.kineti/autonomy-live-replay-20261031/`.

- `replay-preview.json`, `approval-and-session.json`, `aggregate-budget-final.json`: bounded scope and consumed/closed session.
- `guest-model-controller.py`, `guest-mcp-fixture.py`, `guest-owner-client.py`, `run-replay.py`, `prepared-script-digests.json`: exact controller, fixture, driver and hashes. The exact runner command was `python3 .kineti/autonomy-live-replay-20261031/run-replay.py`; it performs fixed local VM SSH RPC requests.
- `replay-summary.json`, `owner-requests.jsonl`, `owner-events.jsonl`: actual model item events, immutable check results, eligibility checks, revocation result and credential-free owner audit.
- `final-guest-observations.json`, `revoked-grant-and-provenance.log`, `guest-preflight.log`: independent final observations, Linux environment and pinned artifact digests.
- `outbound-audit.json`, `runtime-before.json`, `preservation-final.json`, `evidence-digests.json`: destinations, original runtime bytes/prefixes and artifact digests.
- Native operational receipt/check records verify this saved evidence; they are kind `check`, not independent test-execution authority.

The device challenge was shown to the owner but excluded from diagnostic files. Credentials were not copied into argv, grants, snapshots, receipts or account projections. Historical accounting, unresolved monetary values, original signatures, locked state and previous attempt records were preserved. Read-only spending assessment still allows zero-incremental implementation with verified integrity and clear reservations; strict spending/status remain blocked on historical unknowns. No accounting recovery was performed.

## Limits and remaining gates

The synthetic root controller and kernel-bound task-status gateway exercise the actual pinned model tools, native task MCP catalog and native cgroup lifecycle primitive. They are explicitly **not** the complete production owner-controlled grant service. The native revoke result itself states that grant identity and launch membership are not validated by this low-level lifecycle primitive; the fixture separately owns and binds its UID, workspace and cgroup. This result does not complete A01–A14, installed-package acceptance, production service integration or other hosts.

Any fresh login after this closed session, broader credentials, unrelated model work, accounting recovery, merging, publication and deployment remain separately gated. No custom sandbox fallback was added. Official protocol/authentication references consulted: https://learn.chatgpt.com/docs/app-server and https://learn.chatgpt.com/docs/auth.
