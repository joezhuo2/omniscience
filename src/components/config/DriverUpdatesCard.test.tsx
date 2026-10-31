import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { expect, it, vi } from 'vitest';
import { DriverUpdatesCard, type ScanStatus } from './DriverUpdatesCard';

const idle: ScanStatus = { phase: 'idle', updates: [], error: null };
const checking: ScanStatus = { phase: 'checking', updates: [], error: null };
const complete: ScanStatus = {
  phase: 'complete',
  updates: [{ id: 'update-id', title: 'ASUS System Control Interface' }],
  error: null,
};

it('remembers Config card collapse state and hides its controls', () => {
  const first = render(<DriverUpdatesCard />);
  const toggle = screen.getByRole('button', { name: 'Driver & software updates' });
  fireEvent.click(toggle);
  expect(toggle).toHaveAttribute('aria-expanded', 'false');
  expect(screen.queryByRole('button', { name: 'Check now' })).not.toBeInTheDocument();
  expect(document.getElementById(toggle.getAttribute('aria-controls')!)).toHaveAttribute('inert');
  first.unmount();
  render(<DriverUpdatesCard />);
  expect(screen.getByRole('button', { name: 'Driver & software updates' })).toHaveAttribute(
    'aria-expanded',
    'false',
  );
  fireEvent.click(screen.getByRole('button', { name: 'Driver & software updates' }));
  expect(screen.getByRole('button', { name: 'Driver & software updates' })).toHaveAttribute(
    'aria-expanded',
    'true',
  );
});

it('starts only on request and opens the catalog entry for an offered update', async () => {
  const calls: unknown[] = [];
  let status = idle;
  mockIPC((cmd, args) => {
    calls.push([cmd, args]);
    if (cmd === 'scan_driver_updates') status = complete;
    return cmd === 'open_update_download' ? null : status;
  });
  render(<DriverUpdatesCard />);
  expect(calls.some((c) => (c as string[])[0] === 'scan_driver_updates')).toBe(false);
  fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
  expect(await screen.findByText('ASUS System Control Interface')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Download' }));
  await waitFor(() => expect(calls).toContainEqual(['open_update_download', { id: 'update-id' }]));
});

it('shows a scoped up-to-date result only after a successful empty scan', async () => {
  let status = idle;
  mockIPC((cmd) => {
    if (cmd === 'scan_driver_updates') status = { ...complete, updates: [] };
    return status;
  });
  render(<DriverUpdatesCard />);
  expect(screen.queryByText(/are up to date/)).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
  expect(await screen.findByText(/offered by Windows Update are up to date/)).toBeInTheDocument();
});

it('shows checking after reopening and retains finished results without another scan', async () => {
  vi.useFakeTimers();
  try {
    let status = idle;
    let starts = 0;
    mockIPC((cmd) => {
      if (cmd === 'scan_driver_updates') {
        starts++;
        status = checking;
      }
      return status;
    });
    const first = render(<DriverUpdatesCard />);
    await act(async () => {});
    fireEvent.click(screen.getByRole('button', { name: 'Check now' }));
    await act(async () => {});
    first.unmount();
    const second = render(<DriverUpdatesCard />);
    await act(async () => {});
    expect(screen.getByText(/Checking for updates/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Checking…' })).toBeDisabled();
    status = complete;
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1100);
    });
    expect(screen.getByText('ASUS System Control Interface')).toBeInTheDocument();
    second.unmount();
    render(<DriverUpdatesCard />);
    await act(async () => {});
    expect(screen.getByText('ASUS System Control Interface')).toBeInTheDocument();
    expect(starts).toBe(1);
  } finally {
    vi.useRealTimers();
  }
});

it('clears stale success and allows retry after a failed background scan', async () => {
  let fail = false;
  let status = idle;
  mockIPC((cmd) => {
    if (cmd === 'scan_driver_updates')
      status = fail
        ? { phase: 'failed', updates: [], error: 'Windows Update is offline' }
        : { ...complete, updates: [] };
    return status;
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
