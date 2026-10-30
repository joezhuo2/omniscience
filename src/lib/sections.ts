/** Stable IDs keep preferences independent of display titles. */
export type SectionId = 'gpu-power' | 'battery' | 'cpu-cores' | 'network' | 'processes';

const PREFIX = 'omniscience:section:';

export function readSectionCollapsed(section: SectionId | undefined): boolean {
  if (!section) return false;
  try {
    return localStorage.getItem(`${PREFIX}${section}`) === 'true';
  } catch {
    return false;
  }
}

export function saveSectionCollapsed(section: SectionId, collapsed: boolean): void {
  try {
    if (collapsed) localStorage.setItem(`${PREFIX}${section}`, 'true');
    else localStorage.removeItem(`${PREFIX}${section}`);
  } catch {
    // The disclosure still works for this session if storage is unavailable.
  }
}
