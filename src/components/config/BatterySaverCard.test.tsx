import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { BatterySaverCard, type BatterySaverStatus } from './BatterySaverCard';

const supported: BatterySaverStatus = { on: false, threshold: 20, supported: true, error: null };

it('shows the state and threshold, and writes each control', async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return cmd === 'set_battery_saver' ? { ...supported, on: true } : supported;
  });
  render(<BatterySaverCard />);
  const select = await screen.findByRole('combobox', { name: 'Turn on at' });
  await waitFor(() => expect(select).toBeEnabled());
  expect(select).toHaveValue('20');
  expect(screen.getByRole('option', { name: 'Off' })).toHaveValue('0');
  expect(screen.getByRole('status')).toHaveTextContent('Battery saver is off.');
  fireEvent.change(select, { target: { value: '50' } });
  await waitFor(() =>
    expect(calls).toContainEqual(['set_battery_saver_threshold', { threshold: 50 }]),
  );
  fireEvent.click(screen.getByRole('button', { name: 'Turn on' }));
  await waitFor(() => expect(calls).toContainEqual(['set_battery_saver', { on: true }]));
  expect(await screen.findByRole('button', { name: 'Turn off' })).toBeEnabled();
  expect(screen.getByRole('status')).toHaveTextContent('Battery saver is on.');
});

it('lists a threshold set elsewhere', async () => {
  mockIPC(() => ({ ...supported, threshold: 35 }));
  render(<BatterySaverCard />);
  expect(await screen.findByRole('option', { name: '35%' })).toBeInTheDocument();
  expect(screen.getByRole('combobox', { name: 'Turn on at' })).toHaveValue('35');
});

it('disables controls and reports errors when unsupported', async () => {
  mockIPC(() => ({ on: null, threshold: null, supported: false, error: 'No power plan' }));
  render(<BatterySaverCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('No power plan');
  expect(screen.getByRole('combobox', { name: 'Turn on at' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Turn on' })).toBeDisabled();
});
