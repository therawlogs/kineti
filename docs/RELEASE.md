# Release checklist

Everything needed to cut a Kineti release and keep getkineti.com honest.

## Cutting a release

1. **Pre-flight** (local): `cargo test --all` · `cargo clippy --all-targets -- -D warnings` · `./scripts/size-gate.sh` · `./scripts/bench-startup.sh` (informational) — all green.
2. **Version**: bump `version` in `Cargo.toml`. The CLI banner, daemon stamp,
   and C-ABI `kineti_version()` all derive from this single field
   (`env!("CARGO_PKG_VERSION")`) — never hand-edit version strings elsewhere.
3. **Changelog**: add `CHANGELOG.md` entry for `vX.Y.Z` (Kept/Demoted/Added/Compatibility). This is the human note for `install.sh -s vX.Y.Z`.
4. **Tag**: `git tag vX.Y.Z && git push origin vX.Y.Z`.
5. **CI publishes automatically** (tag-gated jobs):
   - linux gnu + musl-static: `kineti-linux-x64`, `kineti-linux-arm64`,
     `kineti-linux-x64-static`, `kineti-linux-arm64-static`
   - macos: `kineti-darwin-arm64`, `kineti-darwin-x64`
   - then `release-checksums` aggregates every asset into `SHA256SUMS`
     and uploads it to the same release
   - then `release-header` attaches `include/kineti.h` for C/Python consumers
6. **Release notes**: paste the `CHANGELOG.md` `vX.Y.Z` section into the GitHub release body so `install.sh -s vX.Y.Z` has a human note (not just the tag name):
   ```sh
   gh release edit vX.Y.Z --notes-file <(sed -n '/^## vX.Y.Z/,/^## v/p' CHANGELOG.md | head -n -1)
   # or via GitHub UI: Releases → Edit → paste CHANGELOG section
   ```
   Verify: `gh api repos/therawlogs/kineti/releases/tags/vX.Y.Z --jq .body` must not be empty.
7. **Verify**: download two assets on different platforms and run
   `sha256sum -c` against `SHA256SUMS`; smoke `--version` on each.
8. **Announce** only after steps 6–7 pass.

## getkineti.com contract

The domain has exactly one job: serve `install.sh` from this repository
**byte-identical**, so the curl one-liner in the README can always be
trusted. Binaries are NEVER served from the domain — install.sh fetches
them from GitHub Releases, where SHA256SUMS anchors integrity.

Go-live / sync procedure:

```sh
# what the domain serves right now vs what we ship:
curl -fsSL https://getkineti.com/install.sh | sha256sum
shasum -a 256 install.sh          # macOS; sha256sum install.sh on Linux

# sync: automatic — the `sync-install-sh` CI job mirrors install.sh into
# therawlogs/kineti-website@main/public/install.sh on every change to it
# (requires the WEBSITE_REPO_TOKEN secret). Manual re-sync: re-run the job
# from the Actions tab. Never hand-edit the copy in kineti-website.
```

Prerequisites before announcing the URL anywhere:
- DNS resolves and HTTPS serves with a valid certificate;
- the served bytes match `install.sh` in the tagged commit;
- flip the repo variable `DOMAIN_LIVE=1` — this turns the weekly
  `domain-smoke` workflow from advisory warnings into hard CI checks.

## Rollback

Assets are immutable per tag; to retract a broken release, mark the GitHub
release as pre-release (removes it from `/releases/latest`) and publish a
patch tag. install.sh resolves "latest" from the API, so pre-releases are
skipped automatically.

## First public push (one-time open-source readiness)

Run every check below BEFORE the first `git push`. All must pass.

### A. Git state

- [x] `origin` remote points at `https://github.com/therawlogs/kineti`
- [x] Committer identity set locally (`git config --local user.name/email`)
- [x] All intended work is committed; `git status` clean or only local-only files
- [x] Force-push safety confirmed: remote `main` holds only a LICENSE blob

### B. CI release pipeline (tag-gated jobs)

- [x] Every job that uploads assets carries `permissions: contents: write`
      (release-linux, release-macos, release-checksums, release-header)
- [x] Asset names match install.sh probes exactly:
      kineti-linux-x64 · -arm64 · -x64-static · -arm64-static ·
      kineti-darwin-arm64 · kineti-darwin-x64
- [x] `SHA256SUMS` lines are `<hash>  <asset>` so install.sh's
      `grep " $asset$"` matches
- [x] Static musl targets build with zero C dependencies (pure-Rust dep tree)
- [x] `include/kineti.h` exists in the tagged commit (release-header uploads it)

### C. Pre-flight gates (same as "Cutting a release" step 1)

- [x] `cargo test --all` green
- [x] `cargo clippy --all-targets -- -D warnings` green
- [x] `./scripts/size-gate.sh` passes (<10 MB ETHOS budget)
- [x] `./scripts/bench-startup.sh` recorded (informational)

### D. Clean-files law (ETHOS §8.1) — zero matches required

- [x] No personal names in tracked files (search given-name and surname variants)
- [x] No home paths in tracked files (`$HOME`, `/Users/<name>`, `/home/<name>`)
- [x] No personal email inside tracked files (your mail domain) — git config
      metadata is exempt; file contents are not
- [x] No secrets: API keys, tokens, private keys (`AKIA`, `ghp_`, `sk-`,
      `BEGIN.*PRIVATE KEY`, hardcoded passwords)

### E. Repo hygiene

- [x] `.gitignore` covers `target/ .kineti/ .DS_Store demo/ dist/`
- [x] No binary artifacts or editor junk tracked (`git ls-files` review)
- [x] No tracked file >1 MB (GitHub hard limit 100 MB, warn 50 MB)
- [x] `LICENSE` present and matches `license` field in Cargo.toml
- [x] README install one-liner uses `https://getkineti.com/install.sh`
- [x] Version in Cargo.toml is the intended first tag (`v0.1.0`)

### F. Post-push order (do not skip sequence)

1. Push `main`; confirm the `ci` workflow runs green on the push event,
   including `sync-install-sh` (needs the WEBSITE_REPO_TOKEN secret).
2. Tag `vX.Y.Z` and push the tag; watch all four release jobs finish.
3. Verify release assets + SHA256SUMS on GitHub Releases.
4. Confirm Vercel redeployed and getkineti.com serves the new bytes
   byte-identical (hash compare above). The sync bot does this
   automatically whenever install.sh changes on green main — never
   hand-edit the copy in kineti-website.
5. Flip repo variable `DOMAIN_LIVE=1`.
6. Only now announce the one-liner.

## Go-live record

Executed 2026-08-25, all checks above passed at go-live:

- First published release: `v0.1.0` (tag → `219e618a`) — 8 assets:
  6 binaries (linux gnu/musl × x64/arm64, darwin arm64/x64),
  `SHA256SUMS`, `kineti.h`; both spot-checked binaries hash-verified,
  `--version` smoke OK.
- getkineti.com serves install.sh byte-identical to main
  (`185d8f8c…`, quiet-probe version); end-to-end one-liner executed
  cleanly: fetch → resolve latest → download → checksum → run
  `kineti 0.1.0`.
- Sync bot live: edits to install.sh on green main propagate to
  kineti-website automatically (~2 min, 201/200-aware, newline-safe
  byte compare).
- Repo variable `DOMAIN_LIVE=1` set — weekly `domain-smoke` is a hard
  gate from here on.
- CI hardening landed during go-live (kept as regression guards):
  identity-less runners need repo-local git config; `cargo build`
  before test so phase9 finds the cdylib; musl targets need
  musl-tools; aarch64 builds natively on ubuntu-24.04-arm; tag pushes
  must be in `on.push.tags`.
- v0.2.0 (2026-08-27): cut to gateway + receipt (`evidence → ship-check → verify`), release body now holds the `CHANGELOG.md` `v0.2.0` section (pasted via `gh release edit v0.2.0 --notes-file`), so `curl ... | sh -s v0.2.0` has a human note.
