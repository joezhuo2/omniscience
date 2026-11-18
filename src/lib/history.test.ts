import { describe, expect, it } from 'vitest';
import {
  HOUR_MS,
  areaPath,
  bucketize,
  frame,
  lastIndex,
  linePath,
  niceMax,
  rangeFor,
  segments,
  eventIndexes,
  seriesStats,
  zoomFrame,
  shapeCharts,
  type HistorySample,
} from './history';

const MIN = 60_000;
const NOW = 1_700_000_000_000 - (1_700_000_000_000 % MIN) + 30_000;

function sample(ts_ms: number, patch: Partial<HistorySample> = {}): HistorySample {
  return {
    ts_ms,
    cpu_pct: 10,
    cpu_temp_c: null,
    ram_pct: 50,
    igpu_pct: null,
    dgpu_pct: null,
    igpu_temp_c: null,
    dgpu_temp_c: null,
    disk_pct: null,
    net_down_bps: 0,
    net_up_bps: 0,
    battery_pct: null,
    package_watts: null,
    ...patch,
  };
}

describe('history ranges', () => {
  it('limits the range to the retention setting', () => {
    expect(rangeFor(24, 24).label).toBe('24h');
    expect(rangeFor(720, 24).label).toBe('24h');
    expect(rangeFor(168, 6).label).toBe('6h');
    expect(rangeFor(24, 1).label).toBe('1h');
    expect(rangeFor(1, 720).label).toBe('1h');
  });
});

describe('frame', () => {
  it('uses one-minute buckets ending with the last completed minute', () => {
    const f = frame(1, NOW);
    expect(f.step).toBe(MIN);
    expect(f.count).toBe(60);
    expect(f.from + f.count * f.step).toBe(NOW - 30_000);
  });

  it('caps long ranges at the point budget', () => {
    const f = frame(720, NOW, 240);
    expect(f.step).toBe(180 * MIN);
    expect(f.count).toBe(240);
    expect(f.count * f.step).toBe(720 * HOUR_MS);
    expect(f.from + f.count * f.step).toBeGreaterThan(NOW - MIN);
  });
});

describe('bucketize', () => {
  const f = { from: 0, step: 2 * MIN, count: 3 };

  it('averages by default, keeps maxima on request and leaves empty buckets null', () => {
    const samples = [sample(0, { cpu_pct: 10 }), sample(MIN, { cpu_pct: 30 }), sample(4 * MIN)];
    expect(bucketize(samples, f, (s) => s.cpu_pct)).toEqual([20, null, 10]);
    expect(bucketize(samples, f, (s) => s.cpu_pct, 'max')).toEqual([30, null, 10]);
  });

  it('skips missing readings and samples outside the frame', () => {
    const samples = [
      sample(-MIN, { cpu_temp_c: 99 }),
      sample(0, { cpu_temp_c: null }),
      sample(MIN, { cpu_temp_c: 60 }),
      sample(6 * MIN, { cpu_temp_c: 99 }),
    ];
    expect(bucketize(samples, f, (s) => s.cpu_temp_c)).toEqual([60, null, null]);
  });
});

describe('paths', () => {
  it('breaks lines at gaps instead of interpolating', () => {
    expect(segments([1, 2, null, 3, null, null, 4, 5])).toEqual([
      [
        [0, 1],
        [1, 2],
      ],
      [[3, 3]],
      [
        [6, 4],
        [7, 5],
      ],
    ]);
    expect(linePath([0, 100, null, 50], 100)).toBe('M0 100L1 0M2.5 50h1');
  });

  it('fills areas only under multi-point runs and clamps to the axis', () => {
    expect(areaPath([50, 200, null, 10], 100)).toBe('M0 100L0 50L1 0L1 100Z');
  });

  it('rounds the axis maximum up to a readable number', () => {
    expect(niceMax([[12, null, 87]], 50)).toBe(90);
    expect(niceMax([[null]], 50)).toBe(50);
    expect(niceMax([[3_400_000], [10]], 1024)).toBe(4_000_000);
  });
});

describe('shapeCharts', () => {
  it('drops charts without data and reports the newest point', () => {
    const f = frame(1, NOW);
    const charts = shapeCharts(
      [
        sample(f.from, { igpu_pct: 5, cpu_temp_c: 70 }),
        sample(f.from + 10 * MIN, { igpu_pct: 7, cpu_temp_c: 72 }),
      ],
      f,
    );
    expect(charts.map((c) => c.title)).toEqual(['CPU', 'Memory', 'GPU', 'Temperatures', 'Network']);
    const gpu = charts.find((c) => c.title === 'GPU')!;
    expect(gpu.series.map((s) => s.label)).toEqual(['iGPU']);
    expect(gpu.max).toBe(100);
    expect(charts.find((c) => c.title === 'Temperatures')!.max).toBe(80);
    expect(lastIndex(gpu)).toBe(10);
  });
});

describe('history zoom, stats and events', () => {
  it('builds a frame for an arbitrary window', () => {
    const small = zoomFrame(NOW, NOW + 10 * MIN);
    expect(small.step).toBe(MIN);
    expect(small.count).toBeGreaterThanOrEqual(10);
    expect(small.from).toBeLessThanOrEqual(NOW);
    const wide = zoomFrame(NOW, NOW + 30 * 24 * HOUR_MS);
    expect(wide.step).toBe(180 * MIN);
    expect(wide.count).toBeLessThanOrEqual(240);
    expect(wide.from + wide.count * wide.step).toBeGreaterThanOrEqual(NOW + 29 * 24 * HOUR_MS);
    expect(zoomFrame(NOW, NOW).count).toBe(1);
  });

  it('summarizes present values only', () => {
    expect(seriesStats([null, 2, 4, null, 9])).toEqual({ min: 2, avg: 5, max: 9 });
    expect(seriesStats([null, null])).toBeNull();
  });

  it('maps events to bucket indexes inside the frame', () => {
    const f = { from: NOW, step: MIN, count: 10 };
    const ev = (ts_ms: number) => ({ ts_ms, kind: 'alert' as const, label: 'x' });
    const found = eventIndexes(
      [ev(NOW - 1), ev(NOW), ev(NOW + 3 * MIN + 5), ev(NOW + 10 * MIN)],
      f,
    );
    expect(found.map((e) => e.index)).toEqual([0, 3]);
  });
});
