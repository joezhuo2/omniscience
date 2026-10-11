import type { Level } from '../types/telemetry';

/** Per-metric accent used while a reading is normal (iOS system colors, as in islet). */
export const ACCENTS = {
  cpu: '#64D2FF',
  ram: '#5E5CE6',
  igpu: '#66D4CF',
  dgpu: '#30D158',
  disk: '#BF5AF2',
  net: '#0A84FF',
} as const;

export type MetricKey = keyof typeof ACCENTS;

export const WARNING = '#FFB340';
export const CRITICAL = '#FF453A';

/** Ring color for a level: the metric accent when normal, otherwise amber or red. */
export function levelColor(level: Level, accent: string): string {
  if (level === 'critical') return CRITICAL;
  if (level === 'warning') return WARNING;
  return accent;
}
