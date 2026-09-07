// src/components/ui/sheet.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface SheetProps {
  id: string;
  title: string;
  description?: string;
  side?: "left" | "right" | "top" | "bottom";
  content: string;
  footer?: string;
}

export function Sheet({
  id,
  title,
  description = "",
  side = "right",
  content,
  footer = "",
}: SheetProps): string {
  const sidePositions = {
    right: "inset-y-0 right-0 h-full w-3/4 max-w-md border-l animate-in slide-in-from-right",
    left: "inset-y-0 left-0 h-full w-3/4 max-w-md border-r animate-in slide-in-from-left",
    top: "inset-x-0 top-0 h-1/3 border-b animate-in slide-in-from-top",
    bottom: "inset-x-0 bottom-0 h-1/3 border-t animate-in slide-in-from-bottom",
  }[side];

  return `
    <div id="${id}" class="fixed inset-0 z-50 hidden bg-black/80 backdrop-blur-sm transition-opacity">
      <div class="fixed ${sidePositions} border-zinc-800 bg-zinc-900 p-6 shadow-2xl flex flex-col justify-between">
        <div>
          <div class="flex items-center justify-between pb-4 border-b border-zinc-800 mb-4">
            <div>
              <h3 class="text-base font-semibold text-white tracking-tight">${title}</h3>
              ${description ? `<p class="text-xs text-zinc-400 mt-1">${description}</p>` : ""}
            </div>
            <button onclick="document.getElementById('${id}').classList.add('hidden')" class="rounded-md p-1.5 text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800 focus:outline-none">
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
            </button>
          </div>
          <div class="text-sm text-zinc-300 space-y-4 overflow-y-auto">
            ${content}
          </div>
        </div>
        ${footer ? `<div class="pt-4 border-t border-zinc-800 mt-4 flex justify-end gap-2">${footer}</div>` : ""}
      </div>
    </div>
  `;
}
