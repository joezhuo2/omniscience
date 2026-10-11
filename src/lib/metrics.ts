import type { Arc } from '../components/Ring';
import type { CoreMetrics, Metric } from '../types/telemetry';
import { ACCENTS, type MetricKey } from './levels';

/**
 * Ring stroke widths. Single rings are thick with a small hole; the concentric network pair stays
 * thin so both layers fit.
 */
export const RING_STROKE = {
  pill: 3.6,
  pillPair: 2.4,
  strip: 6.5,
  stripPair: 4,
} as const;

export interface RingSpec {
  key: MetricKey;
  label: string;
  outer: Arc;
  inner?: Arc;
}

const missing = (accent: string): Arc => ({ percent: null, level: 'normal', accent });

function arc(m: Metric | null | undefined, accent: string): Arc {
  return m ? { percent: m.percent, level: m.level, accent } : missing(accent);
}

/** The six rings in display order. With no data yet, every ring is in the missing state. */
export function ringSpecs(core: CoreMetrics | null | undefined): RingSpec[] {
  const net = core?.net;
  return [
    { key: 'cpu', label: 'CPU', outer: arc(core?.cpu, ACCENTS.cpu) },
    {
      key: 'ram',
      label: 'RAM',
      outer: core
        ? { percent: core.ram.percent, level: core.ram.level, accent: ACCENTS.ram }
        : missing(ACCENTS.ram),
    },
    { key: 'igpu', label: 'iGPU', outer: arc(core?.igpu, ACCENTS.igpu) },
    { key: 'dgpu', label: 'dGPU', outer: arc(core?.dgpu, ACCENTS.dgpu) },
    { key: 'disk', label: 'Disk', outer: arc(core?.disk, ACCENTS.disk) },
    {
      key: 'net',
      label: 'Network',
      outer: net
        ? { percent: net.down_pct, level: net.level, accent: ACCENTS.net }
        : missing(ACCENTS.net),
      inner: net
        ? { percent: net.up_pct, level: net.level, accent: ACCENTS.net }
        : missing(ACCENTS.net),
    },
  ];
}
