---
name: browse
description: A browser verification utility to visually verify rendered output.
---

This is the browser verification utility. Used by /qa and /design to visually verify rendered output.

**Step 1: Start Development Server**
Run a command in the terminal to start the project's development server (e.g., `npm run dev`).

**Step 2: Navigate to Target**
Open the target URL in a browser environment (default: `http://localhost:3000` or `http://localhost:5173`).

**Step 3: Capture & Verify**
Capture a screenshot of the rendered page and save the screenshot directly inside the project repository under `<project_root>/design/screenshots/` or `<project_root>/tests/screenshots/`. Compare the visual output against the design direction specified during the design phase. Check the following:
- Layout alignment matches the design specification.
- Typography renders correctly with the specified fonts.
- Color tokens are applied properly according to the HSL values.
- Responsive behavior works correctly at mobile (375px) and desktop (1280px) viewports.
- Interactive elements display proper hover and focus states.

**Step 4: Report**
Print the verification results as text output, clearly stating pass/fail for each of the checked items.
