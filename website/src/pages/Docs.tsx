import { Link, useParams } from 'react-router-dom';
import { repo, tools } from '../data';
import { Section } from '../components/Layout';
import { Title } from '../components/Title';
import { buttonClass } from '../components/ui/button';

export function DocDetail(): JSX.Element {
  const { slug } = useParams();
  const tool = tools.find((t) => t.slug === slug);
  if (!tool) {
    return (
      <section className="py-8">
        <Title text="Not found" />
        <h1 className="text-4xl font-semibold tracking-tight">Not found.</h1>
        <p className="mt-2 text-muted">No docs for that tool yet.</p>
        <Link to="/tools" className="mt-4 inline-block font-semibold text-foreground">← Back to tools</Link>
      </section>
    );
  }
  return (
    <>
      <Title text={tool.name} />
      <section className="border-b border-border py-8">
        <p className="text-xs font-semibold uppercase tracking-[0.12em] text-primary">Docs · {tool.tier}</p>
        <h1 className="mt-2 text-[clamp(30px,4.5vw,48px)] font-semibold tracking-[-0.04em]">{tool.name}</h1>
        <p className="mt-2 max-w-[660px] text-[16px] text-muted">{tool.detail}</p>
      </section>
      <Section label="What it does">
        <p className="max-w-[700px] leading-relaxed text-foreground/90">{tool.docs}</p>
      </Section>
      <Section label="Who it is for">
        <p className="max-w-[700px] leading-relaxed text-foreground/90">{tool.who}</p>
      </Section>
      <Section label="Status">
        <p className="max-w-[700px] leading-relaxed text-foreground/90">{tool.status} · {tool.date}</p>
        <div className="mt-6 flex flex-wrap gap-2.5">
          <Link to="/tools" className={buttonClass({ variant: 'secondary' })}>← All tools</Link>
          <a href={repo} target="_blank" rel="noreferrer" className={buttonClass({ variant: 'default' })}>View source</a>
        </div>
      </Section>
    </>
  );
}
