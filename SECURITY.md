# Security Policy

Kineti records checks, detects changed inputs, and blocks participating CI commands
when evidence or reported spending fails policy. The source candidate is the
0.6.1 native gate, a single `kineti-cli` crate providing `kineti`. Today's local
repair candidate needs fresh acceptance; it does not replace the dated evidence
for already published artifacts. This policy
explains support, reporting commitments and the limits of those checks. It is not
a sandbox or a claim that a candidate package has been accepted or published.

## 1. Supported Versions

We provide security updates for the current active release line.

| Version | Supported | Notes |
| ------- | --------- | ----- |
| 0.6.x   | Yes       | Native gate; the 0.6.1 candidate awaits fresh acceptance before any new release/platform claims. |
| 0.5.x   | Yes       | Existing security support is retained for this legacy release line; its protected writes remain experimental. |
| 0.4.x   | No        | Please upgrade to 0.5.0 or newer. |
| 0.3.x   | No        | Please upgrade to 0.5.0 or newer. |
| < 0.3.0 | No        | Please upgrade to 0.5.0 or newer. |

## 2. Reporting a Vulnerability

If you discover a security issue or vulnerability in Kineti, please report it privately. Do not open a public GitHub issue.

You can report vulnerabilities privately through either of these channels:

1. **Email**: Send your findings directly to [security@getkineti.com](mailto:security@getkineti.com).
2. **GitHub Security Advisory**: Open a private report at [GitHub Security Advisories](https://github.com/therawlogs/kineti/security/advisories/new).

Please do not open a public issue. We acknowledge all reports within 24 hours.

### Disclosure policy

1. **Private first**: we confirm the issue with you before anything is published.
2. **Advisory**: fixed issues are published as a GitHub Security Advisory, usually alongside the patch release.
3. **Timing**: we aim to publish within 14 days of your report (see the SLA below). If a fix needs longer, we agree a new date with you.
4. **Credit**: researchers are credited in the advisory unless they ask to stay anonymous.
5. **No PGP key**: we do not publish one. Use the email address or the GitHub advisory form above; both are private.

### What to include in your report

Please include the following information:

1. **Description**: Clear description of the vulnerability and the affected components.
2. **Reproduction steps**: Numbered steps to reproduce the issue, including commands, payloads, or sample configurations.
3. **Impact**: Potential impact on local credentials, spend ceilings, execution integrity, or network egress.
4. **Environment**: Operating system and architecture, exact Kineti version and executable digest, and Rust/Cargo version when building from source. For a legacy report, include Bun/Node versions where relevant. Do not include credentials or private keys.

## 3. Response Process and SLA

When you submit a report, we follow these steps:

1. **Acknowledgement**: We acknowledge receipt of your report within 24 hours.
2. **Triage**: We confirm the issue, evaluate severity, and respond with findings within 72 hours.
3. **Fix and Release**: We develop and test a fix, release a patched version, and publish a security advisory within 14 days.
4. **Credit**: We credit researchers in release notes and security advisories unless you request anonymity.

## 4. Scope

The following components are in scope for security reports:

1. **Native CLI and gate** (`core-native/crates/kineti-cli/src/`): `gate_main.rs` validates commands; `gate.rs` checks exact declared command trust and participating CI policy; `receipt.rs` supervises commands and signs/checks receipts; `compat.rs` verifies supported historical signatures.
2. **Accounting and undo**: `spend.rs` checks reported amounts, limits, reservations and reviewed recovery; `journal.rs` checks legacy accounting integrity; `undo.rs` records intent, completion and explicit inspection before uncertain replay; `store.rs` handles durable files and operation locks.
3. **Host and package boundaries**: `mcp.rs` exposes gate tools over stdin/stdout JSON-RPC, and `init.rs` previews/applies owned integration configuration. `install.cjs` and `bin/kineti` select the checksummed native npm payload. `self_test.rs` demonstrates both gate behavior and fabricated-output limitations.
4. **Supported legacy release components**: Reports about 0.5.x CLI tools, native crates (including process boundaries, vector clocks and protocol gateways), companion server and workflow skills remain in scope under the support policy. Their code and dated release records remain in Git history. The dashboard, model routing, memory product and protected-write service are not active native-gate runtime surfaces.

### Out of Scope

1. Reports concerning local modifications or unreleased forks that do not reproduce on the main branch.
2. Denial of service attacks requiring direct root or administrator access to the user machine.
3. Theoretical issues without a reproducible proof-of-concept.

## 5. Security Principles in Kineti

Kineti is designed around defensive defaults:

1. **Reported spending**: Amounts use integer monetary units, total and stage limits, and a 95% stop. Corruption, unresolved monetary reservations and unknown costs deny. This is reported accounting, not provider-bill verification. Agents and participating commands must act on the denial; Kineti does not intercept unrelated tools. The approved subscription-only local implementation exception preserves historical unknowns and authorizes $0 new paid spending. It does not authorize publication or bypass integrity failures or uncertain monetary writes.
2. **Runtime surfaces**: Native MCP uses stdin/stdout; it opens no companion HTTP listener. The legacy companion's loopback binding and Host/Origin validation belong to its dated release behavior, not to a new native server guarantee. Codex integration proves only the tested versions and calls. Cursor and Antigravity remain unverified.
3. **New-file permissions and existing files**: Atomic replacement/state and issuer-key creation request `0600`. New append files and operation-lock files also explicitly request `0600`, even with a permissive umask. Opening an existing append file preserves its mode; existing or interrupted locks are not silently changed or reclaimed. Historical broad modes remain unresolved pending an exact, separately approved metadata-only proposal. An inventory is not a repair. Mode bits alone prove neither parent-directory security nor cross-account access; broad log permissions alone do not establish private-key exposure. Runtime `.kineti/` records remain excluded from Git.
4. **No credentials in logs policy**: Secrets, passwords and raw auth tokens must not be written to logs or state. Automatic argument redaction is not implemented: receipts and undo records preserve exact argv, so user-supplied secret arguments can be persisted. Do not place credentials in command arguments, labels, fixture files or outputs. Secret-like declared environment names are rejected and allowed environment values are hashed; this does not scan arbitrary command arguments or sanitize inherited subprocess environments. Runtime secret filtering needs a separate decision. The historical companion authorization-token exception remains limited to that legacy component: its token is printed once to the local terminal for login, not intentionally logged.

## 6. Evidence and execution limits

A receipt signature establishes integrity relative to the trusted local issuer;
it does not prove honest tests, independent review or control by a separate
account. Test commands can print fabricated passing summaries. Native checks
also require exact argv, declared inputs, dependencies, relevant environment and
runtime identity to remain fresh. Historical signature validity does not grant
new native execution authority. Confirmation flags and interactive trust hashes
record acknowledgment; they do not establish human identity.

The shared runner uses shell-free argv and a monotonic execution/capture deadline.
A complete result requires definite child status and EOF on both streams before
that deadline. It retains at most 8,000,000 raw bytes per stream while continuing
to drain; this is not a total process-memory limit. Completed truncation cannot
authorize passing evidence or verification. Incomplete capture preserves uncertain
undo intent rather than authorizing replay. Cleanup has at most one additional
monotonic second; unconfirmed cleanup reports code 3 with identity/diagnostics.
Execution denies unsupported child-wait control, including automatic reaping or
external child-reaper signal handlers. Confirmed cleanup includes direct-child
reaping and closure of owned pipes; it does not prove every descendant has died.
After complete capture, the existing group-termination attempt remains best effort.
This bounds supervision, not hashing, persistence, OS suspension or escaped
descendants, and is not a hard real-time or sandbox guarantee. Actual lifecycle
proof on every Linux/macOS x64/arm64 target remains required before new platform
claims; local tests and cross-compilation do not establish installed acceptance.

The native gate provides no same-account adversary boundary, universal command
interception, protected writes or automatic native-tool hooks. A participating
gate check does not atomically authorize a later write. Preserve original bytes,
signatures and unresolved records; accounting or historical-permission recovery,
exact command trust, publication, merging and deployment require their separate
reviews and approvals. See the [acceptance contract](docs/GATE-0.6.0-CONTRACT.md)
and [repair contract](docs/NATIVE-GATE-REPAIR-IMPLEMENTATION-CONTRACT-20261026.md).
