---
name: investigate
description: Emergency debugging skill.
---

**Step 1: Capture Error Context**
Read the system output to collect the exact error message, full stack trace, the file name and line number, and a list of recently modified files.

**Step 2: 5-Whys Root Cause Analysis**
Write out a sequence of 5 "why" questions based on the error chain to identify the fundamental root cause of the issue, rather than just the immediate symptom.

**Step 3: Isolate & Fix**
Based on the identified root cause:
1. If it is a logic error: trace the execution path in the code, identify the incorrect assumption, and modify the file to apply a targeted fix.
2. If it is a dependency error: check the package.json versions, read environment variables, and check for port conflicts using system commands.
3. If it is a data error: check database connection strings, look for constraint violations in the schema, and read the database migration state.

**Step 4: Verify Fix**
Run the failing command or test again. If it still fails, attempt another fix. After 3 failed attempts, run `git stash` or `git checkout` to rollback the files to the last working state, and report the attempted fixes.

**Step 5: Report**
Print a summary detailing the root cause that was found, the fix that was applied, and the result of the verification step.
