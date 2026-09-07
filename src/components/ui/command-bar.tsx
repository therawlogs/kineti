// src/components/ui/command-bar.tsx
// macOS Spotlight & Master Design System Command Bar

export interface CommandBarProps {
  placeholder?: string;
  shortcut?: string;
  className?: string;
}

export function CommandBar({
  placeholder = "Type a command or search (⌘K)...",
  shortcut = "⌘K",
  className = "",
}: CommandBarProps): string {
  return `
    <div class="relative flex items-center w-full max-w-lg bg-zinc-900/90 border border-zinc-800 rounded-[12px] px-3.5 py-2 shadow-[inset_0_1px_1px_rgba(0,0,0,0.4),0_2px_8px_rgba(0,0,0,0.25)] backdrop-blur-md focus-within:border-[#0A84FF] focus-within:ring-2 focus-within:ring-[#0A84FF]/20 transition-all ${className}">
      <svg class="w-4 h-4 text-zinc-400 mr-2.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
      <input type="text" placeholder="${placeholder}" class="w-full bg-transparent text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none tracking-tight font-normal" />
      <kbd class="hidden sm:inline-flex items-center px-1.5 py-0.5 text-[10px] font-mono text-zinc-400 bg-zinc-800/80 border border-zinc-700/60 rounded-[5px] shadow-sm ml-2">${shortcut}</kbd>
    </div>
  `;
}
