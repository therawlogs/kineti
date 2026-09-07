# Apple Design Standards & Material Library Guide

This guide explains how Kineti uses Apple's Human Interface Guidelines (HIG), material library, and design kits.

## 1. Core Principles

1. **Clarity**: Text is easy to read, buttons are obvious, and contrast is sharp.
2. **Deference**: Content and tasks are the primary focus. Shadows and blurs support the content without distraction.
3. **Depth**: Translucent glass layers show hierarchy. The active window sits above secondary surfaces.

## 2. Apple Materials Library

Apple materials use background blur, saturation, and subtle specular borders:

| Material | Blur / Tint | Purpose |
|---|---|---|
| **Ultra Thin** | `blur(20px)`, 5% surface tint | Subtle overlays and headers |
| **Thin** | `blur(30px)`, 55% surface tint | Standard cards and content areas |
| **Regular** | `blur(40px)`, 78% surface tint | Main macOS window surface |
| **Thick** | `blur(50px)`, 88% surface tint | Elevated dialogs and popovers |
| **Chrome** | `blur(45px)`, 85% surface tint | Window title bar with traffic lights |

All cards and windows include a top hairline highlight:
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
- **System Gray** (`#8E8E93`): Supporting labels and metadata.

## 4. Typography (SF Pro)

- **Main Font**: `-apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", system-ui, sans-serif`
- **Code Font**: `"SF Mono", Menlo, Monaco, Consolas, monospace`
- **Tracking**: Tight letter-spacing for large titles (`-0.02em`) and relaxed spacing for captions (`+0.01em`).

## 5. Controls & Design Kits

1. **Segmented Controls**: Recessed track with a floating capsule thumb for switching views.
2. **Switches / Toggles**: Smooth capsule switch that turns System Green (`#30D158`) when active.
3. **Pill Buttons**: Rounded buttons with subtle press scaling (`transform: scale(0.97)`).
4. **macOS Chrome**: Window title bar with red, yellow, and green traffic lights.
5. **Continuous Radii**: Smooth corner curvature (10px, 14px, 18px, 22px, and 26px squircle scales).

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
