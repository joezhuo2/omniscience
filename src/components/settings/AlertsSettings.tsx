import { GlassCheckbox } from '@crystalflux/checkbox-glass';
import { useEffect, useState } from 'react';
import { formatSnoozeEnd, isSnoozed, minutesToTime, timeToMinutes } from '../../lib/alerts';
import type { Settings } from '../../settings';
import type { AlertRule } from '../../types/generated/AlertRule';
import type { AlertSettings as Alerts } from '../../types/generated/AlertSettings';
import { Row, ThresholdInput, Toggle } from './SettingsView';

type RuleKey = {
  [K in keyof Alerts]: Alerts[K] extends AlertRule ? K : never;
}[keyof Alerts];

interface RuleSpec {
  key: RuleKey;
  label: string;
  prefix: string;
  min?: number;
  max?: number;
  unit?: string;
  duration?: boolean;
  needsPing?: boolean;
}

const PERCENT = { prefix: 'above', min: 1, max: 100, unit: '%', duration: true };
const TEMP = { prefix: 'above', min: 30, max: 110, unit: '°C', duration: true };

const RULES: RuleSpec[] = [
  { key: 'cpu', label: 'CPU', ...PERCENT },
  { key: 'memory', label: 'Memory', ...PERCENT },
  { key: 'disk', label: 'Disk activity', ...PERCENT },
  { key: 'igpu', label: 'Integrated GPU', ...PERCENT },
  { key: 'dgpu', label: 'Discrete GPU', ...PERCENT },
  { key: 'cpu_temp', label: 'CPU temperature', ...TEMP },
  { key: 'gpu_temp', label: 'GPU temperature', ...TEMP },
  { key: 'disk_space', label: 'Low disk space', prefix: 'below', min: 1, max: 90, unit: '% free' },
  {
    key: 'network_down',
    label: 'Network down',
    prefix: 'no reply',
    duration: true,
    needsPing: true,
  },
  {
    key: 'ping_spike',
    label: 'Ping spike',
    prefix: 'above',
    min: 10,
    max: 5000,
    unit: 'ms',
    duration: true,
    needsPing: true,
  },
  { key: 'battery_full', label: 'Unplug reminder', prefix: 'at', min: 50, max: 100, unit: '%' },
  { key: 'battery_low', label: 'Low battery', prefix: 'at', min: 5, max: 50, unit: '%' },
];

const HOUR_MS = 3_600_000;
const SNOOZES = [1, 8];

export function AlertsSettings({
  settings,
  set,
}: {
  settings: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  const { alerts } = settings;
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 15_000);
    return () => clearInterval(t);
  }, []);

  const patch = (change: Partial<Alerts>) => set({ alerts: { ...alerts, ...change } });
  const patchRule = (key: RuleKey, change: Partial<AlertRule>) =>
    patch({ [key]: { ...alerts[key], ...change } });
  const snooze = (until: number) => {
    setNow(Date.now());
    patch({ snooze_until_ms: until });
  };
  const snoozed = isSnoozed(alerts.snooze_until_ms, now);
  const timeField = (label: string, field: 'quiet_start_min' | 'quiet_end_min') => (
    <input
      aria-label={label}
      type="time"
      value={minutesToTime(alerts[field])}
      onChange={(e) => {
        const min = timeToMinutes(e.target.value);
        if (min !== null) patch({ [field]: min });
      }}
      className="rounded border border-island-faint bg-island-hover px-1.5 py-0.5 text-island-text"
    />
  );
  const button =
    'rounded border border-island-faint px-2 py-0.5 text-[10px] text-island-text hover:border-island-muted';

  return (
    <>
      <Row label="Alerts" hint="Desktop notifications when a reading stays past its limit.">
        <Toggle
          label="Alerts"
          checked={settings.sustained_alerts}
          onChange={(v) => set({ sustained_alerts: v })}
        />
      </Row>
      {settings.sustained_alerts && (
        <>
          {RULES.map(({ key, label, prefix, min, max, unit, duration, needsPing }) => {
            const rule = alerts[key];
            const off = !rule.enabled;
            return (
              <div key={key} className="py-1.5 text-[11px] text-island-text">
                <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
                  <GlassCheckbox
                    className="text-[11px] text-island-text"
                    checked={rule.enabled}
                    onChange={(enabled) => patchRule(key, { enabled })}
                    label={label}
                  />
                  <span
                    className={`flex flex-wrap items-center gap-1 ${off ? 'pointer-events-none opacity-40' : ''}`}
                    inert={off}
                  >
                    <span className="text-island-muted">{prefix}</span>
                    {unit && min !== undefined && max !== undefined && (
                      <ThresholdInput
                        key={`${key}-t-${rule.threshold}`}
                        label={`${label} threshold`}
                        value={rule.threshold}
                        min={min}
                        max={max}
                        unit={unit}
                        onCommit={(threshold) => patchRule(key, { threshold })}
                      />
                    )}
                    {duration && (
                      <>
                        <span className="text-island-muted">for</span>
                        <ThresholdInput
                          key={`${key}-d-${rule.duration_s}`}
                          label={`${label} duration`}
                          value={rule.duration_s}
                          min={0}
                          max={3600}
                          unit="s"
                          onCommit={(duration_s) => patchRule(key, { duration_s })}
                        />
                      </>
                    )}
                  </span>
                </div>
                {needsPing && settings.ping_target === '' && (
                  <p className="text-[10px] text-island-muted">
                    Needs a ping target. Set one under Ping target.
                  </p>
                )}
              </div>
            );
          })}
          <Row label="Quiet hours" hint="No notifications during these hours.">
            <Toggle
              label="Quiet hours"
              checked={alerts.quiet_hours}
              onChange={(quiet_hours) => patch({ quiet_hours })}
            />
          </Row>
          {alerts.quiet_hours && (
            <Row label="Quiet from / to">
              <span className="flex items-center gap-1.5">
                {timeField('Quiet hours from', 'quiet_start_min')}
                <span className="text-island-muted">to</span>
                {timeField('Quiet hours to', 'quiet_end_min')}
              </span>
            </Row>
          )}
          <Row
            label={snoozed ? `Snoozed until ${formatSnoozeEnd(alerts.snooze_until_ms)}` : 'Snooze'}
          >
            {snoozed ? (
              <button className={button} onClick={() => snooze(0)}>
                Resume
              </button>
            ) : (
              <span className="flex gap-1.5">
                {SNOOZES.map((h) => (
                  <button
                    key={h}
                    className={button}
                    onClick={() => snooze(Date.now() + h * HOUR_MS)}
                  >
                    {h} h
                  </button>
                ))}
              </span>
            )}
          </Row>
        </>
      )}
    </>
  );
}
