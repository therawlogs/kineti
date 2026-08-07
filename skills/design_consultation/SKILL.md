---
name: /design-consultation
description: Sets foundational design tokens and deterministic layout rules before interface coding using pure Plain English.
---

# Skill: /design-consultation (Stage 03 - Design Rules & Token System)

## When to Use
Use before frontend layout building.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow this sequence using pure Plain English:

  ### Step 1: Industry & Domain Visual Research
  1. **Domain Competitive Audit**: Conduct research into top-tier production applications for the user's specific industry domain (e.g., Stripe, Linear, Vercel, Apple, Bloomberg, Epic). Generic canned presets (dark mode, glassmorphism, analog cards) are STRICTLY PROHIBITED.
  2. **Bespoke Design Token Systems**: Formulate 3 domain-tailored visual design systems detailing typography pairings (e.g., Inter + JetBrains Mono, Outfit + Roboto), grid spacing (8px rhythm vs 4px micro-density), color palettes (tailored HSL tokens), and component border/shadow rules.
  3. **Render Screenshot Mockups**: Use `generate_image` to generate high-fidelity UI layout previews for each token system.

  ### Step 2: Present Options & HARD PAUSE
  Present the 3 bespoke design systems, rationale, and screenshot previews to the user, then **STOP EXECUTION IMMEDIATELY**:

  > [!IMPORTANT]
  > **HARD PAUSE DIRECTIVE**: You MUST present the 3 domain design token systems and screenshot previews, and STOP YOUR TURN IMMEDIATELY. You are STRICTLY FORBIDDEN from auto-selecting design choices or writing `index.css` before the user explicitly selects a design system.

  ### Step 3: Write Tokens to CSS (After User Selects System)
  Once the user selects their preferred design system, write the chosen design tokens directly to `index.css` and log the decision in `.northstar/decisions/design_tokens.md`.