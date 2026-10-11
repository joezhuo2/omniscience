import { mockIPC } from '@tauri-apps/api/mocks';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import { useToastStore } from '../../toasts';
import type { KillReport } from '../../types/telemetry';
import { Dashboard } from './Dashboard';
import { CONFIRM_MS } from '../../lib/processes';

const withDetail = { ...sampleSnapshot, detail: sampleDetail };

describe('Dashboard', () => {
  beforeEach(() => {
    useTelemetryStore.setState({ snapshot: null, lastTickAt: null });
    useToastStore.setState({ toasts: [] });
  });

  it('shows skeletons until detail data arrives', () => {
    act(() => useTelemetryStore.getState().setSnapshot(sampleSnapshot));
    render(<Dashboard />);
    expect(screen.getAllByTestId('skeleton').length).toBeGreaterThan(0);
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    expect(screen.queryAllByTestId('skeleton')).toHaveLength(0);
  });

  it('shows metric values, GPU memory and power', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    expect(screen.getByTestId('value-cpu')).toHaveTextContent('23%');
    expect(screen.getByTestId('value-ram')).toHaveTextContent('12/31 GB');
    expect(screen.getByTestId('value-cpu')).toHaveTextContent('3.2 GHz');
    expect(screen.getByTestId('value-net')).toHaveTextContent('↓ 1 MB/s');
    expect(screen.getByTestId('value-net')).toHaveTextContent('↑ 80 KB/s');
    expect(screen.getByTestId('value-dgpu')).toHaveTextContent('--');
    expect(screen.getByText(/5\.2 \/ 8\.0 GB/)).toBeInTheDocument();
    expect(screen.getAllByText('8.2 W').length).toBeGreaterThan(0);
  });

  it('shows a tooltip for a hovered core', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    fireEvent.mouseEnter(screen.getAllByTestId('core-cell')[1]!);
    const tip = screen.getByRole('tooltip');
    expect(tip).toHaveTextContent('Core 1');
    expect(tip).toHaveTextContent('5100 MHz');
    expect(tip).toHaveTextContent('Usage 95%');
    expect(tip).toHaveTextContent('48°C');
  });

  it('needs two clicks to end a process and removes it optimistically', async () => {
    const calls: unknown[] = [];
    mockIPC((cmd, args) => {
      if (cmd === 'kill_process_group') {
        calls.push(args);
        return { key: 'chrome', results: [{ pid: 10, outcome: 'killed', message: null }] };
      }
      return null;
    });
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    fireEvent.click(screen.getByLabelText('End chrome.exe'));
    expect(calls).toHaveLength(0);
    fireEvent.click(screen.getByLabelText('Confirm end chrome.exe'));
    expect(screen.queryByText('chrome')).toBeNull();
    await waitFor(() => expect(calls).toEqual([{ key: 'chrome' }]));
    expect(useToastStore.getState().toasts).toHaveLength(0);
  });

  it('disarms the confirmation after a few seconds', () => {
    vi.useFakeTimers();
    try {
      act(() => useTelemetryStore.getState().setSnapshot(withDetail));
      render(<Dashboard />);
      fireEvent.click(screen.getByLabelText('End Code.exe'));
      expect(screen.getByLabelText('Confirm end Code.exe')).toBeInTheDocument();
      act(() => vi.advanceTimersByTime(CONFIRM_MS + 10));
      expect(screen.getByLabelText('End Code.exe')).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it('restores the row and shows a toast when the kill is refused', async () => {
    const refused: KillReport = {
      key: 'code',
      results: [{ pid: 20, outcome: 'access_denied', message: null }],
    };
    mockIPC((cmd) => (cmd === 'kill_process_group' ? refused : null));
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    fireEvent.click(screen.getByLabelText('End Code.exe'));
    fireEvent.click(screen.getByLabelText('Confirm end Code.exe'));
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent(/administrator/));
    expect(screen.getByText('Code')).toBeInTheDocument();
  });

  it('shows the top 3 processes with a toggle for the rest', () => {
    const many = {
      ...sampleDetail,
      processes: Array.from({ length: 6 }, (_, i) => ({
        key: `p${i}`,
        name: `p${i}.exe`,
        pids: [i + 1],
        cpu_pct: 10 - i,
        mem_bytes: 1,
      })),
    };
    act(() => useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: many }));
    render(<Dashboard />);
    const rows = () => screen.getByTestId('process-rows').querySelectorAll('li');
    expect(rows()).toHaveLength(3);
    fireEvent.click(screen.getByRole('button', { name: /Show 3 more/ }));
    expect(rows()).toHaveLength(6);
    fireEvent.click(screen.getByRole('button', { name: /Show less/ }));
    expect(rows()).toHaveLength(3);
  });
});
