/** Stable IDs keep preferences independent of display titles. */
export const DEFAULT_SECTION_ORDER = [
  'gpu-power',
  'battery',
  'cpu-cores',
  'network',
  'processes',
] as const;
export type SectionId = (typeof DEFAULT_SECTION_ORDER)[number];
export const CONFIG_SECTION_ORDER = [
  'driver-updates',
  'gpu-mode',
  'power-limits',
  'cpu-boost',
] as const;
export type ConfigSectionId = (typeof CONFIG_SECTION_ORDER)[number];
export type CardSectionId = SectionId | ConfigSectionId;

export const SECTION_LABELS: Record<SectionId, string> = {
  'gpu-power': 'GPU & Power',
  battery: 'Battery',
  'cpu-cores': 'CPU cores',
  network: 'Network',
  processes: 'Top processes',
};

const PREFIX = 'omniscience:section:';
const ORDER_KEY = 'omniscience:section-order';

/** Drop invalid and duplicate IDs, then append any newly introduced sections. */
export function normalizeSectionOrder(value: unknown): SectionId[] {
  const known = new Set<string>(DEFAULT_SECTION_ORDER);
  const saved = Array.isArray(value)
    ? value.filter((id): id is SectionId => typeof id === 'string' && known.has(id))
    : [];
  return [...new Set([...saved, ...DEFAULT_SECTION_ORDER])];
}

export function readSectionOrder(): SectionId[] {
  try {
    return normalizeSectionOrder(JSON.parse(localStorage.getItem(ORDER_KEY) ?? 'null'));
  } catch {
    return [...DEFAULT_SECTION_ORDER];
  }
}

export function saveSectionOrder(order: SectionId[]): void {
  try {
    localStorage.setItem(ORDER_KEY, JSON.stringify(normalizeSectionOrder(order)));
  } catch {
    // Reordering still works for this dashboard session.
  }
}

export function moveSection(order: SectionId[], active: string, over: string): SectionId[] {
  const from = order.findIndex((id) => id === active);
  const to = order.findIndex((id) => id === over);
  if (from < 0 || to < 0 || from === to) return order;
  const next = [...order];
  next.splice(to, 0, next.splice(from, 1)[0]!);
  return next;
}

export function readSectionCollapsed(section: CardSectionId | undefined): boolean {
  if (!section) return false;
  try {
    return localStorage.getItem(`${PREFIX}${section}`) === 'true';
  } catch {
    return false;
  }
}

export function saveSectionCollapsed(section: CardSectionId, collapsed: boolean): void {
  try {
    if (collapsed) localStorage.setItem(`${PREFIX}${section}`, 'true');
    else localStorage.removeItem(`${PREFIX}${section}`);
  } catch {
    // The disclosure still works for this session if storage is unavailable.
  }
}
