---
name: design
description: The build and visual quality gate that executes design research, visual prototyping, and application building.
---

This is the build and visual quality gate. Run after /autoplan. This is the MOST IMPORTANT skill for output quality.

This skill MUST do these things in order:

**Step 1: Load State**
Read `.northstar/sprint_state.json` to verify it exists and that the `last_completed_gate` field equals `autoplan`.

**Step 2: Domain Visual Research (Chief Product Officer persona)**
Perform a web search for top-tier production applications in the user's specific domain (e.g., Stripe/Mercury/Wise for fintech, Linear/Vercel/Raycast for dev tools).
Evaluate the research findings from 4 angles:
- User POV: Identify friction points, accessibility requirements, and cognitive load.
- Data density POV: Determine how much information per screen is necessary.
- Business POV: Note conversion mechanisms, premium feel indicators, and trust signals.
- Workflow velocity POV: Identify common keyboard shortcuts, bulk actions, and speed requirements.
Generic canned presets (like standard dark mode, glassmorphic, or retro monochrome) are STRICTLY PROHIBITED.

**Step 3: Component Library Selection**
Research and recommend a component library (e.g., Radix/Shadcn UI, Mantine, Ant Design, Headless UI) based on the domain-specific research findings.

**Step 4: Formulate 3 Bespoke Visual Directions**
For each of the 3 directions, specify the following details:
- Typography pairing (using Google Fonts: e.g., Inter + JetBrains Mono).
- Color palette (using HSL tokens for primary, secondary, accent, surface, error, and success).
- Spatial density (choose between an 8px or 4px grid rhythm and define the component spacing scale).
- Border, shadow, and radius rules.
- Micro-interaction patterns for hover states, transitions, and loading states.

**Step 5: Render Screenshot Previews**
Execute the `generate_image` tool to create high-fidelity UI previews for all 3 visual directions (or for all requested application screens).
- **Strict Asset Boundary Rule:** ALL generated screen images, previews, mockups, and wireframes MUST be saved directly into the project repository under `<project_root>/design/screens/` or `<project_root>/design/mockups/`. Never write design artifacts to temporary system directories (`/tmp`), global application directories (`~/.gemini`), or scratch folders outside the active project workspace.
Present these image files to the user alongside the research rationale for each.

**Step 6: Build**
Wait for the user to select a direction. Once selected (or if a logical default choice is made):
1. Write the selected design tokens directly into `<project_root>/index.css`.
2. Write code to build production-grade UI components directly inside the project repository (e.g., `<project_root>/src/`, `<project_root>/public/`).
3. Run `/review` to verify code quality.
4. Run `/qa` to test the UI across multiple viewports.
5. Run `/cso` to perform security checks.

**Step 7: Save State**
Update `.northstar/sprint_state.json` to reflect the completed build state. Write the chosen design decisions into `.northstar/decisions/design_tokens.md`.
End with a checkpoint summary that lists what was built and presents the next options (proceed to /ship or refine).
