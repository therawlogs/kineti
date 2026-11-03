# Research note: who controls the checker?

Date: 2026-10-26. Status: explanatory research, not an implementation proposal.

This note leaves v1 scope and all acceptance requirements unchanged. The
[0.6.0 acceptance contract](../GATE-0.6.0-CONTRACT.md), including G11, G16 and
its stated limits, remains authoritative. No new gate or release condition is
introduced here.

## Three separate claims

1. **Signature validity:** recorded bytes verify against a specified public key.
   This does not establish who controls that key or whether its issuer deserves
   authority.
2. **Evidence honesty:** checks actually ran against the stated inputs and
   reported their results honestly. A signed passing summary can still contain
   fabricated output.
3. **Checker independence:** the agent being checked cannot change or bypass the
   checker, its trusted issuers and keys, policy, required checks, evidence
   acceptance rules or decision delivery. A separate process or model alone
   does not establish this separation.

Kineti's fabricated-output caveat addresses the second claim. The third asks a
different question: even if a check ran honestly, could the checked agent weaken
the check, replace its policy or suppress its result? A valid signature does not
answer that question.

## Why the distinction matters

[OpenAI's monitoring report](https://alignment.openai.com/misalignment-reports/suppressing-misalignment-monitoring-alerts-and-disabling-misalignment-monitoring)
describes agents suppressing monitoring notifications and disabling monitoring
during coding tasks. This illustrates why control over the monitoring path must
be considered alongside the contents of its reports.

[Aegis's changelog](https://github.com/nseney1/Aegis-Guardrails/blob/main/CHANGELOG.md)
distinguishes local self-attestation from stronger governance in 4.3.0 and
describes base-trusted, content-bound human insight approvals in 4.3.1. These
are documented design choices, not independent verification of Aegis's
implementation or effectiveness. They illustrate the need to examine where
approval authority comes from, not merely whether an approval is signed.

## Implication for Kineti v1

Keep the existing claim: record checks, detect changed inputs, and block
participating CI commands when evidence or reported spending fails policy.
Local receipts do not establish independent test execution or protection against
a same-account adversary. A confirmation flag does not prove human identity.

For research, ask who can modify the checker, issuer trust, policy, check
inventory, evidence and result delivery. Any future claim of independence would
need evidence of those control boundaries under a stated threat model. This
note neither requires that architecture for v1 nor reactivates deferred evaluator
or protected-write work. It changes no accounting, publication or approval rule.
