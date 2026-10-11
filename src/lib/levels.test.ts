import { describe, expect, it } from 'vitest';
import { ACCENTS, CRITICAL, WARNING, levelColor } from './levels';

describe('levelColor', () => {
  it('uses the accent when normal', () => {
    expect(levelColor('normal', ACCENTS.cpu)).toBe('#22D3EE');
  });

  it('switches to orange and red above the thresholds', () => {
    expect(levelColor('warning', ACCENTS.cpu)).toBe(WARNING);
    expect(levelColor('critical', ACCENTS.ram)).toBe(CRITICAL);
  });

  it('keeps accents clear of the alert colors', () => {
    for (const accent of Object.values(ACCENTS)) {
      expect([WARNING, CRITICAL]).not.toContain(accent);
    }
  });
});
