# Website-only proprietary distribution and registry retirement

Date: 2026-10-31. Owner direction: repository private; remove current MIT declarations; future distribution through the website only. The owner separately selected npm deprecation and crates.io yanking while preserving historical records.

## Current source policy

Root/crate license files now contain a prospective proprietary, all-rights-reserved notice. npm metadata is `private: true` and `license: UNLICENSED`, and its prepublish hook now refuses registry publication (fresh offline exit 1); Cargo uses a license file and `publish = false`. Current contributor/PR wording and source terms no longer declare Kineti MIT-licensed. Third-party licenses and historical notices remain intact. README/site copy describes website-only future distribution without inventing a downloadable artifact. The active local MCP registry descriptor no longer advertises an npm package.

This owner direction replaces future npm/crates.io distribution requirements in older release plans; it does not rewrite dated contracts, old archives, signatures, accounting or already distributed license terms. Existing packages are historical releases. Future website binaries still need exact artifact, footprint, platform, notice, accounting and release approval. No new version, archive, merge, GitHub source push, website publication or deployment was performed.

GitHub's connector confirmed `therawlogs/kineti` is private. License/source changes are prepared locally and remain subject to the separate GitHub merge/publication gate. They have not been applied to the remote default branch by this work. No private Actions job was triggered or paid CI resource used.

## Registry retirement

Existing access was used; no crates.io login, token creation or credential-scope expansion was performed.

| Crate | Versions verified yanked |
| --- | ---: |
| kineti-cli | 10 |
| kineti-core | 9 |
| kineti-reflex | 9 |
| kineti-memory | 9 |
| kineti-connectors | 9 |
| kineti-actions | 9 |
| kineti-gateway | 8 |
| kineti-harness | 8 |
| Total | 71 |

Only crates whose repository matched this Kineti repository were selected. Public API verification confirms every targeted version yanked and all original archive checksums and license metadata unchanged. Yanking prevents normal new dependency resolution; it does not delete archives, revoke old permissions or stop direct downloads/existing lockfiles.

npm `kineti` now has the approved deprecation notice on all 13 targeted versions, 0.3.1 through 0.6.1. The owner approved one existing-account browser/MFA step; an expired uncompleted challenge was replaced, the owner completed verification, and the CLI exited 0. Independent public registry checks confirmed every notice, with original license fields, archive integrity values and tarball URLs unchanged. The existing npm credential file's pre/post SHA-256 matches; no new persistent login token or access scope was created. No Codex login, model inference or release action occurred.

The applied deprecation notice is:

> Kineti registry distribution is retired. Future releases are distributed only at https://getkineti.com. Existing license terms remain unchanged.

## Evidence and undo

Evidence root: `.kineti/autonomy-model-supervision-20261031/`.

- `registry-before.json` and `registry-retirement-plan.json`: exact npm/crate versions, original license/deprecation/yank metadata, checksums and approved notice.
- `registry-actions.jsonl`, `registry-verification.json`, and `npm-retirement-completed.json`: per-version mutation results and independent public-state checks, including the final npm verification.
- `registry-compatible-undo-registration.json`: native registered exact prior-state restoration script with pinned script/plan hashes. Restoring registry availability is itself a separate reviewed external action; no rollback was executed.
- `active-license-audit.json`, source backups and native undo registrations: current metadata changes and exact preserved predecessors.

The first registry script stopped before any mutation because the available Python lacks `tomllib`; the compatible retry and its separate guarded undo are retained. Historical failure records were not cleared. No published archive or original license notice was edited, deleted or replaced. No monetary record was valued or reset.

Official limitations: npm registry content is immutable (https://docs.npmjs.com/policies/unpublish/); crates yanking leaves downloads and lockfile use intact (https://doc.rust-lang.org/cargo/commands/cargo-yank.html); private GitHub repositories do not retrieve previously made copies (https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository).
