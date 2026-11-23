export interface CardSources {
  core: unknown;
  detail: unknown;
  data: unknown;
}

const MISSING = '--';

/** Dashboard section whose sampling provides each `detail` field. */
const DETAIL_SECTIONS: Record<string, string> = {
  gpus: 'gpu-power',
  power: 'gpu-power',
  cores: 'cpu-cores',
  cpu_limit_pct: 'cpu-cores',
  net_adapters: 'network',
  ping_ms: 'network',
  ping_jitter_ms: 'network',
  ping_loss_pct: 'network',
  public_ip: 'network',
  wifi: 'network',
  metered: 'network',
  drives: 'drives',
  disks: 'drives',
  memory: 'memory',
  power_events: 'system',
  bluetooth: 'bluetooth',
  fans: 'fans',
  processes: 'processes',
};

/** Sections a card's templates read from, so they are sampled while the card is open. */
export function sampledSections(templates: readonly string[]): string[] {
  const out = new Set<string>();
  for (const t of templates)
    for (const [, field] of t.matchAll(/\{\s*detail\.(\w+)/g)) {
      const section = field && DETAIL_SECTIONS[field];
      if (section) out.add(section);
    }
  return [...out];
}

function lookup(sources: CardSources, path: string): unknown {
  let node: unknown = sources;
  for (const key of path.split('.')) {
    if (typeof node !== 'object' || node === null || !Object.hasOwn(node, key)) return undefined;
    node = (node as Record<string, unknown>)[key];
  }
  return node;
}

function show(value: unknown, decimals: number | null): string {
  if (typeof value === 'number') {
    if (!Number.isFinite(value)) return MISSING;
    return decimals === null ? String(value) : value.toFixed(decimals);
  }
  if (typeof value === 'string' || typeof value === 'boolean') return String(value);
  return MISSING;
}

export function fillTemplate(template: string, sources: CardSources): string {
  return template.replace(/\{([^{}|]*)(?:\|(\d+))?\}/g, (_, path: string, decimals?: string) =>
    show(lookup(sources, path.trim()), decimals === undefined ? null : Number(decimals)),
  );
}
