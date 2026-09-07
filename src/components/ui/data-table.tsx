// src/components/ui/data-table.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface Column<T> {
  key: keyof T | string;
  header: string;
  render?: (row: T) => string;
}

export interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  emptyMessage?: string;
  className?: string;
}

export function DataTable<T extends Record<string, any>>({
  columns,
  data,
  emptyMessage = "No records found.",
  className = "",
}: DataTableProps<T>): string {
  const headerHtml = columns.map(c => `<th class="text-left py-2.5 px-3 text-xs font-semibold text-zinc-400 uppercase tracking-wider border-b border-zinc-800">${c.header}</th>`).join("");

  const rowsHtml = data.length === 0
    ? `<tr><td colspan="${columns.length}" class="py-8 text-center text-sm text-zinc-500">${emptyMessage}</td></tr>`
    : data.map(row => {
        const cells = columns.map(c => {
          const val = c.render ? c.render(row) : String(row[c.key as string] ?? "");
          return `<td class="py-3 px-3 text-xs text-zinc-300 font-mono border-b border-zinc-800/50">${val}</td>`;
        }).join("");
        return `<tr class="hover:bg-zinc-800/40 transition-colors">${cells}</tr>`;
      }).join("");

  return `
    <div class="overflow-x-auto rounded-lg border border-zinc-800 bg-zinc-950/60 ${className}">
      <table class="w-full border-collapse text-left">
        <thead><tr class="bg-zinc-900/50">${headerHtml}</tr></thead>
        <tbody>${rowsHtml}</tbody>
      </table>
    </div>
  `;
}
