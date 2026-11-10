import { mockIPC } from '@tauri-apps/api/mocks';
import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, expect, it } from 'vitest';
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

beforeEach(() => {
  requests = [];
  useSettingsStore.setState({ settings: { history_hours: 720 } as Settings });
  const minute = Date.now() - (Date.now() % MIN);
  mockIPC((cmd, args) => {
    if (cmd !== 'get_telemetry_history') return null;
    requests.push(args);
    return [sample(minute - 3 * MIN, 20), sample(minute - 2 * MIN, 37)];
  });
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
