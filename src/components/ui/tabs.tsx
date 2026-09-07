// src/components/ui/tabs.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface TabItem {
  id: string;
  label: string;
  content: string;
  active?: boolean;
}

export interface TabsProps {
  id: string;
  tabs: TabItem[];
  className?: string;
}

export function Tabs({ id, tabs, className = "" }: TabsProps): string {
  const triggerButtons = tabs
    .map(
      (t) => `
      <button
        onclick="switchTab('${id}', '${t.id}')"
        data-tab-id="${t.id}"
        class="tab-btn px-3 py-1.5 text-xs font-medium rounded-md transition-all ${
          t.active
            ? "bg-zinc-800 text-zinc-100 shadow-sm"
            : "text-zinc-400 hover:text-zinc-200"
        }"
      >
        ${t.label}
      </button>
    `
    )
    .join("");

  const tabPanels = tabs
    .map(
      (t) => `
      <div id="${id}-content-${t.id}" class="tab-panel ${t.active ? "" : "hidden"} mt-4">
        ${t.content}
      </div>
    `
    )
    .join("");

  return `
    <div id="${id}" class="w-full ${className}">
      <div class="inline-flex h-9 items-center justify-center rounded-lg bg-zinc-900 p-1 border border-zinc-800 text-zinc-400">
        ${triggerButtons}
      </div>
      <div class="tab-contents">
        ${tabPanels}
      </div>
    </div>
  `;
}
