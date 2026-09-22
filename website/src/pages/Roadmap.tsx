import { Link } from 'react-router-dom';
import { RequestForm } from '../components/Forms';
import { Section } from '../components/Layout';
import { Title } from '../components/Title';
import { repo, tools, weeks } from '../data';

export function Roadmap(): JSX.Element {
  const next = tools.filter((t) => t.tier === 'The wedge');

  return (
    <>
      <Title text="Roadmap" />
      <section className="border-b border-border py-8">
        <h1 className="text-[clamp(30px,4.5vw,48px)] font-semibold tracking-[-0.04em]">
          Roadmap.
        </h1>
      </section>
      <Section label="Six-week sprint">
        <ol className="border-t border-border">
          {weeks.map((w, i) => (
            <li key={w[0]} className="grid grid-cols-[44px_1fr] gap-4 border-b border-border py-4 last:border-b-0">
              <span className="grid size-9 place-items-center rounded-full bg-foreground font-semibold text-white">
                {i + 1}
              </span>
              <div>
                <p className="text-[13px] text-muted">{w[0]}</p>
                <h3 className="mt-0.5 text-[16px] font-semibold">{w[1]}</h3>
                <p className="text-sm text-muted">{w[2]}</p>
              </div>
            </li>
          ))}
        </ol>
      </Section>
      <Section label="Next up">
        <ul className="grid max-w-[700px] gap-2">
          {next.map((t) => (
            <li key={t.number} className="text-[15px]">
              <Link to={`/docs/${t.slug}`} className="font-semibold text-foreground">{t.name}</Link>
              <span className="text-muted"> · {t.status}</span>
            </li>
          ))}
          <li className="text-[15px]">
            <span className="font-semibold">5 research papers</span>
            <span className="text-muted"> · Coming soon</span>
          </li>
        </ul>
        <p className="mt-4 text-[15px] text-muted">
          Full lists live on <Link to="/tools" className="font-semibold text-foreground">Tools</Link> and{' '}
          <Link to="/research" className="font-semibold text-foreground">Research</Link>. Like it?{' '}
          <a href={repo} target="_blank" rel="noreferrer" className="font-semibold text-foreground">⭐ Star on GitHub</a>.
        </p>
      </Section>
      <Section label="Shape what comes next">
        <RequestForm />
      </Section>
    </>
  );
}
