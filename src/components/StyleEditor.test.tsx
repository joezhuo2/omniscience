import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  ACCENT_KEY,
  THEME_PRESETS,
  applyAppearance,
  loadAppearance,
  saveAppearance,
} from '../lib/appearance';
import { StyleEditor } from './StyleEditor';

Element.prototype.scrollIntoView = vi.fn();

describe('StyleEditor themes', () => {
  beforeEach(() => {
    mockIPC((cmd) => (cmd === 'get_accent_color' ? '#cc3300' : null));
  });

  afterEach(() => {
    clearMocks();
    saveAppearance({});
    applyAppearance({});
  });

  it('picks a preset and keeps the font and size', async () => {
    saveAppearance({ 'font-size': '110%', '--color-island-text': '#123456' });
    render(<StyleEditor onBack={() => {}} />);
    fireEvent.click(screen.getByRole('combobox', { name: 'Theme' }));
    fireEvent.click(await screen.findByRole('option', { name: 'Light' }));
    expect(loadAppearance()).toEqual({ ...THEME_PRESETS[1]!.overrides, 'font-size': '110%' });
    expect(document.documentElement.style.getPropertyValue('--color-island-bg')).toBe('#f5f5f7');
  });

  it('goes back to the default with Dark', async () => {
    saveAppearance(THEME_PRESETS[2]!.overrides);
    render(<StyleEditor onBack={() => {}} />);
    fireEvent.click(screen.getByRole('combobox', { name: 'Theme' }));
    fireEvent.click(await screen.findByRole('option', { name: 'Dark' }));
    expect(loadAppearance()).toEqual({});
  });

  it('stores the accent toggle and applies the Windows accent', async () => {
    render(<StyleEditor onBack={() => {}} />);
    fireEvent.click(screen.getByLabelText('Follow Windows accent color'));
    expect(loadAppearance()[ACCENT_KEY]).toBe('1');
    await waitFor(() =>
      expect(document.documentElement.style.getPropertyValue('--color-chart-1')).toBe('#cc3300'),
    );
    fireEvent.click(screen.getByLabelText('Follow Windows accent color'));
    expect(loadAppearance()[ACCENT_KEY]).toBeUndefined();
    expect(document.documentElement.style.getPropertyValue('--color-chart-1')).toBe('');
  });
});
