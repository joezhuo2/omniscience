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

it('shows firmware state when another utility overrides a saved manual selection', async () => {
  mockIPC(() => ({ ...supported, selected: 'min', current: 'med' }));
  render(<GpuModeCard />);
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Med' })).toHaveAttribute('aria-pressed', 'true'),
  );
  expect(screen.getByRole('button', { name: 'Min' })).toHaveAttribute('aria-pressed', 'false');
});

it('describes only the selected GPU mode', async () => {
  mockIPC(() => ({ ...supported, selected: 'min', current: 'min' }));
  render(<GpuModeCard />);
  expect(await screen.findByText('iGPU only: dGPU disabled')).toBeInTheDocument();
  expect(screen.queryByText('Hybrid graphics: both GPUs available')).not.toBeInTheDocument();
  expect(
    screen.queryByText('iGPU on battery, hybrid graphics when plugged in'),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByText('dGPU display mode: requires a supported MUX and Windows restart'),
  ).not.toBeInTheDocument();
});

it('renders no mode description when nothing is selected', async () => {
  mockIPC(() => ({ ...supported, selected: null, current: null, supported: false }));
  render(<GpuModeCard />);
  await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('Requires a supported'));
  expect(screen.queryByText(/GPUs available|dGPU disabled|on battery|display mode/)).toBeNull();
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
