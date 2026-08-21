# Skill: /spec

## Purpose
Compile the complete public specification contract (`spec.md`) and enforce the mandatory human approval gate before code generation.

## Execution Rules
1. Combine audience rules, feature priorities (P1/P2/P3), visual tokens, modular infrastructure Lego blocks (Supabase, Resend, Vercel, or custom alternatives), and architecture schemas into a single comprehensive `<project_root>/spec.md`.
2. Ensure all API request/response payloads, database column constraints, and environment secrets are strictly typed.
3. **HARD PAUSE (Schema Gate):**

```
# ============================================================
SPECIFICATION GATE: Please review <project_root>/spec.md

Type 'yes' or 'build' to approve the spec and launch sub-agent code generation.
```

4. Do not proceed to `/build` without explicit confirmation. Update `state.json` upon approval.
