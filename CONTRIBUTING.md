# Contributing to Kineti

Plain words. Numbered steps. Every change must be safe, tested, and proven.

## 1. Setup

1. Install bun: `curl -fsSL https://bun.sh/install | bash`.
2. Clone and enter the repo.
3. Run `bun install` at the root. For the website, run `bun install` inside `website/` too.
4. Run `./setup.sh` to install skills into your editors (or `kineti init`).
5. Start the dashboard: `bun run companion`, open `http://127.0.0.1:8788`.

## 2. Git branch workflow

1. Fork the repository at `https://github.com/therawlogs/kineti` to your own GitHub account.
2. Clone your fork locally and switch to a dedicated topic branch:
   - Features: `git checkout -b feat/<short-name>`
   - Bug fixes: `git checkout -b fix/<short-name>`
   - Documentation: `git checkout -b docs/<short-name>`
   - Maintenance: `git checkout -b chore/<short-name>`
3. Keep commits atomic and focused on a single change.
4. Clean commits: never commit private names, machine names, personal home paths (use `$HOME`), or secret keys. Search your diff before committing.

## 3. Daily loop

1. Read `.kineti/state.json` before starting anything.
2. Check spend: `bun bin/kineti-spend.ts check`. Stop at $50.00.
3. Save an undo step before changing files: `bun bin/kineti-saga.ts push "<undo-command>"`.
4. Make small edits. Run tests. Save proof (see below).
5. Speak in plain words, offer numbered choices (1, 2, 3).

## 4. Tests and proof (required)

1. Run the full suite: `bun test tests/`.
2. Save proof bound to the code: `bun bin/kineti-evidence.ts run --label <name> -- bun test tests/`.
3. Check it: `bun bin/kineti-evidence.ts check --label <name>`. It must say FRESH.
4. Rust engine: `cargo test --manifest-path core-native/Cargo.toml`.
5. Typecheck: `bun run typecheck`. Installer smoke: `./tests/test-setup.sh`.
6. Never claim tests passed without running them. If code changed after a proof run, run it again.

## 5. Developer Certificate of Origin (DCO)

We follow the Developer Certificate of Origin (DCO) version 1.1. When you contribute code or documentation to Kineti, you certify that you wrote the changes or have the legal right to submit them under the project's MIT License.

1. Add a sign-off line to the bottom of every commit message:
   `Signed-off-by: Full Name <email@example.com>`
2. You can have Git add this automatically by passing the `-s` flag:
   `git commit -s -m "feat: add spend ceiling notification"`
3. Pull requests without a valid DCO sign-off cannot be merged into main.

## 6. Pull request review lifecycle

Every contribution follows a structured 5-step review process:

1. **Open the pull request**: Create your PR targeting the `main` branch. Fill out `.github/pull_request_template.md` completely. Select the type of change and confirm all safety checklist items.
2. **Automated CI checks**: GitHub Actions automatically runs the test suite (`bun test tests/` and `cargo test`), validates TypeScript types (`bun run typecheck`), and verifies installer scripts.
3. **Cryptographic proof verification**: Verify that test proof recorded with `bin/kineti-evidence.ts` is fresh and bound to the code.
4. **Human code review**: Maintainers review changes for correctness, security boundaries, and plain English. If edits are requested, make adjustments on your branch and push updates.
5. **Squash and merge**: When all checks pass and approvals are granted, maintainers squash-merge your PR into `main`.

## 7. Money and safety rules

1. Only the human raises the project ceiling ($50 default, $1-$1000 at mirror time). Every change is audit logged.
2. Costs flow down only: project ceiling splits into swarm budgets splits into agent caps.
3. Every outbound web or API call gets recorded first (`bin/kineti-egress.ts`).
4. Never commit secrets, tokens, private names, or home paths (use `$HOME`).

## 8. How to add a `bin/` tool

1. Create `bin/kineti-<name>.ts` with `#!/usr/bin/env bun`, plain-words output, and numbered choices.
2. Fail closed: bad input exits non-zero, never disables a safety check.
3. Audit log every state change (who, what, old, new). Never log secret values.
4. Add one row to `bin/README.md`.
5. Add tests in `tests/<name>.test.ts` that back up and restore any `.kineti/` files they touch, and redirect `KINETI_MACHINE_DIR` to a temp dir. Tests must pass on a bare checkout with no `.kineti/` present.
6. If the tool reads the spend ceiling, use `loadLimits()` from `bin/lib.ts` so per-project mirror ceilings apply.

## 9. How to add a test

1. Put it in `tests/<area>.test.ts` using `bun:test`.
2. Test real code in `bin/` or `src/`. Do not commit tests that only assert local literals.
3. Back up and restore every `.kineti/` file touched. Redirect `KINETI_MACHINE_DIR` to a temp dir for audit writes.
4. Keep the suite green in CI: no network, no ports outside test servers, no absolute paths.

## 10. How to add a skill

1. One skill does one job. Create `skills/<name>/SKILL.md` with the exact frontmatter of existing skills.
2. Prove it worked twice before packaging.
3. Re-run `./setup.sh` so all hosts receive it as `kineti-<name>`.
4. Record the skill and first-run notes in the journal.

## 11. Packages: npm, website, Rust

1. npm package `kineti`: entry is committed `bin/kineti.js`, generated from `bin/kineti.ts` via `bun run build:router`. Never hand-edit it. Pack contents are controlled by `.npmignore` (a `files` allowlist crashes npm 11). Verify with `npm pack --dry-run`.
2. Website package `kineti-website`: separate app in `website/` with its own lockfile. Build with `bun run build` inside `website/`. It is excluded from the npm pack. The audited landing stays `public/waitlist.html`.
3. Rust workspace `core-native/`: 8 crates, `publish = false`. Bump `[workspace.package] version` together with `package.json`. `Cargo.lock` at `core-native/` is tracked; there is no root lockfile.
4. Version rule: `package.json`, `kineti.config.json`, and Cargo workspace move together. `tests/versions.test.ts` enforces the trio.

## 12. Docs rules

1. `AGENTS.md` holds all agent rules. `GOOD_ROADMAP.md` is the living plan. `ROADMAP.md` is the frozen original spec, do not extend it.
2. `docs/PLAN.md` and `docs/AUDIT.md` are frozen historical snapshots. Add dated note headers, never rewrite their bodies.
3. Root `README.md` structure tree must match the real tree. Update it when adding top-level dirs or tools.
4. Website copy lives in `website/src/`. Keep version strings and tool counts in sync with the repo.

## 13. Ship checklist

1. Typecheck clean, full bun suite green with FRESH proof, Rust suite green.
2. Landing check: `gzip -c public/waitlist.html | wc -c` under 35,000.
3. Adversarial suite: `bun test tests/adversarial_prompt_injection.test.ts`, 0 fail.
4. Bump the version trio, commit, push, tag `vX.Y.Z`.
5. Tag push triggers CI binaries. Attach to the GitHub release (retry on API lag, it resolves).
6. `npm publish` needs a human with 2FA. Record publish plus release in `GOOD_ROADMAP.md` section 13.

