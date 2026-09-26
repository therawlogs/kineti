// Shared data for all 5 routes. MIT.
export const repo = 'https://github.com/therawlogs/kineti';
export const author = 'Kineti Research Team';

export interface Tool {
  number: number;
  slug: string;
  name: string;
  detail: string;
  docs: string;
  who: string;
  status: string;
  tier: string;
  date: string;
  /** GitHub issue number for voting. Absent until an issue exists. */
  issue?: number;
}

export const tools: Tool[] = [
  { number: 1, slug: 'gate-mcp', name: 'Kineti Gate for MCP', detail: 'Local proxy: allow, ask, or deny. Approval binds to the exact action.', docs: 'A local Rust proxy that sits in front of MCP tool calls. Set allow, ask, or deny policies for tool and argument patterns. Human approval binds to the exact action hash: change one argument and the approval voids. Hard per-run and per-project spend stops. Every call writes a dual-signed evidence receipt.', who: 'Teams whose agents touch code, production, or customer data.', status: 'In progress', tier: 'In progress', date: 'Next' },
  { number: 2, slug: 'verifier-spec', name: 'Kineti Verifier + Spec', detail: 'Check any receipt without trusting Kineti cloud.', docs: 'An independent CLI plus the public receipt and policy spec. Anyone can verify a Kineti receipt without running Kineti or trusting its servers. The spec, the verifier, and the gate are all MIT.', who: 'Buyers, auditors, skeptics.', status: 'In progress', tier: 'In progress', date: 'Next' },
  { number: 3, slug: 'kineti-talk', name: 'kineti_talk', detail: 'One MCP tool across 10 editors and hosts.', docs: 'One MCP tool that puts Kineti inside 10 editors and agent hosts. Talk normal, no commands. Shipped and working in v0.3.4.', who: 'Solo builders running agents in their editor.', status: 'Shipped in v0.3.4', tier: 'Shipped', date: 'v0.3.4' },
  { number: 4, slug: 'spend-stop', name: 'Kineti Spend Stop', detail: 'Hard spend ceiling with human-only reset.', docs: 'A hard spending ceiling for any agent setup. Per-agent splits, per-run and per-project stops. Only a human resets it. For anyone burned by a runaway agent bill.', who: 'Anyone paying model bills.', status: 'Shipped in v0.3.4', tier: 'Shipped', date: 'v0.3.4' },
  { number: 5, slug: 'undo', name: 'Kineti Undo', detail: 'Every change registers its inverse. Undo needs your yes.', docs: 'Undo for agent actions. Every change registers its inverse before it runs. Nothing reverses without your approval. Where an action has no inverse, Kineti says so up front.', who: 'Teams letting agents change real systems.', status: 'Shipped in v0.3.4', tier: 'Shipped', date: 'v0.3.4' },
  { number: 6, slug: 'memory', name: 'Kineti Memory', detail: 'Project-scoped memory. Forgets with a proof receipt.', docs: 'Memory that cannot leak across projects. Scope isolation per project, certainty tiers per fact, tombstones on delete. Forgetting writes a proof receipt.', who: 'Teams running agents across many projects.', status: 'Shipped in v0.3.4', tier: 'Shipped', date: 'v0.3.4' },
  { number: 7, slug: 'ovt', name: 'Kineti OVT', detail: 'Scores dollars per true outcome.', docs: 'Proof of what your agent actually achieved. Outcome verification plus dollars-per-true-outcome scoring. Measures results, not model vibes.', who: 'Teams justifying agent spend.', status: 'Shipped in v0.3.4', tier: 'Shipped', date: 'v0.3.4' },
  { number: 8, slug: 'dashboard', name: 'Kineti Dashboard', detail: 'Token-locked local view: goals, spend, undo, proof.', docs: 'The local dashboard: goal, spend, undo, proof, on/off switch, one talk box. Token locked on your machine. Encrypted sync between machines.', who: 'Operators watching agents daily.', status: 'Shipped in v0.3.4', tier: 'Shipped', date: 'v0.3.4' },
  { number: 9, slug: 'policy-packs', name: 'Kineti Policy Packs', detail: 'Starter rules for code, money, data, production.', docs: 'Starter policy templates for the gate. Code changes, money movement, customer data, production operations. Pick one, tighten it, ship.', who: 'New gate users.', status: 'Planned', tier: 'Planned', date: 'Planned' },
  { number: 10, slug: 'hostile-100', name: 'Kineti Hostile 100', detail: '100 hostile tool calls. Method and failures public.', docs: 'A public benchmark of 100 hostile tool calls against the gate. Method published, every failure published. The bar the gate must clear.', who: 'Skeptics and contributors.', status: 'Planned', tier: 'Planned', date: 'Planned' },
  { number: 11, slug: 'receipt-viewer', name: 'Kineti Receipt Viewer', detail: 'Renders a signed receipt readable.', docs: 'Turns signed receipt JSON into a readable record: action, policy, approver, hash, outcome. For humans checking what happened.', who: 'Auditors and operators.', status: 'Planned', tier: 'Planned', date: 'Planned' },
  { number: 12, slug: 'demo', name: 'Kineti Demo', detail: 'Approve $10, try $100, watch it block.', docs: 'The 90-second dangerous-action demo as a runnable script. Approval set for $10, model changes it to $100, Kineti blocks it and writes a verifiable receipt.', who: 'Evaluators with ten minutes.', status: 'Planned', tier: 'Planned', date: 'Planned' },
  { number: 13, slug: 'fleet', name: 'Kineti Fleet', detail: 'One view across many projects.', docs: 'Many projects, one view. Postponed until the wedge proves out.', who: 'Teams running Kineti widely.', status: 'Planned', tier: 'Planned', date: 'Planned' },
  { number: 14, slug: 'cloud-link', name: 'Kineti Cloud Link', detail: 'Browser view via one-use codes and encrypted store.', docs: 'The cloud dashboard link. One-use pairing codes, GitHub login, encrypted store. Postponed until the wedge proves out.', who: 'Teams wanting a browser view.', status: 'Planned', tier: 'Planned', date: 'Planned' },
  { number: 15, slug: 'activity-feed', name: 'Kineti Activity Feed', detail: 'Embeddable audit trail.', docs: 'The view-only audit, proof, and spend trail as an embeddable component. Postponed until the wedge proves out.', who: 'Teams embedding proof in their own dashboards.', status: 'Planned', tier: 'Planned', date: 'Planned' },
];

export const tiers = [
  'Shipped',
  'In progress',
  'Planned',
];

export interface Paper {
  number: number;
  title: string;
  author: string;
  field: string;
  detail: string;
  status: string;
  date: string;
  issue?: number;
}

export const papers: Paper[] = [
  { number: 1, title: 'The Autonomous Nervous System', author, field: 'AI agent verification', detail: 'Agents need an OS layer: bounded memory, transactional rollback, execution lineage, spend caps.', status: 'Coming soon', date: 'TBD' },
  { number: 2, title: 'The Physics of Context', author, field: 'AI agent verification', detail: 'Moving context costs ~100x more than computing it. Stop copying payloads between agents.', status: 'Coming soon', date: 'TBD' },
  { number: 3, title: 'Beyond Vector Search', author, field: 'AI agent verification', detail: 'Similarity search goes stale in loops. Memory needs time, cause, and rollback.', status: 'Coming soon', date: 'TBD' },
  { number: 4, title: 'Sensory Reflex and Style', author, field: 'AI agent verification', detail: 'Cheap signals deserve cheap reflexes, not full model calls.', status: 'Coming soon', date: 'TBD' },
  { number: 5, title: 'Outcome Engineering', author, field: 'AI agent verification', detail: 'Score dollars per true outcome, not tickets closed or tokens burned.', status: 'Coming soon', date: 'TBD' },
];

export const fields = ['AI agent verification'];

export const phases: Array<[string, string, string]> = [
  ['Phase 1', 'Proxy Foundation', 'One real MCP tool call passes through with allow/ask/deny policy.'],
  ['Phase 2', 'Exact-Action Policy & Receipts', 'Human approval binds to exact action hash; argument changes void approval.'],
  ['Phase 3', 'Spend Stops + Verifier CLI', 'Independent CLI to verify receipts without running Kineti cloud.'],
  ['Phase 4', 'Hostile 100 Benchmark', 'Public evaluation of 100 hostile tool calls against the gate.'],
  ['Phase 5', 'Reference Workflows & Ecosystem', 'Tested production templates for teams running autonomous coding agents.'],
];

export function issueUrl(title: string, body: string, labels: string): string {
  return `${repo}/issues/new?title=${encodeURIComponent(title)}&body=${encodeURIComponent(body)}&labels=${encodeURIComponent(labels)}`;
}

/** Prefilled request link for an item with no issue yet. */
export function requestUrl(kind: 'tool' | 'paper', name: string): string {
  return issueUrl(
    `[${kind === 'tool' ? 'Tool request' : 'Paper request'}] ${name}`,
    `## Request\n${name}\n\n## Why\n[why this should exist]\n\n## License\nI understand the project is MIT licensed.`,
    kind === 'tool' ? 'tool-request' : 'paper-request',
  );
}
