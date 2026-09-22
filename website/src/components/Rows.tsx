import type { ReactNode } from 'react';

/** One aligned row for tools, papers, and docs: number, content, right slot. */
export function ItemRow({
  num,
  title,
  byline,
  detail,
  right,
}: {
  num: number;
  title: ReactNode;
  byline?: string;
  detail: string;
  right: ReactNode;
}): JSX.Element {
  return (
    <li className="grid grid-cols-[32px_1fr] items-start gap-3.5 rounded-lg border-b border-border px-2 py-4 transition-colors last:border-b-0 hover:bg-card sm:grid-cols-[40px_1fr_auto] sm:items-center">
      <span className="grid size-[30px] place-items-center rounded-full bg-foreground text-[13px] font-semibold text-white">
        {num}
      </span>
      <div className="min-w-0">
        <div className="text-[15px] font-semibold">{title}</div>
        {byline && <p className="mt-0.5 text-[13px] text-muted">{byline}</p>}
        <p className="mt-1 text-sm leading-relaxed text-muted">{detail}</p>
      </div>
      <div className="col-start-2 justify-self-start sm:col-start-3 sm:justify-self-end">{right}</div>
    </li>
  );
}
