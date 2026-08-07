---
name: /design-html
description: Generate production-quality Pretext-native HTML/CSS layouts based on selected visual marks and styling libraries using Smolagents Code-First Actions in Plain English.
---

# Skill: /design-html (Stage 04 - Visual Theme & HTML Layout Compilation)

## When to Use
Use during the `/design` build gate to generate theme mockups and compile production HTML/CSS.

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** follow these steps using Smolagents Code-First Actions in pure Plain English:

  ### Step 1: Render Theme Mockup Previews (User Sovereignty Gate)
  1. Generate 3 visual screenshot mockup options using the `generate_image` tool:
     * **Option 1**: Sleek Cyber Dark Theme
     * **Option 2**: Glassmorphic Light Theme
     * **Option 3**: Retro Monochrome / Clean High-Contrast Theme
  2. Present mockups in chat and prompt the user to choose their preferred theme.
  3. **User Sovereignty Rule**: Do NOT compile production HTML code until the user approves their chosen visual mark.

  ### Step 2: Code-First HTML Compilation (Smolagents Pattern)
  1. Write direct, clean HTML/CSS code files (zero heavy JSON tool-call drag).
  2. Apply chosen theme tokens, typography, and 8px grid ratios.
  3. Inject `data-agent-action="[action_name]"` tags and unique IDs on all interactive buttons, forms, and data panels.
  4. Ensure sub-second Core Web Vitals targets ($LCP < 2.5s$, $INP < 200ms$).

  Upon completion, print this summary:
  ```
  === /design-html Complete ===
  - Theme screenshot previews rendered via generate_image (User Sovereignty verified).
  - Production Pretext-native HTML/CSS compiled on disk (Smolagents Code-First pattern).
  - Screen action tags (data-agent-action) and unique IDs injected.
  ```
