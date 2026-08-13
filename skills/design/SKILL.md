---
name: design
description: The build and visual quality gate that executes design research, visual prototyping, and application building using the pre-fixed Master Design System Standard.
---

This is the build and visual quality gate. Run after /autoplan. This is the MOST IMPORTANT skill for output quality.

This skill MUST do these things in order:

**Step 1: Load State & Architecture**
Read `.northstar/sprint_state.json` to verify it exists and that the `last_completed_gate` field equals `autoplan`.

**Step 2: Pre-Fixed Design System & Domain Visual Audit (Chief Product Officer persona)**
Apply the **Master Design System Standard** defined in `ETHOS.md` Part 4:
- Core Stack: `shadcn/ui` + `Radix UI` primitives + `Lucide Icons` (`lucide-react`) + `Motion` (`motion/react`).
- Perform a targeted web search for top-tier production applications in the user's specific domain (e.g., Stripe/Mercury for fintech, Linear/Vercel for dev tools).
- Map the project domain directly to one of the **3 Pre-Approved Master Visual Archetypes**:
  * **Archetype A (Modern Technical SaaS)**: Dark Zinc (`hsl(240 10% 3.9%)`), Inter + JetBrains Mono, 4px micro-grid.
  * **Archetype B (High-Trust Enterprise & Fintech)**: Slate Pearl (`hsl(210 20% 98%)`), Navy, Plus Jakarta Sans + Fira Code, 8px rhythm.
  * **Archetype C (Premium Editorial & Consumer)**: Warm Alabaster (`hsl(40 20% 97%)`), Space Grotesk + Source Sans 3, spacious padding.

**Step 3: Component Library Standard**
Use the pre-fixed `shadcn/ui` + `Radix UI` component foundation. Do NOT spend time configuring unproven third-party libraries.

**Step 4: Formulate 3 Tailored Visual Directions**
Using the chosen Archetype as the foundation, present 3 subtle variations tailored to the project:
- Typography pairings (Google Fonts).
- HSL/OKLCH color token scales (`--background`, `--foreground`, `--primary`, `--muted`, `--accent`, `--border`).
- Spatial density rhythm (4px or 8px grid).
- Micro-interaction transitions (spring physics `cubic-bezier(0.16, 1, 0.3, 1)`).

**Step 5: Render Screenshot Previews (Entire Application / Screens)**
Execute the `generate_image` tool to create high-fidelity UI previews for all requested application screens or visual directions.
- **Strict Asset Boundary Rule:** ALL generated screen images, previews, mockups, and wireframes MUST be saved directly into the project repository under `<project_root>/design/screens/` or `<project_root>/design/mockups/`. Never write design artifacts to temporary system directories (`/tmp`), global application directories (`~/.gemini`), or scratch folders outside the active project workspace.
Present these image files to the user alongside the research rationale for each.

**Step 6: Repeat-and-Rinse Component Build**
Wait for the user to select a direction. Once selected (or if a logical default choice is made):
1. Write the selected design tokens directly into `<project_root>/index.css`.
2. Build UI components directly inside `<project_root>/src/components/ui/` or `<project_root>/src/components/` leveraging the **12 Repeat-and-Rinse Components** (Header & Command Bar, Hero Banner, Card, Data Table, Form Controls, Modal, Sheet, Toasts, Tabs, Status Badges, Skeleton Loaders, Buttons).
3. Run `/review` to verify code quality.
4. Run `/qa` to test the UI across multiple viewports.
5. Run `/cso` to perform security checks.

**Step 7: Save State**
Update `.northstar/sprint_state.json` to reflect the completed build state. Write the chosen design decisions into `.northstar/decisions/design_tokens.md`.
End with a checkpoint summary that lists what was built and presents the next options (proceed to /ship or refine).
