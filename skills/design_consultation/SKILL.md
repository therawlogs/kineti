---
name: /design-consultation
description: Sets foundational design tokens and deterministic layout rules before interface coding using pure Plain English.
---

# Skill: /design-consultation (Stage 03 — CPO Domain Visual Research & Token System)

## When to Use
Use before frontend layout building.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow this sequence using pure Plain English:

  ### Step 1: CPO Industry & Domain Visual Research
  Adopt the **Chief Product Officer (CPO) & Head of UX Design** persona.
  1. **Deep Domain Competitive Audit**: Conduct live web research into top-tier production applications for the user's specific industry domain (e.g., Stripe, Linear, Vercel, Apple, Bloomberg, Epic). Evaluate from user POV (friction, accessibility), data density POV (information density requirements), business POV (conversion, premium quality), and workflow velocity POV (keyboard shortcuts, bulk actions). Generic canned presets (dark mode, glassmorphism, analog cards) are STRICTLY PROHIBITED.
  2. **Component Library Selection**: Research and recommend component libraries (Radix/Shadcn UI, Mantine, Ant Design) with domain-specific rationale.
  3. **Bespoke Design Token Systems**: Formulate 3 domain-tailored visual design systems detailing typography pairings (e.g., Inter + JetBrains Mono, Outfit + Roboto), grid spacing (8px rhythm vs 4px micro-density), color palettes (tailored HSL tokens), and component border/shadow rules.
  4. **Render Screenshot Mockups**: Use `generate_image` to generate high-fidelity UI layout previews for each token system.

  ### Step 2: Present Options
  Present the 3 bespoke design systems, rationale, and screenshot previews to the user, make a logical default choice if needed, and proceed to the next step.

  ### Step 3: Write Tokens to CSS
  Write the chosen design tokens directly to `index.css` and log the decision in `.northstar/decisions/design_tokens.md`.