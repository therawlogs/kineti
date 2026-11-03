# Production model repair and immediate crash supervision

Date: 2026-10-31. Status: implemented and verified offline; authenticated production model acceptance remains gated and unrun.

## Implemented changes

The protected owner grant accepts an optional, explicit `model_repair` mode. Existing command-repair requests keep their original meaning. Model mode requires `resource_class: included-subscription`, empty `repair_argv`, an exact model, an owner-reviewed prompt, a persistent turn ceiling, provider `codex-chatgpt-subscription`, and data scope `approved-task-workspace`. Credential-value fields, arbitrary provider selection, task network and paid fallback remain unsupported. The protected request digest and service executable pin still bind approval. Adding model authority to an old request requires a new reviewed grant; project files cannot expand authority.

Model grants also pin the SHA-256 of the matching official sibling `codex-code-mode-host`. The owner validates its protected executable path, the host profile grants read access to that exact public runtime file, and every use/reuse checks its digest. A missing or changed helper stops work before a fresh login or model turn.

The production runner now performs the model turn between a definitely failing check and a fresh owner-orchestrated retest. Model, repair and verification counters persist across pause/resume and cannot reset on missing counter data. A turn is counted before transmission, so uncertain outcomes consume authority conservatively. Only the copied workspace is edited; original project files remain outside task writes. Early completion notifications are retained rather than lost while awaiting the start response. Failed/interrupted/incomplete turns preserve uncertainty and partial work.

Before each model turn, and periodically during a long turn, the service requires a ChatGPT account, ordinary allowance, explicit no-purchased-credit evidence and quota below the 95% stop. Missing evidence or possible incremental spending stops work. The approved model/provider cannot silently fall back. Subagent modes, image generation, connected apps/plugins, web search, skill search and memories are disabled in the pinned host. Host safety requests are denied, never automatically approved. Observed model actions are hashed without persisting their raw arguments/output. Numeric host usage observations are recorded with thread scope and unknown monetary valuation; they are not provider bills or inferred dollar costs.

Normal `task serve` does not start a login. The new root-console `task serve-model` entry point asks the independent owner console to arm exactly one memory-only device login; it must be invoked only after separate authorization for that login and the model-turn batch. Device challenge values go to the controlling terminal, not grants, receipts or audit files. Startup rejects persistent auth caches. Codex-generated fixed-workspace trust markers are allowed, while other inherited config settings and redirected/oversized config files are rejected. A missing account returns `needs-action-approval` without consuming a model turn or repair attempt.

## Immediate crash supervision

A separate owner-only native helper pins the protected cgroup and acknowledges readiness before task code launches. It waits on a private pipe held only by the owner service; process death closes the pipe and triggers the existing Linux `cgroup.kill` stop. There is no PID reuse dependency, cgroup delegation, process-group fallback or new sandbox. On a normal shutdown, the owner drains the group before disarming/reaping the helper. Unexpected helper exit is detected by the owner's bounded checks and stops dependent work.

With the real pinned official Codex 0.166.0, the fixture verified a detached child’s kernel session and cgroup membership across the sandbox PID namespace, killed its own owner-service process, and observed an empty cgroup about 11 ms from the owner kill (about 10 ms after exit) **before any restart**. The delayed write did not occur. Restart then removed the empty group and refused uncertain replay/recovery while retaining counters. This verifies the tested single owner-process crash, not simultaneous controller/watchdog failure or every kernel/service-manager failure mode.

## Fresh offline acceptance

Evidence: `.kineti/autonomy-model-supervision-20261031/`.

| Check | Observed outcome |
| --- | --- |
| Real pinned app-server crash | `detached-session-crash-final.jsonl`: kernel-verified detached session; stopped descendants without restart; delayed effect absent; resume stayed uncertain |
| Matching helper runtime | `helper-runtime-smoke.jsonl`: real official helper arithmetic/protocol passed without auth, inference or enabled external tools |
| Eight model protocol cases | Repair/retest, early completion, no auth, quota stop, model mismatch, interrupted completion and quota loss during a turn, and matching-helper drift passed |
| Real official host, model mode without auth | `real-model-helper-no-auth.jsonl`: actual sidecar started and the deliberately failing check ran; `needs-action-approval`; zero login calls and zero model turns |
| Model authority counters | Offline double repaired twice, retained counters after pause/resume and stopped a third turn before login/inference |
| Privacy projection | Numeric-only usage retained; fixture secret text absent from returned state |
| macOS arm64 | 26 library and 59 integration tests passed separately under UTC and Asia/Kolkata; strict Clippy passed |
| Linux arm64 | 26 library and 59 integration tests passed separately under UTC and Asia/Kolkata; strict Clippy passed |
| Formatting/accounting | Format and whitespace checks passed; strict spending/status retain exit 3; read-only zero-incremental eligibility retains verified integrity and clear monetary reservations |

The eight protocol cases use a deterministic local app-server double, explicitly not a real account, model, sandbox or official-host acceptance. Their `turn/start` messages do not contact a provider. The real crash and no-auth cases use the official pinned host. No fresh Codex login or actual acceptance inference occurred; the earlier seven aggregate real turns remain historical and unchanged.

Commands and environment are retained in `verification-results.json`, `metadata-and-local-checks.json`, `linux-helper-tests.log`, `mac-helper-tests-*.log`, `mac-helper-clippy.log` and `guest-sealed-audit.log`. Cargo commands use `--locked --offline --manifest-path core-native/Cargo.toml`; Linux uses existing vendored sources. Both explicit `TZ=UTC` and `TZ=Asia/Kolkata` runs are retained. Source/executable hashes are in `final-source-digests.json` and `artifact-digests.json`. Exact fixture programs and earlier diagnostic failures remain saved; failed attempts are not counted as passes. The strengthened detached-child check translates sandbox PIDs with kernel `NSpid` metadata; an earlier un-translated diagnostic was rejected rather than counted as a pass.

Guarded native undo was registered before each source/document change. Original dirty work was retained, no Git clean/reset/stash was performed, and protected owner-store predecessors were hash-checked by every successful fixture. The reused test account's generated configuration was preserved; a new empty unprivileged fixture account was provisioned rather than deleting it. No credentials were created or copied into that account. Per-fixture logs show cgroup drain/service shutdown before VM stop. Durable closed-grant audits were recollected in an audit-only restart, with no task execution; no task processes or auth caches were present then. A failed read against the already stopped VM and its projection-recollection record are retained. The disposable VM is stopped with its disk retained.

## Remaining acceptance and gates

This is source implementation plus offline verification, not authenticated production acceptance. A future separately approved batch must use the real owner service and official model for edit/build/fail/repair/retest, MCP status and descendant revocation, with explicit workspace outbound-data approval. The earlier successful synthetic authenticated controller is not substituted for this test. Broader direct-edit/escape tests, independent review, all supported release targets, installed website-artifact acceptance and complete A01–A14 acceptance remain pending.

An illustrative model request extension is:

```json
{
  "resource_class": "included-subscription",
  "repair_argv": [],
  "model_repair": {
    "model": "gpt-6.1-sol",
    "prompt": "Inspect Kineti task status and repair only the copied workspace for the declared checks",
    "max_turns": 3,
    "helper_sha256": "587436a943a59c18cabdbe83f07800d237b60ddd2e0f54d05b23d8b37a0b36f4",
    "provider": "codex-chatgpt-subscription",
    "data_scope": "approved-task-workspace"
  }
}
```

This example grants nothing. After the complete request is planned and approved at the separate owner console, an authorized fresh-login batch can use `kineti task serve-model --owner-store /var/lib/kineti-owner-native-v1 --id <approved-id>` and a bound `task run` client. This work did not invoke that login entry point. Fresh login, additional real model turns, credential expansion, accounting recovery, merge, website publication and deployment remain separately gated. No new paid spending was authorized or used.

Official protocol reference: https://learn.chatgpt.com/docs/app-server. Website-only distribution policy is recorded separately in `WEBSITE-ONLY-DISTRIBUTION-20261031.md`.
