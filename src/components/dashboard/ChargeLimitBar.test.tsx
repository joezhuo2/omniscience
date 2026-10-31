import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { ChargeLimitBar, type ChargeLimitStatus } from './ChargeLimitBar';

const supported: ChargeLimitStatus = { limit: 100, supported: true, error: null };

it('writes only on release and shows the value the backend reports', async () => {
  const writes: unknown[] = [];
  mockIPC((cmd, args) => {
    if (cmd === 'get_charge_limit') return supported;
    writes.push(args);
    return { ...supported, limit: 60 };
  });
  render(<ChargeLimitBar />);
  const slider = await screen.findByRole('slider', { name: 'Charge limit' });
  fireEvent.change(slider, { target: { value: '80' } });
  fireEvent.change(slider, { target: { value: '65' } });
  expect(screen.getByText('Charge limit 65%')).toBeInTheDocument();
  expect(writes).toHaveLength(0);
  fireEvent.pointerUp(slider);
  await waitFor(() => expect(screen.getByText('Charge limit 60%')).toBeInTheDocument());
  expect(writes).toEqual([{ limit: 65 }]);
});

it('renders nothing when unsupported', async () => {
  mockIPC(() => ({ limit: 100, supported: false, error: 'ASUS driver unavailable' }));
  const { container } = render(<ChargeLimitBar />);
  await new Promise((r) => setTimeout(r, 20));
  expect(container).toBeEmptyDOMElement();
});
