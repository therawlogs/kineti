#!/usr/bin/env python3
"""
Causality Graph Builder (v1.0)
Generates 4-Level SQL/PGQ Relational Property Graphs for AI SDLC OS:
  Level 1: Agent Level Causality (Roles, Subagents, Tool Calls, Contracts)
  Level 2: Loop Level Causality (Autonomic Patch Loops, Retries, State Machine Undos)
  Level 3: Graph Level Causality (Code AST Imports, Function Calls, PageRank Scores)
  Level 4: Project Level Causality (Business Goals, 5-Whys, Cost Ceilings, Veto Gates)
  Master: Cumulative Causality Graph (Unified SQL/PGQ Property Graph)

Zero external dependencies - Uses Python 3 standard library.
"""

import ast
import json
import os
import re
import sys
from pathlib import Path

def parse_ast_dependencies(project_root):
    """Parses AST for Python/JS/TS files to extract dependencies and compute PageRank."""
    nodes = []
    edges = []
    file_map = {}
    adjacency = {}

    root = Path(project_root)
    code_extensions = {".py", ".js", ".ts", ".jsx", ".tsx", ".go"}
    
    # 1. Collect files
    ignore_dirs = {".git", "node_modules", "venv", ".venv", "__pycache__", ".northstar", "dist", "build"}
    for p in root.rglob("*"):
        if any(ignored in p.parts for ignored in ignore_dirs):
            continue
        if p.suffix in code_extensions and p.is_file():
            rel_path = str(p.relative_to(root))
            file_map[rel_path] = p
            nodes.append({"id": f"FILE:{rel_path}", "type": "CodeFile", "path": rel_path})
            adjacency[rel_path] = set()

    # 2. Parse Python AST Imports
    for rel_path, full_path in file_map.items():
        if full_path.suffix == ".py":
            try:
                content = full_path.read_text(encoding="utf-8", errors="ignore")
                tree = ast.parse(content, filename=str(full_path))
                for node in ast.walk(tree):
                    if isinstance(node, ast.Import):
                        for alias in node.names:
                            target_py = alias.name.replace(".", "/") + ".py"
                            if target_py in file_map:
                                edges.append({
                                    "source": f"FILE:{rel_path}",
                                    "target": f"FILE:{target_py}",
                                    "label": "IMPORTS"
                                })
                                adjacency[rel_path].add(target_py)
                    elif isinstance(node, ast.ImportFrom):
                        if node.module:
                            target_py = node.module.replace(".", "/") + ".py"
                            if target_py in file_map:
                                edges.append({
                                    "source": f"FILE:{rel_path}",
                                    "target": f"FILE:{target_py}",
                                    "label": "IMPORTS"
                                })
                                adjacency[rel_path].add(target_py)
            except Exception:
                pass
        elif full_path.suffix in {".js", ".ts", ".jsx", ".tsx"}:
            try:
                content = full_path.read_text(encoding="utf-8", errors="ignore")
                import_matches = re.findall(r'(?:import|from)\s+[\'"]([^\'"]+)[\'"]', content)
                for imp in import_matches:
                    for ext in ["", ".ts", ".js", ".tsx", ".jsx", "/index.ts", "/index.js"]:
                        candidate = str(Path(rel_path).parent / (imp + ext))
                        candidate = os.path.normpath(candidate)
                        if candidate in file_map:
                            edges.append({
                                "source": f"FILE:{rel_path}",
                                "target": f"FILE:{candidate}",
                                "label": "IMPORTS"
                            })
                            adjacency[rel_path].add(candidate)
                            break
            except Exception:
                pass

    # 3. Compute PageRank
    num_files = len(file_map)
    pagerank = {f: 1.0 / max(num_files, 1) for f in file_map}
    d = 0.85
    for _ in range(20):
        new_pagerank = {}
        for node in file_map:
            rank_sum = 0.0
            for other, neighbors in adjacency.items():
                if node in neighbors:
                    rank_sum += pagerank[other] / max(len(neighbors), 1)
            new_pagerank[node] = (1 - d) / max(num_files, 1) + d * rank_sum
        pagerank = new_pagerank

    # Add PageRank property to nodes
    for n in nodes:
        f_path = n["path"]
        n["pagerank_score"] = round(pagerank.get(f_path, 0.0), 4)

    return nodes, edges, pagerank

def build_sql_pgq_table(nodes, edges, graph_name):
    """Formats nodes and edges as SQL/PGQ Property Graph Tables."""
    lines = []
    lines.append(f"# SQL/PGQ Relational Property Graph: {graph_name}")
    lines.append(f"*Generated automatically by causality_graph_builder.py*\n")
    
    lines.append("## Vertex Table (Nodes)")
    lines.append("| Node ID | Type | Properties |")
    lines.append("|---|---|---|")
    for n in nodes:
        props = json.dumps({k: v for k, v in n.items() if k not in ("id", "type")})
        lines.append(f"| `{n['id']}` | `{n['type']}` | `{props}` |")
    
    lines.append("\n## Edge Table (Relationships)")
    lines.append("| Source Node | Relationship | Target Node | Properties |")
    lines.append("|---|---|---|---|")
    for e in edges:
        props = json.dumps({k: v for k, v in e.items() if k not in ("source", "target", "label")})
        lines.append(f"| `{e['source']}` | **{e['label']}** | `{e['target']}` | `{props}` |")
        
    return "\n".join(lines)

def main():
    project_root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    northstar_dir = Path(project_root) / ".northstar" / "graphs"
    northstar_dir.mkdir(parents=True, exist_ok=True)

    # Level 3: Graph Level Causality (Code AST & PageRank)
    ast_nodes, ast_edges, pagerank = parse_ast_dependencies(project_root)
    graph_sql = build_sql_pgq_table(ast_nodes, ast_edges, "Level 3: Graph Level Code Dependency Graph")
    (northstar_dir / "graph_causality.md").write_text(graph_sql, encoding="utf-8")

    # Level 1: Agent Level Causality
    agent_nodes = [
        {"id": "AGENT:planner", "type": "SubAgent", "role": "Strategic Planner"},
        {"id": "AGENT:architect", "type": "SubAgent", "role": "Technical Architect"},
        {"id": "AGENT:builder", "type": "SubAgent", "role": "Code Builder"},
        {"id": "AGENT:qa_verifier", "type": "SubAgent", "role": "QA Verifier"},
        {"id": "TOOL:generate_image", "type": "MCPTool", "action": "Screenshot Theme Mockup"},
        {"id": "TOOL:run_command", "type": "MCPTool", "action": "Shell Execution"},
    ]
    agent_edges = [
        {"source": "AGENT:planner", "target": "AGENT:architect", "label": "HANDSOFF_SPEC"},
        {"source": "AGENT:architect", "target": "AGENT:builder", "label": "DELEGATES_BUILD"},
        {"source": "AGENT:builder", "target": "TOOL:generate_image", "label": "INVOKES_PREVIEW"},
        {"source": "AGENT:builder", "target": "AGENT:qa_verifier", "label": "TRIGGERS_QA"},
        {"source": "AGENT:qa_verifier", "target": "TOOL:run_command", "label": "EXECUTES_TESTS"},
    ]
    agent_sql = build_sql_pgq_table(agent_nodes, agent_edges, "Level 1: Agent Level Causality Graph")
    (northstar_dir / "agent_causality.md").write_text(agent_sql, encoding="utf-8")

    # Level 2: Loop Level Causality
    loop_nodes = [
        {"id": "LOOP:officehours_stage0_1", "type": "DiscoveryLoop", "status": "COMPLETED"},
        {"id": "LOOP:autoplan_stage2_3", "type": "PlanningLoop", "status": "COMPLETED"},
        {"id": "LOOP:grok_build_autonomic_patch", "type": "TestFixLoop", "max_retries": 5},
        {"id": "LOOP:langgraph_saga_undo", "type": "RollbackStack", "strategy": "LIFO"},
    ]
    loop_edges = [
        {"source": "LOOP:officehours_stage0_1", "target": "LOOP:autoplan_stage2_3", "label": "TRIGGERS_PLANNING"},
        {"source": "LOOP:autoplan_stage2_3", "target": "LOOP:grok_build_autonomic_patch", "label": "ENFORCES_TEST_FIX"},
        {"source": "LOOP:grok_build_autonomic_patch", "target": "LOOP:langgraph_saga_undo", "label": "FALLBACK_ON_BREACH"},
    ]
    loop_sql = build_sql_pgq_table(loop_nodes, loop_edges, "Level 2: Loop Level Causality Graph")
    (northstar_dir / "loop_causality.md").write_text(loop_sql, encoding="utf-8")

    # Level 4: Project Level Causality
    proj_nodes = [
        {"id": "GOAL:business_objective", "type": "BusinessGoal", "desc": "Customer Growth & >80% Margin"},
        {"id": "GATE:veto_holder_approval", "type": "HardStopGate", "status": "PASSED"},
        {"id": "GATE:spend_circuit_breaker", "type": "CostGuardrail", "ceiling_usd": 50.0},
        {"id": "GATE:pii_redaction_firewall", "type": "SecurityGuardrail", "status": "ACTIVE"},
    ]
    proj_edges = [
        {"source": "GOAL:business_objective", "target": "GATE:veto_holder_approval", "label": "REQUIRES_SIGN_OFF"},
        {"source": "GOAL:business_objective", "target": "GATE:spend_circuit_breaker", "label": "BOUNDED_BY"},
        {"source": "GOAL:business_objective", "target": "GATE:pii_redaction_firewall", "label": "PROTECTED_BY"},
    ]
    proj_sql = build_sql_pgq_table(proj_nodes, proj_edges, "Level 4: Project Level Causality Graph")
    (northstar_dir / "project_causality.md").write_text(proj_sql, encoding="utf-8")

    # Cumulative Master Causality Graph (Unified Level 1 + 2 + 3 + 4)
    all_nodes = agent_nodes + loop_nodes + ast_nodes + proj_nodes
    all_edges = agent_edges + loop_edges + ast_edges + proj_edges
    cumulative_sql = build_sql_pgq_table(all_nodes, all_edges, "Unified Cumulative Master Property Graph (Levels 1-4)")
    (northstar_dir / "cumulative_causality.md").write_text(cumulative_sql, encoding="utf-8")

    keystone_file = max(pagerank.items(), key=lambda x: x[1])[0] if pagerank else "None"
    print(f"=== Causality Graph Builder Complete ===")
    print(f"Level 1 (Agent Causality): {len(agent_nodes)} nodes -> .northstar/graphs/agent_causality.md")
    print(f"Level 2 (Loop Causality):  {len(loop_nodes)} nodes -> .northstar/graphs/loop_causality.md")
    print(f"Level 3 (Graph Causality): {len(ast_nodes)} nodes -> .northstar/graphs/graph_causality.md")
    print(f"Level 4 (Project Causality): {len(proj_nodes)} nodes -> .northstar/graphs/project_causality.md")
    print(f"Unified Master Graph:       {len(all_nodes)} nodes -> .northstar/graphs/cumulative_causality.md")
    print(f"#1 Keystone Code File (PageRank): {keystone_file}")

if __name__ == "__main__":
    main()
