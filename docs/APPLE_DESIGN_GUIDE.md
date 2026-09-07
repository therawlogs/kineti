# Apple Design Standards & Material Library Guide

This guide explains how Kineti uses Apple's Human Interface Guidelines (HIG), material library, and design kits across its web interfaces and companion dashboard.

## 1. Core Principles

1. **Clarity**: Text uses plain words, buttons are obvious, and contrast is sharp.
2. **Deference**: Tasks, goals, and test proofs are the primary focus. Shadows, blurs, and borders support the content without distraction.
3. **Native Feel**: Clean single-viewport web presentation designed to feel native in Safari and modern browsers, without fake window frames or mock traffic lights.
4. **Depth**: Translucent glass layers show hierarchy. Pinned navigation bars and slide-out sheet panels float above the content canvas.

## 2. Apple Materials Library

Apple materials use backdrop blur, saturation, and subtle specular borders:

| Material | Blur / Tint | Purpose |
|---|---|---|
| **Ultra Thin** | `blur(20px)`, 5% surface tint | Subtle overlays and headers |
| **Thin** | `blur(30px)`, 55% surface tint | Standard cards and content surfaces |
| **Regular** | `blur(40px)`, 78% surface tint | Pinned navigation bar and toolbars |
| **Thick** | `blur(50px)`, 88% surface tint | System dialogs, modals, and alerts |
| **Sheet** | `blur(40px)`, 95% surface tint | Slide-out settings drawer panel |

All cards and panels include a top hairline highlight:
```css
box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);
```

## 3. Apple System Colors

Kineti uses Apple's standard system palette:

- **System Blue** (`#0A84FF`): Primary actions, active links, focus rings.
- **System Green** (`#30D158`): Safe status, passing tests, active toggles.
- **System Orange** (`#FF9F0A`): Attention needed, pending approvals.
- **System Red** (`#FF453A`): Spending limits reached, failed checks.
- **System Purple** (`#BF5AF2`): Tasks and goals.
- **System Teal** (`#64D2FF`): Secondary status and undo readiness.
- **System Gray** (`#8E8E93`): Supporting labels and metadata.

## 4. Typography (SF Pro & SF Mono)

- **Main Font**: `-apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", system-ui, sans-serif`
- **Code & Number Font**: `"SF Mono", Menlo, Monaco, Consolas, monospace`
- **Tracking**: Tight letter-spacing for large titles (`-0.02em`) and relaxed spacing for captions (`+0.01em`).

## 5. Controls & Design Kits

1. **Pinned Apple Navigation Bar**: Sticky frosted navigation bar (`height: 52px`) with blur and hairline divider.
2. **Repository Switcher**: Frosted capsule dropdown with instant search and checkmarks.
3. **Segmented Controls**: Recessed track with a floating capsule thumb for switching views (`Dashboard`, `Fleet View`, `Logs & Proofs`).
4. **Slide-Out Sheet Drawer**: Smooth right-aligned drawer with frosted backdrop for settings and integrations.
5. **Switches / Toggles**: Smooth capsule switch that turns System Green (`#30D158`) when active.
6. **Pill Buttons**: Rounded buttons with subtle press scaling (`transform: scale(0.97)`).
7. **Continuous Radii**: Smooth squircle corner curvature (10px, 14px, 18px, 22px, and 26px scales).

## 6. How to Use in Code

Import design tokens directly:

```ts
import { AppleColors, AppleMaterials, AppleRadii } from "./src/design/apple-design-tokens";
```

Or use the pre-built UI components:

```tsx
import { Button, Card, Tabs, FormToggle } from "./src/components/ui";

// Apple filled action button
const cta = Button({ label: "Approve Gate", variant: "apple-filled" });

// Apple switch toggle
const toggle = FormToggle({ id: "safe-mode", label: "Safe Mode", variant: "apple", checked: true });
```
