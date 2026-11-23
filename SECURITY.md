# Security Policy

Kineti records checks, detects changed inputs and PR-diff policy/test tampering,
and blocks participating CI commands when evidence or reported spending fails
policy. The active open-core (MIT) release line is **Kineti `1.0.0`**, built
from the single `kineti-cli` crate providing `kineti` and the composite GitHub
Action (`action.yml`). Local candidate builds require fresh verification and do
not authorize publication or deployment without separate owner approval. This
policy explains support, private reporting expectations, and the technical
limits of Kineti's checks. It is not a sandbox, automatic command interceptor,
or warranty.

## 1. Supported Versions

Security triage is provided on a best-effort basis for the active `1.0.x` native
gate line only. Older release lines are unsupported and remain preserved solely
in Git history.

| Version | Supported | Notes |
| --- | --- | --- |
| `1.0.x` | Yes (best-effort) | Active open-core (MIT) native gate & Linux GitHub Actions PR anti-tampering line (`kineti-cli`). |
| `0.6.x` | No | Historical registry packages are retired (deprecated on npm, yanked on crates.io). |
| `0.5.x` and earlier | No | Legacy release line preserved in Git history only; no longer supported. |

## 2. Reporting a Vulnerability

If you discover a security issue or vulnerability in Kineti, please report it
privately. Do not open a public issue or disclose exploit details publicly
before coordination.

You can report vulnerabilities privately through:

1. **Email (Primary):** Send findings directly to [security@getkineti.com](mailto:security@getkineti.com).
2. **GitHub Security Advisory (for authorized repository collaborators):** Open a private advisory in the repository's Security tab.

### Disclosure policy

1. **Private first:** We confirm the issue with you privately before publishing details.
2. **Coordinated remediation:** Fix timing is agreed directly with the reporter based on severity and maintainer capacity; we do not make an unconditional 14-day mandatory patch SLA.
3. **Credit:** Researchers are credited in release notes or advisories unless they request anonymity.
4. **No PGP key:** We do not publish a PGP key; use private email (`security@getkineti.com`) or a private repository security advisory.

### What to include in your report

Please include:

1. **Description:** Clear explanation of the vulnerability and affected components.
2. **Reproduction steps:** Minimal numbered steps, commands, diffs, or fixtures to reproduce.
3. **Impact:** Potential impact on base-branch policy enforcement, PR-diff tampering checks, command execution boundaries, receipt integrity, or local credential exposure.
4. **Environment:** OS and architecture, exact Kineti version and executable SHA-256 digest, and Rust/Cargo version if built from source. Do **not** send real secrets, API tokens, or private signing keys.

## 3. Solo-Maintainer Response Process

Kineti is maintained by a solo founder. Our realistic best-effort triage
commitments are:

1. **Acknowledgement:** Best-effort acknowledgement within **5 business days**.
2. **Triage & prioritization:** Critical security flaws (policy bypasses, unintended command execution, signature-verification defects, or secret leakage) are prioritized ahead of feature work.
3. **Fix & disclosure coordination:** We develop and verify a fix on a best-effort schedule and agree on disclosure timing with the reporter.

## 4. Scope

The following active native-gate components are in scope:

1. **Native CLI & PR anti-tampering gate** (`core-native/crates/kineti-cli/src/` and `action.yml`): `gate_main.rs` argument validation; `pr_guard.rs` base-branch `.kineti/gate.json` loading, PR-diff policy/test tampering detection, and `kineti-pr-audit-v1` signing; `gate.rs` fail-closed declared command enforcement; `receipt.rs` POSIX command supervision, parser contract, and receipt verification; `compat.rs` historical signature compatibility.
2. **Accounting, undo & durable storage:** `spend.rs` integer reported-spend limits and reviewed recovery; `journal.rs` legacy accounting integrity; `undo.rs` CLI-only intent/completion and uncertain-replay guards; `store.rs` `O_NOFOLLOW` two-tier inventory hashing, atomic writes, and locking.
3. **Host boundaries:** `mcp.rs` stdin/stdout JSON-RPC server (strictly allowlisted `kineti_evidence_record` matching `.kineti/gate.json`; undo/saga tools are intentionally removed from MCP) and `init.rs` previewable setup and backup handling.

### Out of Scope

1. Reports targeting retired `0.6.x` / `0.5.x` legacy lines, archived harness/memory/companion code in Git history, or modified forks that do not reproduce on the active branch.
2. Local attacks assuming root/administrator privileges or same-account shell access outside a protected CI runner (see Section 6).
3. Theoretical findings without a reproducible proof-of-concept.

## 5. Security Principles in Kineti

1. **Base-branch trust boundary:** Kineti 1.0 enforces `.kineti/gate.json` read strictly from the protected base commit on a protected CI runner. Same-account local execution on a developer machine cannot prevent an agent with local write permissions from modifying files outside Kineti.
2. **Strict command allowlisting:** `kineti ci` fails closed when a required label lacks an exact declared `argv` command mapping. Native MCP (`kineti mcp`) speaks stdin/stdout JSON-RPC with no HTTP listener, strictly enforces `.kineti/gate.json` command allowlists on `kineti_evidence_record`, and does not expose `undo` / `saga` tools over MCP. Local MCP has previously been tested only on specific macOS arm64 Codex versions; **Cursor and Antigravity remain unverified**.
3. **Reported spending:** Reported spending uses integer monetary units, total and stage ceilings, and a 95% stop. Corruption, unresolved reservations, and unknown historical costs fail closed. This is operator-reported accounting, not provider-bill verification. Historical monetary unknowns remain preserved and are never reset to zero.
4. **File permissions & runtime state:** Atomic state writes, issuer keys, new append files, and lock files request `0600`. Existing append files preserve their mode; existing or interrupted locks are not silently reclaimed. Runtime `.kineti/` state must remain excluded from Git.
5. **Exact-`argv` secret-handling limits:** Automatic command-argument redaction is **not** implemented. Test receipts and undo records persist exact `argv`, so credentials passed on the command line can be written to `.kineti/` state or audit outputs. Never place secrets, passwords, or auth tokens in command arguments, labels, fixture files, or test output. Secret-like declared environment names are rejected and allowed environment values are hashed; Kineti does not scan arbitrary command arguments or sanitize inherited subprocess environments.

## 6. Evidence and Execution Limits

- **Signatures vs. assertion quality:** An Ed25519 signature on a receipt or `kineti-pr-audit-v1` bundle establishes integrity relative to the trusted signing key—not that tests are honest, meaningful, or free of fabricated output (`"independent_assertion_quality_proven": false`).
- **Two-tier test parsers:** Built-in count-aware parsing (`--parser auto`) extracts observed pass/fail/skip counts for `cargo test` and multi-suite `bun test`. Allowlisted polyglot runners (`pytest`, `jest`, `vitest`, `go test`, `node --test`, `make test`) executed via `--parser exit-code` enforce exit `0`, exact `argv`, required test presence, untruncated capture, and workspace freshness while explicitly recording `"count_observed": false`.
- **Confirmation flags:** CLI confirmation flags and interactive trust hashes record acknowledgment; they do not establish human identity.
- **Process supervision limits:** The runner executes shell-free `argv` under a monotonic deadline, requires definite child exit status and EOF on both streams, retains at most 8,000,000 raw bytes per stream while draining (not a total process-memory limit), and rejects truncated output from authorizing passing evidence. Cleanup allows at most one additional monotonic second; confirmed cleanup covers direct-child reaping and pipe closure, not proof that every escaped descendant has exited. This is bounded execution supervision, not a hard real-time guarantee or kernel sandbox.
- **No automatic interception or protected writes:** Kineti provides no universal command interception, no kernel sandboxing, no protected writes, no automatic native-tool hooks, and no atomic authorization of later writes. Publication, merging, and deployment require separate owner approval. See [docs/GATE-0.6.0-CONTRACT.md](docs/GATE-0.6.0-CONTRACT.md).
