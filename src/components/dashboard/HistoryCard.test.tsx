import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it } from 'vitest';
import { useToastStore } from '../../toasts';
import type { HistorySample } from '../../lib/history';
import { useSettingsStore, type Settings } from '../../settings';
import { HistoryCard } from './HistoryCard';

const MIN = 60_000;

function sample(ts_ms: number, cpu_pct: number): HistorySample {
  return {
    ts_ms,
    cpu_pct,
    cpu_temp_c: null,
    ram_pct: 40,
    igpu_pct: null,
    dgpu_pct: null,
    igpu_temp_c: null,
    dgpu_temp_c: null,
    disk_pct: null,
    net_down_bps: 0,
    net_up_bps: 0,
    battery_pct: null,
    package_watts: null,
  };
}

let requests: unknown[];
let exported: unknown[];
let exportFails: boolean;
let events: { ts_ms: number; kind: string; label: string }[];

beforeEach(() => {
  requests = [];
  exported = [];
  exportFails = false;
  useToastStore.setState({ toasts: [] });
  useSettingsStore.setState({ settings: { history_hours: 720 } as Settings });
  const minute = Date.now() - (Date.now() % MIN);
  mockIPC((cmd, args) => {
    if (cmd === 'get_history_events') return events;
    if (cmd === 'export_history') {
      exported.push(args);
      if (exportFails) throw 'disk full';
      return 'C:/Downloads/h.csv';
    }
    if (cmd !== 'get_telemetry_history') return null;
    requests.push(args);
    return [sample(minute - 3 * MIN, 20), sample(minute - 2 * MIN, 37)];
  });
  events = [
    { ts_ms: minute - 2 * MIN, kind: 'throttle', label: 'Thermal throttle' },
    { ts_ms: minute - 90 * 24 * 60 * MIN, kind: 'alert', label: 'Old alert' },
  ];
});

it('charts only metrics that have data and reads out the newest point', async () => {
  render(<HistoryCard />);
  expect(await screen.findByRole('img', { name: 'CPU history' })).toBeInTheDocument();
  expect(screen.queryByRole('img', { name: 'Battery history' })).not.toBeInTheDocument();
  const cpu = () => screen.getByRole('img', { name: 'CPU history' }).closest('figure')!;
  fireEvent.click(screen.getByRole('radio', { name: '1h' }));
  expect(screen.getByRole('radio', { name: '1h' })).toBeChecked();
  await waitFor(() => expect(cpu()).toHaveTextContent('37%'));
  expect(requests).toHaveLength(2);
});

const cpuSvg = () => screen.getByRole('img', { name: 'CPU history' });

function mockBox() {
  cpuSvg().getBoundingClientRect = () => ({ left: 0, width: 240, top: 0, height: 10 }) as DOMRect;
}

it('draws markers only for events in view and describes them', async () => {
  render(<HistoryCard />);
  await screen.findByRole('img', { name: 'CPU history' });
  const marks = cpuSvg().querySelectorAll('[data-testid="history-event"]');
  expect(marks).toHaveLength(1);
  expect(marks[0]).toHaveAttribute('stroke', 'var(--color-island-warn)');
  expect(cpuSvg()).toHaveAccessibleDescription('1 event shown');
});

it('shows event labels and statistics', async () => {
  render(<HistoryCard />);
  await screen.findByRole('img', { name: 'CPU history' });
  expect(cpuSvg().closest('figure')).toHaveTextContent('min 29% avg 29% max 29%');
  mockBox();
  const mark = cpuSvg().querySelector('[data-testid="history-event"]')!;
  const x = Number(mark.getAttribute('x1'));
  const width = 239;
  fireEvent.mouseMove(cpuSvg(), { clientX: (x / width) * 240 });
  expect(cpuSvg().closest('figure')).toHaveTextContent('Thermal throttle');
});

it('zooms by dragging and resets', async () => {
  render(<HistoryCard />);
  await screen.findByRole('img', { name: 'CPU history' });
  mockBox();
  expect(screen.queryByRole('button', { name: 'Reset zoom' })).not.toBeInTheDocument();
  fireEvent.mouseDown(cpuSvg(), { clientX: 200 });
  fireEvent.mouseMove(cpuSvg(), { clientX: 202 });
  fireEvent.mouseUp(window);
  expect(screen.queryByRole('button', { name: 'Reset zoom' })).not.toBeInTheDocument();
  fireEvent.mouseDown(cpuSvg(), { clientX: 100 });
  fireEvent.mouseMove(cpuSvg(), { clientX: 200 });
  expect(screen.getAllByTestId('history-selection').length).toBeGreaterThan(0);
  fireEvent.mouseUp(window);
  fireEvent.click(await screen.findByRole('button', { name: 'Reset zoom' }));
  expect(screen.queryByRole('button', { name: 'Reset zoom' })).not.toBeInTheDocument();
});

it('exports the visible frame and toasts the result', async () => {
  render(<HistoryCard />);
  await screen.findByRole('img', { name: 'CPU history' });
  fireEvent.click(screen.getByRole('button', { name: 'Export history as CSV' }));
  await waitFor(() =>
    expect(useToastStore.getState().toasts.at(-1)?.text).toBe('Saved to C:/Downloads/h.csv'),
  );
  expect(exported[0]).toMatchObject({ format: 'csv' });
  exportFails = true;
  fireEvent.click(screen.getByRole('button', { name: 'Export history as JSON' }));
  await waitFor(() => expect(useToastStore.getState().toasts.at(-1)?.text).toBe('disk full'));
});
