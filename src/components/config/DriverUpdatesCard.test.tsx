import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it } from 'vitest';
import { DriverUpdatesCard } from './DriverUpdatesCard';

it('checks on request and opens the catalog entry for an offered update', async () => {
  const calls: unknown[] = [];
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    return cmd === 'scan_driver_updates'
      ? [{ id: 'update-id', title: 'ASUS System Control Interface' }]
      : null;
  });
  render(<DriverUpdatesCard />);
  expect(calls).toEqual([]);
  fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
  expect(await screen.findByText('ASUS System Control Interface')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Download' }));
  await waitFor(() => expect(calls).toContainEqual(['open_update_download', { id: 'update-id' }]));
});

it('shows a scoped up-to-date result only after a successful empty scan', async () => {
  mockIPC(() => []);
  render(<DriverUpdatesCard />);
  expect(screen.queryByText(/are up to date/)).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
  expect(await screen.findByText(/offered by Windows Update are up to date/)).toBeInTheDocument();
});

it('clears stale success and allows retry after a failed scan', async () => {
  let fail = false;
  mockIPC(() => {
    if (fail) throw new Error('Windows Update is offline');
    return [];
  });
  render(<DriverUpdatesCard />);
  fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
  await screen.findByText(/are up to date/);
  fail = true;
  fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
  expect(await screen.findByRole('alert')).toHaveTextContent('offline');
  expect(screen.queryByText(/are up to date/)).not.toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Check now' })).toBeEnabled();
});
