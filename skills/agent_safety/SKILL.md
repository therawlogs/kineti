---
name: /agent_safety
description: Enforces Persona B (Frontier AI) agent safety guardrails, including content filtering, hallucination detection, tool-use sandboxing, human escalation triggers, and adversarial prompt injection defense.
---

# Skill: /agent_safety (Frontier AI Safety Guardrail Gate)

## When to Use
Use during the planning or ship phases for Persona B (Frontier AI Research Founder) projects to design and verify user-facing agent safety.

## Protocol & Actions
When this command is run, you **MUST** evaluate and present configuration options for:
1. **Adversarial Input Defense**: Mitigating prompt injection and jailbreak attempts from end-users.
2. **Tool-Use Sandboxing**: Defining strict execution boundaries (e.g., read-only DB access for user-facing agents).
3. **Hallucination Detection & Human Escalation**: Setting confidence thresholds where the agent halts and routes to a human operator.
4. **Content Filtering**: Output sanitization and PII egress prevention (in addition to `/cso`).

Present a comprehensive Agent Safety Strategy to the user, and **HALT FOR USER CONFIRMATION**.
