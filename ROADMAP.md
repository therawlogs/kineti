# Project Roadmap

This document lists future planned improvements.

## Planned Updates

| Feature | Requirement | First Step |
|---|---|---|
| **Similarity Search** | Requires an embedding API key. Keyword search works today. | Configure an embedding provider in `gbrain`. |
| **Second Opinion Skill** | Requires a second command-line tool installed (such as `codex` or `gemini`). | Install a second CLI and test `kineti-second-opinion`. |
| **Gemini Integration Test** | Confirm tool detection in Gemini. | Run an interactive session in Gemini and verify memory commands. |

## Future Considerations

| Improvement | Purpose | Next Step |
|---|---|---|
| **Automated Output Quality Evaluation** | Evaluate generated responses against verified human examples. | Collect 50 example pairs in `.kineti/golden.jsonl` and create `bin/kineti-eval.ts`. |
| **Native Graph Storage** | Store cause-and-effect records directly in a graph database. | Add link scoring to `kineti-memory-job` before changing databases. |
| **Team Mode** | Allow multiple team members to share memory securely. | Add workspace permissions to memory storage. |

## Completed in Version 3

- Direct task entry (`bugfix`, `feature`, `refactor`, `audit`) and full project workflow
- Spend limit protection ($50 USD max)
- Step-by-step undo tracking
- Test verification with file hashes
- Web companion dashboard with plain English summaries
- Automated weekly integrity checks
