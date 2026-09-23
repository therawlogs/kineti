// src/components/ui/button.tsx
// Apple HIG & Master Design System Button

export interface ButtonProps {
  variant?: "primary" | "secondary" | "outline" | "ghost" | "destructive" | "apple-filled" | "apple-tinted" | "apple-gray";
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
  // Apple HIG base styles: continuous squircle radius, smooth active scale, font smoothing
  const baseStyles = "inline-flex items-center justify-center font-medium transition-all duration-150 ease-out active:scale-[0.98] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#0A84FF] focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-40 select-none cursor-pointer";
  
  const sizeStyles = {
    sm: "h-7 px-3 text-xs rounded-[8px] font-medium",
    md: "h-9 px-4 text-sm rounded-[10px] font-medium",
    lg: "h-11 px-6 text-base rounded-[14px] font-semibold",
  }[size];

  const variantStyles = {
    primary: "bg-violet-600 text-white shadow-[0_1px_2px_rgba(0,0,0,0.2),inset_0_1px_0_rgba(255,255,255,0.2)] hover:bg-violet-500 active:bg-violet-700",
    secondary: "bg-zinc-800 text-zinc-100 hover:bg-zinc-700 border border-zinc-700 shadow-sm",
    outline: "border border-zinc-800 bg-transparent hover:bg-zinc-800 hover:text-zinc-100 text-zinc-300",
    ghost: "hover:bg-zinc-800/60 hover:text-zinc-100 text-zinc-400",
    destructive: "bg-red-500/10 text-red-400 hover:bg-red-500/20 border border-red-500/30",
    "apple-filled": "bg-[#0A84FF] text-white shadow-[0_2px_8px_rgba(10,132,255,0.35),inset_0_1px_0_rgba(255,255,255,0.25)] hover:bg-[#0071E3] active:bg-[#005bb5]",
    "apple-tinted": "bg-[#0A84FF]/15 text-[#0A84FF] hover:bg-[#0A84FF]/25 border border-[#0A84FF]/30",
    "apple-gray": "bg-white/10 text-white hover:bg-white/15 border border-white/10 backdrop-blur-md",
  }[variant];

  const clickAttr = onClick ? ` onclick="${onClick}"` : "";
  const disabledAttr = disabled ? " disabled" : "";

  return `<button class="${baseStyles} ${sizeStyles} ${variantStyles} ${className}"${clickAttr}${disabledAttr}>${label}</button>`;
}
