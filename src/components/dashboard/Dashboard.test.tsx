import { mockIPC } from '@tauri-apps/api/mocks';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useSettingsStore, type Settings } from '../../settings';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import { useToastStore } from '../../toasts';
import type { DetailMetrics, KillReport } from '../../types/telemetry';
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
    const battery = screen.getByText('Battery').closest('section')!;
    expect(within(battery).getByTestId('skeleton')).toBeInTheDocument();
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    expect(screen.queryAllByTestId('skeleton')).toHaveLength(0);
    expect(screen.getByTestId('battery-percent')).toHaveTextContent('78%');
  });

  it('removes the battery skeleton when detail confirms no battery', () => {
    render(<Dashboard />);
    expect(screen.getByText('Battery')).toBeInTheDocument();
    act(() =>
      useTelemetryStore.getState().setSnapshot({
        ...sampleSnapshot,
        detail: { ...sampleDetail, battery: null },
      }),
    );
    expect(screen.queryByText('Battery')).not.toBeInTheDocument();
  });

  it('minimizes every section independently while keeping metric rings and summaries visible', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    const { unmount } = render(<Dashboard />);
    for (const title of ['GPU & Power', 'Battery', 'CPU cores', 'Network', 'Top processes']) {
      const button = screen.getByRole('button', { name: title });
      fireEvent.click(button);
      expect(button).toHaveAttribute('aria-expanded', 'false');
      expect(document.getElementById(button.getAttribute('aria-controls')!)).toHaveAttribute(
        'aria-hidden',
        'true',
      );
    }
    expect(screen.getByTestId('value-cpu')).toBeVisible();
    expect(screen.getByTitle('Package power')).toBeVisible();
    expect(screen.getByText('2 threads')).toBeVisible();
    expect(
      screen.getByRole('region', { name: 'Battery' }).querySelector('header'),
    ).toHaveTextContent('78%');
    unmount();
    render(<Dashboard />);
    for (const title of ['GPU & Power', 'Battery', 'CPU cores', 'Network', 'Top processes']) {
      const button = screen.getByRole('button', { name: title });
      expect(button).toHaveAttribute('aria-expanded', 'false');
      fireEvent.click(button);
      expect(button).toHaveAttribute('aria-expanded', 'true');
      expect(document.getElementById(button.getAttribute('aria-controls')!)).toBeVisible();
    }
    expect(screen.getByTestId('battery-percent')).toBeVisible();
    expect(screen.getByRole('searchbox')).toBeVisible();
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

  it('shows package watts once and keeps temperature out of the header', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard onSettings={() => {}} />);
    expect(screen.getAllByText('8.2 W')).toHaveLength(1);
    expect(screen.getByTitle('Package power')).toHaveTextContent('8.2 W');
    const header = screen.getByText('Omniscience').closest('header')!;
    expect(header).not.toHaveTextContent(/W|°C/);
    expect(within(header).getByRole('button', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByTestId('value-cpu')).toHaveTextContent('48°C');
  });

  it('shows GPU fan speed and throttling', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    expect(screen.getByText(/2100 rpm/)).toBeInTheDocument();
    expect(screen.getAllByText('Throttling')).toHaveLength(1);
  });

  it('shows temperatures under CPU and GPU rings, with the ACPI source explained', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    expect([...screen.getByTestId('value-cpu').children].map((el) => el.textContent)).toEqual([
      '23%',
      '3.2 GHz',
      '48°C',
    ]);
    expect(within(screen.getByTestId('value-cpu')).getByText('48°C')).toHaveAttribute(
      'title',
      expect.stringContaining('ACPI'),
    );
    expect([...screen.getByTestId('value-igpu').children].map((el) => el.textContent)).toEqual([
      '4%',
      '45°C',
    ]);
    expect([...screen.getByTestId('value-dgpu').children].map((el) => el.textContent)).toEqual([
      '--',
      '—',
    ]);
    const gpuCard = screen.getByText('GPU & Power').closest('section')!;
    expect(within(gpuCard).queryByText(/°C/)).not.toBeInTheDocument();
  });

  it('flags CPU throttling only past the noise floor', () => {
    const limited = { ...sampleDetail, cpu_limit_pct: 22.4 };
    act(() => useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: limited }));
    const { unmount } = render(<Dashboard />);
    expect(screen.getByText('Throttled −22%')).toBeInTheDocument();
    unmount();
    const jitter = { ...sampleDetail, cpu_limit_pct: 3 };
    act(() => useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail: jitter }));
    render(<Dashboard />);
    expect(screen.queryByText(/Throttled/)).not.toBeInTheDocument();
  });

  it.each([null, 0, 100, -1, NaN, Infinity])(
    'hides unsupported CPU loss readings (%s)',
    (cpu_limit_pct) => {
      act(() =>
        useTelemetryStore
          .getState()
          .setSnapshot({ ...sampleSnapshot, detail: { ...sampleDetail, cpu_limit_pct } }),
      );
      render(<Dashboard />);
      expect(screen.queryByText(/Throttled/)).not.toBeInTheDocument();
    },
  );

  it('lists network adapters, ping and public IP', () => {
    useSettingsStore.setState({ settings: { ping_target: '1.1.1.1' } as Settings });
    const detail = { ...sampleDetail, public_ip: '203.0.113.9' };
    act(() => useTelemetryStore.getState().setSnapshot({ ...sampleSnapshot, detail }));
    render(<Dashboard />);
    expect(screen.getByText('Wi-Fi')).toBeInTheDocument();
    expect(screen.getByText('192.168.1.20')).toBeInTheDocument();
    expect(screen.getByTestId('ping')).toHaveTextContent('14 ms');
    const row = screen.getByText('Wi-Fi').parentElement!;
    expect(within(row).getByText('192.168.1.20')).toBeInTheDocument();
    expect(within(row).getByTestId('ping')).toHaveTextContent('14 ms');
    const card = screen.getByRole('heading', { name: 'Network' }).closest('section')!;
    expect(card).not.toHaveTextContent(/↓|↑|MB\/s|KB\/s/);
    expect(within(card).getAllByTestId('ping')).toHaveLength(1);
    expect(screen.getByText('203.0.113.9')).toBeInTheDocument();
  });

  it('shows battery charge, drain and time left', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    expect(screen.getByTestId('battery-percent')).toHaveTextContent('78%');
    expect(screen.getByTestId('battery-summary')).toHaveTextContent('2h 30m left');
    expect(screen.getByText('−12 W')).toBeInTheDocument();
  });

  it('shows charging and plugged-in states, and nothing without a battery', () => {
    const show = (battery: DetailMetrics['battery']) => {
      act(() =>
        useTelemetryStore
          .getState()
          .setSnapshot({ ...sampleSnapshot, detail: { ...sampleDetail, battery } }),
      );
      return render(<Dashboard />);
    };
    let view = show({ percent: 60, state: 'charging', watts: 30, secs_remaining: 1800 });
    expect(screen.getByTestId('battery-summary')).toHaveTextContent('30m to full');
    view.unmount();
    view = show({ percent: 80, state: 'idle', watts: null, secs_remaining: null });
    expect(screen.getByTestId('battery-summary')).toHaveTextContent('Plugged in, not charging');
    view.unmount();
    show(null);
    expect(screen.queryByText('Battery')).not.toBeInTheDocument();
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
    await waitFor(() => expect(screen.getByText(/needs administrator rights/)).toBeInTheDocument());
    expect(screen.getByText('Code')).toBeInTheDocument();
  });

  it('switches process columns to GPU usage and memory on the GPU tab', () => {
    act(() => useTelemetryStore.getState().setSnapshot(withDetail));
    render(<Dashboard />);
    const rows = () => screen.getByTestId('process-rows').querySelectorAll('li');
    expect(rows()[0]).toHaveTextContent('chrome');
    expect(rows()[0]).toHaveTextContent('6.0%');
    fireEvent.click(screen.getByRole('tab', { name: 'gpu' }));
    expect(screen.getByRole('tab', { name: 'gpu' })).toHaveAttribute('aria-selected', 'true');
    expect(rows()[0]).toHaveTextContent('chrome');
    expect(rows()[0]).toHaveTextContent('2.0%');
    expect(rows()[0]).toHaveTextContent('307 MB');
  });

  it('sorts by disk and network rates and preserves the selected tab during search', () => {
    vi.useFakeTimers();
    try {
      act(() => useTelemetryStore.getState().setSnapshot(withDetail));
      render(<Dashboard />);
      const rows = () => screen.getByTestId('process-rows').querySelectorAll('li');
      fireEvent.click(screen.getByRole('tab', { name: 'disk' }));
      expect(rows()[0]).toHaveTextContent('Code');
      expect(rows()[0]).toHaveTextContent('4 MB/s');
      fireEvent.click(screen.getByRole('tab', { name: 'network' }));
      expect(rows()[0]).toHaveTextContent('chrome');
      expect(rows()[0]).toHaveTextContent('2 MB/s');
      fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'Code' } });
      act(() => vi.advanceTimersByTime(200));
      expect(screen.getByRole('tab', { name: 'network' })).toHaveAttribute('aria-selected', 'true');
      expect(rows()).toHaveLength(1);
      expect(rows()[0]).toHaveTextContent('1 KB/s');
      act(() =>
        useTelemetryStore.getState().setSnapshot({
          ...withDetail,
          detail: {
            ...sampleDetail,
            processes: sampleDetail.processes.map((p) => ({ ...p, net_bps: null })),
          },
        }),
      );
      expect(rows()[0]).toHaveTextContent('—');
      expect(screen.getByText(/Per-app throughput is unavailable/)).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it('shows skeletons while searching, ignores superseded queries, and clears immediately', () => {
    vi.useFakeTimers();
    try {
      act(() => useTelemetryStore.getState().setSnapshot(withDetail));
      render(<Dashboard />);
      const search = screen.getByRole('searchbox', { name: 'Search processes' });
      const card = screen.getByText('Top processes').closest('section')!;
      fireEvent.change(search, { target: { value: 'chrome' } });
      expect(within(card).getAllByTestId('skeleton')).toHaveLength(5);
      expect(screen.queryByLabelText('End chrome.exe')).not.toBeInTheDocument();
      act(() => vi.advanceTimersByTime(100));
      fireEvent.change(search, { target: { value: ' CODE ' } });
      act(() => vi.advanceTimersByTime(100));
      expect(within(card).getAllByTestId('skeleton')).toHaveLength(5);
      act(() => vi.advanceTimersByTime(100));
      expect(within(card).queryAllByTestId('skeleton')).toHaveLength(0);
      expect(screen.getByTestId('process-rows').querySelectorAll('li')).toHaveLength(1);
      expect(screen.getByLabelText('End Code.exe')).toBeInTheDocument();
      fireEvent.click(screen.getByRole('tab', { name: 'gpu' }));
      expect(screen.getByLabelText('End Code.exe')).toBeInTheDocument();
      fireEvent.change(search, { target: { value: 'not-running' } });
      act(() => vi.advanceTimersByTime(200));
      expect(screen.getByText('No matching processes')).toBeInTheDocument();
      fireEvent.change(search, { target: { value: 'pending' } });
      fireEvent.click(screen.getByRole('button', { name: 'Clear process search' }));
      expect(search).toHaveValue('');
      expect(screen.getByTestId('process-rows').querySelectorAll('li')).toHaveLength(2);
      act(() => vi.advanceTimersByTime(200));
      expect(within(card).queryAllByTestId('skeleton')).toHaveLength(0);
      expect(screen.getByTestId('process-rows').querySelectorAll('li')).toHaveLength(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it('updates filtered rows from live telemetry and disarms end confirmation on search', () => {
    vi.useFakeTimers();
    try {
      act(() => useTelemetryStore.getState().setSnapshot(withDetail));
      render(<Dashboard />);
      fireEvent.click(screen.getByLabelText('End chrome.exe'));
      fireEvent.change(screen.getByRole('searchbox'), { target: { value: '11' } });
      act(() => vi.advanceTimersByTime(200));
      expect(screen.getByLabelText('End chrome.exe')).toBeInTheDocument();
      expect(screen.queryByLabelText('End Code.exe')).not.toBeInTheDocument();
      act(() =>
        useTelemetryStore.getState().setSnapshot({
          ...withDetail,
          detail: { ...sampleDetail, processes: [sampleDetail.processes[1]!] },
        }),
      );
      expect(screen.getByText('No matching processes')).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
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
        gpu_pct: 0,
        gpu_mem_bytes: 0,
        disk_bps: null,
        net_bps: null,
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
