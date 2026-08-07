# Project ETHOS: Engineering Principles

This document defines the core coding constraints and product philosophy for this repository. All AI agents and developers **MUST** align with this ethos.

---

## 1. Zero Structural Drag
*   **Simple & Minimal**: Prioritize simple, readable code. Avoid writing complex architectural layers or abstractions unless explicitly requested.
*   **Low Dependency**: Use standard library primitives first. Do not add external package dependencies to solve simple problems.
*   **Single-File Preference**: When creating helper logic or utilities, prefer a single consolidated file over a multi-file hierarchy to prevent codebase clutter and token sprawl.

## 2. Eliminate "AI Slop"
*   **No Placeholders**: Never write code comments like `// TODO: Implement later` or `// Your logic here`. All code generated must be fully implemented, syntactically correct, and operational.
*   **No Redundant Documentation**: Do not write superficial docstrings or comments that merely repeat what the code does. Keep documentation dense and meaningful.

## 3. High Interaction Velocity
*   **Friction Elimination**: Design features to expose crucial inputs immediately. Strip unnecessary layers, popups, and clicks.
*   **Direct Aesthetics**: Keep UI layouts clean, using a structured token spacing grid (e.g. 8px multiples). Avoid messy styling hacks.

## 4. Testability & Safety
*   **Validate Boundaries**: Treat all input ingestion points as hostile. Write explicit, strict type validations.
*   **Checkpoints**: Always save codebase state (e.g. git WIP commit) before starting complex, multi-file code modifications.