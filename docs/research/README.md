# Research notes

Research notes, not part of the CLI. The items below are design claims and proposals, not guarantees provided by Kineti v0.4.0. Check the root README and release notes for supported behavior.

## The 360º Human Model & Epistemic Engine

Personalized agents must understand the full human context without hallucinating, overriding user statements, or quietly mutating goals when third-party systems push back:

- **Multi-Scope Context Isolation**: Strict context isolation across `Global`, `Domain` (Health, Work, Finance, Schedule, Taste), and `Relationship` (person-to-person) scopes. Scoped facts never leak into generic or un-scoped queries.
- **Epistemic Certainty Tiers**: Enforces `DirectlyKnown` (explicit user ground truth) > `ObservedPattern` (behavioral patterns) > `Inferred` (hypotheses). Machine inferences are strictly blocked from overwriting explicit user statements.
- **Rule-Exception Hierarchies**: Resolves complex user lifestyles unambiguously: `BaselineRule` (e.g. Vegetarian) $\to$ `PermittedException` (e.g. Eats eggs) $\to$ `Preference` (e.g. Prefers low dairy) $\to$ `SafetyCeiling` (e.g. Peanut allergy).
- **Verbatim Root Goal & Anti-Drift Engine**: Anchors autonomous task chains to the exact, unmodified words uttered by the user and their explicit definition of "Done". Every step is inspected against the original ask, eliminating the multi-step "telephone game". Intermediate steps and tools are treated as expendable scaffolding.
- **Friction Triage Ladder ("Clean No over Dirty Yes")**: Triages real-world obstacles through 3 levels:
  1. *Noise*: Transient blips auto-retry with exponential backoff.
  2. *Broken Surface*: Broken websites or portals silently reroute to alternatives.
  3. *Real Constraint*: Hard third-party refusals escalate immediately with a clean impossibility report. Sunk costs are written off ($0 sunk-cost fallacy), and quiet compromises (such as accepting budget overruns) are strictly blocked.
- **Commitment-Time Verification**: Re-checks perishable facts (fares, seat availability, stock levels, auth tokens) at the exact millisecond of external or financial commit, never trusting cached plan snapshots.
- **Asymmetric Gap-Filling**: Cheap, reversible gaps are filled automatically and disclosed in audit evidence; expensive or irreversible gaps halt execution to ask the user.
- **Reputation Gating & Ingress Defense**: Outbound communication is treated as a non-regenerating resource (knowing an identity does not equal permission to contact). Incoming external messages and webhooks arrive as untrusted data, never instructions.
