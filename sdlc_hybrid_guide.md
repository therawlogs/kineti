# SDLC Hybrid Guide: Upgraded Plain English Edition (v7)

This guide explains the step-by-step process for planning, building, testing, and deploying software using **pure Plain English** and concrete physical steps integrated with **6 AI Frameworks & 5 Graph/AST Primitives**.

---

## 1. The 9-Step Process for Every Stage

Every stage in the building process follows these 9 simple steps:

```
1. Ask clear Plain English questions for the current stage.
2. Ask plain questions specific to the project's industry or domain (Pi.dev Socratic Memory).
3. Draw a clear map showing how data and steps connect (LlamaIndex Sub-Query Breakdown & SQL/PGQ Property Graph).
4. Update the combined map showing the full system picture.
5. Check if any needed information is missing.
6. Resolve missing information (user answers now, fills a simple template, or uses temporary mock data).
7. Save the conversation transcript to the local memory folder (.northstar/dialogue/).
8. Update the local memory index file (manifest.json).
9. Move to the next stage.
```

---

## 2. Local Memory Directory Structure (`.northstar/`)

All project records are saved locally on your computer. A `.gitignore` file prevents sending these records to public online code repositories:

*   `dialogue/`: Contains plain transcript records of all questions and answers from Stage 0 to Stage 7.
*   `gaps/`: Logs missing information and records how each gap was resolved.
*   `graphs/`: Contains clear visual maps showing how system parts connect (stored in SQL/PGQ relational property graph format).
*   `decisions/`: Stores approved choices and records rejected ideas with their failure reasons.
*   `templates/`: Stores blank data templates for missing information that needs filling.
*   `mocks/`: Stores temporary mock data tagged `[PROVISIONAL]` for testing.
*   `manifest.json`: Stores the complete list of all saved local memory files.
