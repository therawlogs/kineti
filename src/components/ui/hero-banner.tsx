// src/components/ui/hero-banner.tsx
// Apple HIG Hero Banner & Master Design System

export interface HeroBannerProps {
  badgeText?: string;
  title: string;
  subtitle: string;
  ctaButton?: string;
  className?: string;
}

export function HeroBanner({
  badgeText = "KINETI OS RUNTIME",
  title,
  subtitle,
  ctaButton = "",
  className = "",
}: HeroBannerProps): string {
  return `
    <div class="relative overflow-hidden rounded-[20px] border border-zinc-800 bg-gradient-to-b from-zinc-900/90 to-zinc-950/90 p-8 shadow-[inset_0_1px_0_rgba(255,255,255,0.1),0_12px_32px_rgba(0,0,0,0.3)] backdrop-blur-xl ${className}">
      <div class="inline-flex items-center gap-2 rounded-full border border-violet-500/30 bg-violet-500/10 px-3 py-1 text-xs font-semibold text-violet-300 mb-4 shadow-sm">
        <span>${badgeText}</span>
      </div>
      <h1 class="text-3xl font-bold tracking-tight text-white sm:text-4xl mb-2.5 font-[-apple-system,BlinkMacSystemFont]">${title}</h1>
      <p class="max-w-2xl text-base text-zinc-400 mb-6 leading-relaxed">${subtitle}</p>
      ${ctaButton ? `<div class="flex gap-3">${ctaButton}</div>` : ""}
    </div>
  `;
}
