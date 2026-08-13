---
name: cso
description: Security audit skill.
---

**Step 1: Spend Circuit Breaker Check**
1. Read the `kineti.config.json` file and verify API cost ceilings are set.
2. Check the source code to verify that auto-stop middleware is implemented for runaway API costs.

**Step 2: Private Data Protection**
1. Read the data flow paths in the code to check for personally identifiable information (names, emails, credit cards, health data).
2. Verify that the code strips PII before sending data to external APIs.
3. Verify that the code encrypts sensitive data before writing it to a database.

**Step 3: OWASP Top 10 Audit**
Read the source code to verify the following:
1. SQL injection: Ensure only parameterized queries are used for database operations.
2. Cross-site scripting (XSS): Ensure output encoding is applied to all user-rendered content.
3. Missing authorization: Ensure every API endpoint contains logic to check user permissions.
4. Unencrypted secrets: Ensure all API keys are loaded from environment variables and not hardcoded. Check that `.env` is listed in `.gitignore`.
5. CSRF: Ensure state-changing endpoints include CSRF protection.

**Step 4: Report**
Print a summary showing a pass/fail status for each security check, along with specific remediation steps for any failures.
