import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, describe, expect, it } from 'vitest';
import {
  ACCENT_KEY,
  COLOR_TOKENS,
  THEME_PRESETS,
  applyAppearance,
  loadAppearance,
  presetLabel,
  saveAppearance,
  syncAccent,
  withPreset,
} from './appearance';

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

describe('themes', () => {
  it('gives every non-default preset the full token set', () => {
    expect(THEME_PRESETS.map((p) => p.label)).toEqual([
      'Dark',
      'Light',
      'Midnight',
      'High contrast',
    ]);
    expect(THEME_PRESETS[0]?.overrides).toEqual({});
    for (const { overrides } of THEME_PRESETS.slice(1)) {
      for (const { name } of COLOR_TOKENS) expect(overrides[name]).toMatch(/^#[0-9a-f]{6}$/);
    }
    const backgrounds = THEME_PRESETS.slice(1).map((p) => p.overrides['--color-island-bg']);
    expect(new Set(backgrounds).size).toBe(3);
  });

  it('replaces colors but keeps font, size and the accent flag', () => {
    const light = THEME_PRESETS[1]!.overrides;
    const next = withPreset(
      { '--color-island-text': '#123456', 'font-size': '110%', [ACCENT_KEY]: '1' },
      light,
    );
    expect(next).toEqual({ ...light, 'font-size': '110%', [ACCENT_KEY]: '1' });
    expect(presetLabel(next)).toBe('Light');
    expect(presetLabel({ 'font-size': '110%' })).toBe('Dark');
    expect(presetLabel({ '--color-island-text': '#123456' })).toBeNull();
  });

  it('applies the light scheme variables and clears them again', () => {
    const style = document.documentElement.style;
    applyAppearance(THEME_PRESETS[1]!.overrides);
    expect(style.getPropertyValue('color-scheme')).toBe('light');
    expect(style.getPropertyValue('--glass-hi')).toBe('0 0 0');
    applyAppearance({});
    expect(style.getPropertyValue('color-scheme')).toBe('');
  });
});

describe('accent color', () => {
  const chart = () => document.documentElement.style.getPropertyValue('--color-chart-1');
  afterEach(() => {
    clearMocks();
    saveAppearance({});
    applyAppearance({});
  });

  it('overrides chart 1 only while the toggle is on', async () => {
    mockIPC((cmd) => (cmd === 'get_accent_color' ? '#aa5500' : null));
    saveAppearance({ [ACCENT_KEY]: '1', '--color-chart-1': '#111111' });
    await syncAccent();
    expect(chart()).toBe('#aa5500');
    saveAppearance({});
    applyAppearance({});
    await syncAccent();
    expect(chart()).toBe('');
  });

  it('ignores a missing or malformed accent', async () => {
    saveAppearance({ [ACCENT_KEY]: '1' });
    mockIPC(() => null);
    await syncAccent();
    expect(chart()).toBe('');
    mockIPC(() => 'red');
    await syncAccent();
    expect(chart()).toBe('');
  });
});
