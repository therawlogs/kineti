# SQL/PGQ Relational Property Graph: Level 2: Loop Level Causality Graph
*Generated automatically by causality_graph_builder.py*

## Vertex Table (Nodes)
| Node ID | Type | Properties |
|---|---|---|
| `LOOP:officehours_stage0_1` | `DiscoveryLoop` | `{"status": "COMPLETED"}` |
| `LOOP:autoplan_stage2_3` | `PlanningLoop` | `{"status": "COMPLETED"}` |
| `LOOP:grok_build_autonomic_patch` | `TestFixLoop` | `{"max_retries": 5}` |
| `LOOP:langgraph_saga_undo` | `RollbackStack` | `{"strategy": "LIFO"}` |

## Edge Table (Relationships)
| Source Node | Relationship | Target Node | Properties |
|---|---|---|---|
| `LOOP:officehours_stage0_1` | **TRIGGERS_PLANNING** | `LOOP:autoplan_stage2_3` | `{}` |
| `LOOP:autoplan_stage2_3` | **ENFORCES_TEST_FIX** | `LOOP:grok_build_autonomic_patch` | `{}` |
| `LOOP:grok_build_autonomic_patch` | **FALLBACK_ON_BREACH** | `LOOP:langgraph_saga_undo` | `{}` |