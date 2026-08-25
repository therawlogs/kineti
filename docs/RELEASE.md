# Release checklist

Everything needed to cut a Kineti release and keep getkineti.com honest.

## Cutting a release

1. **Pre-flight** (local): `cargo test --all` · `cargo clippy --all-targets -- -D warnings` · `./scripts/size-gate.sh` · `./scripts/bench-startup.sh` (informational) — all green.
2. **Version**: bump `version` in `Cargo.toml`. The CLI banner, daemon stamp,
   and C-ABI `kineti_version()` all derive from this single field
   (`env!("CARGO_PKG_VERSION")`) — never hand-edit version strings elsewhere.
3. **Tag**: `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. **CI publishes automatically** (tag-gated jobs):
   - linux gnu + musl-static: `kineti-linux-x64`, `kineti-linux-arm64`,
     `kineti-linux-x64-static`, `kineti-linux-arm64-static`
   - macos: `kineti-darwin-arm64`, `kineti-darwin-x64`
   - then `release-checksums` aggregates every asset into `SHA256SUMS`
     and uploads it to the same release
   - then `release-header` attaches `include/kineti.h` for C/Python consumers
5. **Verify**: download two assets on different platforms and run
   `sha256sum -c` against `SHA256SUMS`; smoke `--version` on each.
6. **Announce** only after step 7 passes.

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

- [ ] `origin` remote points at `https://github.com/therawlogs/kineti`
- [ ] Committer identity set locally (`git config --local user.name/email`)
- [ ] All intended work is committed; `git status` clean or only local-only files
- [ ] Force-push safety confirmed: remote `main` holds only a LICENSE blob

### B. CI release pipeline (tag-gated jobs)

- [ ] Every job that uploads assets carries `permissions: contents: write`
      (release-linux, release-macos, release-checksums, release-header)
- [ ] Asset names match install.sh probes exactly:
      kineti-linux-x64 · -arm64 · -x64-static · -arm64-static ·
      kineti-darwin-arm64 · kineti-darwin-x64
- [ ] `SHA256SUMS` lines are `<hash>  <asset>` so install.sh's
      `grep " $asset$"` matches
- [ ] Static musl targets build with zero C dependencies (pure-Rust dep tree)
- [ ] `include/kineti.h` exists in the tagged commit (release-header uploads it)

### C. Pre-flight gates (same as "Cutting a release" step 1)

- [ ] `cargo test --all` green
- [ ] `cargo clippy --all-targets -- -D warnings` green
- [ ] `./scripts/size-gate.sh` passes (<10 MB ETHOS budget)
- [ ] `./scripts/bench-startup.sh` recorded (informational)

### D. Clean-files law (ETHOS §8.1) — zero matches required

- [ ] No personal names in tracked files (search given-name and surname variants)
- [ ] No home paths in tracked files (`$HOME`, `/Users/<name>`, `/home/<name>`)
- [ ] No personal email inside tracked files (your mail domain) — git config
      metadata is exempt; file contents are not
- [ ] No secrets: API keys, tokens, private keys (`AKIA`, `ghp_`, `sk-`,
      `BEGIN.*PRIVATE KEY`, hardcoded passwords)

### E. Repo hygiene

- [ ] `.gitignore` covers `target/ .kineti/ .DS_Store demo/ dist/`
- [ ] No binary artifacts or editor junk tracked (`git ls-files` review)
- [ ] No tracked file >1 MB (GitHub hard limit 100 MB, warn 50 MB)
- [ ] `LICENSE` present and matches `license` field in Cargo.toml
- [ ] README install one-liner uses `https://getkineti.com/install.sh`
- [ ] Version in Cargo.toml is the intended first tag (`v0.1.0`)

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
