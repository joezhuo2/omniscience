import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { PowerLimitsCard, type PowerLimitsStatus } from './PowerLimitsCard';

const supported: PowerLimitsStatus = {
  ac: true,
  ac_limits: null,
  battery_limits: null,
  supported: true,
  error: null,
};

it('writes once on release and keeps boost at or above sustained', async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return supported;
  });
  render(<PowerLimitsCard />);
  const spl = screen.getByRole('slider', { name: 'Sustained (SPL)' });
  await waitFor(() => expect(spl).toBeEnabled());
  fireEvent.change(spl, { target: { value: '70' } });
  fireEvent.change(spl, { target: { value: '75' } });
  expect(screen.getByRole('slider', { name: 'Boost (SPPT)' })).toHaveValue('75');
  expect(calls.filter(([c]) => c === 'set_power_limits')).toHaveLength(0);
  fireEvent.pointerUp(spl);
  await waitFor(() =>
    expect(calls.filter(([c]) => c === 'set_power_limits')).toEqual([
      ['set_power_limits', { ac: true, spl: 75, sppt: 75 }],
    ]),
  );
});

it('disables sliders when unsupported', async () => {
  mockIPC(() => ({ ...supported, supported: false, error: 'ASUS driver unavailable' }));
  render(<PowerLimitsCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('ASUS driver unavailable');
  expect(screen.getByRole('slider', { name: 'Sustained (SPL)' })).toBeDisabled();
});

it('selects the active source tab and edits the other profile separately', async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return { ...supported, ac: false, ac_limits: { spl: 60, sppt: 70 } };
  });
  render(<PowerLimitsCard />);
  const battery = screen.getByRole('tab', { name: 'Battery' });
  await waitFor(() => expect(battery).toHaveAttribute('aria-selected', 'true'));
  expect(screen.getByRole('slider', { name: 'Sustained (SPL)' })).toHaveValue('45');
  fireEvent.click(screen.getByRole('tab', { name: 'Plugged in' }));
  const spl = screen.getByRole('slider', { name: 'Sustained (SPL)' });
  expect(spl).toHaveValue('60');
  fireEvent.change(spl, { target: { value: '65' } });
  fireEvent.pointerUp(spl);
  await waitFor(() =>
    expect(calls).toContainEqual(['set_power_limits', { ac: true, spl: 65, sppt: 70 }]),
  );
  fireEvent.click(screen.getByRole('button', { name: 'Reset' }));
  await waitFor(() => expect(calls).toContainEqual(['reset_power_limits', { ac: true }]));
});
