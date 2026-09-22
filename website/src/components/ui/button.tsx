import * as React from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '../../lib/utils';

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-[15px] font-semibold transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-4 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'bg-foreground text-white hover:brightness-125',
        primary: 'bg-primary text-white hover:brightness-125',
        secondary: 'border border-border bg-card text-foreground hover:border-primary',
        ghost: 'text-foreground hover:bg-primary-soft',
      },
      size: {
        default: 'h-[46px] px-[18px]',
        sm: 'h-10 px-3 text-sm',
        xs: 'h-8 px-2.5 text-[13px]',
      },
    },
    defaultVariants: { variant: 'default', size: 'default' },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

export function Button({ className, variant, size, type = 'button', ...props }: ButtonProps): JSX.Element {
  return <button type={type} className={cn(buttonVariants({ variant, size }), className)} {...props} />;
}

export function buttonClass(opts?: VariantProps<typeof buttonVariants> & { className?: string }): string {
  const { className, ...variants } = opts ?? {};
  return cn(buttonVariants(variants), className);
}
