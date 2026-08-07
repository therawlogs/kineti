---
name: /agent_memory
description: Designs Persona B (Frontier AI) production agent memory architectures, including episodic memory, procedural memory, working memory, and cross-tenant privacy boundaries.
---

# Skill: /agent_memory (Frontier AI Production Memory Architecture)

## When to Use
Use during the planning phases (`/autoplan` or `/spec`) for Persona B projects to design how the agent remembers users and tasks at scale.

## Protocol & Actions
When this command is run, you **MUST** evaluate and design the agent's memory systems:
1. **Episodic Memory (User History)**: How the agent remembers past conversations, preferences, and facts about specific users across sessions (e.g., using Mem0 or Zep).
2. **Procedural Memory (Tool Mastery)**: How the agent learns and retrieves the correct workflow steps for complex tasks.
3. **Working Memory (Context Window Management)**: Summarization triggers, token pruning, and KV-cache optimization to keep context windows under the maximum limits.
4. **Memory Privacy Boundaries**: Hard schema constraints to ensure User A's memory vectors cannot be retrieved during User B's session.

Present the Memory Architecture blueprint to the user, and **HALT FOR USER CONFIRMATION**.
