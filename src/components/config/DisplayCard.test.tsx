import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { DisplayCard, type DisplayStatus } from './DisplayCard';

const supported: DisplayStatus = {
  device: 'Built-in Display',
  current: { width: 2560, height: 1600, hz: 120 },
  resolutions: [
    [2560, 1600],
    [1920, 1200],
  ],
  rates: [60, 120],
  auto: false,
  supported: true,
  error: null,
};

it('lists modes and waits for the re-read state before changing selection', async () => {
  let finish: (value: DisplayStatus) => void = () => {};
  const calls: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === 'get_display_mode') return supported;
    calls.push(args);
    return new Promise<DisplayStatus>((resolve) => {
      finish = resolve;
    });
  });
  render(<DisplayCard />);
  expect(await screen.findByText('Built-in Display')).toBeInTheDocument();
  await waitFor(() => expect(screen.getByRole('button', { name: '60 Hz' })).toBeEnabled());
  expect(screen.getByRole('button', { name: '120 Hz' })).toHaveAttribute('aria-pressed', 'true');
  expect(screen.getByRole('button', { name: '2560×1600' })).toHaveAttribute('aria-pressed', 'true');
  fireEvent.click(screen.getByRole('button', { name: '60 Hz' }));
  expect(calls).toEqual([{ hz: 60 }]);
  expect(screen.getByRole('button', { name: '60 Hz' })).toHaveAttribute('aria-pressed', 'false');
  finish({ ...supported, current: { width: 2560, height: 1600, hz: 60 } });
  await waitFor(() =>
    expect(screen.getByRole('button', { name: '60 Hz' })).toHaveAttribute('aria-pressed', 'true'),
  );
});

it('marks Auto and clears manual rate highlight when automatic is active', async () => {
  mockIPC(() => ({ ...supported, auto: true }));
  render(<DisplayCard />);
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Auto' })).toHaveAttribute('aria-pressed', 'true'),
  );
  expect(screen.getByRole('button', { name: '120 Hz' })).toHaveAttribute('aria-pressed', 'false');
});

it('disables controls and reports unsupported displays', async () => {
  mockIPC(() => ({
    ...supported,
    supported: false,
    current: null,
    rates: [],
    resolutions: [],
    error: 'No display',
  }));
  render(<DisplayCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('No display');
  for (const button of within(screen.getByRole('group', { name: 'Refresh rate' })).getAllByRole(
    'button',
  ))
    expect(button).toBeDisabled();
});

it('retains actual state and reports a refused write', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_display_mode') return supported;
    throw new Error('Windows rejected the display mode');
  });
  render(<DisplayCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: '1920×1200' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: '1920×1200' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('rejected');
  expect(screen.getByRole('button', { name: '2560×1600' })).toHaveAttribute('aria-pressed', 'true');
});
