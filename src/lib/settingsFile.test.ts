import { describe, expect, it } from 'vitest';
import { buildSettingsDocument, importedSettings, parseSettingsDocument } from './settingsFile';
import { RESET_GROUPS, resetPatch } from './settingsReset';
import type { Settings } from '../settings';

const alerts = (snooze: number, quiet = false) =>
  ({ quiet_hours: quiet, snooze_until_ms: snooze }) as Settings['alerts'];

const make = (over: Partial<Settings> = {}): Settings =>
  ({
    warn_threshold: 70,
    crit_threshold: 90,
    cpu_temp_warn_c: 80,
    cpu_temp_crit_c: 90,
    hotkey: 'Alt+O',
    launch_at_startup: false,
    pill_rings: ['cpu'],
    pill_numbers: 'off',
    hover_open_delay_ms: 0,
    hover_close_delay_ms: 0,
    dodge_browser_tabs: false,
    poll_interval_ms: 1000,
    history_hours: 24,
    ping_target: '',
    show_public_ip: false,
    sensor_details: false,
    energy_price_per_kwh: 0,
    energy_currency: '$',
    temp_unit: 'celsius',
    net_unit: 'bytes',
    sustained_alerts: true,
    profiles: [],
    active_profile: '',
    metric_thresholds: { cpu: null, ram: null, igpu: null, dgpu: null, disk: null, net: null },
    alerts: alerts(0),
    ...over,
  }) as unknown as Settings;

const glass = { on: true, opacity: 60 };

describe('settings document', () => {
  it('round-trips', () => {
    const settings = make({ temp_unit: 'fahrenheit' });
    const text = JSON.stringify(buildSettingsDocument(settings, { 'font-size': '110%' }, glass));
    expect(parseSettingsDocument(text, make())).toEqual({
      app: 'omniscience',
      version: 1,
      settings,
      appearance: { 'font-size': '110%' },
      glass,
    });
  });

  it('rejects the wrong shape', () => {
    const good = buildSettingsDocument(make(), {}, glass);
    const bad = (doc: unknown) => () => parseSettingsDocument(JSON.stringify(doc), make());
    expect(() => parseSettingsDocument('nope', make())).toThrow('valid JSON');
    expect(bad([])).toThrow('Omniscience settings');
    expect(bad({ ...good, app: 'other' })).toThrow('Omniscience settings');
    expect(bad({ ...good, version: 2 })).toThrow('Omniscience settings');
    expect(bad({ ...good, settings: null })).toThrow('settings');
    expect(bad({ ...good, settings: { ...good.settings, hotkey: 5 } })).toThrow('"hotkey"');
    expect(bad({ ...good, settings: { ...good.settings, pill_rings: 'cpu' } })).toThrow(
      '"pill_rings"',
    );
    const missing = { ...good.settings } as Record<string, unknown>;
    delete missing.temp_unit;
    expect(bad({ ...good, settings: missing })).toThrow('"temp_unit"');
    expect(bad({ ...good, appearance: { a: 1 } })).toThrow('appearance');
    expect(bad({ ...good, glass: { on: 'yes', opacity: 50 } })).toThrow('glass');
  });

  it('drops unknown settings and clamps glass opacity', () => {
    const good = buildSettingsDocument(make(), {}, { on: false, opacity: 500 });
    const text = JSON.stringify({ ...good, settings: { ...good.settings, extra: 1 } });
    const doc = parseSettingsDocument(text, make());
    expect('extra' in doc.settings).toBe(false);
    expect(doc.glass).toEqual({ on: false, opacity: 95 });
  });

  it('keeps the machine-specific values when applying', () => {
    const doc = buildSettingsDocument(
      make({ launch_at_startup: true, alerts: alerts(99, true) }),
      {},
      glass,
    );
    const applied = importedSettings(doc, make({ alerts: alerts(5) }));
    expect(applied.launch_at_startup).toBe(false);
    expect(applied.alerts).toEqual(alerts(5, true));
  });
});

describe('resetPatch', () => {
  const defaults = make({ warn_threshold: 75, pill_rings: ['cpu', 'ram'] });
  const current = make({ alerts: alerts(7, true) });

  it('touches only the group fields', () => {
    const thresholds = resetPatch('thresholds', defaults, current);
    expect(Object.keys(thresholds).sort()).toEqual([
      'cpu_temp_crit_c',
      'cpu_temp_warn_c',
      'crit_threshold',
      'metric_thresholds',
      'warn_threshold',
    ]);
    expect(thresholds.warn_threshold).toBe(75);
    expect(resetPatch('island', defaults, current).pill_rings).toEqual(['cpu', 'ram']);
    expect(Object.keys(resetPatch('monitoring', defaults, current))).toContain('net_unit');
    expect(Object.keys(resetPatch('profiles', defaults, current)).sort()).toEqual([
      'active_profile',
      'profiles',
    ]);
  });

  it('resets alert rules but keeps the snooze', () => {
    const patch = resetPatch('alerts', defaults, current);
    expect(Object.keys(patch).sort()).toEqual(['alerts', 'sustained_alerts']);
    expect(patch.alerts).toEqual(alerts(7));
  });

  it('knows every group and returns nothing for an unknown one', () => {
    expect(RESET_GROUPS.map((g) => g.id)).toEqual([
      'thresholds',
      'island',
      'monitoring',
      'alerts',
      'profiles',
    ]);
    expect(resetPatch('nope', defaults, current)).toEqual({});
  });
});
