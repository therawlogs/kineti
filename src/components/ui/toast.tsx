// src/components/ui/toast.tsx
// Apple Dynamic Island / HUD Notification Pill & Master Design System

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
    default: "border-zinc-800 bg-[#1C1C1E]/95 text-zinc-100",
    success: "border-emerald-500/30 bg-[#1C1C1E]/95 text-emerald-300",
    warning: "border-amber-500/30 bg-[#1C1C1E]/95 text-amber-300",
    destructive: "border-red-500/30 bg-[#1C1C1E]/95 text-red-300",
  }[variant];

  return `
    <div id="${id}" class="pointer-events-auto flex w-full max-w-sm items-center justify-between space-x-4 overflow-hidden rounded-[18px] border p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.15),0_12px_32px_rgba(0,0,0,0.4)] backdrop-blur-2xl transition-all ${variantStyles} animate-in slide-in-from-top-2">
      <div class="grid gap-0.5">
        <div class="text-sm font-semibold tracking-tight">${title}</div>
        ${description ? `<div class="text-xs opacity-80">${description}</div>` : ""}
      </div>
      <button onclick="document.getElementById('${id}').remove()" class="rounded-full p-1 opacity-70 hover:opacity-100 hover:bg-white/10 transition-colors focus:outline-none cursor-pointer">
        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
  `;
}
