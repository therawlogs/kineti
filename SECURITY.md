# Security Policy

We take the security of Kineti OS seriously. This document explains what versions we support, how to report security vulnerabilities, and how we handle reports.

## 1. Supported Versions

We provide security updates for the current active release line.

| Version | Supported | Notes |
| ------- | --------- | ----- |
| 0.3.x   | Yes       | Current active release series. |
| < 0.3.0 | No        | Please upgrade to 0.3.8 or newer. |

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

1. **Self-reported spend limits**: the agent records each model call; the breaker trips on recorded totals once cost thresholds are reached.
2. **Loopback-only binding**: The companion web server binds only to `127.0.0.1` and validates Host and Origin headers.
3. **Local data isolation**: Runtime state in `.kineti/` is written with owner-only permissions (`0600`) and is excluded from git tracking.
4. **No credentials in logs**: Secrets, passwords, and raw auth tokens are never written to log files or state files. The one deliberate exception is the companion's local authorization token, which is printed once to your terminal (never to a log file) so you can paste it into the login form on the same machine.
