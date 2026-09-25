# Security Policy

We take the security of Kineti OS seriously. This document explains what versions we support, how to report security vulnerabilities, and how we handle reports.

## 1. Supported Versions

We provide security updates for the current active release line.

| Version | Supported | Notes |
| ------- | --------- | ----- |
| 0.3.x   | Yes       | Current active release series. |
| < 0.3.0 | No        | Please upgrade to 0.3.4 or newer. |

## 2. Reporting a Vulnerability

If you discover a security issue or vulnerability in Kineti, please report it privately. Do not open a public GitHub issue.

Report vulnerabilities only through GitHub native Private Vulnerability Reporting:

1. Open a private report at [GitHub Security Advisories](https://github.com/therawlogs/kineti/security/advisories/new).
2. Do not open a public issue and do not send email. No email address is needed.

### What to include in your report

Please include the following information:

1. **Description**: Clear description of the vulnerability and the affected components.
2. **Reproduction steps**: Numbered steps to reproduce the issue, including commands, payloads, or sample configurations.
3. **Impact**: Potential impact on local credentials, spend ceilings, execution integrity, or network egress.
4. **Environment**: Operating system, Bun version, and Rust version.

## 3. Response Process and SLA

When you submit a report, we follow these steps:

1. **Acknowledgement**: We acknowledge receipt of your report within 24 hours.
2. **Triage**: We confirm the issue, evaluate severity, and respond with findings within 72 hours.
3. **Fix and Release**: We develop and test a fix, release a patched version, and publish a security advisory within 14 days.
4. **Credit**: We credit researchers in release notes and security advisories unless you request anonymity.

## 4. Scope

The following components are in scope for security reports:

1. **Control plane and CLI tools** (`bin/`): Spending limit controls (`kineti-spend.ts`), saga rollback system (`kineti-saga.ts`), cryptographic test evidence verification (`kineti-evidence.ts`), and network egress tracking (`kineti-egress.ts`).
2. **Native nervous system** (`core-native/`): Memory safety, process boundaries, vector clocks, and protocol gateways.
3. **Companion dashboard** (`bin/kineti-companion.ts`): Local web server authentication, host header validation, origin gating, and session token storage.
4. **Skills library** (`skills/`): Workflow skills and prompt safety boundaries.

### Out of Scope

1. Reports concerning local modifications or unreleased forks that do not reproduce on the main branch.
2. Denial of service attacks requiring direct root or administrator access to the user machine.
3. Theoretical issues without a reproducible proof-of-concept.

## 5. Security Principles in Kineti

Kineti is designed around defensive defaults:

1. **Fail-closed spend limits**: Tools halt execution immediately if cost thresholds are reached.
2. **Loopback-only binding**: The companion web server binds only to `127.0.0.1` and validates Host and Origin headers.
3. **Local data isolation**: Runtime state in `.kineti/` uses strict file permissions and is excluded from git tracking.
4. **Zero credential leakage**: Tools never print secrets, passwords, or raw auth tokens to logs or console outputs.
