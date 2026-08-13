---
name: review
description: Code quality audit skill.
---

**Step 1: Production-Grade Quality Checklist**
Read all recently modified files and check them against these criteria:
1. Type safety: Confirm all functions have explicit parameter and return types.
2. Error handling: Confirm all asynchronous operations use try/catch blocks with specific error types.
3. Modularity: Confirm business logic is separated from UI rendering and vendor-specific code.
4. No hardcoded values: Confirm configuration values are read from environment variables or configuration files.
5. Accessibility: Confirm interactive HTML elements have aria labels and use semantic HTML tags.
6. Unique IDs: Confirm all interactive HTML elements have unique, descriptive IDs for testing purposes.

**Step 2: Dependency Audit**
Check the imports in files that depend on the modified files to ensure no paths or names were broken by the changes.

**Step 3: Score & Report**
Calculate a code quality score from 0 to 100 based on the checks. Print specific feedback for each failing check, including the exact file path and line number where the issue was found.
