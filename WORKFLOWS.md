# WORKFLOWS.md — Execution Guide

## The Greenfield Pipeline

```
/brainstorm ──> /design ──> /architecture ──> /spec ──> /build ──> /qa ──> /ship
(Intake)      (UI/UX)       (System)      (Contract)   (Code)    (Test)   (Launch)
```

### Step-by-Step Overview
1. **`/brainstorm`**: Takes raw idea via voice or notes, prompts for target audience (1. Internal, 2. B2C, 3. B2B), runs autonomous market research, validates Lego-block relevance (Supabase, Resend, Vercel), and buckets features into P1, P2, and P3.
2. **`/design`**: Selects 1 of the 3 Visual Archetypes, enforces the 12 UI components, and renders prototype screen previews to `design/screens/`.
3. **`/architecture`**: Maps selected Lego-block infrastructure, generates PostgreSQL data schemas and API contracts, and presents 3 selectable dials (1. Balanced, 2. Zero Cost, 3. Enterprise Fortress).
4. **`/spec`**: Generates typed API schemas, database contracts, and infrastructure specs in `<project_root>/spec.md`. **[HARD PAUSE: User Approval]**
5. **`/build`**: Spawns isolated sub-agents with immutable root goals and Saga LIFO rollback handlers to write code in `src/`.
6. **`/qa`**: Runs automated Playwright multi-viewport testing (Desktop, Tablet, Mobile) with a 5-attempt self-healing loop.
7. **`/ship`**: Prompts for environment API keys (Supabase, Resend), launches local interactive preview, and executes production deployment to Vercel. **[HARD PAUSE: User Deploy Confirmation]**
