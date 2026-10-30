import type { Arc } from '../components/Ring';
import type { CoreMetrics, Metric } from '../types/telemetry';
import { ACCENTS, type MetricKey } from './levels';
import type { Settings } from '../settings';

export const CPU_TEMP_SOURCE =
  'Hottest ACPI thermal zone (system temperature; may differ from CPU package temperature)';

/** Temperature uses the same warning bands, on a 0–105°C scale. */
export function temperatureArc(
  celsius: number | null | undefined,
  accent: string,
  settings?: Pick<Settings, 'warn_threshold' | 'crit_threshold'> | null,
): Arc | undefined {
  if (celsius == null || !Number.isFinite(celsius)) return undefined;
  const percent = Math.min(100, Math.max(0, (celsius / 105) * 100));
  return {
    percent,
    accent,
    level:
      percent > (settings?.crit_threshold ?? 90)
        ? 'critical'
        : percent > (settings?.warn_threshold ?? 70)
          ? 'warning'
          : 'normal',
  };
}

/**
 * Ring stroke widths. Single rings are thick with a small hole; concentric pairs stay
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
export function ringSpecs(
  core: CoreMetrics | null | undefined,
  settings?: Pick<Settings, 'warn_threshold' | 'crit_threshold'> | null,
): RingSpec[] {
  const net = core?.net;
  return [
    {
      key: 'cpu',
      label: 'CPU',
      outer: arc(core?.cpu, ACCENTS.cpu),
      inner: temperatureArc(core?.cpu_temp_c, ACCENTS.cpu, settings),
    },
    {
      key: 'ram',
      label: 'RAM',
      outer: core
        ? { percent: core.ram.percent, level: core.ram.level, accent: ACCENTS.ram }
        : missing(ACCENTS.ram),
    },
    {
      key: 'igpu',
      label: 'iGPU',
      outer: arc(core?.igpu, ACCENTS.igpu),
      inner: temperatureArc(core?.igpu_temp_c, ACCENTS.igpu, settings),
    },
    {
      key: 'dgpu',
      label: 'dGPU',
      outer: arc(core?.dgpu, ACCENTS.dgpu),
      inner: temperatureArc(core?.dgpu_temp_c, ACCENTS.dgpu, settings),
    },
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
