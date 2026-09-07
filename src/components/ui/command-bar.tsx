// src/components/ui/command-bar.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface CommandBarProps {
  placeholder?: string;
  shortcut?: string;
  className?: string;
}

export function CommandBar({ placeholder = "Type a command or search (⌘K)...", shortcut = "⌘K", className = "" }: CommandBarProps): string {
  return `
    <div class="relative flex items-center w-full max-w-lg bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 shadow-inner focus-within:border-violet-500 focus-within:ring-1 focus-within:ring-violet-500 transition-all ${className}">
      <svg class="w-4 h-4 text-zinc-500 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
      <input type="text" placeholder="${placeholder}" class="w-full bg-transparent text-sm text-zinc-100 placeholder-zinc-500 focus:outline-none" />
      <kbd class="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono text-zinc-400 bg-zinc-800 border border-zinc-700 rounded">${shortcut}</kbd>
    </div>
  `;
}
