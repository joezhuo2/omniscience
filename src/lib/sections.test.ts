import { afterEach, describe, expect, it, vi } from 'vitest';
import { readSectionCollapsed, saveSectionCollapsed } from './sections';

afterEach(() => {
  vi.restoreAllMocks();
  localStorage.clear();
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
