# Contributing

Thanks for looking at Kineti. The project is small and opinionated on purpose.

## Ground rules

1. **Governance features must be mechanical.** If a safeguard depends on the
   model choosing to obey, it does not ship. Enforcement lives in control flow.
2. **Every module ships with tests.** `cargo test` and `cargo clippy -- -D warnings`
   must pass. Proof gates apply to the harness itself.
3. **No secrets, ever.** Keys come from environment variables. Nothing that
   could authenticate is ever written to disk or logs. Egress records redact.
4. **Binary stays under 10 MB.** Prefer small dependencies; justify each new one
   in the PR description with a size measurement (`cargo build --release`).

## Workflow

```sh
git clone https://github.com/iwpraveen/kineti && cd kineti
cargo test                    # prove before you propose
cargo clippy --all-targets -- -D warnings
```

Open a pull request with: what failure mode it addresses, how the enforcement
works mechanically, and which tests pin the behavior.

## Reporting bugs

Security issues: see [SECURITY.md](SECURITY.md). Everything else: GitHub Issues
with reproduction steps and (if possible) a failing test.
