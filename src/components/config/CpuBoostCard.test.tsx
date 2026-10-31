import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { CpuBoostCard, type CpuBoostStatus } from './CpuBoostCard';

const supported: CpuBoostStatus = {
  current: 1,
  label: 'Enabled',
  plan: 'Balanced',
  mismatch: false,
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
  finish({ ...supported, current: 2, label: 'Aggressive' });
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Aggressive' })).toHaveAttribute(
      'aria-pressed',
      'true',
    ),
  );
});

it('names other boost modes and flags AC/battery mismatch with no button pressed', async () => {
  mockIPC(() => ({ ...supported, current: 3, label: 'Efficient enabled', mismatch: true }));
  render(<CpuBoostCard />);
  expect(await screen.findByText(/Efficient enabled\./)).toBeInTheDocument();
  expect(screen.getByText(/different boost settings/)).toBeInTheDocument();
  for (const button of within(screen.getByRole('group', { name: 'CPU boost' })).getAllByRole(
    'button',
  ))
    expect(button).toHaveAttribute('aria-pressed', 'false');
});

it('disables controls and reports errors when unsupported', async () => {
  mockIPC(() => ({ ...supported, supported: false, current: null, error: 'No power plan' }));
  render(<CpuBoostCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('No power plan');
  for (const button of within(screen.getByRole('group', { name: 'CPU boost' })).getAllByRole(
    'button',
  ))
    expect(button).toBeDisabled();
});
