import { Link } from 'react-router-dom';
import { Actions, Section } from '../components/Layout';
import { Title } from '../components/Title';

export function Home(): JSX.Element {
  return (
    <>
      <Title text="Open research for human progress" />
      <section className="relative overflow-hidden border-b border-border py-16 sm:py-20">
        <div
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 bg-[radial-gradient(#ddd5cb_1px,transparent_1px)] [background-size:22px_22px] [mask-image:linear-gradient(to_bottom,black,transparent_85%)] opacity-60"
        />
        <div className="relative">
        <p className="text-xs font-semibold uppercase tracking-[0.12em] text-primary">
          Kineti · Open research for human progress
        </p>
        <h1 className="mt-2.5 max-w-[800px] text-[clamp(40px,5.5vw,64px)] font-semibold leading-[1.04] tracking-[-0.04em]">
          Independent research and open tools for how software should work.
        </h1>
        <Actions />
        </div>
      </section>

      <Section label="Start here">
        <ol className="mb-4 grid max-w-[600px] list-decimal gap-1.5 pl-5 text-muted">
          <li>Install Kineti.</li>
          <li>Wrap one MCP server.</li>
          <li>Pick a policy pack.</li>
          <li>Run the dangerous test.</li>
          <li>Verify the receipt.</li>
        </ol>
        <Link to="/tools" className="font-semibold text-foreground">See all 15 tools →</Link>
      </Section>
    </>
  );
}
