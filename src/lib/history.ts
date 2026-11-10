import type { HistorySample } from '../types/generated/HistorySample';
import { formatPercent, formatRate, formatTemp, formatWatts } from './format';

export type { HistorySample };

const MINUTE_MS = 60_000;
export const HOUR_MS = 60 * MINUTE_MS;
export const MAX_POINTS = 240;

export const RANGES = [
  { label: '1h', hours: 1 },
  { label: '6h', hours: 6 },
  { label: '24h', hours: 24 },
  { label: '7d', hours: 168 },
  { label: '30d', hours: 720 },
] as const;

export type Range = (typeof RANGES)[number];

export function rangeFor(chosenHours: number, retentionHours: number): Range {
  const limit = Math.min(chosenHours, retentionHours);
  return RANGES.filter((r) => r.hours <= limit).at(-1) ?? RANGES[0];
}

export interface Frame {
  from: number;
  step: number;
  count: number;
}

// The newest bucket ends with the last completed minute; the current one is not recorded yet.
export function frame(hours: number, now: number, maxPoints = MAX_POINTS): Frame {
  const minutes = hours * 60;
  const stepMinutes = Math.ceil(minutes / maxPoints);
  const step = stepMinutes * MINUTE_MS;
  const count = Math.ceil(minutes / stepMinutes);
  const end = Math.floor((now - MINUTE_MS) / step) * step + step;
  return { from: end - count * step, step, count };
}

type Pick = (s: HistorySample) => number | null;

export function bucketize(
  samples: HistorySample[],
  f: Frame,
  pick: Pick,
  agg: 'mean' | 'max' = 'mean',
): (number | null)[] {
  const sum = new Array<number>(f.count).fill(0);
  const n = new Array<number>(f.count).fill(0);
  const peak = new Array<number>(f.count).fill(-Infinity);
  for (const s of samples) {
    const i = Math.floor((s.ts_ms - f.from) / f.step);
    const v = pick(s);
    if (i < 0 || i >= f.count || v == null || !Number.isFinite(v)) continue;
    sum[i]! += v;
    n[i]!++;
    peak[i] = Math.max(peak[i]!, v);
  }
  return n.map((count, i) => (count === 0 ? null : agg === 'max' ? peak[i]! : sum[i]! / count));
}

export function segments(values: (number | null)[]): [number, number][][] {
  const out: [number, number][][] = [];
  let run: [number, number][] | null = null;
  values.forEach((v, i) => {
    if (v == null) {
      run = null;
      return;
    }
    if (!run) out.push((run = []));
    run.push([i, v]);
  });
  return out;
}

const y = (v: number, max: number) =>
  +(100 - (Math.min(Math.max(v, 0), max) / max) * 100).toFixed(2);

export function linePath(values: (number | null)[], max: number): string {
  return segments(values)
    .map((seg) => {
      if (seg.length === 1) return `M${seg[0]![0] - 0.5} ${y(seg[0]![1], max)}h1`;
      return seg.map(([i, v], k) => `${k ? 'L' : 'M'}${i} ${y(v, max)}`).join('');
    })
    .join('');
}

export function areaPath(values: (number | null)[], max: number): string {
  return segments(values)
    .filter((seg) => seg.length > 1)
    .map((seg) => {
      const first = seg[0]![0];
      const last = seg.at(-1)![0];
      return `M${first} 100${seg.map(([i, v]) => `L${i} ${y(v, max)}`).join('')}L${last} 100Z`;
    })
    .join('');
}

export function niceMax(values: (number | null)[][], floor: number): number {
  let m = floor;
  for (const row of values) for (const v of row) if (v != null && v > m) m = v;
  const mag = 10 ** Math.floor(Math.log10(m));
  return Math.ceil(m / mag) * mag;
}

interface ChartSpec {
  title: string;
  format: (v: number) => string;
  max?: number;
  floor?: number;
  agg?: 'max';
  series: { label: string; pick: Pick }[];
}

const CHARTS: ChartSpec[] = [
  {
    title: 'CPU',
    format: formatPercent,
    max: 100,
    series: [{ label: 'CPU', pick: (s) => s.cpu_pct }],
  },
  {
    title: 'Memory',
    format: formatPercent,
    max: 100,
    series: [{ label: 'RAM', pick: (s) => s.ram_pct }],
  },
  {
    title: 'GPU',
    format: formatPercent,
    max: 100,
    series: [
      { label: 'iGPU', pick: (s) => s.igpu_pct },
      { label: 'dGPU', pick: (s) => s.dgpu_pct },
    ],
  },
  {
    title: 'Temperatures',
    format: formatTemp,
    floor: 50,
    agg: 'max',
    series: [
      { label: 'CPU', pick: (s) => s.cpu_temp_c },
      { label: 'iGPU', pick: (s) => s.igpu_temp_c },
      { label: 'dGPU', pick: (s) => s.dgpu_temp_c },
    ],
  },
  {
    title: 'Disk',
    format: formatPercent,
    max: 100,
    series: [{ label: 'Active', pick: (s) => s.disk_pct }],
  },
  {
    title: 'Network',
    format: formatRate,
    floor: 1024,
    series: [
      { label: 'Down', pick: (s) => s.net_down_bps },
      { label: 'Up', pick: (s) => s.net_up_bps },
    ],
  },
  {
    title: 'Battery',
    format: formatPercent,
    max: 100,
    series: [{ label: 'Charge', pick: (s) => s.battery_pct }],
  },
  {
    title: 'Power',
    format: formatWatts,
    floor: 10,
    series: [{ label: 'Package', pick: (s) => s.package_watts }],
  },
];

export interface ShapedSeries {
  label: string;
  values: (number | null)[];
}

export interface ShapedChart {
  title: string;
  format: (v: number) => string;
  max: number;
  series: ShapedSeries[];
}

export function shapeCharts(samples: HistorySample[], f: Frame): ShapedChart[] {
  return CHARTS.flatMap((spec) => {
    const series = spec.series
      .map((s) => ({ label: s.label, values: bucketize(samples, f, s.pick, spec.agg) }))
      .filter((s) => s.values.some((v) => v != null));
    if (series.length === 0) return [];
    const max =
      spec.max ??
      niceMax(
        series.map((s) => s.values),
        spec.floor ?? 1,
      );
    return [{ title: spec.title, format: spec.format, max, series }];
  });
}

export function lastIndex(chart: ShapedChart): number | null {
  for (let i = chart.series[0]!.values.length - 1; i >= 0; i--) {
    if (chart.series.some((s) => s.values[i] != null)) return i;
  }
  return null;
}
