import type { Settings } from '../settings';

export const RESET_GROUPS: { id: string; label: string; keys: (keyof Settings)[] }[] = [
  {
    id: 'thresholds',
    label: 'Thresholds and units',
    keys: [
      'warn_threshold',
      'crit_threshold',
      'cpu_temp_warn_c',
      'cpu_temp_crit_c',
      'metric_thresholds',
      'temp_unit',
      'net_unit',
    ],
  },
  {
    id: 'island',
    label: 'Island and pill',
    keys: [
      'hover_open_delay_ms',
      'hover_close_delay_ms',
      'dodge_browser_tabs',
      'pill_rings',
      'pill_numbers',
    ],
  },
  {
    id: 'monitoring',
    label: 'Monitoring',
    keys: [
      'poll_interval_ms',
      'history_hours',
      'ping_target',
      'show_public_ip',
      'sensor_details',
      'energy_price_per_kwh',
      'energy_currency',
    ],
  },
  { id: 'alerts', label: 'Alerts', keys: ['sustained_alerts', 'alerts'] },
  { id: 'integrations', label: 'Integrations', keys: ['integrations'] },
  { id: 'profiles', label: 'Profiles', keys: ['profiles', 'active_profile'] },
];

export function resetPatch(id: string, defaults: Settings, current: Settings): Partial<Settings> {
  const group = RESET_GROUPS.find((g) => g.id === id);
  const patch = Object.fromEntries((group?.keys ?? []).map((k) => [k, defaults[k]]));
  if (group?.keys.includes('alerts')) {
    patch.alerts = { ...defaults.alerts, snooze_until_ms: current.alerts.snooze_until_ms };
  }
  return patch;
}
