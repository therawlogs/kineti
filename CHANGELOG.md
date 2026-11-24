## 1.0.1 — Open-Core AI-PR Verification & Anti-Tampering Gate (2026-11-24)

- **Base-branch PR policy & anti-tampering gate (`kineti pr-check`):** Load `.kineti/gate.json` strictly from the protected base commit (`git show <base>:.kineti/gate.json`), fail closed on unauthorized policy/workflow edits (`policy_tamper_detected`), and scan PR diffs across Rust, Python, JS/TS, and Go for deleted required test files, net test deletions, added skip/ignore/only/xfail/todo markers (including inline and multi-line block-comment smuggling), and weakened runner flags or `package.json` `"test"` scripts.
- **Signed compliance packs (`kineti-pr-audit-v1`):** Emit canonical Ed25519-signed JSON audit bundles and Markdown PR summary tables with `--require-ci` consistency verification and explicit honesty flags (`independent_assertion_quality_proven: false`, `ci_runner_trust_boundary: true`, `automatic_interception: false`).
- **Upgraded Linux GitHub Action (`action.yml`):** Preflight fail-fast `pr-check`, base-branch command/parser/min-passed extraction, supervised `kineti test run` + `kineti ci`, final `pr-check --require-ci --require-evidence`, and `$GITHUB_STEP_SUMMARY` output with read-only permissions and zero `${{` inside `run:` blocks.
- **Two-tier streaming `O_NOFOLLOW` freshness & constant-size receipts:** Stream-hash files in 64 KiB `O_NOFOLLOW` chunks (Tier 1 Git index + root/nested `.gitignore` + `.git/info/exclude`; Tier 2 bounded pure-Rust walker without `legacy`/`build`/`target` blind spots); store constant-size `inventory_sha256` + `inventory_count` in `.kineti/evidence.jsonl` with single-buffer appends and locked reads.
- **Polyglot test verification & parser fixes:** Fix multi-suite `bun test` parsing to sum all suites; add `--allow-ignored` and unified scalar/map `min_passed` support; add allowlisted `--parser exit-code` (`kineti-exit-code-v1`, `count_observed: false`) for `pytest`, `jest`, `vitest`, `go test`, `node --test`, and `make test`.
- **Security hardening:** Close both MCP host-RCE vectors (strict `.kineti/gate.json` command allowlisting on `kineti_evidence_record`; remove `kineti_saga_push` and `kineti_saga_rollback` from MCP); fail closed in `kineti ci` when required labels lack declared command mappings; fix multi-project Codex `init` collision and keep backups out of project trees (`<codex-home>/.kineti-backups`, `0600`); delete the unbounded `.kineti/mirror.json` ceiling override; resolve relative executables against `--root`; enforce `O_NOFOLLOW` across state/key/audit paths; add `kineti status --project-only` and subcommand `--help`.
- **Open-core (MIT) & offline pilot kit:** Keep the CLI, GitHub Action, and 4-scene offline demo (`examples/pilot-demo/run-demo.sh`) open-core MIT alongside optional paid Founding Team Pilot onboarding; redesign `website/` with zero trackers or external CDN scripts.
- **Published archives & 4-platform acceptance:** Tag `v1.0.1` (`de407ee559d4828e390e166fc30cbbf4c79fa43c`) leaves the dated `2026-09-07` historical `v1.0.0` GitHub tag/release (`1d1d63b`) untouched and preserves all 13 unresolved historical monetary records at `$0` new paid spending under the owner-approved narrow publication-policy exception. Exact bit-for-bit verified archives published to `crates.io` (`kineti-cli 1.0.1`, `100,288` bytes, SHA-256 `37088afe55bf7b45b050e3da49932ecd982827b6d86c88bd3a61554078e51c2f`), `npm` (`kineti@1.0.1`, `3,884,179` bytes, SHA-256 `81c5729a51d21fe7f414c33433f6055bdc0dcfe498ab25fee7c94612dc08f886`), and GitHub Release `v1.0.1`; post-publication exact-archive acceptance passed across `linux-x64`, `linux-arm64`, `darwin-x64`, and `darwin-arm64` (Actions run `45355978482`).

The historical 0.6.1 and earlier entries below are retained verbatim.

## 0.6.1 — unreleased native gate repair candidate

- Use one deadline for child completion and both output streams; bound cleanup and preserve uncertain undo intent.
- Keep exact argv, exit meanings, output retention and legacy signature compatibility; add live fault and lifecycle regressions.
- Request owner-only permissions for new append and lock files. Historical files keep their modes pending separately approved recovery.
- Refresh native contributor and security guidance, retaining disclosure/support commitments and documenting exact-argv secret-handling limits.
- Declare the exact Cargo verification argv without granting trust or rewriting old receipts.
- Prepare consistent Cargo/npm/runtime metadata for 0.6.1. Exact final artifacts, installed footprints, fresh host/platform evidence and publication approval remain separate requirements.

The previous 0.6.0 candidate entry below is retained verbatim as history. Its
[dated closeout](docs/GATE-0.6.0-CLOSEOUT.md) records the later published result;
those original artifacts and approvals do not authorize 0.6.1 publication.

## 0.6.0 — unreleased native gate candidate

- Replace the assistant/harness product with one native safety and reported-spending gate.
- Limit platforms to Linux and macOS, x64 and arm64. Cursor and Antigravity are unverified.
- Preserve historical signatures and records; add versioned native receipts and self-test.
- Ship one Cargo crate and one npm package with four native targets; Node/Bun are not runtime requirements.
- Remove evaluator/protected-write guarantees. Separate reviewed artifact and accounting approval blocks publication.

# Changelog

Release notes explain what changed and what remains limited. Kineti v0.5.0 is a local tooling release; protected writes are experimental.

## 0.5.0

- Validate receipt issuer trust, age, declared inputs, local revocation and exact write requests through shared checks.
- Preserve exact command arguments, including spaces and empty values, and document the legacy substring check.
- Serialize execution logging, block damaged ledgers and keep generated CI reports in runtime state.
- Add direct negative, replay, concurrency and write-failure tests.
- Simplify documentation introductions and website copy while keeping setup and reference details.
- Verify package contents before publishing npm and all eight Rust crates. Omit incomplete standalone downloads.

### Compatibility and limits

Bun is required for npm governance commands; the native kineti-cli is a separate Rust tool. Protected writes require an explicit local trust policy. Existing receipt checks and protected-write result codes are unchanged. Forged-success rejection and atomic authorization plus effect remain unresolved. Direct editor writes bypass the gate, and spend is reported by the agent rather than verified against provider bills.

## 0.4.0

### Added

- Public `kineti seed` command for creating missing CI state and spend files from `kineti.config.json`. Existing locked state and spend are preserved.
- CI support for checking each required evidence label, its result, age, and workspace fingerprint.
- Ed25519 signatures for native OVT tickets using the maintained `ed25519-dalek` library.
- Worktree creation now reports Git failures instead of silently returning an empty directory.

### Changed

- CI reports now say **checks passed** or **blocked**. They are not safety certificates.
- GitHub verification runs with read-only permissions; PR comments are posted by a separate, limited-permission job.
- HTTP requests identify as Kineti, not another vendor.
- Documentation distinguishes local test receipts, reported spend, demo features, and unavailable services.

### Removed or disabled

- Removed the local simulated email service and dedicated `agent_email` connector.
- Removed fake external-data purge and forget receipts.
- Disabled Companion connector toggles, credential storage, TOTP seed storage, and test-message dispatch. Existing local files are left untouched.
- Disabled Companion cloud pairing claims. No cloud pairing service is included.
- Removed the external-data purge coordinator that returned local tombstones as an external deletion.

### Limits

- Ed25519 verifies signatures against supplied public keys; it does not authenticate those keys or prove the signed evidence is true.
- Git worktrees separate file changes; they do not sandbox processes, network access, credentials, or the host.
- Kineti still records spend reported by the agent and does not mediate every action available to the agent.
- OpenShell integration, hosted Agent Safety CI, signed deployment certificates, and deployment verification are deferred.

The release workflow runs its checks before publishing. It publishes the native crates, the npm package, and the GitHub release assets. Package availability can differ briefly while registry indexes update.
