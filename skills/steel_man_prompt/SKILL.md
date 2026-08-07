---
name: /steel-man-prompt
description: Hardens prompts via red-teaming, simulating jailbreaks and leaks to reinforce instructions against adversarial input.
---

# Skill: /steel-man-prompt (Step 10 - Adversarial Prompt Hardening)

## When to Use
During application prompt engineering cycles to validate behavior under stress.

## How to Use
Enter `/steel-man-prompt` targeted at a proposed core system prompt.

## Sequencing
- **Phase**: `03_deterministic_ai_runtime`
- **Step**: 10 (Prompt security and predictability optimization gate).

## Protocol & Actions
- **Mode**: `SYNTHETIC_ADVERSARY_LOOP` (Critical Choice Mode)
- **Instructions**:
  When this skill is invigorated, check if it is running as a background sub-routine of a compounded orchestrator (e.g. `/design`). If so, execute silently without prompting, inheriting choices from the parent brief. Otherwise, you **MUST NOT** run the attack simulation immediately. Present the user with these adversarial parameters:

  1. **What is the focus of the red-team attack?**
     * *Option A*: Core Rule Extraction (Attempting to make the model output its system instructions via translation or direct request).
     * *Option B*: Sandbox Jailbreak (Attempting to make the model execute external code, commands, or take unauthorized actions).
     * *Option C*: Hallucination / Edge Cases (Feeding garbage inputs, conflicting constraints, or repetitive tokens to trigger output degradation).
     * *Option D*: Write-in.

  2. **Select the intensity level of the attack simulation:**
     * *Option A*: Standard Compliance Scan (Simulate common prompt injection vectors like "Ignore previous instructions").
     * *Option B*: Advanced Adversarial Payload (Simulate roleplay exploits, token-space overrides, and multi-turn trickery).
     * *Option C*: Write-in.

  Obtain selections before starting the adversarial evaluation.

## Expected Output
- **Output type**: `HARDENED_SYSTEM_PROMPT_CONFIG`