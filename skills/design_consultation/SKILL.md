---
name: design_consultation
description: A standalone design token system skill to establish or change the visual design system independently.
---

This is the standalone design token system skill. Use before any frontend layout building, or when the user wants to establish/change the visual design system independently.

**Step 1: Domain Competitive Audit**
Perform a web search for top-tier production applications in the user's specific domain. Evaluate the research findings from 4 angles:
- User POV: Identify friction points, accessibility requirements, and cognitive load.
- Data density POV: Determine how much information per screen is necessary.
- Business POV: Note conversion mechanisms, premium feel indicators, and trust signals.
- Workflow velocity POV: Identify common keyboard shortcuts, bulk actions, and speed requirements.
Generic canned presets are STRICTLY PROHIBITED.

**Step 2: Formulate 3 Design Token Systems**
For each of the 3 token systems, specify the following:
- Typography pairings (using Google Fonts).
- Grid spacing (such as 8px rhythm vs 4px micro-density).
- Color palettes (using tailored HSL tokens).
- Component border, shadow, and radius rules.
- Responsive breakpoint strategy.

**Step 3: Render Mockups**
Execute the `generate_image` tool to create visual mockups for each of the 3 token systems.

**Step 4: Present & Wait**
Show all 3 mockups to the user along with the research rationale. Pause execution and wait for the user to make a selection.

**Step 5: Write Tokens**
Write the chosen design tokens directly into `index.css`. Record the decision by writing it to `.northstar/decisions/design_tokens.md`.
