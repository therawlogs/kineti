// src/components/ui/button.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface ButtonProps {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "destructive";
  size?: "sm" | "md" | "lg";
  label: string;
  onClick?: string;
  disabled?: boolean;
  className?: string;
}

export function Button({
  variant = "primary",
  size = "md",
  label,
  onClick,
  disabled = false,
  className = "",
}: ButtonProps): string {
  const baseStyles = "inline-flex items-center justify-center font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-violet-500 disabled:pointer-events-none disabled:opacity-50 select-none";
  
  const sizeStyles = {
    sm: "h-8 px-3 text-xs rounded-md",
    md: "h-9 px-4 text-sm rounded-md",
    lg: "h-10 px-6 text-sm rounded-lg",
  }[size];

  const variantStyles = {
    primary: "bg-violet-600 text-white shadow hover:bg-violet-500 active:bg-violet-700",
    secondary: "bg-zinc-800 text-zinc-100 hover:bg-zinc-700 border border-zinc-700",
    outline: "border border-zinc-800 bg-transparent hover:bg-zinc-800 hover:text-zinc-100 text-zinc-300",
    ghost: "hover:bg-zinc-800 hover:text-zinc-100 text-zinc-400",
    destructive: "bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/30",
  }[variant];

  const clickAttr = onClick ? ` onclick="${onClick}"` : "";
  const disabledAttr = disabled ? " disabled" : "";

  return `<button class="${baseStyles} ${sizeStyles} ${variantStyles} ${className}"${clickAttr}${disabledAttr}>${label}</button>`;
}
