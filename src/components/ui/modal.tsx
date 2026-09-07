// src/components/ui/modal.tsx
// Archetype A: Modern Technical SaaS (Linear / Vercel style)

export interface ModalProps {
  id: string;
  title: string;
  description?: string;
  body: string;
  primaryAction?: string;
  secondaryAction?: string;
}

export function Modal({ id, title, description = "", body, primaryAction = "", secondaryAction = "" }: ModalProps): string {
  return `
    <div id="${id}" class="fixed inset-0 z-50 hidden items-center justify-center bg-black/80 backdrop-blur-sm p-4">
      <div class="relative w-full max-w-lg rounded-xl border border-zinc-800 bg-zinc-900 p-6 shadow-2xl animate-in fade-in zoom-in-95">
        <div class="flex flex-col space-y-1.5 text-center sm:text-left mb-4">
          <h3 class="text-lg font-semibold leading-none tracking-tight text-white">${title}</h3>
          ${description ? `<p class="text-sm text-zinc-400">${description}</p>` : ""}
        </div>
        <div class="py-2 text-sm text-zinc-300">${body}</div>
        <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 mt-6">
          ${secondaryAction}
          ${primaryAction}
        </div>
      </div>
    </div>
  `;
}
