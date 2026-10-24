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
