# Skill: /build

## Purpose
Autonomously assemble production frontend components and backend logic using isolated sub-agents, OTD prompt envelopes, and Saga rollback safety.

## Execution Rules
1. Verify `spec.md` is approved in `state.json`.
2. Initialize the `CostAwareCircuitBreaker` with a $50.00 USD hard spending cap.
3. Spawn isolated sub-agents for each feature component. Wrap every prompt in an Ontology Trigger Data (OTD) envelope containing:
   - Immutable `root_goal`
   - Target component task
   - Authority Tier (Tier 1 Execution: blocked from modifying root configurations or cloud infrastructure)
4. Assemble UI components in `src/components/ui/` strictly using the 12 standard primitives.
5. Wire backend routes, database handlers, and API endpoints.
6. Register every file creation and mutation on a Saga LIFO rollback stack. If a sub-agent execution fails, pop and execute compensations in reverse order.
7. Once build completes, automatically trigger `/qa`.
