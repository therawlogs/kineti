# CLAUDE.md: Project Environment Directives

This file details the commands and paths required to build, test, and lint this project. Refer to this configuration to avoid execution ambiguity.

---

## 1. Primary Commands

*   **Build**: `[Insert build command, e.g., npm run build or make]`
*   **Test**: `[Insert test run command, e.g., npm test or pytest]`
*   **Lint & Format**: `[Insert lint command, e.g., eslint . or black .]`
*   **Dev Server**: `[Insert local run command, e.g., npm run dev]`

## 2. Structural Guidelines
*   Ensure all new code compiles without errors or warnings before requesting user review.
*   Ensure code passes formatting checks natively using the command specified above.

## 3. Sprint State Machine Validation
*   All agents and sub-routines **MUST** respect the state machine sequence tracked in `.sprint_state.json`.
*   Verify that preceding stages are completed and approved before attempting to execute downstream gates. The sequence is strictly `/officehours` ➔ `/autoplan` ➔ `/design` ➔ `/ship`.