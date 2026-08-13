---
name: qa
description: Testing and verification skill.
---

**Step 1: Start Dev Server**
Check if the development server is running on the expected port. If it is not running, start it.

**Step 2: Run Test Suite**
Execute the project's test suite, including all unit tests and integration tests. Read the output and report the results.

**Step 3: Multi-Viewport Browser Verification**
Use a browser automation tool to perform the following:
1. Load the application in a desktop viewport (1280px width).
2. Load the application in a mobile viewport (375px width).
3. Execute user click paths and check that the DOM state matches expected outcomes.
4. Capture and save screenshots at each viewport size.

**Step 4: Automated Fix Loop**
If any test fails during Step 2 or Step 3:
1. Read the exact error message and stack trace from the test output.
2. Write a code modification that addresses the root cause of the error.
3. Apply the fix to the relevant file.
4. Re-run the failing test.
5. Repeat this process up to 5 times. If the test still fails after 5 attempts, print a failure report and stop.

**Step 5: Report**
Print a summary of the test run, including the number of tests passed and failed, the viewports verified, and any automated fixes that were applied.
