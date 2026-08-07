---
name: /cso
description: Chief Security Officer threat modeling gate. Runs OWASP Top 10, STRIDE, PII Egress Redaction, and Financial Circuit Breakers using pure Plain English.
---

# Skill: /cso (Stage 06 - Security Threat Audit & Spend Circuit Breakers)

## When to Use
Use during the build audit phase (`/design`).

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** execute these security checks using pure Plain English:

  ### Step 1: Meta AI Financial Spend Circuit Breaker
  1. Audit API key call paths and set automated daily/monthly cost ceilings.
  2. Inject auto-stop circuit breaker middleware: if API spending breaches cost cap, instantly freeze downstream API loops and alert project owner.

  ### Step 2: Two-Way Private Data Redaction (PII Scrubber)
  1. Audit data egress and ingress points.
  2. Inject automated regex and NLP redaction middleware to strip names, credit cards, passwords, and private health data before sending requests to external LLM APIs.

  ### Step 3: OWASP Top 10 & STRIDE Audit
  1. Check for SQL injection, cross-site scripting (XSS), missing authorization headers, and unencrypted secrets.
  2. Verify all API keys are loaded from environment variables (`.env`) and excluded from git repositories.

  Upon completion, print this summary:
  ```
  === /cso Complete ===
  - Meta AI Financial Spend Circuit Breakers set and standby active.
  - Two-way PII Redaction Middleware active (private data scrubbed before egress).
  - OWASP Top 10 & STRIDE threat audit passed cleanly (0 high-severity security bugs).
  ```