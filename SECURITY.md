# Security Policy

## Reporting

Report vulnerabilities privately via GitHub
[Security Advisories](https://github.com/iwpraveen/kineti/security/advisories/new).
Please do not open public issues for exploitable findings. Expect a response
within 7 days.

## Scope

Kineti is an agent harness that executes model-chosen commands on your machine.
The following are in scope:

- Path-fence escapes (tools escaping the project root, symlink tricks)
- Prompt-injection paths that lead to tool execution from untrusted content
- Hash-chain verification bypasses (tampered journals verifying as OK)
- Spend breaker bypasses
- Secret leakage into logs, tool output, or child process environments

## Honest limitations (v0.1)

- `bash` runs with your user's permissions inside the project directory; there
  is no syscall sandbox yet (planned: platform sandbox profiles).
- Quarantine is heuristic pattern-matching, not true dual-model isolation.
- Network access of spawned processes is not restricted.

Treat Kineti as blast-radius reduction and audit infrastructure — not as a
hard security boundary against a fully compromised host.
