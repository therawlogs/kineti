# SQL/PGQ Relational Property Graph: Unified Cumulative Master Property Graph (Levels 1-4)
*Generated automatically by causality_graph_builder.py*

## Vertex Table (Nodes)
| Node ID | Type | Properties |
|---|---|---|
| `AGENT:planner` | `SubAgent` | `{"role": "Strategic Planner"}` |
| `AGENT:architect` | `SubAgent` | `{"role": "Technical Architect"}` |
| `AGENT:builder` | `SubAgent` | `{"role": "Code Builder"}` |
| `AGENT:qa_verifier` | `SubAgent` | `{"role": "QA Verifier"}` |
| `TOOL:generate_image` | `MCPTool` | `{"action": "Screenshot Theme Mockup"}` |
| `TOOL:run_command` | `MCPTool` | `{"action": "Shell Execution"}` |
| `LOOP:officehours_stage0_1` | `DiscoveryLoop` | `{"status": "COMPLETED"}` |
| `LOOP:autoplan_stage2_3` | `PlanningLoop` | `{"status": "COMPLETED"}` |
| `LOOP:grok_build_autonomic_patch` | `TestFixLoop` | `{"max_retries": 5}` |
| `LOOP:langgraph_saga_undo` | `RollbackStack` | `{"strategy": "LIFO"}` |
| `FILE:scripts/sync_kinetios.py` | `CodeFile` | `{"path": "scripts/sync_kinetios.py", "pagerank_score": 0.075}` |
| `FILE:scripts/causality_graph_builder.py` | `CodeFile` | `{"path": "scripts/causality_graph_builder.py", "pagerank_score": 0.075}` |
| `GOAL:business_objective` | `BusinessGoal` | `{"desc": "Customer Growth & >80% Margin"}` |
| `GATE:veto_holder_approval` | `HardStopGate` | `{"status": "PASSED"}` |
| `GATE:spend_circuit_breaker` | `CostGuardrail` | `{"ceiling_usd": 50.0}` |
| `GATE:pii_redaction_firewall` | `SecurityGuardrail` | `{"status": "ACTIVE"}` |

## Edge Table (Relationships)
| Source Node | Relationship | Target Node | Properties |
|---|---|---|---|
| `AGENT:planner` | **HANDSOFF_SPEC** | `AGENT:architect` | `{}` |
| `AGENT:architect` | **DELEGATES_BUILD** | `AGENT:builder` | `{}` |
| `AGENT:builder` | **INVOKES_PREVIEW** | `TOOL:generate_image` | `{}` |
| `AGENT:builder` | **TRIGGERS_QA** | `AGENT:qa_verifier` | `{}` |
| `AGENT:qa_verifier` | **EXECUTES_TESTS** | `TOOL:run_command` | `{}` |
| `LOOP:officehours_stage0_1` | **TRIGGERS_PLANNING** | `LOOP:autoplan_stage2_3` | `{}` |
| `LOOP:autoplan_stage2_3` | **ENFORCES_TEST_FIX** | `LOOP:grok_build_autonomic_patch` | `{}` |
| `LOOP:grok_build_autonomic_patch` | **FALLBACK_ON_BREACH** | `LOOP:langgraph_saga_undo` | `{}` |
| `GOAL:business_objective` | **REQUIRES_SIGN_OFF** | `GATE:veto_holder_approval` | `{}` |
| `GOAL:business_objective` | **BOUNDED_BY** | `GATE:spend_circuit_breaker` | `{}` |
| `GOAL:business_objective` | **PROTECTED_BY** | `GATE:pii_redaction_firewall` | `{}` |