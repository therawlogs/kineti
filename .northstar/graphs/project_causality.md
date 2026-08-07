# SQL/PGQ Relational Property Graph: Level 4: Project Level Causality Graph
*Generated automatically by causality_graph_builder.py*

## Vertex Table (Nodes)
| Node ID | Type | Properties |
|---|---|---|
| `GOAL:business_objective` | `BusinessGoal` | `{"desc": "Customer Growth & >80% Margin"}` |
| `GATE:veto_holder_approval` | `HardStopGate` | `{"status": "PASSED"}` |
| `GATE:spend_circuit_breaker` | `CostGuardrail` | `{"ceiling_usd": 50.0}` |
| `GATE:pii_redaction_firewall` | `SecurityGuardrail` | `{"status": "ACTIVE"}` |

## Edge Table (Relationships)
| Source Node | Relationship | Target Node | Properties |
|---|---|---|---|
| `GOAL:business_objective` | **REQUIRES_SIGN_OFF** | `GATE:veto_holder_approval` | `{}` |
| `GOAL:business_objective` | **BOUNDED_BY** | `GATE:spend_circuit_breaker` | `{}` |
| `GOAL:business_objective` | **PROTECTED_BY** | `GATE:pii_redaction_firewall` | `{}` |