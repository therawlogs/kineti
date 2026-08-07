---
name: /agent-orchestration
description: Multi-Agent Orchestration Pattern design for production agent products. Evaluates supervisor-worker hierarchies, peer-to-peer collaboration, swarm execution, handoff protocols, and fault isolation.
---

# Skill: /agent-orchestration (Tier 5 — Frontier AI Multi-Agent Orchestration)

## When to Use
Use when the project is classified as **Persona B (Frontier AI Research Founder)** or **A+B Hybrid**, and the agent product involves multiple collaborating agents.

## How to Use
Enter `/agent-orchestration` in the active workspace.

## Sequencing
- **Phase**: `02_technical_planning` (runs alongside `/autoplan`)
- **Pre-requisite**: Persona B or A+B Hybrid detected in `/officehours`.
- **Downstream Blockers**: Blocks `/design` and `/ship` until orchestration architecture is locked.

## Protocol & Actions
- **Instructions**:
  When this command is run, you **MUST** follow this sequence:

  ### Step 1: Assert Persona & Load Context
  1. Verify `detected_persona` includes `B` or `A+B`.
  2. Load agent product description and system architecture from `.northstar/`.

  ### Step 2: Orchestration Architecture Research (CTO Persona)
  Adopt the **CTO & Principal Architect** persona. Research multi-agent orchestration patterns for the project's domain.

  1. **Supervisor → Worker Hierarchies**: Evaluate manager agent designs that decompose complex tasks and delegate to specialist agents (e.g., routing agent → research agent → drafting agent → review agent). Research frameworks (LangGraph, CrewAI, AutoGen, OpenAI Swarm).
  2. **Peer-to-Peer Agent Collaboration**: Evaluate designs where agents operate as equals with shared memory, collaborating on tasks without a central supervisor. Suitable for brainstorming, multi-perspective analysis.
  3. **Swarm Execution for Parallel Tasks**: Evaluate embarrassingly parallel patterns where identical agent instances process independent items concurrently (e.g., 100 documents reviewed simultaneously).
  4. **Agent Handoff Protocols**: Design context preservation during agent-to-agent handoffs. When Agent A routes to Agent B, what context transfers? Evaluate structured handoff schemas vs. full conversation forwarding.
  5. **Fault Isolation & Circuit Breakers**: Design failure boundaries so one agent's crash doesn't cascade to the entire orchestration. Evaluate retry policies, Dead Letter Queues for failed agent tasks, and graceful degradation (if specialist agent fails, supervisor handles directly).
  6. **Cost Routing & Model Selection Per Agent**: Different agents in the orchestration may use different models. Route simple classification agents to cheap/fast models, route complex reasoning agents to expensive/capable models.

  ### Step 3: Present Orchestration Architecture & HARD PAUSE
  Present the orchestration pattern recommendation, framework options, fault isolation strategy, and cost routing plan to the user, then **STOP EXECUTION IMMEDIATELY**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the orchestration architecture, framework options, and fault isolation strategy to the user, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from implementing orchestration code before the user explicitly approves.

  ### Step 4: Implement Approved Orchestration Architecture (After User Approves)
  Once approved, generate orchestration scaffolding, agent routing logic, handoff schemas, and circuit breaker middleware. Log decisions to `.northstar/decisions/agent_orchestration.md`.
