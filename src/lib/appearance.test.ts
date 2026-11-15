import { describe, expect, it } from 'vitest';
import { applyAppearance, loadAppearance, saveAppearance } from './appearance';

describe('appearance', () => {
  it('round-trips overrides through storage', () => {
    expect(loadAppearance()).toEqual({});
    saveAppearance({ '--color-island-text': '#112233' });
    expect(loadAppearance()).toEqual({ '--color-island-text': '#112233' });
    saveAppearance({});
    expect(loadAppearance()).toEqual({});
  });

  it('sets and clears a CSS variable', () => {
    const style = document.documentElement.style;
    applyAppearance({ '--color-island-text': '#112233' });
    expect(style.getPropertyValue('--color-island-text')).toBe('#112233');
    applyAppearance({});
    expect(style.getPropertyValue('--color-island-text')).toBe('');
  });
});
