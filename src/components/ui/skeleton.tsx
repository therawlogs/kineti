// src/components/ui/skeleton.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface SkeletonProps {
  className?: string;
  width?: string;
  height?: string;
  rounded?: "sm" | "md" | "lg" | "full";
}

export function Skeleton({
  className = "",
  width = "w-full",
  height = "h-4",
  rounded = "md",
}: SkeletonProps): string {
  const roundedStyles = {
    sm: "rounded-sm",
    md: "rounded-md",
    lg: "rounded-lg",
    full: "rounded-full",
  }[rounded];

  return `<div class="animate-pulse bg-zinc-800/80 ${roundedStyles} ${width} ${height} ${className}"></div>`;
}
