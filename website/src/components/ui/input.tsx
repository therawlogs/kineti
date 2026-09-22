import * as React from 'react';
import { cn } from '../../lib/utils';

export function Input({ className, ...props }: React.InputHTMLAttributes<HTMLInputElement>): JSX.Element {
  return (
    <input
      className={cn(
        'h-11 w-full rounded-md border border-input-border bg-card px-3 text-[15px] text-foreground placeholder:text-faint focus:outline-2 focus:outline-offset-1 focus:outline-primary',
        className,
      )}
      {...props}
    />
  );
}

export function Textarea({ className, ...props }: React.TextareaHTMLAttributes<HTMLTextAreaElement>): JSX.Element {
  return (
    <textarea
      className={cn(
        'w-full rounded-md border border-input-border bg-card px-3 py-2.5 text-[15px] text-foreground placeholder:text-faint focus:outline-2 focus:outline-offset-1 focus:outline-primary',
        className,
      )}
      {...props}
    />
  );
}

export function Select({ className, ...props }: React.SelectHTMLAttributes<HTMLSelectElement>): JSX.Element {
  return (
    <select
      className={cn(
        'h-11 rounded-md border border-input-border bg-card px-3 text-[15px] text-foreground focus:outline-2 focus:outline-offset-1 focus:outline-primary',
        className,
      )}
      {...props}
    />
  );
}

export function Field({ label, children }: { label: string; children: React.ReactNode }): JSX.Element {
  return (
    <label className="grid gap-1.5 text-sm font-semibold">
      {label}
      {children}
    </label>
  );
}
