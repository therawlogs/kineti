import { useState } from 'react';
import { issueUrl } from '../data';
import { buttonClass } from './ui/button';
import { Field, Input, Textarea } from './ui/input';

export function RequestForm(): JSX.Element {
  const [name, setName] = useState('');
  const [who, setWho] = useState('');
  const [problem, setProblem] = useState('');
  const [proof, setProof] = useState('');
  const href = issueUrl(
    `[Tool request] ${name || 'New tool'}`,
    `## Tool\n${name || '[name]'}\n\n## Who needs it\n${who || '[user or team]'}\n\n## Risky action or painful workflow\n${problem || '[describe the workflow]'}\n\n## What proof should it produce?\n${proof || '[describe the trusted result]'}\n\n## Open-source intent\nI understand the project is MIT licensed.`,
    'tool-request',
  );
  return (
    <form className="grid max-w-[720px] gap-4" onSubmit={(e) => e.preventDefault()}>
      <Field label="Tool name">
        <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="A name a developer gets immediately" />
      </Field>
      <Field label="Who needs it?">
        <Input value={who} onChange={(e) => setWho(e.target.value)} placeholder="Team or workflow" />
      </Field>
      <Field label="What action or problem should it control?">
        <Textarea value={problem} onChange={(e) => setProblem(e.target.value)} rows={4} />
      </Field>
      <Field label="What should the proof show?">
        <Textarea value={proof} onChange={(e) => setProof(e.target.value)} rows={3} />
      </Field>
      <a className={buttonClass({ variant: 'default', className: 'justify-self-start' })} href={href} target="_blank" rel="noreferrer">
        Open public tool request
      </a>
      <p className="text-[13px] text-muted">This opens a prefilled GitHub issue. Nothing is submitted from this page.</p>
    </form>
  );
}

export function ContributorForm(): JSX.Element {
  const [name, setName] = useState('');
  const [contact, setContact] = useState('');
  const [area, setArea] = useState('');
  const [work, setWork] = useState('');
  const [plan, setPlan] = useState('');
  const href = issueUrl(
    `[Contributor] ${area || 'I want to help'}`,
    `## Name\n${name || '[name]'}\n\n## Contact\n${contact || '[GitHub, email, or other preferred route]'}\n\n## What I want to build or fix\n${area || '[area]'}\n\n## My approach\n${plan || '[brief plan]'}\n\n## Past work\n${work || '[links]'}\n\n## License\nI am comfortable contributing under the MIT license.`,
    'contributor',
  );
  return (
    <form className="grid max-w-[720px] gap-4" onSubmit={(e) => e.preventDefault()}>
      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Name">
          <Input value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Contact">
          <Input value={contact} onChange={(e) => setContact(e.target.value)} placeholder="GitHub, email, or other route" />
        </Field>
      </div>
      <Field label="What do you want to build or fix?">
        <Input value={area} onChange={(e) => setArea(e.target.value)} />
      </Field>
      <Field label="How would you approach it?">
        <Textarea value={plan} onChange={(e) => setPlan(e.target.value)} rows={4} />
      </Field>
      <Field label="Links to past work">
        <Textarea value={work} onChange={(e) => setWork(e.target.value)} rows={3} />
      </Field>
      <a className={buttonClass({ variant: 'default', className: 'justify-self-start' })} href={href} target="_blank" rel="noreferrer">
        Open contributor request
      </a>
      <p className="text-[13px] text-muted">This opens a prefilled GitHub issue for public review. Nothing is submitted from this page.</p>
    </form>
  );
}
