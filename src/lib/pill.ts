import type { CoreMetrics } from '../types/telemetry';
import type { PillNumbers } from '../types/generated/PillNumbers';
import { formatRateRounded, toDisplayTemp } from './format';
import type { MetricKey } from './levels';

export const ALL_RINGS: MetricKey[] = ['cpu', 'ram', 'igpu', 'dgpu', 'disk', 'net'];

export function pillWidth(ringCount: number, numbersOn: boolean): number {
  const n = ringCount;
  return 88 + n * 22 + (n - 1) * 8 + (numbersOn ? n * 24 : 0);
}

const percentText = (p: number | null | undefined) =>
  p == null || !Number.isFinite(p) ? '–' : `${Math.round(p)}%`;

const tempText = (c: number | null | undefined) =>
  c == null || !Number.isFinite(c) ? null : `${Math.round(toDisplayTemp(c))}°`;

function compactRate(bps: number): string {
  return formatRateRounded(bps).replace('/s', '').replace(' ', '').replace(/[Bb]$/, '');
}

export function pillNumber(
  key: MetricKey,
  core: CoreMetrics | null | undefined,
  mode: PillNumbers,
): string {
  if (!core || mode === 'off') return '–';
  if (key === 'net') return core.net ? compactRate(core.net.down_bps) : '–';
  const temp = { cpu: core.cpu_temp_c, igpu: core.igpu_temp_c, dgpu: core.dgpu_temp_c }[
    key as 'cpu' | 'igpu' | 'dgpu'
  ];
  if (mode === 'temp' && temp !== undefined) {
    const t = tempText(temp);
    if (t) return t;
  }
  const metric = key === 'ram' ? core.ram : core[key as 'cpu' | 'igpu' | 'dgpu' | 'disk'];
  return percentText(metric?.percent);
}
