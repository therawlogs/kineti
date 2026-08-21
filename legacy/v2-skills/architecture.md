# Skill: /architecture

## Purpose
Design system boundaries, data flows, database schemas, and present selectable architecture dials.

## Execution Rules
1. Read `<project_root>/spec.md` to extract data contracts, feature priorities, visual tokens, and selected Lego blocks.
2. Generate a clean Markdown system map outlining:
   - Modular infrastructure mapping (Supabase Auth, Resend, Supabase PostgreSQL Database, Supabase User Metadata, Vercel hosting or chosen alternatives).
   - PostgreSQL schema models and relational keys.
   - Core REST/gRPC API route contracts.
   - Multi-tenancy isolation rules (RLS or dedicated schemas).
3. Present the 3 Architecture Tradeoff Dials:

```
Select your architectural priority dial:

1. Balanced Standard (Default cloud setup with edge routing and managed database)
2. Lean API / Zero Cost (Replaces paid APIs with Redis caching and local open-weight inference)
3. Enterprise Fortress (Enforces bidirectional PII redaction middleware and strict tenant isolation)
```

4. Save the chosen architecture profile and data schemas into `.northstar/decisions/architecture.md` and append to `spec.md`.
5. Prompt the operator:

```
Architecture plan generated. Type 'Approve' to lock contracts and generate specifications via /spec.
```
