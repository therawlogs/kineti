import { Link, NavLink } from 'react-router-dom';
import type { ReactNode } from 'react';
import { ArrowRight, ArrowUpRight } from 'lucide-react';
import { repo } from '../data';
import { cn } from '../lib/utils';
import { buttonClass } from './ui/button';

/**
 * One rhythm for every section on every page: same top padding,
 * same eyebrow label, same hairline below. This is what makes
 * the site feel aligned.
 */
export function Section({ label, children, className }: { label: string; children: ReactNode; className?: string }): JSX.Element {
  return (
    <section className={cn('border-b border-border py-12 last:border-b-0', className)}>
      <p className="mb-5 text-xs font-semibold uppercase tracking-[0.12em] text-primary">{label}</p>
      {children}
    </section>
  );
}

export function Layout({ children }: { children: ReactNode }): JSX.Element {
  return (
    <div className="mx-auto w-full max-w-[1060px] px-5 pb-10 sm:px-6">
      <header className="sticky top-0 z-10 -mx-5 mb-8 border-b border-border bg-background/95 px-5 backdrop-blur sm:-mx-6 sm:px-6">
        <div className="flex min-h-14 items-center justify-between gap-4">
          <Link to="/" className="flex shrink-0 items-center gap-2.5 text-lg font-semibold text-foreground no-underline">
            <span className="grid size-[30px] place-items-center rounded-lg bg-foreground font-mono text-white">K</span>
            <span className="hidden sm:inline">Kineti</span>
          </Link>
          <nav aria-label="Kineti pages" className="flex gap-4 overflow-x-auto sm:gap-6">
            {[
              ['/', 'Home'],
              ['/tools', 'Tools'],
              ['/research', 'Research'],
              ['/roadmap', 'Roadmap'],
              ['/contribute', 'Contribute'],
            ].map(([to, label]) => (
              <NavLink
                key={to}
                to={to}
                end={to === '/'}
                className={({ isActive }) =>
                  cn(
                    'inline-flex min-h-14 shrink-0 items-center text-[15px] text-muted no-underline transition-colors hover:text-foreground',
                    isActive && 'text-foreground shadow-[inset_0_-2px_0_var(--color-primary)]',
                  )
                }
              >
                {label}
              </NavLink>
            ))}
          </nav>
        </div>
      </header>
      <main>{children}</main>
      <footer className="mt-4 border-t border-border pt-6 text-[13px] text-muted">
        Open research for human progress. MIT licensed.{' '}
        <a href={repo} target="_blank" rel="noreferrer" className="text-foreground">Source</a>
      </footer>
    </div>
  );
}

export function Actions(): JSX.Element {
  return (
    <div className="mb-3 mt-6 flex flex-wrap gap-2.5">
      <a href={repo} target="_blank" rel="noreferrer" className={buttonClass({ variant: 'default' })}>View source <ArrowUpRight /></a>
      <Link to="/contribute" className={buttonClass({ variant: 'secondary' })}>Build with us <ArrowRight /></Link>
    </div>
  );
}
