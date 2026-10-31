import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { PowerModeCard, type PowerStatus } from './PowerModeCard';

const supported: PowerStatus = {
  selected: null,
  current: 'balanced',
  supported: true,
  error: null,
};

it('shows the effective mode and waits for a write before changing selection', async () => {
  let finish: (value: PowerStatus) => void = () => {};
  mockIPC((cmd) =>
    cmd === 'get_power_mode'
      ? supported
      : new Promise<PowerStatus>((resolve) => {
          finish = resolve;
        }),
  );
  render(<PowerModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Performance' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Performance' }));
  expect(screen.getByRole('button', { name: 'Performance' })).toHaveAttribute(
    'aria-pressed',
    'false',
  );
  expect(screen.getByRole('button', { name: 'Balanced' })).toBeDisabled();
  finish({ ...supported, selected: 'performance', current: 'performance' });
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Performance' })).toHaveAttribute(
      'aria-pressed',
      'true',
    ),
  );
});

it('marks Auto when selected and disables controls when unsupported', async () => {
  mockIPC(() => ({ ...supported, selected: 'auto', current: 'performance' }));
  const { unmount } = render(<PowerModeCard />);
  await waitFor(() =>
    expect(screen.getByRole('button', { name: 'Auto' })).toHaveAttribute('aria-pressed', 'true'),
  );
  unmount();
  mockIPC(() => ({ ...supported, supported: false, current: null, error: 'No overlay' }));
  render(<PowerModeCard />);
  expect(await screen.findByRole('alert')).toHaveTextContent('No overlay');
  for (const button of within(screen.getByRole('group', { name: 'Power mode' })).getAllByRole(
    'button',
  ))
    expect(button).toBeDisabled();
});

it('retains actual state and reports a refused write', async () => {
  mockIPC((cmd) => {
    if (cmd === 'get_power_mode') return supported;
    throw new Error('Windows refused the power mode');
  });
  render(<PowerModeCard />);
  await waitFor(() => expect(screen.getByRole('button', { name: 'Efficiency' })).toBeEnabled());
  fireEvent.click(screen.getByRole('button', { name: 'Efficiency' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('refused');
  expect(screen.getByRole('button', { name: 'Balanced' })).toHaveAttribute('aria-pressed', 'true');
});
