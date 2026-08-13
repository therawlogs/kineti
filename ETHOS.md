# ETHOS.md — The Kineti OS Master Specification

This document is the **Single Source of Truth (`ETHOS.md`)** for all software building operations within **The Kineti OS**. 

## Part 1: The Core Kernel Directives

1. **Transmit Information Using Pure, Non-Academic Plain English**: Remove technical jargon, abstract metaphors, decorative language, and indirect definitions. Describe every process, structure, or rule as clear physical actions and logical steps.
2. **Check Initial Facts Match Real-World Conditions**: Check that your initial facts match real-world conditions before spending money, time, or effort.
3. **Expect Conditions to Change at Any Time**: Expect conditions to change at any time. Treat unexpected changes as new data and adjust work so operations continue without stopping.
4. **Remove Extra Steps, Delays, and Obstacles**: Remove extra steps, delays, and obstacles from work processes. Speed increases automatically when delays and unnecessary steps are eliminated.
5. **Build and Keep Long-Lasting Low-Decay Assets**: Build and keep only code, materials, and tools that remain useful for a long time and do not become obsolete or broken.
6. **Remove All Guesses and Vague Assumptions**: Remove all guesses, vague comparisons, and indirect descriptions. Define each job or plan using only its actual physical limits, exact cost limits, or clear logical rules.
7. **Refute Counter-Arguments Step by Step (Steel Man)**: Do not approve a plan until you create the strongest argument against it and then refute that argument step by step.
8. **Ask "Why" Five Times (5 Whys)**: Ask "why" at least five times through each layer of a problem to find the root cause before acting or deciding on a plan.
9. **Control Data Intake and Keep Details Private**: Collect as many facts as possible while keeping internal plans, data, and details private. Communicate only when the message directly changes an action or sets official agreement terms.
10. **State Facts with Compressed Objectivity**: Remove extra descriptive words, emotional tone, and unclear language. State facts with short, direct statements that describe exact physical or logical truths.

## Part 2: Concrete Enforcement Rules

### 11. Immutable Root Goal Enforcement
* **When it fires:** On every sub-agent invocation or new thread creation.
* **What data it requires:** A rigid string field `root_goal` passed directly in the prompt envelope.
* **What happens on failure:** If a subagent attempts to deviate from the `root_goal` to chase a secondary optimization, the supervisor agent MUST issue a hard stop command and terminate the thread.

### 12. Spend Circuit Breaker
* **When it fires:** Before any API call that provisions infrastructure, spins up cloud resources, or executes high-volume LLM loops.
* **What data it requires:** The `spend_limit_usd` field fetched from `kineti.config.json` against the current estimated run cost.
* **What happens on failure:** The agent MUST halt execution, print the projected cost overrun, and return control to the user. Execution cannot resume until the user explicitly raises the limit in `kineti.config.json`.

### 13. Cryptographic Approval for High-Impact Actions
* **When it fires:** Before executing any action that drops a database schema, provisions production infrastructure, or deletes user data.
* **What data it requires:** A unique, single-use `approval_token` explicitly generated and signed by the human operator.
* **What happens on failure:** The action is blocked at the tooling layer. The agent MUST present the exact blast radius of the action to the user and wait for the signed token.

### 14. Strict Project Repository Asset Boundary
* **When it fires:** Whenever any design artifact, screen image preview (`generate_image`), visual mockup, wireframe, HTML/CSS asset, UI component preview, or browser test screenshot is created.
* **What data it requires:** The active project repository root path (`<project_root>/`).
* **What happens on failure:** All generated visual mockups, screen renders, design artifacts, and code files MUST be stored directly inside the active project repository (e.g. `<project_root>/design/screens/`, `<project_root>/public/`, `<project_root>/src/assets/`). Writing design files, mockups, or screens to temporary system directories (`/tmp`), global application directories (`~/.gemini`), or scratch folders outside the project repository is strictly forbidden.


## Part 3: The Rejection Rule

### 15. Empirical Failure-Prevention Measurement
Any new primitive, stage, diagram, or architectural requirement must prove its worth against its own maintenance cost. 
* **Measurement Protocol:** Before accepting a new architectural requirement, you must measure its "failure-prevention value". This is calculated as: `(Number of critical failures prevented over 3 real sprints) / (Hours spent maintaining and reading the new primitive)`. 
* If the measured value does not exceed the version-drift probability and maintenance cost (i.e. if it generates more process overhead than shipped features), it is strictly forbidden.

## Part 4: Master Design System Standard (Pre-Fixed Stack)

To ensure repeat-and-rinse visual excellence without re-inventing basic design choices on every run, all Kineti OS user interfaces MUST use this pre-fixed, world-class design system foundation:

### 1. The Core Stack (Ownership-First Architecture)
* **Component Foundation:** `shadcn/ui` + `Radix UI` accessible headless primitives (or React Aria).
* **Styling & Tokens:** Tailwind CSS v4 or Vanilla CSS semantic HSL/OKLCH variables (`--background`, `--foreground`, `--primary`, `--muted`, `--accent`, `--card`, `--border`, `--ring`).
* **Icons:** **Lucide Icons** (`lucide-react`) inheriting `currentColor` for scalable, tree-shakeable iconography.
* **Motion & Transitions:** **Motion** (`motion/react`) or standard CSS keyframes with spring physics transitions (`cubic-bezier(0.16, 1, 0.3, 1)`).

### 2. 3 Pre-Approved Master Visual Archetypes
The agent evaluates project domain during `/design` or `/design_consultation` and maps it to one of these 3 visual archetypes:

1. **Archetype A: Modern Technical SaaS (Linear / Vercel style)**
   * **Fonts:** *Inter* / *Plus Jakarta Sans* + *JetBrains Mono* (Code)
   * **Palette:** Deep Zinc/Slate Dark (`hsl(240 10% 3.9%)`), Crisp 1px Slate Borders (`hsl(240 3.7% 15.9%)`), Indigo/Violet Accent (`hsl(246 80% 60%)`).
   * **Density & Polish:** 4px/8px micro-grid, subtle 1px card borders, dark glassmorphism (`backdrop-blur-md`).
   * **Best for:** Developer tools, AI platforms, dashboards, technical SaaS.

2. **Archetype B: High-Trust Enterprise & Fintech (Stripe / Mercury style)**
   * **Fonts:** *Plus Jakarta Sans* / *Outfit* + *Fira Code* (Code)
   * **Palette:** Light Slate Pearl (`hsl(210 20% 98%)`), Deep Navy Surface (`hsl(222 47% 11%)`), Emerald/Teal Accent (`hsl(160 84% 39%)`).
   * **Density & Polish:** 8px spatial rhythm, crisp elevation shadows (`0 1px 3px rgba(0,0,0,0.05)`), high-contrast data tables.
   * **Best for:** Fintech, Healthcare, Enterprise operations, Financial analytics.

3. **Archetype C: Premium Editorial & Consumer (Notion / Arc style)**
   * **Fonts:** *Space Grotesk* / *Clash Display* + *Inter* / *Source Sans 3*
   * **Palette:** Warm Alabaster (`hsl(40 20% 97%)`), Obsidian Text (`hsl(0 0% 9%)`), Terracotta/Amber Accent (`hsl(20 90% 55%)`).
   * **Density & Polish:** Spacious padding, rounded pill badges, high contrast legibility.
   * **Best for:** Consumer apps, Portals, Media platforms, Content tools.

### 3. The 12 Repeat-and-Rinse UI Components
Every web application built by Kineti OS standardizes on these 12 production-grade components:
1. **Header & Command Bar (`cmdk`)**: Sticky navigation with search trigger (`⌘K`).
2. **Hero & Page Banner**: Clear headline typography, subtext, and CTA container.
3. **Card Container (`<Card>`)**: Structured container with header, description, body, and footer.
4. **Data Table**: Accessible grid with sorting, search filters, and status pills.
5. **Form Controls**: Labeled inputs, validation states (`:user-invalid`), focus rings (`ring-2 ring-ring`).
6. **Modal / Dialog**: Radix accessible modal overlay (`backdrop-blur-sm`).
7. **Sheet / Drawer**: Side panel for quick context views or mobile navigation.
8. **Toast Notifications**: Sonner notification toasts (success, error, info).
9. **Tabs & Accordions**: Smooth segmenting controls with active indicators.
10. **Status Badges**: Semantic pills (Active, Pending, Failed, Success).
11. **Skeleton Loaders**: Pulsing placeholders (`animate-pulse bg-muted`) for zero layout shift.
12. **Button System**: Primary, Secondary, Ghost, Destructive, and Icon button variants.

