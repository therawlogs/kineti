// src/components/ui/toast.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface ToastProps {
  id: string;
  title: string;
  description?: string;
  variant?: "default" | "success" | "warning" | "destructive";
  durationMs?: number;
}

export function Toast({
  id,
  title,
  description = "",
  variant = "default",
}: ToastProps): string {
  const variantStyles = {
    default: "border-zinc-800 bg-zinc-900 text-zinc-100",
    success: "border-emerald-500/30 bg-emerald-950/40 text-emerald-300",
    warning: "border-amber-500/30 bg-amber-950/40 text-amber-300",
    destructive: "border-red-500/30 bg-red-950/40 text-red-300",
  }[variant];

  return `
    <div id="${id}" class="pointer-events-auto flex w-full max-w-sm items-center justify-between space-x-4 overflow-hidden rounded-lg border p-4 shadow-lg transition-all ${variantStyles} animate-in slide-in-from-top-2">
      <div class="grid gap-1">
        <div class="text-sm font-semibold">${title}</div>
        ${description ? `<div class="text-xs opacity-90">${description}</div>` : ""}
      </div>
      <button onclick="document.getElementById('${id}').remove()" class="rounded-md p-1 opacity-70 hover:opacity-100 focus:outline-none">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
  `;
}
