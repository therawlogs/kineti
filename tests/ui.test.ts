// tests/ui.test.ts
// Tests for the 12 Master Design System UI components (Archetype A)

import { describe, expect, it } from "bun:test";
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

describe("Master Design System - 12 Mandatory UI Components", () => {
  it("1. Button renders with Archetype A styles and variants", () => {
    const primary = Button({ label: "Approve Gate", variant: "primary" });
    expect(primary).toContain("bg-violet-600");
    expect(primary).toContain("Approve Gate");

    const destructive = Button({ label: "Rollback", variant: "destructive" });
    expect(destructive).toContain("text-red-400");
  });

  it("2. Card renders structured surface with 1px border", () => {
    const card = Card({
      title: "Cost Meter",
      children: "<div>$0.024</div>",
      subtitle: "Current session",
    });
    expect(card).toContain("border-zinc-800");
    expect(card).toContain("bg-zinc-900");
    expect(card).toContain("Cost Meter");
  });

  it("3. CommandBar renders ⌘K trigger", () => {
    const bar = CommandBar({ placeholder: "Search commands..." });
    expect(bar).toContain("⌘K");
    expect(bar).toContain("Search commands...");
  });

  it("4. DataTable renders sortable grid with headers and rows", () => {
    const table = DataTable({
      columns: [
        { key: "stage", header: "Stage" },
        { key: "status", header: "Status" },
      ],
      data: [
        { stage: "spec", status: "passed" },
        { stage: "build", status: "in-progress" },
      ],
    });
    expect(table).toContain("Stage");
    expect(table).toContain("passed");
    expect(table).toContain("in-progress");
  });

  it("5. Form controls render input, select, and toggle", () => {
    const input = FormInput({ id: "project-name", label: "Project Name" });
    expect(input).toContain("<input");
    expect(input).toContain("Project Name");

    const select = FormSelect({
      id: "archetype",
      label: "Archetype",
      options: [{ value: "A", label: "Modern Technical SaaS" }],
    });
    expect(select).toContain("<select");

    const toggle = FormToggle({ id: "gate-strict", label: "Strict Gate" });
    expect(toggle).toContain("type=\"checkbox\"");
  });

  it("6. Modal renders dialog overlay with actions", () => {
    const modal = Modal({
      id: "gate-modal",
      title: "Confirm Gate Release",
      body: "Are you sure you want to approve Stage 6 (Spec)?",
    });
    expect(modal).toContain("gate-modal");
    expect(modal).toContain("Confirm Gate Release");
  });

  it("7. Sheet renders slide-out drawer panel", () => {
    const sheet = Sheet({
      id: "trace-sheet",
      title: "Causal Trace Detail",
      content: "<p>Event details</p>",
      side: "right",
    });
    expect(sheet).toContain("trace-sheet");
    expect(sheet).toContain("slide-in-from-right");
    expect(sheet).toContain("Causal Trace Detail");
  });

  it("8. Toast renders status alert", () => {
    const toast = Toast({
      id: "toast-1",
      title: "Merkle DAG Verified",
      variant: "success",
    });
    expect(toast).toContain("border-emerald-500/30");
    expect(toast).toContain("Merkle DAG Verified");
  });

  it("9. Tabs renders segmented switcher", () => {
    const tabs = Tabs({
      id: "view-tabs",
      tabs: [
        { id: "pipeline", label: "Pipeline", content: "<div>Pipeline View</div>", active: true },
        { id: "audit", label: "Audit Log", content: "<div>Audit View</div>" },
      ],
    });
    expect(tabs).toContain("switchTab('view-tabs', 'pipeline')");
    expect(tabs).toContain("Pipeline View");
  });

  it("10. StatusBadge renders semantic pills", () => {
    const badge = StatusBadge({ status: "pass", label: "Verified" });
    expect(badge).toContain("bg-emerald-500/10");
    expect(badge).toContain("Verified");
  });

  it("11. Skeleton renders pulsing loader", () => {
    const skeleton = Skeleton({ width: "w-32", height: "h-6" });
    expect(skeleton).toContain("animate-pulse");
    expect(skeleton).toContain("w-32");
    expect(skeleton).toContain("h-6");
  });

  it("12. HeroBanner renders page title and CTA container", () => {
    const banner = HeroBanner({
      title: "Kineti Causal Harness",
      subtitle: "Runtime Governance & Verification",
      badgeText: "v0.3.0",
    });
    expect(banner).toContain("Kineti Causal Harness");
    expect(banner).toContain("v0.3.0");
  });
});
