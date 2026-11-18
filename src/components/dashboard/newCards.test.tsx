import { act, render, screen } from '@testing-library/react';
import { mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { useSettingsStore } from '../../settings';
import { useTelemetryStore } from '../../store';
import { sampleDetail, sampleSnapshot } from '../../test/fixtures';
import type { Settings } from '../../types/generated/Settings';
import type { DetailMetrics } from '../../types/telemetry';
import { BluetoothCard } from './BluetoothCard';
import { Dashboard } from './Dashboard';
import { DrivesCard } from './DrivesCard';
import { GpuPowerCard } from './GpuPowerCard';
import { MemoryCard } from './MemoryCard';
import { NetworkCard } from './NetworkCard';
import { SystemCard } from './SystemCard';

const GB = 1024 ** 3;

const show = (patch: Partial<DetailMetrics>) =>
  act(() =>
    useTelemetryStore.getState().setSnapshot({
      ...sampleSnapshot,
      detail: { ...sampleDetail, ...patch },
    }),
  );

beforeEach(() => {
  useTelemetryStore.setState({ snapshot: null, lastTickAt: null });
  useSettingsStore.setState({ settings: null });
  localStorage.clear();
});

describe('DrivesCard', () => {
  it('shows volumes, speeds and physical disks', () => {
    show({
      drives: [
        {
          name: 'C:',
          label: 'System',
          total_bytes: 500 * GB,
          free_bytes: 200 * GB,
          read_bps: 1024 * 1024,
          write_bps: 2048,
        },
        {
          name: 'D:',
          label: null,
          total_bytes: 100 * GB,
          free_bytes: 50 * GB,
          read_bps: null,
          write_bps: null,
        },
      ],
      disks: [
        { name: 'Samsung SSD', kind: 'NVMe', health: 'Healthy', temp_c: 41, wear_pct: 3 },
        { name: 'Old HDD', kind: null, health: 'Warning', temp_c: null, wear_pct: null },
      ],
    });
    render(<DrivesCard />);
    const [c, d] = screen.getAllByTestId('drive');
    expect(c).toHaveTextContent('C:System');
    expect(c).toHaveTextContent('300 GB / 500 GB');
    expect(c).toHaveTextContent('200 GB free');
    expect(c).toHaveTextContent('R 1.0 MB/s · W 2.0 KB/s');
    expect(d).not.toHaveTextContent('R ');
    const [ssd, hdd] = screen.getAllByTestId('disk');
    expect(ssd).toHaveTextContent('NVMe · 41°C · Wear 3% · Healthy');
    expect(hdd).toHaveTextContent('Old HDD');
    expect(hdd).toHaveTextContent('Warning');
    expect(screen.getAllByTestId('disk-health')[1]).toHaveStyle({ color: '#FFB340' });
  });

  it('copes with no drives and with no detail', () => {
    const { container } = render(<DrivesCard />);
    expect(container.querySelector('[data-testid="drive"]')).toBeNull();
    show({});
    expect(screen.getByText('No drives found')).toBeInTheDocument();
    expect(container).not.toHaveTextContent(/NaN|undefined/);
  });
});

describe('NetworkCard extras', () => {
  beforeEach(() => {
    mockIPC(() => null);
    useSettingsStore.setState({ settings: { ping_target: '1.1.1.1' } as Settings });
  });

  it('shows wifi, jitter, loss, usage and the metered badge', () => {
    show({
      wifi: { ssid: 'HomeNet', signal_pct: 82, link_mbps: 866 },
      ping_jitter_ms: 3,
      ping_loss_pct: 2,
      data_today_bytes: 2 * GB,
      data_month_bytes: 40 * GB,
      metered: true,
    });
    render(<NetworkCard />);
    expect(screen.getByTestId('wifi')).toHaveTextContent('HomeNet82% · 866 Mbps');
    expect(screen.getByTestId('ping-jitter')).toHaveTextContent('±3 ms');
    expect(screen.getByTestId('ping-loss')).toHaveTextContent('2% loss');
    expect(screen.getByTestId('data-usage')).toHaveTextContent('Today 2.0 GB · This month 40.0 GB');
    expect(screen.getByTestId('metered')).toHaveTextContent('Metered');
  });

  it('omits what is missing', () => {
    show({});
    const { container } = render(<NetworkCard />);
    for (const id of ['wifi', 'ping-jitter', 'ping-loss', 'data-usage', 'metered']) {
      expect(screen.queryByTestId(id)).not.toBeInTheDocument();
    }
    expect(container).not.toHaveTextContent(/NaN|undefined/);
  });

  it('shows no badge when not metered', () => {
    show({ data_today_bytes: 1, data_month_bytes: 2, metered: false });
    render(<NetworkCard />);
    expect(screen.getByTestId('data-usage')).toBeInTheDocument();
    expect(screen.queryByTestId('metered')).not.toBeInTheDocument();
  });
});

describe('GpuPowerCard extras', () => {
  const gpu = { ...sampleDetail.gpus[0]! };
  afterEach(() => useSettingsStore.setState({ settings: null }));

  it('shows clocks, encode/decode and energy with cost', () => {
    useSettingsStore.setState({
      settings: { energy_price_per_kwh: 0.25, energy_currency: 'EUR ' } as Settings,
    });
    show({
      gpus: [{ ...gpu, core_clock_mhz: 1800, mem_clock_mhz: 7000, encode_pct: 5, decode_pct: 10 }],
      energy_today_wh: 120,
    });
    render(<GpuPowerCard />);
    expect(screen.getByTestId('gpu-clocks')).toHaveTextContent(
      'Core 1800 MHz · Mem 7000 MHz · Enc 5% · Dec 10%',
    );
    expect(screen.getByTestId('energy-today')).toHaveTextContent('Energy today 120 Wh · EUR 0.03');
  });

  it('omits cost when the price is zero and clocks when missing', () => {
    useSettingsStore.setState({
      settings: { energy_price_per_kwh: 0, energy_currency: '$' } as Settings,
    });
    show({ gpus: [gpu], energy_today_wh: 12.5 });
    render(<GpuPowerCard />);
    expect(screen.getByTestId('energy-today')).toHaveTextContent(/^Energy today 12\.5 Wh$/);
    expect(screen.queryByTestId('gpu-clocks')).not.toBeInTheDocument();
  });

  it('omits energy when missing', () => {
    show({});
    render(<GpuPowerCard />);
    expect(screen.queryByTestId('energy-today')).not.toBeInTheDocument();
  });
});

describe('MemoryCard', () => {
  it('shows commit, cache and compression', () => {
    show({
      memory: {
        committed_bytes: 20 * GB,
        commit_limit_bytes: 40 * GB,
        cached_bytes: 8 * GB,
        compressed_bytes: 512 * 1024 ** 2,
      },
    });
    render(<MemoryCard />);
    expect(screen.getByTestId('memory-committed')).toHaveTextContent('20.0 GB / 40.0 GB');
    expect(screen.getByTestId('memory-cached')).toHaveTextContent('Cached 8.0 GB');
    expect(screen.getByTestId('memory-compressed')).toHaveTextContent('Compressed 512 MB');
  });

  it('copes with missing data', () => {
    const { container } = render(<MemoryCard />);
    expect(container.querySelector('[data-testid="memory-committed"]')).toBeNull();
    show({
      memory: {
        committed_bytes: 1 * GB,
        commit_limit_bytes: 2 * GB,
        cached_bytes: null,
        compressed_bytes: null,
      },
    });
    expect(screen.queryByTestId('memory-cached')).not.toBeInTheDocument();
    expect(screen.queryByTestId('memory-compressed')).not.toBeInTheDocument();
    show({ memory: null });
    expect(screen.getByText('No memory detail')).toBeInTheDocument();
  });
});

describe('SystemCard', () => {
  it('shows uptime, last boot and the five most recent events', () => {
    const boot = Date.now() - (3 * 86400 + 4 * 3600 + 12 * 60 + 5) * 1000;
    const events = Array.from({ length: 7 }, (_, i) => ({
      ts_ms: Date.now() - i * 60000,
      kind: i % 2 === 0 ? ('wake' as const) : ('sleep' as const),
    }));
    show({ boot_ms: boot, power_events: events });
    render(<SystemCard />);
    expect(screen.getByTestId('uptime')).toHaveTextContent('3d 4h 12m');
    expect(screen.getByTestId('last-boot')).toHaveTextContent(new Date(boot).toLocaleString());
    const rows = screen.getAllByTestId('power-event');
    expect(rows).toHaveLength(5);
    expect(rows[0]).toHaveTextContent('Wake');
    expect(rows[1]).toHaveTextContent('Sleep');
  });

  it('shows -- without a boot time', () => {
    show({});
    render(<SystemCard />);
    expect(screen.getByTestId('uptime')).toHaveTextContent('--');
    expect(screen.queryByTestId('last-boot')).not.toBeInTheDocument();
    expect(screen.queryByTestId('power-event')).not.toBeInTheDocument();
  });
});

describe('BluetoothCard', () => {
  it('lists devices and flags low batteries', () => {
    show({
      bluetooth: [
        { name: 'Mouse', percent: 80 },
        { name: 'Headset', percent: 15 },
      ],
    });
    render(<BluetoothCard />);
    expect(screen.getAllByTestId('bluetooth-device')).toHaveLength(2);
    const [ok, low] = screen.getAllByTestId('bluetooth-percent');
    expect(ok).toHaveTextContent('80%');
    expect(low).toHaveTextContent('15%');
    expect(low).toHaveStyle({ color: '#FFB340' });
    expect(ok).not.toHaveStyle({ color: '#FFB340' });
  });

  it('is hidden from the dashboard without devices', () => {
    show({ bluetooth: [{ name: 'Mouse', percent: 50 }] });
    render(<Dashboard />);
    expect(screen.getByRole('article', { name: 'Bluetooth' })).toBeInTheDocument();
    show({ bluetooth: [] });
    expect(screen.queryByRole('article', { name: 'Bluetooth' })).not.toBeInTheDocument();
    expect(screen.getByRole('article', { name: 'Drives' })).toBeInTheDocument();
    expect(screen.getByRole('article', { name: 'Memory' })).toBeInTheDocument();
    expect(screen.getByRole('article', { name: 'System' })).toBeInTheDocument();
  });
});
