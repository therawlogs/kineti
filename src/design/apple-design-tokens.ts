// src/design/apple-design-tokens.ts
// Apple Human Interface Guidelines (HIG) Design Tokens & Material Library

export const AppleColors = {
  // System Colors (Dark Mode primary with Light Mode variants)
  systemBlue: "#0A84FF",
  systemBlueLight: "#007AFF",
  systemGreen: "#30D158",
  systemGreenLight: "#34C759",
  systemIndigo: "#5E5CE6",
  systemIndigoLight: "#5856D6",
  systemOrange: "#FF9F0A",
  systemOrangeLight: "#FF9500",
  systemPink: "#FF375F",
  systemPinkLight: "#FF2D55",
  systemPurple: "#BF5AF2",
  systemPurpleLight: "#AF52DE",
  systemRed: "#FF453A",
  systemRedLight: "#FF3B30",
  systemTeal: "#64D2FF",
  systemTealLight: "#5AC8FA",
  systemYellow: "#FFD60A",
  systemYellowLight: "#FFCC00",

  // System Grays
  systemGray: "#8E8E93",
  systemGray2: "#636366",
  systemGray3: "#48484A",
  systemGray4: "#3A3A3C",
  systemGray5: "#2C2C2E",
  systemGray6: "#1C1C1E",

  // Background Levels (Dark Mode)
  systemBackground: "#000000",
  secondarySystemBackground: "#1C1C1E",
  tertiarySystemBackground: "#2C2C2E",
  systemGroupedBackground: "#000000",
  secondarySystemGroupedBackground: "#1C1C1E",

  // Labels & Text
  label: "#FFFFFF",
  secondaryLabel: "rgba(235, 235, 245, 0.6)",
  tertiaryLabel: "rgba(235, 235, 245, 0.3)",
  quaternaryLabel: "rgba(235, 235, 245, 0.18)",

  // Separators & Hairlines
  separator: "rgba(84, 84, 88, 0.65)",
  opaqueSeparator: "#38383A",
} as const;

// Apple Materials Library (Vibrancy & Blur)
export const AppleMaterials = {
  ultraThin: {
    background: "rgba(30, 30, 35, 0.45)",
    backdropFilter: "blur(20px) saturate(180%)",
    border: "1px solid rgba(255, 255, 255, 0.08)",
    boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.1)",
  },
  thin: {
    background: "rgba(32, 32, 38, 0.65)",
    backdropFilter: "blur(30px) saturate(190%)",
    border: "1px solid rgba(255, 255, 255, 0.1)",
    boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.12)",
  },
  regular: {
    background: "rgba(36, 36, 44, 0.78)",
    backdropFilter: "blur(40px) saturate(200%)",
    border: "1px solid rgba(255, 255, 255, 0.12)",
    boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.14), 0 8px 32px rgba(0, 0, 0, 0.36)",
  },
  thick: {
    background: "rgba(42, 42, 50, 0.88)",
    backdropFilter: "blur(50px) saturate(210%)",
    border: "1px solid rgba(255, 255, 255, 0.15)",
    boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.16), 0 16px 48px rgba(0, 0, 0, 0.45)",
  },
  chrome: {
    background: "rgba(24, 24, 28, 0.75)",
    backdropFilter: "blur(45px) saturate(200%)",
    borderBottom: "1px solid rgba(255, 255, 255, 0.1)",
    boxShadow: "inset 0 -0.5px 0 rgba(0, 0, 0, 0.4)",
  },
} as const;

// Apple Continuous Corner Radii ("Squircle" scales)
export const AppleRadii = {
  xs: "6px",
  sm: "10px",
  md: "14px",
  lg: "18px",
  xl: "22px",
  sheet: "26px",
  pill: "9999px",
} as const;

// Apple Typography Font Stacks
export const AppleTypography = {
  fontSans: '-apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "SF Pro", "Helvetica Neue", sans-serif',
  fontMono: '"SF Mono", Menlo, Monaco, Consolas, "Liberation Mono", monospace',
  scales: {
    largeTitle: { size: "34px", weight: "700", tracking: "-0.022em", lineHeight: "1.2" },
    title1: { size: "28px", weight: "700", tracking: "-0.02em", lineHeight: "1.25" },
    title2: { size: "22px", weight: "600", tracking: "-0.018em", lineHeight: "1.3" },
    title3: { size: "20px", weight: "600", tracking: "-0.015em", lineHeight: "1.35" },
    headline: { size: "17px", weight: "600", tracking: "-0.012em", lineHeight: "1.4" },
    body: { size: "15px", weight: "400", tracking: "-0.008em", lineHeight: "1.45" },
    callout: { size: "14px", weight: "400", tracking: "-0.005em", lineHeight: "1.4" },
    subheadline: { size: "13px", weight: "400", tracking: "-0.003em", lineHeight: "1.4" },
    footnote: { size: "12px", weight: "400", tracking: "0em", lineHeight: "1.35" },
    caption1: { size: "11px", weight: "500", tracking: "+0.005em", lineHeight: "1.3" },
    caption2: { size: "10px", weight: "600", tracking: "+0.01em", lineHeight: "1.25" },
  },
} as const;

// Spring Animation Curves
export const AppleSpring = {
  snappy: "cubic-bezier(0.16, 1, 0.3, 1)",
  bouncy: "cubic-bezier(0.34, 1.56, 0.64, 1)",
  smooth: "cubic-bezier(0.25, 1, 0.5, 1)",
} as const;
