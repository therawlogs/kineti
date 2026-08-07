---
name: /agent_eval
description: Executes Persona B (Frontier AI) agent evaluation frameworks, including accuracy benchmarks, tool-use correctness, regression detection, and A/B testing infrastructure setup.
---

# Skill: /agent_eval (Frontier AI Quality Benchmarking Gate)

## When to Use
Use before production deployment (`/ship`) for Persona B projects to measure agent quality beyond standard CI/CD code tests.

## Protocol & Actions
When this command is run, you **MUST** formulate and execute an Agent Eval Framework:
1. **Domain-Specific Accuracy Benchmarks**: Generate a test set of 50-100 expected user prompts and expected outcomes.
2. **Tool-Use Correctness**: Verify the agent selects the right tools with correct JSON parameters under ambiguous conditions.
3. **Regression Detection**: Compare current agent iteration against the previous baseline to ensure no capability degradation.
4. **Latency/Quality Pareto Curve**: Measure time-to-first-token (TTFT) and total generation time vs response quality.

Present the Eval Scorecard to the user, and **HALT FOR USER CONFIRMATION**.
