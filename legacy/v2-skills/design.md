# Skill: /design

## Purpose
Establish visual direction, generate screen prototype previews, and lock frontend design tokens to the Master Design System Standard.

## Execution Rules
1. Verify that `<project_root>/spec.md` contains approved audience and P1 feature priorities.
2. Present the 3 Master Visual Archetypes as numbered options:

```
Select your preferred visual taste direction:

1. Option A: Dark Modern Tech (Linear/Vercel style — Dark Zinc, sharp typography, dark glassmorphism)
2. Option B: Clean High-Trust (Stripe/Mercury style — Light Slate Pearl, structured data tables, deep navy)
3. Option C: Premium Editorial (Notion/Arc style — Warm Alabaster canvas, obsidian text, spacious cards)
```

3. Upon selection, render high-fidelity screen preview layouts directly to `<project_root>/design/screens/`.
4. Ensure every screen layout is composed exclusively of the 12 standard primitives in `src/components/ui/`.
5. Append the selected color palette, typography tokens, and screen layout definitions to `spec.md`.
6. Prompt the operator:

```
Visual previews generated in design/screens/.
Type 'Approve' to proceed to /architecture.
```
