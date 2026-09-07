// src/components/ui/card.tsx
// Apple Material & Master Design System Card

export interface CardProps {
  title?: string;
  subtitle?: string;
  children?: string;
  className?: string;
  headerAction?: string;
  material?: "ultraThin" | "thin" | "regular" | "thick";
}

export function Card({
  title,
  subtitle,
  children = "",
  className = "",
  headerAction = "",
}: CardProps): string {
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
    <div class="bg-zinc-900 border border-zinc-800 rounded-2xl p-5 shadow-[inset_0_1px_0_rgba(255,255,255,0.08),0_8px_24px_rgba(0,0,0,0.25)] backdrop-blur-xl ${className}">
      ${header}
      <div class="card-content">${children}</div>
    </div>
  `;
}
