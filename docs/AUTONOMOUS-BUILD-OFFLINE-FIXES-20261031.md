# Offline terminal and MCP status fixes

Date: 2026-10-31. Status: offline fixes verified; authenticated replay remains unaccepted and separately gated.

## Scope and outcome

The approved local implementation exception authorizes $0 new paid spending. This work uses deterministic canned Responses SSE on loopback, the pinned official Codex 0.166.0 app-server and matching helper, and native Kineti. There was no fresh login, real model inference, subscription turn, new API credential, global provider configuration change, merge, publication or deployment. The previous four real acceptance turns remain consumed; this does not authorize another turn.

The native adapter disables optional `features.shell_snapshot` and `features.shell_snapshot_v2` in its child process arguments. The task MCP catalog adds read-only, non-destructive, closed-world hints **only** to `kineti_task_status`. Mutating tools keep their original metadata and owner binding. The existing stock filesystem/network policy, no-new-privileges and cgroup controls remain in use. No custom sandbox or shell replacement was introduced.

## Debug report

1. Terminal reproduction: with the original optional shell-snapshot feature, the actual unified-exec handler exited 2 without output or a marker. With `features.shell_snapshot=false`, the same command, shell and permissions exited 0, printed `terminal-ok` and wrote `ready` in the synthetic workspace. This isolates an optional snapshot startup compatibility failure in this minimal Linux fixture. It does not prove which individual shell-prologue instruction fails. The existing `/bin/bash` shim was preserved.
2. Status reproduction: the original native catalog caused `mcpServer/elicitation/request` for `kineti_task_status`; the fixture rejected it. A per-tool automatic-approval setting alone did not fix it. Read-only metadata did; the final direct native executable completed status without an approval request or a metadata wrapper.
3. Negative control: an intermediate variant annotated mutating tools as closed-world and removed their prompt. That variant was rejected. The final status-only variant caused `kineti_task_run` to request consent, which the fixture denied before owner IPC or execution. No blanket approval policy was added.
4. Regression: `core-native/crates/kineti-cli/tests/codex_host_offline.py` drives the actual pinned tool handlers with canned local protocol responses. It checks an absent account, absent Authorization header, the explicit temporary loopback provider, native cgroup cleanup, kernel-bound task IPC and positive/negative outcomes. It is a standalone Linux-root fixture, not a Cargo-discovered test or authenticated model acceptance.

## Fresh verification

Evidence root: `.kineti/autonomy-offline-fix-20261031/` (ignored local evidence).

| Check | Evidence and result |
| --- | --- |
| Original terminal failure / isolated green | `repro-baseline.jsonl` / `repro-no-snapshot.jsonl` |
| Original status consent / rejected auto setting | `mcp-baseline-v2.jsonl` / `mcp-config-auto.jsonl` |
| Final direct native status / mutating consent | `mcp-status-only-final.jsonl`: status success, zero prompts; run denied, one prompt |
| Final terminal | `terminal-final-and-linux-provenance.log`: exit 0, expected output and marker, zero prompts |
| macOS native suite | `mac-tests-final.log`: 24 library and 59 integration tests passed |
| macOS strict Clippy | `mac-clippy-final.log`: passed |
| Linux native suite and strict Clippy | `linux-tests-clippy-final.log`; final outcome recorded in `verification-results.json` |
| Formatting and whitespace | `local-environment-checks.json`: fmt check and git diff check exit 0 |
| Source and executable digests | `final-source-artifact-digests.json`; Linux artifact/Codex/helper hashes in provenance log |
| Native operational receipt | `native-check-receipt.json`; kind `check`, not independent test authority |

Exact Cargo commands: `cargo test --locked --offline --manifest-path core-native/Cargo.toml`; `cargo clippy --locked --offline --manifest-path core-native/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path core-native/Cargo.toml --check`. Local Cargo home/target are inside the evidence/project area. Linux adds `--config 'source.crates-io.replace-with="vendored-sources"' --config 'source.vendored-sources.directory="vendor"'` with existing vendored sources. Environment records identify actual OS, architecture, rustc and Cargo. Linux uses the existing disposable Alpine arm64 VM; this is not four-platform or installed-package acceptance.

Fixture commands use `sudo python3 core-native/crates/kineti-cli/tests/codex_host_offline.py --case NAME --codex PINNED_CODEX --native CANDIDATE --task-user DEDICATED_USER --expect pass|blocked`. Final cases are `no-snapshot-status-only-final`, `mcp-status-only-final`, and `mcp-write-control-status-only`; the latter expects blocked. The temporary provider override exists only in that child process. No persistent auth/provider file was written. Original pinned Codex and helper hashes are retained in the provenance record.

## Preservation and remaining gates

A complete starting-work archive and exact runtime inventory preceded edits. Native guarded undo entries preceded source changes, including the compile-syntax correction and narrowing the metadata fix. Each inverse checks expected candidate bytes before restoring its saved predecessor; later edits block restoration. No rollback was executed. `starting-work-comparison.json` shows only the two intended existing source files changed, preserving concurrent website work. The new regression and this report are additional files.

`runtime-before.json` and final preservation verification cover original runtime/accounting bytes; append-only evidence/undo prefixes are preserved and the undo guard advances normally. `spend check` and default status retain the historical unknown-cost block (exit 3); read-only spending assessment reports verified integrity, clear monetary reservations and $0 new paid authority. No historical costs were valued, cleared or recovered.

Authenticated terminal/MCP replay and a model-driven detached-child revocation test remain unverified. Fresh login, extra real model turns, credential expansion, accounting recovery, merge, package publication and deployment each retain their separate gates. The implementation remains an experimental native Linux owner-service candidate, not a released or universally enforced agent runtime.

Official supported configuration/protocol references consulted: https://learn.chatgpt.com/docs/config-file/config-reference and https://learn.chatgpt.com/docs/app-server. Pinned source excerpts used for diagnosis are saved alongside the evidence.
