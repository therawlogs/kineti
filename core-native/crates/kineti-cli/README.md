# Kineti 0.6.0 native gate

**Record checks, detect changed inputs, and block participating CI commands when evidence or reported spending fails policy.**

0.6.0 is a gate-only release candidate. Publication requires reviewed artifacts
and separate approval. Only Linux and macOS, x64 and arm64, are supported.
`kineti-cli` changes from its older assistant lineage to this gate. Older
published releases remain available. Harness execution, model routing,
delegation, swarm, dashboard, memory, evaluator services and protected writes
are removed from the active product.

## Install and initialize

After approved publication, the installation commands will be:

```sh
cargo install kineti-cli --version 0.6.0 --locked
# Or:
npm install -g kineti@0.6.0
kineti init --goal "Your explicit project goal"
kineti init --goal "Your explicit project goal" --apply
```

This is not a claim that 0.6.0 is already published. Cargo needs Rust for
installation. npm needs Node and enabled installation scripts for installation.
The installed executable needs neither Node nor Bun. The npm package contains
four checksummed binaries; installation verifies/selects one and removes unused
payloads. Disabled scripts leave an explicit installation error.

From source, use `cargo run --manifest-path core-native/Cargo.toml -p kineti-cli --bin kineti --`.

`init` previews changes, destinations and backups. Review before `--apply`.
New projects require an explicit goal; existing locked goals and historical
records stay intact. Missing fields may be initialized; damaged records require
separate reviewed recovery. When Codex is detected, init proposes native MCP
registration and one instruction skill. `--codex-home <directory>` chooses an
explicit destination. Repetition preserves unrelated settings; edited owned
settings are refused. `init --uninstall --apply` removes owned integration
content and preserves project records. Restart existing Codex sessions after
changes. Cursor and Antigravity configuration is preserved, but those hosts are
unverified in 0.6.0. There is no automatic native-tool interception. Codex integration checks cover only
0.163.0-alpha.11.2 and stable 0.164.0; no intervening version is implied supported.

## Record and check

```sh
kineti test run --label tests --required-test tests -- cargo test
kineti test check --label tests
kineti ci --require-evidence tests
kineti spend log --stage build --model reported --tokens-in 0 --tokens-out 0 --usd 0.25
kineti spend check
kineti undo push --label cleanup -- rm generated-file
kineti undo rollback
kineti undo rollback --yes
kineti self-test
```

Configure required labels in `.kineti/gate.json` or pass them explicitly. Empty
required checks cannot pass. `--evidence-commands <json-file>` binds each label
to an exact argv array. `--env-name NAME` declares relevant environment inputs;
PATH is bound by default. Receipts bind project inputs, dependency files, runtime
identity, exact arguments, declared environment, session reference and observed
results. Commands provide structured JSON; explicit `--json` is also accepted.
`evidence` and `saga` remain aliases. Removed commands fail explicitly.

`verify` runs only its declared command after command-hash trust. Use
`verify --status` and interactive `verify --trust`. It does not intercept other
commands. CI checks evidence, project gates and reported spending policy.

## Trust and recovery

A valid signature does not prove honest tests. A command can fabricate a passing
summary; self-test demonstrates this limitation. Local keys do not protect
against a same-account adversary. Native receipts use `kineti-canonical-v2`;
legacy signatures use the original JavaScript serialization. Legacy evidence
is readable and signature-checkable; unsupported legacy runtime/parser freshness
cannot authorize the native gate. Run fresh native checks after migration.

Spending is reported accounting, not provider-bill verification. Integer
millionths of USD are used, with a 95% total/stage stop. Corruption, unknown costs
and unresolved reservations fail closed. Reset requires audited `--i-am-human`
confirmation; the flag does not prove a human acted and cannot create headroom.

`spend recover` previews recovery. Select a preserved snapshot inside the project
with `--source` and `--evidence`. Apply requires the exact reviewed
`--proposal-sha256`, `--apply` and `--i-am-human`. Originals remain unchanged and
are backed up; a separate recovery view and resolution trail are recorded.
Unknown records are never cleared. Native chain damage requires inspection.

Undo records intent/completion, runs newest first and continues after definite
failures. Uncertain interruption blocks replay until inspected and resolved with
`undo resolve --action-id ID --outcome applied|not-applied --yes`.
If the history guard was interrupted, `undo inspect` previews its exact bytes;
`undo inspect --expected-sha256 HASH --yes` records reviewed inspection and keeps
a byte backup. This does not resolve uncertain action outcomes.
There is no atomic authorization of later writes or protected-write guarantee.

## Acceptance

The [contract](docs/GATE-0.6.0-CONTRACT.md),
[behavior map and exit codes](docs/GATE-0.6.0-BEHAVIOR.md), and
[evidence matrix](docs/GATE-0.6.0-EVIDENCE.md) govern acceptance. Final installed
files for each route must be below **10,000,000 bytes** per target, including
initial integration assets. Download and unpacked-package sizes are separate.
No unmeasured platform-size claim is made.

Exit codes: 0 success; 1 failed command/check or incomplete undo; 2 invalid input;
3 spending/integrity/recovery stop; evidence 4 stale, 5 missing, 6 unsupported
legacy, 7 invalid/untrusted; verify 9 missing/untrusted command.

Dated older results, including the historical 631-test result, are separate from
the new suite inventory. [Historical documents](docs/history/pre-0.6.0/README.md)
and Git history remain available; they are not current product guarantees.
