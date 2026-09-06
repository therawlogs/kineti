with open('docs/HARNESS_STRATEGY_BLUEPRINT.md', 'r') as f:
    text = f.read()

import re
# look for the 20 entities in section 3.3 or appendix
# Let's extract lines around "Universal 20-Entity"
start = text.find("Universal 20-Entity Provenance Kernel")
print("Section start found at:", start)
snippet = text[start:start+4000]

entities = [
    "AgentSession", "Actor", "PersonaRole", "GoalTree", "Intent",
    "Hypothesis", "ActionRequest", "ActionExecution", "ToolInvocation",
    "EnvironmentObservation", "ArtifactSnapshot", "StateDelta", "GateEvaluation",
    "HumanIntervention", "VerificationEvidence", "CausalEdge", "RollbackRecord",
    "MetricSample", "AttestationProof", "OutcomeCommit"
]

print("Checking presence of 20 canonical entities:")
found = 0
for entity in entities:
    pos = text.find(entity)
    if pos != -1:
        found += 1
        print(f"  [x] {entity} (found at char {pos})")
    else:
        print(f"  [ ] {entity} (MISSING)")

print(f"Total entities verified: {found} / {len(entities)}")
