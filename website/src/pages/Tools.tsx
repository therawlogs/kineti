import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { Section } from '../components/Layout';
import { Title } from '../components/Title';
import { ItemRow } from '../components/Rows';
import { Badge } from '../components/ui/badge';
import { Input, Select } from '../components/ui/input';
import { tiers, tools } from '../data';

export function Tools(): JSX.Element {
  const [q, setQ] = useState('');
  const [tier, setTier] = useState('All');
  const [sort, setSort] = useState<'number' | 'name'>('number');

  const items = useMemo(() => {
    const needle = q.trim().toLowerCase();
    const list = tools.filter(
      (t) =>
        (tier === 'All' || t.tier === tier) &&
        (needle === '' ||
          t.name.toLowerCase().includes(needle) ||
          t.detail.toLowerCase().includes(needle)),
    );
    list.sort((a, b) => {
      if (a.number === 1) return -1;
      if (b.number === 1) return 1;
      return sort === 'name' ? a.name.localeCompare(b.name) : a.number - b.number;
    });
    return list;
  }, [q, tier, sort]);

  return (
    <>
      <Title text="Tools" />
      <section className="border-b border-border py-8">
        <h1 className="text-[clamp(30px,4.5vw,48px)] font-semibold tracking-[-0.04em]">
          Tools.
        </h1>
        <div className="mt-5 flex flex-wrap gap-2.5">
          <Input
            className="min-w-[200px] flex-1"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Search tools"
            aria-label="Search tools"
          />
          <Select value={tier} onChange={(e) => setTier(e.target.value)} aria-label="Filter by tier">
            <option value="All">All tiers</option>
            {tiers.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </Select>
          <Select value={sort} onChange={(e) => setSort(e.target.value as 'number' | 'name')} aria-label="Sort tools">
            <option value="number">Sort: number</option>
            <option value="name">Sort: name</option>
          </Select>
        </div>
      </section>
      <Section label={tier === 'All' ? 'All tools' : tier}>
        <ol className="border-t border-border">
          {items.map((t) => (
            <ItemRow
              key={t.number}
              num={t.number}
              title={<Link to={`/docs/${t.slug}`} className="text-foreground hover:text-primary">{t.name}</Link>}
              detail={t.detail}
              right={<Badge variant="outline">{t.status}</Badge>}
            />
          ))}
        </ol>
        {items.length === 0 && <p className="mt-4 text-[13px] text-muted">No tools match.</p>}
      </Section>
    </>
  );
}
