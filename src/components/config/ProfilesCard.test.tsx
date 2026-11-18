import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, expect, it } from 'vitest';
import { useSettingsStore, type Settings } from '../../settings';
import { useToastStore } from '../../toasts';
import type { Profile } from '../../types/generated/Profile';
import { ProfilesCard } from './ProfilesCard';

const profile = (id: string, name: string, extra: Partial<Profile> = {}): Profile => ({
  id,
  name,
  power_mode: null,
  spl_w: null,
  sppt_w: null,
  cpu_boost: null,
  refresh_hz: null,
  fan_mode: null,
  hotkey: '',
  apps: [],
  ...extra,
});

const gaming = profile('gaming', 'Gaming', { power_mode: 'performance', hotkey: 'Ctrl+G' });
const quiet = profile('quiet', 'Quiet');

let calls: [string, unknown][];

const setup = (active_profile = '', handler?: (cmd: string, args: unknown) => unknown) => {
  const settings = { profiles: [gaming, quiet], active_profile } as Settings;
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    if (handler) return handler(cmd, args);
    if (cmd === 'get_settings') return settings;
    if (cmd === 'set_settings') return (args as { settings: Settings }).settings;
  });
  render(<ProfilesCard />);
};

const saved = () =>
  (calls.filter(([c]) => c === 'set_settings').at(-1)![1] as { settings: Settings }).settings;

beforeEach(() => {
  calls = [];
  useSettingsStore.setState({ settings: null, error: null });
  useToastStore.setState({ toasts: [] });
});

afterEach(() => {
  useSettingsStore.setState({ settings: null, error: null });
});

it('lists profiles and marks the active one', async () => {
  setup('quiet');
  expect(await screen.findByText('Gaming')).toBeInTheDocument();
  expect(screen.getAllByText('Active')).toHaveLength(1);
});

it('applies a profile and toasts what was set', async () => {
  setup('', (cmd, args) =>
    cmd === 'apply_profile'
      ? { applied: ['Power mode', 'CPU boost'], errors: [] }
      : cmd === 'get_settings'
        ? { profiles: [gaming], active_profile: '' }
        : args,
  );
  fireEvent.click(await screen.findByRole('button', { name: 'Apply Gaming' }));
  await waitFor(() =>
    expect(useToastStore.getState().toasts[0]?.text).toBe('Gaming applied: Power mode, CPU boost'),
  );
  expect(calls).toContainEqual(['apply_profile', { id: 'gaming' }]);
});

it('includes failures in the apply toast', async () => {
  setup('', (cmd) =>
    cmd === 'apply_profile'
      ? { applied: ['Power mode'], errors: ['Fan mode: unsupported'] }
      : { profiles: [gaming], active_profile: '' },
  );
  fireEvent.click(await screen.findByRole('button', { name: 'Apply Gaming' }));
  await waitFor(() =>
    expect(useToastStore.getState().toasts[0]?.text).toBe(
      'Gaming applied: Power mode\nFan mode: unsupported',
    ),
  );
});

it('toasts when apply is rejected', async () => {
  setup('', (cmd) => {
    if (cmd === 'apply_profile') throw 'System config is off';
    return { profiles: [gaming], active_profile: '' };
  });
  fireEvent.click(await screen.findByRole('button', { name: 'Apply Gaming' }));
  await waitFor(() =>
    expect(useToastStore.getState().toasts[0]?.text).toContain('System config is off'),
  );
});

it('saves the whole profiles array with only the edited field changed', async () => {
  setup();
  fireEvent.click(await screen.findByRole('button', { name: 'Edit Quiet' }));
  fireEvent.click(screen.getByRole('radio', { name: 'Turbo' }));
  await waitFor(() => expect(saved().profiles).toEqual([gaming, { ...quiet, fan_mode: 'turbo' }]));
  const spl = screen.getByRole('textbox', { name: 'Quiet SPL watts' });
  fireEvent.change(spl, { target: { value: '35' } });
  fireEvent.blur(spl);
  await waitFor(() => expect(saved().profiles[1]?.spl_w).toBe(35));
  const apps = screen.getByRole('textbox', { name: 'Quiet apps' });
  fireEvent.change(apps, { target: { value: 'Game.exe, , b.exe' } });
  fireEvent.blur(apps);
  await waitFor(() => expect(saved().profiles[1]?.apps).toEqual(['Game.exe', 'b.exe']));
  expect(saved().profiles[0]).toEqual(gaming);
});

it('clears a hotkey', async () => {
  setup();
  fireEvent.click(await screen.findByRole('button', { name: 'Edit Gaming' }));
  fireEvent.click(screen.getByRole('button', { name: 'Clear Gaming hotkey' }));
  await waitFor(() => expect(saved().profiles[0]?.hotkey).toBe(''));
});

it('adds a profile with a unique id and deletes one', async () => {
  setup();
  fireEvent.click(await screen.findByRole('button', { name: 'Add profile' }));
  await waitFor(() => expect(saved().profiles).toHaveLength(3));
  const ids = saved().profiles.map((p) => p.id);
  expect(new Set(ids).size).toBe(3);
  fireEvent.click(await screen.findByRole('button', { name: 'Delete Quiet' }));
  await waitFor(() => expect(saved().profiles.map((p) => p.id)).not.toContain('quiet'));
});

it('shows the store error when a save is rejected', async () => {
  setup('', (cmd) => {
    if (cmd === 'set_settings') throw 'hotkey already in use';
    return { profiles: [gaming, quiet], active_profile: '' };
  });
  fireEvent.click(await screen.findByRole('button', { name: 'Delete Quiet' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('hotkey already in use');
});
