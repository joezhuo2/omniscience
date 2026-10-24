import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';
import { useSettingsStore, type Settings } from '../../settings';
import { SettingsView } from './SettingsView';

const defaults: Settings = {
  warn_threshold: 70,
  crit_threshold: 90,
  poll_interval_ms: 1000,
  hotkey: 'CommandOrControl+Shift+O',
  launch_at_startup: false,
  auto_check_updates: true,
  hide_over_fullscreen: true,
  ping_target: '1.1.1.1',
  show_public_ip: false,
};

describe('SettingsView', () => {
  let saved: Settings[];

  beforeEach(() => {
    saved = [];
    useSettingsStore.setState({ settings: null, error: null });
    mockIPC((cmd, args) => {
      if (cmd === 'get_settings') return defaults;
      if (cmd === 'set_settings') {
        const s = (args as { settings: Settings }).settings;
        saved.push(s);
        return s;
      }
      return null;
    });
  });

  it('loads and shows current settings', async () => {
    render(<SettingsView onBack={() => {}} />);
    expect(await screen.findByLabelText('Warning threshold')).toHaveValue(70);
    expect(screen.getByLabelText('Show/hide hotkey')).toHaveTextContent('Ctrl+Shift+O');
    expect(screen.getByLabelText('Hide over fullscreen apps')).toHaveAttribute(
      'aria-checked',
      'true',
    );
  });

  it('saves a toggle immediately', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByLabelText('Launch at startup'));
    await waitFor(() => expect(saved.at(-1)?.launch_at_startup).toBe(true));
  });

  it('commits a threshold on blur, not on every keystroke', async () => {
    render(<SettingsView onBack={() => {}} />);
    const input = await screen.findByLabelText('Critical threshold');
    fireEvent.change(input, { target: { value: '8' } });
    fireEvent.change(input, { target: { value: '85' } });
    expect(saved).toHaveLength(0);
    fireEvent.blur(input);
    await waitFor(() => expect(saved.at(-1)?.crit_threshold).toBe(85));
    expect(saved).toHaveLength(1);
  });

  it('commits a trimmed ping target on Enter', async () => {
    render(<SettingsView onBack={() => {}} />);
    const input = await screen.findByLabelText('Ping target');
    fireEvent.change(input, { target: { value: ' 8.8.8.8 ' } });
    expect(saved).toHaveLength(0);
    fireEvent.keyDown(input, { key: 'Enter' });
    await waitFor(() => expect(saved.at(-1)?.ping_target).toBe('8.8.8.8'));
  });

  it('turns the public IP lookup on', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByLabelText('Show public IP'));
    await waitFor(() => expect(saved.at(-1)?.show_public_ip).toBe(true));
  });

  it('steps a threshold with the themed arrows', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByLabelText('Increase Warning threshold'));
    await waitFor(() => expect(saved.at(-1)?.warn_threshold).toBe(71));
  });

  it('picks the update interval from a segmented control', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByRole('radio', { name: '2 s' }));
    await waitFor(() => expect(saved.at(-1)?.poll_interval_ms).toBe(2000));
    await waitFor(() =>
      expect(screen.getByRole('radio', { name: '2 s' })).toHaveAttribute('aria-checked', 'true'),
    );
  });

  it('records a new hotkey', async () => {
    render(<SettingsView onBack={() => {}} />);
    fireEvent.click(await screen.findByLabelText('Show/hide hotkey'));
    expect(screen.getByLabelText('Show/hide hotkey')).toHaveTextContent('Press keys');
    fireEvent.keyDown(window, { key: 'K', code: 'KeyK', altKey: true, shiftKey: true });
    await waitFor(() => expect(saved.at(-1)?.hotkey).toBe('Alt+Shift+K'));
  });
});
