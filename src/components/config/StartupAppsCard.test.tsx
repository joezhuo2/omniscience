import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import type { StartupApp } from '../../types/telemetry';
import { StartupAppsCard } from './StartupAppsCard';

const apps: StartupApp[] = [
  {
    id: 'u1',
    name: 'Discord',
    command: 'C:\\discord.exe',
    source: 'user',
    enabled: true,
    can_toggle: true,
  },
  {
    id: 'm1',
    name: 'Vendor Tool',
    command: 'tool.exe',
    source: 'machine',
    enabled: false,
    can_toggle: false,
  },
  {
    id: 'f1',
    name: 'Sync',
    command: 'sync.lnk',
    source: 'folder',
    enabled: false,
    can_toggle: true,
  },
];

it('lists apps with their source and toggles one', async () => {
  const calls: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === 'get_startup_apps') return apps;
    calls.push(args);
    return apps.map((a) => (a.id === 'f1' ? { ...a, enabled: true } : a));
  });
  render(<StartupAppsCard />);
  expect(await screen.findByText('Discord')).toBeInTheDocument();
  expect(screen.getByText('User')).toBeInTheDocument();
  expect(screen.getByText('All users')).toBeInTheDocument();
  expect(screen.getByText('Startup folder')).toBeInTheDocument();
  expect(screen.getByRole('switch', { name: 'Discord runs at startup' })).toBeChecked();
  fireEvent.click(screen.getByRole('switch', { name: 'Sync runs at startup' }));
  expect(calls).toEqual([{ id: 'f1', enabled: true }]);
  await waitFor(() =>
    expect(screen.getByRole('switch', { name: 'Sync runs at startup' })).toBeChecked(),
  );
});

it('disables entries that need administrator rights', async () => {
  mockIPC(() => apps);
  render(<StartupAppsCard />);
  expect(await screen.findByRole('switch', { name: 'Vendor Tool runs at startup' })).toBeDisabled();
  expect(screen.getByRole('switch', { name: 'Discord runs at startup' })).toBeEnabled();
});

it('shows an empty state', async () => {
  mockIPC(() => []);
  render(<StartupAppsCard />);
  expect(await screen.findByText('No startup apps.')).toBeInTheDocument();
});

it('shows read errors', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_startup_apps') throw 'System config is off';
    return null;
  });
  render(<StartupAppsCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('System config is off');
});

it('shows the error when a toggle is refused', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_startup_apps') return apps;
    throw 'Needs administrator rights';
  });
  render(<StartupAppsCard />);
  fireEvent.click(await screen.findByRole('switch', { name: 'Sync runs at startup' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Needs administrator rights');
});
