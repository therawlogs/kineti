import { ContributorForm } from '../components/Forms';
import { Section } from '../components/Layout';
import { Title } from '../components/Title';
import { ItemRow } from '../components/Rows';

const firsts = [
  { n: 1, name: 'Add a policy pack', detail: 'Code changes, money, customer data, production, or a workflow you know.' },
  { n: 2, name: 'Break the gate', detail: 'Add hostile calls, changed arguments, stale approvals, and evidence edge cases.' },
  { n: 3, name: 'Improve one micro-tool', detail: 'Package an extractable v0.3.0 capability with tests and a clear README.' },
  { n: 4, name: 'Build a reference integration', detail: 'Show exact-action approval and recovery against a real MCP workflow.' },
];

export function Contribute(): JSX.Element {
  return (
    <>
      <Title text="Contribute" />
      <section className="border-b border-border py-8">
        <h1 className="text-[clamp(30px,4.5vw,48px)] font-semibold tracking-[-0.04em]">
          Build it with us.
        </h1>
      </section>
      <Section label="Good first contributions">
        <ol className="border-t border-border">
          {firsts.map((r) => (
            <ItemRow
              key={r.n}
              num={r.n}
              title={r.name}
              detail={r.detail}
              right={<span />}
            />
          ))}
        </ol>
      </Section>
      <Section label="Contributor request">
        <ContributorForm />
      </Section>
    </>
  );
}
