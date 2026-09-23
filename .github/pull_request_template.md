## Description

Explain the change in plain words. What problem does this pull request solve?

## Type of Change

Choose one or more by marking with an `x`:

- [ ] 1. Bug fix (fixes an issue without breaking existing behavior)
- [ ] 2. New feature (adds new capability without breaking existing behavior)
- [ ] 3. Documentation update (improves guides, specs, or comments)
- [ ] 4. Code cleanup or refactor (improves structure without changing behavior)
- [ ] 5. Security fix (resolves a vulnerability or enhances safety boundaries)

## How Was This Tested?

List the commands run and describe the verification steps:

1. Command: `bun test tests/`
2. Command: `cargo test --manifest-path core-native/Cargo.toml`
3. Command: `bun run typecheck`
4. Test proof label: `bun bin/kineti-evidence.ts run --label <label-name> -- bun test tests/`

## Kineti Safety and Governance Checklist

Please review and check all items before submitting:

- [ ] **Tests pass**: `bun test tests/` passes with 0 failures.
- [ ] **Rust engine tests pass**: `cargo test --manifest-path core-native/Cargo.toml` passes with 0 failures.
- [ ] **Typecheck clean**: `bun run typecheck` passes with 0 errors.
- [ ] **Cryptographic proof**: Fresh test proof is recorded via `bun bin/kineti-evidence.ts run --label <name> -- <command>` and verified with `bun bin/kineti-evidence.ts check --label <name>`.
- [ ] **Undo safety**: An undo command was registered via `bun bin/kineti-saga.ts push "<undo-command>"` for any file or system mutation.
- [ ] **Spending limit**: Spending is verified under the $50.00 ceiling via `bun bin/kineti-spend.ts check`.
- [ ] **Secrets and privacy**: Zero secrets, private API keys, personal filesystem paths (`/Users/...`), personal emails, or machine hostnames are included in this PR.
- [ ] **Plain English**: Descriptions and user-facing messages use plain words and avoid technical jargon or metaphors.
- [ ] **Developer Certificate of Origin (DCO)**: Commit messages include a DCO sign-off line (`Signed-off-by: Full Name <email>`) certifying rights under the project license.
