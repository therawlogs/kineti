import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const badgeVariants = cva(
  'inline-flex items-center rounded-full px-2.5 py-1 text-xs font-semibold whitespace-nowrap',
  {
    variants: {
      variant: {
        teal: 'bg-primary-soft text-primary-ink',
        outline: 'border border-border bg-card text-muted',
        danger: 'border border-danger-border bg-card text-danger',
      },
    },
    defaultVariants: { variant: 'teal' },
  },
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps): JSX.Element {
  return <span className={cn(badgeVariants({ variant }), className)} {...props} />;
}
