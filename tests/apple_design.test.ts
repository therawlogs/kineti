// tests/apple_design.test.ts
// Tests for Apple Human Interface Guidelines (HIG), Material Library, and Design Kit

import { describe, expect, it } from "bun:test";
import {
  AppleColors,
  AppleMaterials,
  AppleRadii,
  AppleTypography,
  AppleSpring,
} from "../src/design/apple-design-tokens";
import {
  Button,
  Card,
  CommandBar,
  DataTable,
  FormInput,
  FormSelect,
  FormToggle,
  HeroBanner,
  Modal,
  Sheet,
  Skeleton,
  StatusBadge,
  Tabs,
  Toast,
} from "../src/components/ui";

describe("Apple Design Standards & Material Library", () => {
  it("verifies Apple HIG system color tokens", () => {
    expect(AppleColors.systemBlue).toBe("#0A84FF");
    expect(AppleColors.systemGreen).toBe("#30D158");
    expect(AppleColors.systemRed).toBe("#FF453A");
    expect(AppleColors.systemOrange).toBe("#FF9F0A");
    expect(AppleColors.systemPurple).toBe("#BF5AF2");
    expect(AppleColors.systemBackground).toBe("#000000");
    expect(AppleColors.secondarySystemBackground).toBe("#1C1C1E");
  });

  it("verifies Apple Materials vibrancy, blur, and specular highlights", () => {
    expect(AppleMaterials.ultraThin.backdropFilter).toContain("blur(20px)");
    expect(AppleMaterials.thin.backdropFilter).toContain("blur(30px)");
    expect(AppleMaterials.regular.backdropFilter).toContain("blur(40px)");
    expect(AppleMaterials.regular.boxShadow).toContain("inset 0 1px 0");
    expect(AppleMaterials.thick.backdropFilter).toContain("blur(50px)");
    expect(AppleMaterials.chrome.backdropFilter).toContain("blur(45px)");
  });

  it("verifies Apple typography and continuous corner radii", () => {
    expect(AppleTypography.fontSans).toContain("-apple-system");
    expect(AppleTypography.fontSans).toContain("SF Pro Display");
    expect(AppleTypography.fontMono).toContain("SF Mono");
    expect(AppleRadii.sheet).toBe("26px");
    expect(AppleSpring.snappy).toContain("cubic-bezier");
  });

  it("renders Apple HIG button variants", () => {
    const filled = Button({ label: "Continue", variant: "apple-filled" });
    expect(filled).toContain("bg-[#0A84FF]");
    expect(filled).toContain("Continue");

    const tinted = Button({ label: "Details", variant: "apple-tinted" });
    expect(tinted).toContain("text-[#0A84FF]");

    const glass = Button({ label: "Cancel", variant: "apple-gray" });
    expect(glass).toContain("backdrop-blur-md");
  });

  it("renders Apple switch toggle with system green fill", () => {
    const appleToggle = FormToggle({ id: "haptic", label: "Haptic Feedback", variant: "apple", checked: true });
    expect(appleToggle).toContain("peer-checked:bg-[#30D158]");
    expect(appleToggle).toContain("Haptic Feedback");
  });

  it("renders Apple frosted glass card with continuous squircle corners", () => {
    const card = Card({ title: "Live Metrics", children: "<span>Active</span>" });
    expect(card).toContain("rounded-2xl");
    expect(card).toContain("backdrop-blur-xl");
    expect(card).toContain("inset_0_1px_0");
  });

  it("renders Apple system dialog modal with frosted backdrop and 22px squircle", () => {
    const modal = Modal({ id: "dialog-1", title: "Confirm Action", body: "Proceed with step?" });
    expect(modal).toContain("rounded-[22px]");
    expect(modal).toContain("backdrop-blur-2xl");
    expect(modal).toContain("Confirm Action");
  });

  it("renders Apple slide-out sheet drawer with frosted blur", () => {
    const sheet = Sheet({ id: "sheet-1", title: "Inspection Panel", content: "Details here" });
    expect(sheet).toContain("backdrop-blur-3xl");
    expect(sheet).toContain("slide-in-from-right");
    expect(sheet).toContain("Inspection Panel");
  });

  it("renders Apple Dynamic Island / HUD notification toast", () => {
    const toast = Toast({ id: "toast-hud", title: "Saved to Journal", variant: "default" });
    expect(toast).toContain("rounded-[18px]");
    expect(toast).toContain("backdrop-blur-2xl");
    expect(toast).toContain("Saved to Journal");
  });
});
