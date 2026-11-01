import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { CpuBoostCard, type CpuBoostStatus } from './CpuBoostCard';

const supported: CpuBoostStatus = {
  ac: true,
  plugged: 1,
  plugged_label: 'Enabled',
  battery: 0,
  battery_label: 'Disabled',
  plan: 'Balanced',
  supported: true,
  error: null,
};

it('shows the plan state and updates only after the write returns', async () => {
  let finish: (value: CpuBoostStatus) => void = () => {};
  mockIPC((cmd) =>
    cmd === 'get_cpu_boost'
      ? supported
      : new Promise<CpuBoostStatus>((resolve) => {
          finish = resolve;
        }),
  );
  render(<CpuBoostCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'On' })).toBeEnabled());
  expect(screen.getByRole('button', { name: 'On' })).toHaveAttribute('aria-pressed', 'true');
  expect(screen.getByRole('status')).toHaveTextContent('Power plan: Balanced');
  fireEvent.click(screen.getByRole('button', { name: 'Aggressive' }));
  expect(screen.getByRole('button', { name: 'Aggressive' })).toHaveAttribute(
    'aria-pressed',
    'false',
  );
  finish({ ...supported, plugged: 2, plugged_label: 'Aggressive' });
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Aggressive' })).toHaveAttribute(
      'aria-pressed',
      'true',
    ),
  );
});

it('names other boost modes with no button pressed', async () => {
  mockIPC(() => ({ ...supported, plugged: 3, plugged_label: 'Efficient enabled' }));
  render(<CpuBoostCard />);
  expect(await screen.findByText(/Efficient enabled\./)).toBeInTheDocument();
  for (const button of within(screen.getByRole('group', { name: 'CPU boost' })).getAllByRole(
    'button',
  ))
    expect(button).toHaveAttribute('aria-pressed', 'false');
});

it('shows and writes each power source separately', async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return supported;
  });
  render(<CpuBoostCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'On' })).toBeEnabled());
  expect(screen.getByRole('tab', { name: 'Plugged in' })).toHaveAttribute('aria-selected', 'true');
  fireEvent.click(screen.getByRole('tab', { name: 'Battery' }));
  expect(screen.getByRole('button', { name: 'Off' })).toHaveAttribute('aria-pressed', 'true');
  fireEvent.click(screen.getByRole('button', { name: 'Aggressive' }));
  await waitFor(() => expect(calls).toContainEqual(['set_cpu_boost', { ac: false, mode: 2 }]));
});

it('disables controls and reports errors when unsupported', async () => {
  mockIPC(() => ({
    ...supported,
    supported: false,
    plugged: null,
    battery: null,
    error: 'No power plan',
  }));
  render(<CpuBoostCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('No power plan');
  for (const button of within(screen.getByRole('group', { name: 'CPU boost' })).getAllByRole(
    'button',
  ))
    expect(button).toBeDisabled();
});
