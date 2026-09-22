import { useMemo, useState } from 'react';
import { Section } from '../components/Layout';
import { Title } from '../components/Title';
import { ItemRow } from '../components/Rows';
import { Badge } from '../components/ui/badge';
import { Input, Select } from '../components/ui/input';
import { fields, papers } from '../data';

export function Research(): JSX.Element {
  const [q, setQ] = useState('');
  const [field, setField] = useState('All');
  const [sort, setSort] = useState<'number' | 'title'>('number');

  const items = useMemo(() => {
    const needle = q.trim().toLowerCase();
    const list = papers.filter(
      (p) =>
        (field === 'All' || p.field === field) &&
        (needle === '' ||
          p.title.toLowerCase().includes(needle) ||
          p.author.toLowerCase().includes(needle) ||
          p.detail.toLowerCase().includes(needle)),
    );
    list.sort((a, b) => (sort === 'title' ? a.title.localeCompare(b.title) : a.number - b.number));
    return list;
  }, [q, field, sort]);

  return (
    <>
      <Title text="Research" />
      <section className="border-b border-border py-8">
        <h1 className="text-[clamp(30px,4.5vw,48px)] font-semibold tracking-[-0.04em]">Research.</h1>
        <div className="mt-5 flex flex-wrap gap-2.5">
          <Input
            className="min-w-[200px] flex-1"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Search papers"
            aria-label="Search papers"
          />
          <Select value={field} onChange={(e) => setField(e.target.value)} aria-label="Filter by field">
            <option value="All">All fields</option>
            {fields.map((f) => (
              <option key={f} value={f}>{f}</option>
            ))}
          </Select>
          <Select value={sort} onChange={(e) => setSort(e.target.value as 'number' | 'title')} aria-label="Sort papers">
            <option value="number">Sort: number</option>
            <option value="title">Sort: title</option>
          </Select>
        </div>
      </section>
      <Section label={field === 'All' ? 'All papers' : field}>
        <ol className="border-t border-border">
          {items.map((p) => (
            <ItemRow
              key={p.number}
              num={p.number}
              title={p.title}
              byline={`By ${p.author} · ${p.field}`}
              detail={p.detail}
              right={<Badge variant="outline">{p.status}</Badge>}
            />
          ))}
        </ol>
        {items.length === 0 && <p className="mt-4 text-[13px] text-muted">No papers match.</p>}
      </Section>
    </>
  );
}
