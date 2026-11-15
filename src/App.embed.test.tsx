import { mockIPC } from '@tauri-apps/api/mocks';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import App from './App';
import { initEmbed, invoke } from './lib/ipc';
import { useSettingsStore } from './settings';

describe('embedded mode', () => {
  let calls: { cmd: string; args: unknown }[];

  beforeEach(() => {
    window.history.replaceState(null, '', '/?embed=1');
    useSettingsStore.setState({ settings: null, error: null });
    calls = [];
    mockIPC(
      (cmd, args) => {
        calls.push({ cmd, args });
        if (cmd.endsWith('get_settings'))
          return { history_hours: 6, config_enabled: true, ping_target: '' };
        if (cmd.endsWith('get_telemetry_history')) return [];
        return null;
      },
      { shouldMockEvents: true },
    );
  });

  afterEach(() => {
    window.history.replaceState(null, '', '/');
    document.documentElement.classList.remove('embed');
  });

  it('routes commands through the omniscience plugin only when embedded', async () => {
    await invoke('get_settings');
    window.history.replaceState(null, '', '/');
    await invoke('get_settings');
    expect(calls.map((c) => c.cmd)).toEqual(['plugin:omniscience|get_settings', 'get_settings']);
  });

  it('marks the document so it scrolls with a transparent background', () => {
    initEmbed();
    expect(document.documentElement).toHaveClass('embed');
  });

  it('renders the open dashboard without the pill or config tabs and samples while visible', async () => {
    const { unmount } = render(<App />);
    expect(screen.getByTestId('embedded')).toBeInTheDocument();
    expect(screen.queryByTestId('island')).not.toBeInTheDocument();
    expect(screen.queryByRole('tab', { name: 'config' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /pin/i })).not.toBeInTheDocument();
    await waitFor(() =>
      expect(calls).toContainEqual({
        cmd: 'plugin:omniscience|set_expanded',
        args: { expanded: true },
      }),
    );
    await waitFor(() => expect(screen.getByRole('radio', { name: '24h' })).toBeDisabled());
    expect(screen.getByRole('radio', { name: '6h' })).toBeChecked();
    act(() => unmount());
    expect(calls.filter((c) => c.cmd.endsWith('set_expanded')).at(-1)).toEqual({
      cmd: 'plugin:omniscience|set_expanded',
      args: { expanded: false },
    });
    expect(calls.filter((c) => c.cmd.endsWith('set_process_sampling')).at(-1)).toEqual({
      cmd: 'plugin:omniscience|set_process_sampling',
      args: { enabled: false },
    });
  });

  it('hides standalone-only settings', async () => {
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Settings' }));
    await screen.findByText('Keep history for');
    expect(screen.queryByText('Show/hide hotkey')).not.toBeInTheDocument();
    expect(screen.queryByText('Launch at startup')).not.toBeInTheDocument();
    expect(screen.queryByText('Hide over fullscreen apps')).not.toBeInTheDocument();
    expect(screen.queryByText('Auto-check for updates')).not.toBeInTheDocument();
    expect(screen.queryByText(/System config/)).not.toBeInTheDocument();
    expect(screen.queryByText('Reset to top center')).not.toBeInTheDocument();
    expect(screen.getAllByText('Sustained usage alerts')[0]).toBeInTheDocument();
  });
});
