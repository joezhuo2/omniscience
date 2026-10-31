import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { GpuModeCard, type GpuStatus } from './GpuModeCard';

const supported: GpuStatus = {
  selected: null,
  current: 'med',
  supported: true,
  mux_supported: true,
  restart_required: false,
  error: null,
};

it('shows confirmed firmware state and waits for a write before changing selection', async () => {
  let finish: (value: GpuStatus) => void = () => {};
  mockIPC((cmd) =>
    cmd === 'get_gpu_mode'
      ? supported
      : new Promise<GpuStatus>((resolve) => {
          finish = resolve;
        }),
  );
  render(<GpuModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Min' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Min' }));
  expect(screen.getByRole('button', { name: 'Min' })).toHaveAttribute('aria-pressed', 'false');
  expect(screen.getByRole('button', { name: 'Med' })).toBeDisabled();
  finish({ ...supported, selected: 'min', current: 'min' });
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Min' })).toHaveAttribute('aria-pressed', 'true'),
  );
});

it('disables unavailable controls and reports firmware failures', async () => {
  mockIPC(() => ({
    ...supported,
    supported: false,
    current: null,
    error: 'ASUS driver unavailable',
  }));
  render(<GpuModeCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('ASUS driver unavailable');
  for (const button of within(screen.getByRole('group', { name: 'GPU mode' })).getAllByRole(
    'button',
  ))
    expect(button).toBeDisabled();
});

it('requires restart after a MUX change and prevents further GPU writes', async () => {
  mockIPC((cmd) =>
    cmd === 'get_gpu_mode'
      ? supported
      : { ...supported, selected: 'max', current: 'max', restart_required: true },
  );
  render(<GpuModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Max' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Max' }));
  expect(await screen.findByText(/Auto switching is paused/)).toBeInTheDocument();
  for (const button of within(screen.getByRole('group', { name: 'GPU mode' })).getAllByRole(
    'button',
  ))
    expect(button).toBeDisabled();
});

it('retains actual state and reports a refused write', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_gpu_mode') return supported;
    throw new Error('Firmware refused GPU mode');
  });
  render(<GpuModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Min' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Min' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('Firmware refused');
  expect(screen.getByRole('button', { name: 'Med' })).toHaveAttribute('aria-pressed', 'true');
});
