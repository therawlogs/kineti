// src/components/ui/form-controls.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

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
      ${label ? `<label for="${id}" class="text-xs font-semibold text-zinc-300">${label}</label>` : ""}
      <input
        id="${id}"
        type="${type}"
        placeholder="${placeholder}"
        value="${value}"
        class="h-9 w-full rounded-md border border-zinc-800 bg-zinc-900 px-3 py-1 text-sm text-zinc-100 placeholder-zinc-500 shadow-sm transition-colors focus:border-violet-500 focus:outline-none focus:ring-1 focus:ring-violet-500"
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
      ${label ? `<label for="${id}" class="text-xs font-semibold text-zinc-300">${label}</label>` : ""}
      <select
        id="${id}"
        ${onChange ? `onchange="${onChange}"` : ""}
        class="h-9 w-full rounded-md border border-zinc-800 bg-zinc-900 px-3 py-1 text-sm text-zinc-100 shadow-sm transition-colors focus:border-violet-500 focus:outline-none focus:ring-1 focus:ring-violet-500"
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
}

export function Toggle({
  id,
  label,
  checked = false,
  onChange = "",
}: ToggleProps): string {
  return `
    <label class="flex items-center gap-2.5 cursor-pointer select-none">
      <input type="checkbox" id="${id}" class="sr-only peer" ${checked ? "checked" : ""} ${onChange ? `onchange="${onChange}"` : ""} />
      <div class="w-9 h-5 bg-zinc-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-violet-600"></div>
      <span class="text-xs font-medium text-zinc-300">${label}</span>
    </label>
  `;
}

// Aliases for clear namespacing
export const FormInput = Input;
export const FormSelect = Select;
export const FormToggle = Toggle;
