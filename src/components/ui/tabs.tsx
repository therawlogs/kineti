// src/components/ui/tabs.tsx
// Apple Segmented Control & Master Design System Tabs

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
        class="tab-btn relative px-3.5 py-1 text-xs font-medium rounded-md transition-all duration-150 ease-out cursor-pointer ${
          t.active
            ? "bg-zinc-800 text-white shadow-[0_1px_3px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,255,255,0.12)] font-semibold"
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
      <div class="inline-flex h-9 items-center justify-center rounded-[10px] bg-zinc-900/90 p-1 border border-zinc-800 shadow-[inset_0_1px_2px_rgba(0,0,0,0.4)] backdrop-blur-md text-zinc-400">
        ${triggerButtons}
      </div>
      <div class="tab-contents">
        ${tabPanels}
      </div>
    </div>
  `;
}
