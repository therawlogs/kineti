# SQL/PGQ Relational Property Graph: Level 1: Agent Level Causality Graph
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

## Edge Table (Relationships)
| Source Node | Relationship | Target Node | Properties |
|---|---|---|---|
| `AGENT:planner` | **HANDSOFF_SPEC** | `AGENT:architect` | `{}` |
| `AGENT:architect` | **DELEGATES_BUILD** | `AGENT:builder` | `{}` |
| `AGENT:builder` | **INVOKES_PREVIEW** | `TOOL:generate_image` | `{}` |
| `AGENT:builder` | **TRIGGERS_QA** | `AGENT:qa_verifier` | `{}` |
| `AGENT:qa_verifier` | **EXECUTES_TESTS** | `TOOL:run_command` | `{}` |