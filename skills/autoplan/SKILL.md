---
name: autoplan
description: Architecture and technical planning gate.
---

# Architecture Planning Skill

This skill establishes the technical architecture and infrastructure choices for the system. It runs after the functional specification is approved.

## 1. Stage 2 System Boundaries
Present these questions to determine physical system limits and wait for answers:
- **Latency Target**: What is the maximum acceptable response time for core actions?
- **Peak Concurrency**: How many simultaneous users or requests must the system handle at peak?
- **Failover Plan**: What happens when the primary database or service goes down?

## 2. Stage 3 Experience
Present these questions to determine the client interface and wait for answers:
- **Primary Interaction Surface**: Web, mobile app, CLI, or API-only?
- **User Habit Benchmark**: Which existing application does the user expect this to feel like?
- **Information Density**: Does the UI require sparse, focused screens or high-density dashboards?
- **Non-Text Content**: How will images, video, or audio be handled?

Ask dynamic, domain-specific probes based on the answers provided.

## 3. Enterprise Domain Evaluations
For each of the following technical categories, present 3 researched options suitable for the specific domain, provide a rationale for each, and wait for the user to select an approach:

1. **Database & Persistence**: Evaluate relational (e.g., PostgreSQL + Prisma/Drizzle), document (e.g., MongoDB), vector (e.g., pgvector), and caching (e.g., Redis/Upstash) options with rationale for the specific domain.
2. **Multi-Tenancy & Data Scoping**: Evaluate Row-Level Security vs schema-per-tenant vs DB-per-tenant. Require `tenant_id` on all queries.
3. **Authentication & Access Control**: Evaluate SSO/SAML (e.g., WorkOS/Auth0), OAuth, session security (hashing, JWT rotation), and RBAC/ABAC access control.
4. **Data Pipelines & Async Processing**: Evaluate job queues (e.g., BullMQ/Temporal), ETL (e.g., Airbyte/dbt), and reliability patterns (backoff, Dead Letter Queues).
5. **Data Integrity**: Evaluate transaction patterns, idempotency keys for external calls, foreign keys, soft-delete, and audit logs.
6. **Secret Management**: Evaluate vaults (e.g., Infisical/HashiCorp/AWS Secrets Manager), boot validation preventing startup on missing env vars, and secret leak prevention in CI.
7. **API Contract Strategy**: Evaluate typed contracts (e.g., tRPC/Zod or OpenAPI) that the frontend must compile against.

## 4. Summary and Handoff
Do not automatically run sub-skills or start coding. End the interaction by presenting a summary of the selected architectural choices and listing the next available options.
