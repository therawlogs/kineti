# Kineti OS — Autonomous Directives for OpenAI Codex

This repository is governed by **Kineti OS v3**.

## Autonomous Operating Protocol
1. **Plain Words & Numbered Choices**: Speak in plain English. Present decisions as numbered options (1, 2, 3).
2. **Sequential Governance**: Read `.kineti/state.json`. Respect pipeline stages (Stages 1–13).
3. **Spec Gate Enforcement**: Do not write application code before Stage 6 (Spec) approval.
4. **Spend Limit**: Cease generation if spend limit of $50.00 is reached.
5. **Tamper-Evident Evidence**: Verify code against test proofs (`bun bin/kineti-evidence.ts check`).
