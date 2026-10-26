## Description

Explain the problem and resulting behavior in plain words. Link the approved
scope or contract and identify any incomplete acceptance requirements.

## Type of change

- [ ] 1. Bug fix
- [ ] 2. New feature
- [ ] 3. Documentation
- [ ] 4. Refactor
- [ ] 5. Security fix

## Verification

Record exact commands, exits, assertions, source/executable/artifact hashes,
OS/architecture and environment. Mark unrun, skipped, failed or unsupported checks
explicitly. Replace these examples with the commands actually run:

1. `cargo fmt --manifest-path core-native/Cargo.toml --check`
2. `cargo clippy --locked --manifest-path core-native/Cargo.toml --all-targets -- -D warnings`
3. `TZ=UTC cargo test --locked --manifest-path core-native/Cargo.toml`
4. `TZ=Asia/Kolkata cargo test --locked --manifest-path core-native/Cargo.toml`
5. Native `kineti test run` and `kineti test check` labels, exact command argv,
   declared inputs/environment and fresh results; see `CONTRIBUTING.md`.

Keep runtime `.kineti/` records local. A valid signature does not prove honest
test execution. Local source tests do not establish installed-package or
four-platform acceptance.

## Safety and review

- [ ] Fresh native formatting, Clippy and both timezone suites pass; missing proof is listed above.
- [ ] Fresh native evidence was recorded and checked against exact argv after the final source/runtime change.
- [ ] Intended changes were previewed, exact original bytes preserved, and native undo registered before edits.
- [ ] Locked goals, user changes, historical signatures/records and unresolved costs were preserved.
- [ ] Native `kineti spend check` and `kineti status` results are reported, including denials; no limit or accounting record was cleared to pass.
- [ ] Any approved subscription-only exception stayed at $0 new paid spending; integrity failures and unresolved monetary reservations were not bypassed.
- [ ] No real editor configuration or argv trust was changed without its separate approval.
- [ ] Outbound destinations were checked and local request audit records preserved; no secrets, private paths or unrelated personal data are included.
- [ ] User-facing text uses plain words; tests cover real behavior and denial paths.
- [ ] Every commit has a DCO sign-off certifying the right to submit under the MIT License.
- [ ] Required human review and separate merge approval remain explicit; this PR does not authorize publication, deployment, recovery, paid services or a version bump.
