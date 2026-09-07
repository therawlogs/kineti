// src/components/ui/modal.tsx
// Apple HIG System Dialog & Master Design System Modal

export interface ModalProps {
  id: string;
  title: string;
  description?: string;
  body: string;
  primaryAction?: string;
  secondaryAction?: string;
}

export function Modal({
  id,
  title,
  description = "",
  body,
  primaryAction = "",
  secondaryAction = "",
}: ModalProps): string {
  return `
    <div id="${id}" class="fixed inset-0 z-50 hidden items-center justify-center bg-black/60 backdrop-blur-xl p-4 transition-all">
      <div class="relative w-full max-w-lg rounded-[22px] border border-white/10 bg-[#1C1C1E]/90 p-6 shadow-[inset_0_1px_0_rgba(255,255,255,0.15),0_24px_48px_rgba(0,0,0,0.5)] backdrop-blur-2xl animate-in fade-in zoom-in-95 duration-200">
        <div class="flex flex-col space-y-1.5 text-center sm:text-left mb-4">
          <h3 class="text-lg font-semibold leading-none tracking-tight text-white">${title}</h3>
          ${description ? `<p class="text-sm text-zinc-400 mt-1">${description}</p>` : ""}
        </div>
        <div class="py-2 text-sm text-zinc-300">${body}</div>
        <div class="flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 mt-6 gap-2">
          ${secondaryAction}
          ${primaryAction}
        </div>
      </div>
    </div>
  `;
}
