---
name: design_consultation
description: A standalone design token system skill to establish or change the visual design system using the pre-fixed Master Design System Standard.
---

This is the standalone design token system skill. Use before any frontend layout building, or when the user wants to establish/change the visual design system independently.

**Step 1: Domain Competitive Audit & Archetype Mapping**
Apply the **Master Design System Standard** defined in `ETHOS.md` Part 4:
- Core Stack: `shadcn/ui` + `Radix UI` primitives + `Lucide Icons` (`lucide-react`) + `Motion` (`motion/react`).
- Perform a targeted web search for top-tier production applications in the user's specific domain.
- Map the project domain directly to one of the **3 Pre-Approved Master Visual Archetypes**:
  * **Archetype A (Modern Technical SaaS)**: Dark Zinc, Inter + JetBrains Mono, 4px micro-grid.
  * **Archetype B (High-Trust Enterprise & Fintech)**: Slate Pearl, Navy, Plus Jakarta Sans + Fira Code, 8px rhythm.
  * **Archetype C (Premium Editorial & Consumer)**: Warm Alabaster, Space Grotesk + Source Sans 3, spacious padding.

**Step 2: Formulate 3 Design Token Variations**
Using the chosen Archetype as the foundation, present 3 variations:
- Typography pairings (Google Fonts).
- Grid spacing (8px vs 4px rhythm).
- Color palettes (tailored HSL/OKLCH tokens).
- Component border, shadow, and radius rules.
- Responsive breakpoint strategy.

**Step 3: Render Mockups**
Execute the `generate_image` tool to create visual mockups for each of the 3 token systems.
- **Strict Asset Boundary Rule:** Save all generated mockup images directly inside the project repository under `<project_root>/design/tokens/` or `<project_root>/design/mockups/`. Never save design files to temporary system directories (`/tmp`), global application directories (`~/.gemini`), or scratch folders outside the active project workspace.

**Step 4: Present & Wait**
Show all 3 mockups to the user along with the research rationale. Pause execution and wait for the user to make a selection.

**Step 5: Write Tokens & Component Specs**
Write the chosen design tokens directly into `<project_root>/index.css`. Record the decision by writing it to `<project_root>/.northstar/decisions/design_tokens.md`.
