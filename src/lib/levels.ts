import type { Level } from '../types/telemetry';

/** Per-metric accent used while a reading is normal ("cool neon"). */
export const ACCENTS = {
  cpu: '#22D3EE',
  ram: '#A78BFA',
  igpu: '#34D399',
  dgpu: '#A3E635',
  disk: '#38BDF8',
  net: '#F472B6',
} as const;

export type MetricKey = keyof typeof ACCENTS;

export const WARNING = '#F97316';
export const CRITICAL = '#EF4444';

/** Ring color for a level: the metric accent when normal, otherwise orange or red. */
export function levelColor(level: Level, accent: string): string {
  if (level === 'critical') return CRITICAL;
  if (level === 'warning') return WARNING;
  return accent;
}
