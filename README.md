# Kineti 0.6.1 native gate candidate

**Record checks, detect changed inputs, and block participating CI commands when evidence or reported spending fails policy.**

0.6.1 is an unreleased repair candidate targeting Linux and macOS, x64 and
arm64. It needs fresh acceptance of its exact source, binaries and installed
packages, followed by separate artifact and accounting approval for publication.
`kineti-cli` changes from its older assistant lineage to this gate. Older
published releases remain available. Harness execution, model routing,
delegation, swarm, dashboard, memory, evaluator services and protected writes
are removed from the active product.

## Install and initialize

After approved 0.6.1 publication, choose one installation route:

Cargo (requires Rust for installation):

```sh
cargo install kineti-cli --version '=0.6.1' --locked
```

npm (requires Node and enabled installation scripts for installation):

```sh
npm install -g kineti@0.6.1
```

The already published 0.6.0 packages remain available: [Cargo 0.6.0](https://crates.io/crates/kineti-cli/0.6.0)
and [npm 0.6.0](https://www.npmjs.com/package/kineti/v/0.6.0).
See the [GitHub release](https://github.com/therawlogs/kineti/releases/tag/v0.6.0)
and [release closeout](docs/GATE-0.6.0-CLOSEOUT.md) for dated 0.6.0 publication and acceptance evidence. That approval and those artifacts do not establish 0.6.1 acceptance.

From your project directory, preview initialization with your explicit goal:

```sh
kineti init --goal "Your explicit project goal"
```

Review the preview, then apply initialization:

```sh
kineti init --goal "Your explicit project goal" --apply
```

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
unverified for this candidate. There is no automatic native-tool interception.
Dated 0.6.0 Codex integration checks cover only 0.163.0-alpha.11.2 and stable
0.164.0; no intervening version is implied supported. Fresh 0.6.1 host checks
must bind the new executable and exact Codex version.

## Record and check

```sh
kineti test run --label tests --required-test tests -- cargo test
kineti test check --label tests --expect-argv '["cargo","test"]'
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
legacy signatures use the original JavaScript serialization. Unmarked historical v3 receipts also retain that original signing format. Legacy evidence
is readable and signature-checkable; unsupported legacy runtime/parser freshness
cannot authorize the native gate. Run fresh native checks after migration.

Spending is reported accounting, not provider-bill verification. Integer
millionths of USD are used, with a 95% total/stage stop. A lower configured
safety factor can stop spending earlier; configuration cannot raise the 95% stop. Corruption, unknown costs
and unresolved reservations fail closed. Reset requires audited `--i-am-human`
confirmation; the flag does not prove a human acted and cannot create headroom.

`spend recover` previews recovery. Select a preserved snapshot inside the project
with `--source` and `--evidence`. Apply requires the exact reviewed
`--proposal-sha256`, `--apply` and `--i-am-human`. Originals remain unchanged and
are backed up; a separate recovery view and resolution trail are recorded.
Unknown records are never cleared. Native chain damage requires inspection.
Incomplete recovery, altered recovery history, and changes to a reviewed recovery
view block accounting until inspected.

Undo records intent/completion, runs newest first and continues after definite
failures. Uncertain interruption blocks replay until inspected and resolved with
`undo resolve --action-id ID --outcome applied|not-applied --yes`.
If the history guard was interrupted, `undo inspect` previews its exact bytes;
`undo inspect --expected-sha256 HASH --yes` records reviewed inspection and keeps
a byte backup. This does not resolve uncertain action outcomes.
There is no atomic authorization of later writes or protected-write guarantee.

## Acceptance

The frozen [0.6.0 contract](docs/GATE-0.6.0-CONTRACT.md),
[behavior map and exit codes](docs/GATE-0.6.0-BEHAVIOR.md), and
[dated evidence matrix](docs/GATE-0.6.0-EVIDENCE.md) preserve the behavior and
accounting baseline. The owner separately selected 0.6.1 preparation; its exact
artifacts need fresh acceptance and separate publication approval. Final installed
files for each route must be below **10,000,000 bytes** per target, including
initial integration assets. Download and unpacked-package sizes are separate.
No unmeasured platform-size claim is made.

Exit codes: 0 success; 1 failed command/check or incomplete undo; 2 invalid input;
3 spending/integrity/recovery stop; evidence 4 stale, 5 missing, 6 unsupported
legacy, 7 invalid/untrusted; verify 9 missing/untrusted command.

Dated older results, including the historical 631-test result, are separate from
the new suite inventory. [Historical documents](docs/history/pre-0.6.0/README.md)
and Git history remain available; they are not current product guarantees.

Operational `evidence check` results do not authorize test gates. Test checks and CI
require test receipts. Declared label commands override default test commands;
operational checks never inherit the default test command. Value options accept
`--name=value`; arguments after `--` remain exact, including empty arguments.
