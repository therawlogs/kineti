# Skill: /qa

## Purpose
Execute automated headless browser multi-viewport verification and static code analysis with a 5-attempt self-healing loop.

## Execution Rules
1. Boot the local development server.
2. Launch Playwright headless browser tests across three viewports:
   - Desktop (1440x900)
   - Tablet (768x1024)
   - Mobile (375x667)
3. Save full-page verification screenshots to `<project_root>/design/screenshots/`.
4. Scan for console errors, layout shifts, broken links, and type mismatches.
5. **Self-Healing Loop:** If any test fails, feed the exact error trace back to the coding agent. Apply minimal targeted patches and re-test (maximum 5 attempts).
6. Run static security audit (`/cso`) to confirm zero hardcoded secrets or PII leakage patterns.
7. Print the final QA Status:

```
[✓] Frontend Components Verified
[✓] API & Database Connections Operational
[✓] 12/12 Viewport Tests Passed (Desktop / Tablet / Mobile)
[✓] Security & Spend Scans Clear ($0.00 overruns)
```
