// src/components/ui/status-badge.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface StatusBadgeProps {
  status: "pass" | "fail" | "pending" | "online" | "tripped";
  label?: string;
  className?: string;
}

export function StatusBadge({ status, label, className = "" }: StatusBadgeProps): string {
  const text = label || status.toUpperCase();
  const styles = {
    pass: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
    online: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
    fail: "bg-red-500/10 text-red-400 border-red-500/20",
    tripped: "bg-red-500/10 text-red-400 border-red-500/20",
    pending: "bg-amber-500/10 text-amber-400 border-amber-500/20",
  }[status];

  return `
    <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold border ${styles} ${className}">
      <span class="w-1.5 h-1.5 rounded-full bg-current"></span>
      <span>${text}</span>
    </span>
  `;
}
