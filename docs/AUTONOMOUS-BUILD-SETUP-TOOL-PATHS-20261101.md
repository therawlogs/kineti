# Setup cancellation and broader tool-path verification

Date: 2026-11-01. Status: bounded offline cases passed; full contract acceptance remains pending.

## Implemented changes

The owner service now uses one lifecycle mutex for initial snapshot preparation, host creation and restoration. Stop first signals cancellation and records its intent, waits for that mutex, then drains the task group and records completion. Host creation therefore cannot register a new group after the final stop drain. Groups are registered immediately after creation so subsequent setup errors can reconcile them. Safety drain/closure records remain allowed after halt, while ordinary progress events are rejected; late initialization or completion cannot overwrite a stopped phase.

Initial copying now has cancellation checkpoints before filesystem mutations and between 64 KiB copy chunks, validates the expected source length, and preserves partial blobs/work. Existing source bytes are never overwritten. The unchanged public snapshot helper delegates to this implementation without adding task authority. Paused or interrupted partial setup cannot resume merely because its phase changed: a completed snapshot inventory is required. No custom sandbox, process-group fallback or hidden permission approval was added.

Source, tests and documentation were previewed in staging and exact native undo/backups registered before edits. The final optimized Linux arm64 candidate is protected at a new owner path, SHA-256 `7aa562a1064a55d38137d743099edad728cc9df0d9b4a051340d4b64e7eceec8`. The intermediate `71309e072a7ab1ce57438428ecc15c180e6e38abf7cca8057a4bb7a7ca4f2f27` candidate and earlier accepted samples remain preserved. Official Codex remains 0.166.0, SHA-256 `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`, with matching helper `587436a943a59c18cabdbe83f07800d237b60ddd2e0f54d05b23d8b37a0b36f4`.

## Fresh offline evidence

1. Real snapshot setup: revocation while the initial copy was active preserved the partial checkpoint, kept work/blob file bytes, modes and ownership stable after acknowledgment, and started no host or verification command. The sampled native revoke returned in about 11 ms.
2. Real stock host startup: revocation before capabilities completed drained the group, recorded no task command and permitted no late progress. Sampled return was about 27 ms.
3. Paused setup/restart: pause preserved the partial copy; restarting the same otherwise-valid grant refused partial setup without changing its bytes. The fixture was subsequently revoked and closed.
4. Delayed protocol double: an intentionally delayed initialization response separately exercised the startup critical section and cancellation, with real native group control but no official-host acceptance claim for the double. Sampled return was about 92 ms.
5. Broader stock terminal paths: 23 denied operations and two successful authorized-write controls, including descriptors, atomic replacement, file copying, outside directory creation, fork/exec children, proc-root alias, and symlinks created during execution. Earlier file, exclusion, policy, hard-link, shell-child, TCP and abstract Unix checks were rerun. Outside canaries and original work stayed unchanged; positive-control listener counts showed no task connection.
6. CLI/MCP boundary assertions: wrong session rejected, repeated identical native notification suppressed, same UID outside the launched group denied mutation, fixed MCP status allowed, and scope/argv/paid-limit overrides and an owner-revoke tool name rejected. These are sampled protocol assertions, not full parity or model-consent acceptance.
7. Regression: real detached-child revocation, synchronized recovery cancellation and failed-drain hard stop were rerun successfully on this candidate. Copy-cancellation unit coverage includes both partial blob creation and partial task-file copying, with original bytes and partial data retained.
8. Source suites: macOS arm64 26 library/62 integration tests and Linux arm64 27 library/62 integration tests passed separately under UTC and Asia/Kolkata. Strict Clippy, formatting and whitespace checks passed. Linux uses vendored dependencies with `--locked --offline`.

The times above are individual observations, not latency guarantees. Cancellation is cooperative between filesystem operations; synchronous owner I/O can delay acknowledgment. Stop waits for the protected operation before acknowledging. Work and protected evidence remain available for inspection. Snapshot/event size bounds do not establish a continuous disk quota.

## Newly measured limitation

The current stock profile allowed creation of a new `.env.dynamic-fixture` file inside the authorized work copy. This was a separate capability observation, not counted as a denied-operation pass. Existing tested exclusions and original secret reads were still blocked. Configured filename glob denials therefore do not establish universal denial of newly created matching filenames. Full filename-exclusion enforcement remains unaccepted; this report does not infer that every possible supported host configuration has the same limitation. No custom sandbox fallback was attempted.

These terminal checks do not establish negative model-native direct-edit coverage or all possible inherited-descriptor paths. No fresh login or acceptance model turn was used; the earlier ten real attempts and closed sessions remain unchanged. No additional reviewer agent was used, so independent review of these new lifecycle changes remains pending. The previous independent review and its narrower findings remain historical rather than approval of this new artifact.

## Evidence, preservation and gates

Evidence is retained under `.kineti/autonomy-setup-paths-20261101/`: exact guest programs; `setup-snapshot.jsonl`, `setup-host-real.jsonl`, `setup-snapshot-pause.jsonl`, `setup-host-double.jsonl`; `tool-paths-split.jsonl`; active-child/recovery/cleanup regression logs; closed grant/event/head records, source/artifact hashes, both-platform check logs, spending assessments, native undo and operational verification. The oversized first combined check was rejected by the existing 4096-byte argument bound. It was split into two fixed checks without raising that limit; the rejected request and generated work remain preserved.

Root PTY approval exercises the owner role, not individual human identity. Closed-state collection is authoritative for fixture teardown; the pause sample also retains its intermediate paused state. The sampled kernel and native results are local observations. Chained audits and operational signatures do not supply independent test trust.

The repository's integrity remains verified, monetary reservations clear, all thirteen historical valuation unknowns visible and reported amounts unchanged. Ledger/signature bytes and earlier evidence/undo prefixes are preserved. Prior owner-store files and original synthetic work were hash-checked. All fixture grants/groups/services were closed, no task UID process or persistent auth file remained, and the disposable VM was stopped with its disk retained. No new paid spending, credential expansion, fresh login, acceptance inference, merge, registry change, publication or deployment occurred.

The current acceptance matrix records the remaining gaps. Full A01–A14 acceptance, supported dynamic-exclusion strategy, model-native direct-edit tests, current-candidate independent review/live replay, complete fault/limit grids, four-target installed acceptance and separate release/accounting approvals remain pending.
