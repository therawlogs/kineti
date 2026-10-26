# Contributing to Kineti

Use plain words, keep changes focused, and run checks before asking for review.
The active product is the native gate in the single `kineti-cli` crate. Its
executable is `kineti`; the npm package is `kineti`. Bun, the dashboard and the
old multi-crate harness are historical, not contributor prerequisites.
The current source candidate is 0.6.1. Its exact binaries and installed packages
need fresh acceptance; the dated 0.6.0 publication approval does not authorize it.

## 1. Build from source

1. Install Rust with Cargo, rustfmt and Clippy on Linux or macOS, x64 or arm64.
2. Fork [therawlogs/kineti](https://github.com/therawlogs/kineti), clone your fork,
   enter the repository and create a focused topic branch.
3. Read `AGENTS.md`, `.kineti/state.json` when present, and
   [the acceptance contract](docs/GATE-0.6.0-CONTRACT.md). Preserve existing locked
   goals, user edits, signatures, costs and interrupted work.
4. Build the native executable:

```sh
cargo build --locked --manifest-path core-native/Cargo.toml
KINETI_BIN="$PWD/core-native/target/debug/kineti"
"$KINETI_BIN" --version
```

The source is in `core-native/crates/kineti-cli/src/`; integration tests are in
`core-native/crates/kineti-cli/tests/native_gate.rs`. Runtime execution needs
neither Node nor Bun. Node is needed only for npm installation/package tooling.
The static website in `website/` has no install or build step.

For a new disposable project, choose its goal explicitly and preview `kineti init`
before `--apply`. Use a project-local `--codex-home` when testing integration;
do not alter your real editor configuration. Restart existing Codex sessions
after integration changes. Existing projects retain their locked goal.

## 2. Before each change

1. Run `"$KINETI_BIN" spend check` and `"$KINETI_BIN" status`. Retain the exact
   result, including denials. Do not clear accounting records to obtain a pass.
2. Check the configured total and stage limits and the 95% stop. Spending is
   reported accounting, not provider-bill verification. Only the human owner may
   approve a limit increase; confirmation flags do not prove human identity.
3. Preview the intended diff and save the current bytes inside the project.
   Register the exact inverse before editing. For example, after saving an
   existing file to `.kineti/backups/example.rs`, register its restoration:

```sh
"$KINETI_BIN" undo push --run-id local-change --label restore-example -- \
  /bin/cp .kineti/backups/example.rs core-native/crates/kineti-cli/src/example.rs
"$KINETI_BIN" undo rollback --run-id local-change
```

The example paths must be replaced with real files and an exact saved backup;
use a fresh run ID and label. `rollback` without `--yes` previews pending actions.
After reviewing that preview, `undo rollback --run-id local-change --yes` runs
them newest first. Definite failures continue; uncertain effects stop for
inspection. Never blindly replay an uncertain action or restore over newer user
edits. Keep backups, generated files and isolated test homes inside the project.

The owner's documented 0.6.0 exception permits included subscription usage for
local implementation with **$0 new paid spending**, while preserving specified
historical monetary unknowns. In this repository, `spend check` and `status` may
therefore return exit 3 with `unresolved historical cost ... remains recorded;
any implementation-only subscription exception does not authorize publication`.
That denial remains visible. The exception does not cover integrity failures,
uncertain monetary writes, unresolved monetary reservations, possible extra
charges or publication. Stop affected work if those conditions arise; never
guess missing costs or force the expected recorded $55 into a ledger.

## 3. Checks and fresh evidence

Run the same source checks as `.github/workflows/kineti-gate.yml`:

```sh
cargo fmt --manifest-path core-native/Cargo.toml --check
cargo clippy --locked --manifest-path core-native/Cargo.toml --all-targets -- -D warnings
TZ=UTC cargo test --locked --manifest-path core-native/Cargo.toml
TZ=Asia/Kolkata cargo test --locked --manifest-path core-native/Cargo.toml
```

Record and check fresh native evidence for each timezone after the final edit:

```sh
TZ=UTC "$KINETI_BIN" test run --label native-utc --timeout 600 \
  --required-test core-native/crates/kineti-cli/src \
  --required-test core-native/crates/kineti-cli/tests --env-name TZ -- \
  cargo test --locked --manifest-path core-native/Cargo.toml
TZ=UTC "$KINETI_BIN" test check --label native-utc \
  --expect-argv '["cargo","test","--locked","--manifest-path","core-native/Cargo.toml"]'
TZ=Asia/Kolkata "$KINETI_BIN" test run --label native-kolkata --timeout 600 \
  --required-test core-native/crates/kineti-cli/src \
  --required-test core-native/crates/kineti-cli/tests --env-name TZ -- \
  cargo test --locked --manifest-path core-native/Cargo.toml
TZ=Asia/Kolkata "$KINETI_BIN" test check --label native-kolkata \
  --expect-argv '["cargo","test","--locked","--manifest-path","core-native/Cargo.toml"]'
```

Successful checks return JSON with `result.status` equal to `fresh`. Preserve
exact argv, exits, output, timezone/environment, OS/architecture, source and
executable hashes. If using a project-local Cargo cache offline, add `--offline`
to the actual Cargo command and expected argv and declare `CARGO_HOME` with
`--env-name CARGO_HOME`; report that different command accurately. Rebuild after
native source changes and run fresh evidence again after any bound input or
runtime changes. A signed receipt is distinct from trustworthy test execution:
commands can fabricate passing summaries, as `kineti self-test` demonstrates.
Never claim a test passed unless freshly run. Keep `.kineti/` records local.

`kineti verify --status` shows the declared argv and its trust state. An untrusted
declaration denies with exit 9 and `verify command not trusted; review and run
kineti verify --trust`. Missing declarations also deny with exit 9. Review of an
exact new argv and an interactive trust grant are separate actions; do not grant
trust merely to make a check pass. Changed declarations invalidate old evidence
and old argv trust does not carry over. `verify` gates its participating command;
it does not intercept unrelated commands or provide protected writes.

## 4. Tests and documentation

1. Extend meaningful existing Rust tests; use Cargo's test framework. Exercise
   real behavior and denial paths, not copies of implementation constants.
2. Use disposable project-local fixtures and isolated homes/configuration.
   Never modify real accounting, editor settings or global test-process umask.
3. Preserve legacy signature fixtures, dated evidence and historical records.
   Report skipped, failed, unsupported or missing proof explicitly.
4. Update active documentation to match source. Dated documents under
   `docs/history/` and release records remain history. Keep the root README's
   paths and website claims accurate and bounded.
5. Check outbound destinations before transmitting project information and
   preserve local request audit records. Do not send credentials or private
   project data as part of a test.

## 5. DCO and review

We follow the Developer Certificate of Origin (DCO) version 1.1. By contributing,
you certify that you wrote the changes or have the legal right to submit them
under the project's MIT License.

1. Sign off every commit: `git commit -s -m "fix: describe the concrete change"`.
   The message must include `Signed-off-by: Full Name <email@example.com>`.
   Reviewers require the sign-off; the repository does not currently configure
   an automated DCO enforcement check.
2. Keep commits focused. Review the diff for secrets, personal filesystem paths,
   machine names and unrelated changes. Required DCO identity belongs in the
   commit sign-off; do not copy private identities into source or fixtures.
3. Open a pull request to `main` and fill out
   `.github/pull_request_template.md`, including actual commands and denials.
   Never push directly to `main`.
4. The native gate workflow runs formatting, Clippy and the native suite under
   UTC and Asia/Kolkata. Human reviewers check behavior, security boundaries,
   fresh evidence and plain language. Address requested changes and rerun checks.
5. Maintainers may squash-merge only after required checks, review and separate
   merge approval. Green local tests or a signed receipt do not grant release
   authority.

## 6. Packages and publication

`core-native/Cargo.toml` contains one member, `kineti-cli`, which provides
`kineti`. The npm package `kineti` selects a checksummed native binary through
`install.cjs` and `bin/kineti`. The four targets are Linux musl and macOS, each
on x64 and arm64. Runtime feasibility on a development host is not proof of an
installed Cargo or npm package on any other target.

The [0.6.0 contract](docs/GATE-0.6.0-CONTRACT.md), its approved accounting
amendment, the [repair contract](docs/NATIVE-GATE-REPAIR-IMPLEMENTATION-CONTRACT-20261026.md)
retain the behavior/accounting baseline. The owner's later 0.6.1 preparation
instruction selects the new candidate version; its artifact acceptance and
publication/accounting approval remain separate. Preserve older
published crates and dated release records. No automatic version bump, tag,
registry upload, remote job dispatch or release follows from this workflow.
The current `release.yml` prepares candidates and has no publishing jobs.
Publication needs reviewed accounting, exact artifact approval and separate
authorization; merging and deployment also need separate approval. Never assume
checked-in workflow configuration proves live provider enforcement.
