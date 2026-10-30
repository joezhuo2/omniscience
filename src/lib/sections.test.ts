import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  DEFAULT_SECTION_ORDER,
  moveSection,
  normalizeSectionOrder,
  readSectionOrder,
  saveSectionOrder,
  readSectionCollapsed,
  saveSectionCollapsed,
} from './sections';

afterEach(() => {
  vi.restoreAllMocks();
  localStorage.clear();
});

describe('section ordering', () => {
  it('repairs partial, duplicated and unknown saved sections', () => {
    expect(normalizeSectionOrder(['network', 'network', 'unknown', 42, 'processes'])).toEqual([
      'network',
      'processes',
      'gpu-power',
      'battery',
      'cpu-cores',
    ]);
    expect(normalizeSectionOrder({})).toEqual([...DEFAULT_SECTION_ORDER]);
    localStorage.setItem('omniscience:section-order', '{invalid');
    expect(readSectionOrder()).toEqual([...DEFAULT_SECTION_ORDER]);
  });

  it('moves cards without losing hidden sections or changing minimize preferences', () => {
    const original = [...DEFAULT_SECTION_ORDER];
    const moved = moveSection(original, 'processes', 'gpu-power');
    expect(moved).toEqual(['processes', 'gpu-power', 'battery', 'cpu-cores', 'network']);
    expect(original).toEqual([...DEFAULT_SECTION_ORDER]);
    saveSectionCollapsed('network', true);
    saveSectionOrder(moved);
    expect(readSectionOrder()).toEqual(moved);
    expect(readSectionCollapsed('network')).toBe(true);
    expect(moveSection(moved, 'unknown', 'network')).toBe(moved);
    expect(moveSection(moved, 'network', 'network')).toBe(moved);
  });

  it('survives denied preference storage', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('blocked');
    });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('blocked');
    });
    expect(readSectionOrder()).toEqual([...DEFAULT_SECTION_ORDER]);
    expect(() => saveSectionOrder([...DEFAULT_SECTION_ORDER])).not.toThrow();
  });
});

describe('section preferences', () => {
  it('defaults to expanded and remembers each section independently', () => {
    expect(readSectionCollapsed(undefined)).toBe(false);
    expect(readSectionCollapsed('cpu-cores')).toBe(false);
    saveSectionCollapsed('cpu-cores', true);
    expect(readSectionCollapsed('cpu-cores')).toBe(true);
    expect(readSectionCollapsed('network')).toBe(false);
    saveSectionCollapsed('cpu-cores', false);
    expect(readSectionCollapsed('cpu-cores')).toBe(false);
  });

  it('ignores invalid stored values', () => {
    localStorage.setItem('omniscience:section:battery', 'broken');
    expect(readSectionCollapsed('battery')).toBe(false);
  });

  it('falls back gracefully when storage is blocked or full', () => {
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('blocked');
    });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('full');
    });
    vi.spyOn(Storage.prototype, 'removeItem').mockImplementation(() => {
      throw new Error('blocked');
    });
    expect(readSectionCollapsed('network')).toBe(false);
    expect(() => saveSectionCollapsed('network', true)).not.toThrow();
    expect(() => saveSectionCollapsed('network', false)).not.toThrow();
  });
});
