# Contributing

Thanks for looking at Kineti. The project is small and opinionated on purpose.

**Product:** v0.2.2 is ship proof + spend fuse for any agent — `evidence → ship-check → verify` + `gateway` + `swarm --tasks` (any verification via `[artifacts]` + `[proof].command`). The 13-stage runner is frozen on tag `v0.1.0` (`docs/v0.1.md`, `kineti run --legacy --goal` hidden, prints legacy warning; without `--legacy` still works) — bug fixes only, no new stages.

## Ground rules

1. **Governance features must be mechanical.** If a safeguard depends on the
   model choosing to obey, it does not ship. Enforcement lives in control flow.
2. **Every module ships with tests.** `cargo test` and `cargo clippy -- -D warnings`
   must pass. Proof gates apply to the harness itself.
3. **No secrets in the wrong place, ever.** Provider credentials live in
   environment variables or OAuth tokens stored at `~/.kineti/auth/` with
   mode 0600 (Phase 8). Nothing that could authenticate is ever written to
   project files, logs, or tool output. Egress records redact.
4. **Binary stays under 10 MB.** Prefer small dependencies; justify each new one
   in the PR description with a size measurement (`cargo build --release`).

## Workflow

```sh
git clone https://github.com/therawlogs/kineti && cd kineti
cargo build                   # phase9 asserts symbols in target/debug cdylib
cargo test --all              # prove before you propose (any verify cmd also works)
cargo clippy --all-targets -- -D warnings
./scripts/size-gate.sh        # <10 MB ETHOS budget (informational locally, hard in CI)
kineti evidence --cmd "cargo test --all" && kineti ship-check  # any cmd: pytest / npm test / ./verify.sh
# or: kineti evidence         # uses [proof].command from kineti.toml
```

CI runs the same gates on every push and PR — tests execute twice per commit
(direct and daemon backend matrix), so both transports must stay green. The
required check `kineti-receipt` (`.github/actions/kineti-receipt`, `with: verify-command` / `proof-command`)
runs `evidence --cmd` then `ship-check`+`verify --all` — see `README.md`.
Releases are tag-driven; maintainers cut them per [docs/RELEASE.md](docs/RELEASE.md)
(paste `CHANGELOG.md` section into the GitHub release body).

Open a pull request with: what failure mode it addresses, how the enforcement
works mechanically, and which tests pin the behavior.

## License

By contributing you agree your contributions are licensed under the
Apache License 2.0, the project's license.

## Reporting bugs

Security issues: see [SECURITY.md](SECURITY.md). Everything else: GitHub Issues
with reproduction steps and (if possible) a failing test.
