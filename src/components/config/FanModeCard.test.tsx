import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { FanModeCard, type FanModeStatus } from './FanModeCard';

const supported: FanModeStatus = { mode: 'performance', supported: true, error: null };

it('shows the current mode and sets a new one', async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return cmd === 'get_fan_mode' ? supported : { ...supported, mode: 'turbo' };
  });
  render(<FanModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Turbo' })).toBeEnabled());
  expect(screen.getByRole('button', { name: 'Performance' })).toHaveAttribute(
    'aria-pressed',
    'true',
  );
  fireEvent.click(screen.getByRole('button', { name: 'Turbo' }));
  await waitFor(() => expect(calls).toContainEqual(['set_fan_mode', { mode: 'turbo' }]));
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Turbo' })).toHaveAttribute('aria-pressed', 'true'),
  );
});

it('disables controls when unsupported', async () => {
  mockIPC(() => ({ mode: null, supported: false, error: null }));
  render(<FanModeCard />);
  expect(await screen.findByText(/Requires a supported ASUS/)).toBeInTheDocument();
  for (const name of ['Silent', 'Performance', 'Turbo'])
    expect(screen.getByRole('button', { name })).toBeDisabled();
});

it('reports a refused write', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_fan_mode') return supported;
    throw new Error('Fan write refused');
  });
  render(<FanModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Silent' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Silent' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('refused');
});
