import * as React from 'react';
import { cn } from '../../lib/utils';

export function Card({ className, ...props }: React.HTMLAttributes<HTMLDivElement>): JSX.Element {
  return (
    <div
      className={cn('rounded-lg border border-border bg-card p-5 shadow-[0_1px_2px_rgba(37,33,31,0.05)] transition-[border-color,box-shadow] duration-150 hover:border-[#c9c0b8] hover:shadow-[0_6px_20px_rgba(37,33,31,0.09)]', className)}
      {...props}
    />
  );
}

export function CardTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>): JSX.Element {
  return <h3 className={cn('text-[17px] font-semibold tracking-tight', className)} {...props} />;
}

export function CardText({ className, ...props }: React.HTMLAttributes<HTMLParagraphElement>): JSX.Element {
  return <p className={cn('mt-1.5 text-[15px] leading-relaxed text-muted', className)} {...props} />;
}
