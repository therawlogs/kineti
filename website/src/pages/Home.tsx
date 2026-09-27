import { Link } from 'react-router-dom';
import { Actions, Section } from '../components/Layout';
import { Title } from '../components/Title';
import { useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { repo } from '../data';

export function Home(): JSX.Element {
  const [copied, setCopied] = useState<string | null>(null);

  const copy = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopied(id);
    setTimeout(() => setCopied(null), 2000);
  };

  return (
    <>
      <Title text="Spend cap, undo, and proof receipts for AI coding agents" />
      <section className="relative overflow-hidden border-b border-border py-16 sm:py-20">
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 bg-[radial-gradient(#ddd5cb_1px,transparent_1px)] [background-size:22px_22px] [mask-image:linear-gradient(to_bottom,black,transparent_85%)] opacity-60"
        />
        <div className="relative">
          <p className="text-xs font-semibold uppercase tracking-[0.12em] text-primary">
            Kineti · Safety & Memory for AI Coding Agents
          </p>
          <h1 className="mt-2.5 max-w-[800px] text-[clamp(40px,5.5vw,64px)] font-semibold leading-[1.04] tracking-[-0.04em]">
            Spend cap, undo, and proof receipts for AI coding agents.
          </h1>
          <p className="mt-4 max-w-[650px] text-lg text-muted">
            A lightweight, open-source (MIT) safety layer that runs under Claude Code, Cursor, OpenCode, Codex, Antigravity, Cline, and fx.sh.
          </p>

          <div className="mt-6 flex max-w-[600px] flex-col gap-2.5 sm:flex-row">
            <div className="flex flex-1 items-center justify-between rounded-lg border border-border bg-card px-3.5 py-2.5 font-mono text-sm text-foreground">
              <span>npm install -g kineti</span>
              <button
                type="button"
                onClick={() => copy('npm install -g kineti', 'npm')}
                className="ml-2 text-muted hover:text-foreground"
                aria-label="Copy npm command"
              >
                {copied === 'npm' ? <Check className="size-4 text-green-600" /> : <Copy className="size-4" />}
              </button>
            </div>
            <div className="flex flex-1 items-center justify-between rounded-lg border border-border bg-card px-3.5 py-2.5 font-mono text-sm text-foreground">
              <span>cargo install kineti-cli</span>
              <button
                type="button"
                onClick={() => copy('cargo install kineti-cli', 'cargo')}
                className="ml-2 text-muted hover:text-foreground"
                aria-label="Copy cargo command"
              >
                {copied === 'cargo' ? <Check className="size-4 text-green-600" /> : <Copy className="size-4" />}
              </button>
            </div>
          </div>

          <Actions />
        </div>
      </section>

      <Section label="Supported Agents & Editors">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          <div className="rounded-lg border border-border bg-card/60 p-4">
            <h3 className="font-semibold text-foreground">Claude Code & Desktop</h3>
            <p className="mt-1 font-mono text-xs text-muted">claude mcp add kineti</p>
          </div>
          <div className="rounded-lg border border-border bg-card/60 p-4">
            <h3 className="font-semibold text-foreground">Cursor & Windsurf</h3>
            <p className="mt-1 font-mono text-xs text-muted">.cursor/mcp.json</p>
          </div>
          <div className="rounded-lg border border-border bg-card/60 p-4">
            <h3 className="font-semibold text-foreground">OpenCode & Codex</h3>
            <p className="mt-1 font-mono text-xs text-muted">opencode / codex mcp</p>
          </div>
          <div className="rounded-lg border border-border bg-card/60 p-4">
            <h3 className="font-semibold text-foreground">Antigravity & fx.sh</h3>
            <p className="mt-1 font-mono text-xs text-muted">kineti init (auto-hook)</p>
          </div>
        </div>
      </Section>

      <Section label="Start here">
        <ol className="mb-4 grid max-w-[600px] list-decimal gap-1.5 pl-5 text-muted">
          <li>Install Kineti (<code className="rounded bg-muted/20 px-1 py-0.5 font-mono text-xs text-foreground">npm install -g kineti</code>).</li>
          <li>Set up project rules (<code className="rounded bg-muted/20 px-1 py-0.5 font-mono text-xs text-foreground">kineti init</code>).</li>
          <li>Check your spend ceiling (<code className="rounded bg-muted/20 px-1 py-0.5 font-mono text-xs text-foreground">kineti spend check</code>).</li>
          <li>Run tests and capture receipts (<code className="rounded bg-muted/20 px-1 py-0.5 font-mono text-xs text-foreground">kineti test -- bun test</code>).</li>
          <li>Run the 30s spend-stop demo (<code className="rounded bg-muted/20 px-1 py-0.5 font-mono text-xs text-foreground">bash scripts/demo-spend-cap.sh</code>).</li>
        </ol>
        <p className="mt-2 text-[15px] text-muted">
          <Link to="/tools" className="font-semibold text-foreground">See all 15 tools →</Link>
          {' · '}
          <a href={repo} target="_blank" rel="noreferrer" className="font-semibold text-foreground">⭐ Star on GitHub</a>
        </p>
      </Section>
    </>
  );
}
