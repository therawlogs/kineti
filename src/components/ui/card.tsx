// src/components/ui/card.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface CardProps {
  title?: string;
  subtitle?: string;
  children?: string;
  className?: string;
  headerAction?: string;
}

export function Card({ title, subtitle, children = "", className = "", headerAction = "" }: CardProps): string {
  const header = title ? `
    <div class="flex items-center justify-between pb-3 mb-4 border-b border-zinc-800">
      <div>
        <h3 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">${title}</h3>
        ${subtitle ? `<p class="text-xs text-zinc-500 mt-0.5">${subtitle}</p>` : ""}
      </div>
      ${headerAction ? `<div>${headerAction}</div>` : ""}
    </div>
  ` : "";

  return `
    <div class="bg-zinc-900/80 border border-zinc-800/80 rounded-xl p-5 shadow-lg backdrop-blur-sm ${className}">
      ${header}
      <div class="card-content">${children}</div>
    </div>
  `;
}
