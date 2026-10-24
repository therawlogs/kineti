# Kineti 0.6.0 evidence matrix

Contract: [GATE-0.6.0-CONTRACT.md](GATE-0.6.0-CONTRACT.md).
This is checkpoint evidence, not release acceptance or publication approval.

## Approved implementation accounting exception

The owner approved subscription-only implementation with $0 new paid spending
on 2026-10-25. The contract records the approval and its implementation-only
boundary. Historical records and monetary unknowns remain unchanged. The latest
account checkpoint allowed ordinary usage, reported plan_type=prolite, no extra
credits and 6% account-wide weekly usage. This is not historical billing proof.

## G03: native runner access verified

Public repository: therawlogs/kineti. Actions enabled; authenticated account has
repository/workflow access. Only standard public-repository runner labels were
used, with no checkout, external actions, secrets, paid services or deployment.
GitHub documents these standard runners as free for public repositories:
https://docs.github.com/en/actions/reference/runners/github-hosted-runners

Run: https://github.com/therawlogs/kineti/actions/runs/40124627870

Workflow commit: `82f39b53700106317e485ef6d36c607ecc1ea11b` on
`codex/gate-060-feasibility`. This is a feasibility-only remote commit, not the
0.6.0 release commit. Main and release tags were not changed.

| Target | Standard label | Observed native architecture | Result |
| --- | --- | --- | --- |
| Linux x64 | ubuntu-24.04 | x86_64 | Passed |
| Linux arm64 | ubuntu-24.04-arm | aarch64 | Passed |
| macOS x64 | macos-15-intel | x86_64 | Passed |
| macOS arm64 | macos-15 | arm64 | Passed |

Exact OS versions, kernel/build identities and Rust/Cargo versions are retained
in `.kineti/gate-060-feasibility/runner-40124627870.log`; job IDs and the exact
workflow head are in `runner-40124627870.json` alongside it. These jobs establish
actual runner access only. Cargo/npm installation and size acceptance remain
unrun and must use the same actual target environments later.

## G04: pinned alpha protocol feasibility verified

Codex: `0.163.0-alpha.11.2`.

Executable SHA-256:
`248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`

The executable's own generated JSON schemas established the actual protocol
parameters. The fixture performed initialize/initialized, config/read,
thread/start (ephemeral, no turn), mcpServerStatus/list and mcpServer/tool/call.
Codex reported the native fixture connected, discovered
`kineti_feasibility_status` and returned `kineti-native-feasibility-ok` from a
real call. The fixture is compiled Rust; it is not the completed gate.

HOME, CODEX_HOME, temporary and XDG directories were isolated inside
`.kineti/gate-060-feasibility`. The child inherited no API credentials and its
PATH excluded Node/Bun. Its model provider points to an unused local port; the
request allowlist contains no model-turn method. No real Codex configuration or
credential files were loaded or modified by the probe.

Reproduce with `python3 scripts/check-gate-060-codex.py` using the pinned installed
Codex and the cached Rust dependencies. This probe is host-local acceptance, not
a clean-machine release installation check. Exact generated config/binary hashes,
results, protocol transcript and stderr are retained in the project-local
feasibility directory. Stable Codex and the final native gate remain unverified.

## Remaining requirements

G01 is amended for implementation only, not publication. G03 and the early alpha
portion of G04 have evidence above; the final integration/stable acceptance in
G04 is still open. G02 and G05–G20 remain unaccepted. No native port, source
deletion, package publication, size acceptance or stable-host support is claimed.
