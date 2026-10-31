import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { PowerLimitsCard, type PowerLimitsStatus } from './PowerLimitsCard';

const supported: PowerLimitsStatus = { spl: null, sppt: null, supported: true, error: null };

it('writes once on release and keeps boost at or above sustained', async () => {
  const calls: [string, unknown][] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return cmd === 'set_power_limits' ? { ...supported, ...(args as object) } : supported;
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
      ['set_power_limits', { spl: 75, sppt: 75 }],
    ]),
  );
});

it('disables sliders when unsupported', async () => {
  mockIPC(() => ({ ...supported, supported: false, error: 'ASUS driver unavailable' }));
  render(<PowerLimitsCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('ASUS driver unavailable');
  expect(screen.getByRole('slider', { name: 'Sustained (SPL)' })).toBeDisabled();
});
