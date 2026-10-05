# Changelog

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
