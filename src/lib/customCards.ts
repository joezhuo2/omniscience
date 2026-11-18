export interface CardSources {
  core: unknown;
  detail: unknown;
  data: unknown;
}

const MISSING = '--';

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
