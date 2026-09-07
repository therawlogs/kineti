// src/components/ui/form-controls.tsx
// Apple HIG & Master Design System Form Controls

export interface InputProps {
  id: string;
  label?: string;
  type?: "text" | "number" | "password";
  placeholder?: string;
  value?: string;
  helperText?: string;
  className?: string;
}

export function Input({
  id,
  label,
  type = "text",
  placeholder = "",
  value = "",
  helperText = "",
  className = "",
}: InputProps): string {
  return `
    <div class="flex flex-col gap-1.5 w-full ${className}">
      ${label ? `<label for="${id}" class="text-xs font-semibold text-zinc-300 tracking-tight">${label}</label>` : ""}
      <input
        id="${id}"
        type="${type}"
        placeholder="${placeholder}"
        value="${value}"
        class="h-9 w-full rounded-[10px] border border-zinc-800 bg-zinc-900 px-3 py-1 text-sm text-zinc-100 placeholder-zinc-500 shadow-sm transition-all focus:border-[#0A84FF] focus:outline-none focus:ring-2 focus:ring-[#0A84FF]/20"
      />
      ${helperText ? `<span class="text-[11px] text-zinc-500">${helperText}</span>` : ""}
    </div>
  `;
}

export interface SelectOption {
  value: string;
  label: string;
  selected?: boolean;
}

export interface SelectProps {
  id: string;
  label?: string;
  options: SelectOption[];
  className?: string;
  onChange?: string;
}

export function Select({
  id,
  label,
  options,
  className = "",
  onChange = "",
}: SelectProps): string {
  const optionsMarkup = options
    .map(
      (opt) =>
        `<option value="${opt.value}" ${opt.selected ? "selected" : ""}>${opt.label}</option>`
    )
    .join("");

  return `
    <div class="flex flex-col gap-1.5 w-full ${className}">
      ${label ? `<label for="${id}" class="text-xs font-semibold text-zinc-300 tracking-tight">${label}</label>` : ""}
      <select
        id="${id}"
        ${onChange ? `onchange="${onChange}"` : ""}
        class="h-9 w-full rounded-[10px] border border-zinc-800 bg-zinc-900 px-3 py-1 text-sm text-zinc-100 shadow-sm transition-all focus:border-[#0A84FF] focus:outline-none focus:ring-2 focus:ring-[#0A84FF]/20 cursor-pointer"
      >
        ${optionsMarkup}
      </select>
    </div>
  `;
}

export interface ToggleProps {
  id: string;
  label: string;
  checked?: boolean;
  onChange?: string;
  variant?: "standard" | "apple";
}

export function Toggle({
  id,
  label,
  checked = false,
  onChange = "",
  variant = "standard",
}: ToggleProps): string {
  // Apple HIG toggle switch styling with fluid spring feel
  const activeColorClass = variant === "apple" ? "peer-checked:bg-[#30D158]" : "peer-checked:bg-violet-600";

  return `
    <label class="flex items-center gap-2.5 cursor-pointer select-none">
      <input type="checkbox" id="${id}" class="sr-only peer" ${checked ? "checked" : ""} ${onChange ? `onchange="${onChange}"` : ""} />
      <div class="relative w-10 h-6 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-4 peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:shadow-sm after:transition-transform after:duration-200 after:ease-out ${activeColorClass}"></div>
      <span class="text-xs font-medium text-zinc-300">${label}</span>
    </label>
  `;
}

// Aliases for clear namespacing
export const FormInput = Input;
export const FormSelect = Select;
export const FormToggle = Toggle;
