import { BatteryCharging, BatteryMedium, PlugZap } from 'lucide-react';
import { formatDuration, formatWatts } from '../../lib/format';
import { useTelemetryStore } from '../../store';
import type { Battery } from '../../types/telemetry';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

/** Below this charge the bar turns amber, below half of it red. */
const LOW_PCT = 20;

function summary(b: Battery): string {
  switch (b.state) {
    case 'discharging':
      return b.secs_remaining != null ? `${formatDuration(b.secs_remaining)} left` : 'On battery';
    case 'charging':
      return b.secs_remaining != null ? `${formatDuration(b.secs_remaining)} to full` : 'Charging';
    case 'idle':
      return 'Plugged in, not charging';
  }
}

function barColor(b: Battery): string {
  if (b.state !== 'discharging') return 'var(--color-island-ok)';
  if (b.percent < LOW_PCT / 2) return 'var(--color-island-crit)';
  return b.percent < LOW_PCT ? 'var(--color-island-warn)' : 'var(--color-island-ok)';
}

/** Laptop battery. Not rendered on machines without one. */
export function BatteryCard() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  if (!detail) {
    return (
      <Card section="battery" title="Battery">
        <Skeleton className="h-6" />
      </Card>
    );
  }
  const battery = detail.battery;
  if (!battery) return null;
  const Icon =
    battery.state === 'charging'
      ? BatteryCharging
      : battery.state === 'idle'
        ? PlugZap
        : BatteryMedium;
  const sign = battery.state === 'discharging' ? '−' : battery.state === 'charging' ? '+' : '';
  return (
    <Card
      title="Battery"
      section="battery"
      summary={
        <span className="text-[11px] text-island-text tabular-nums">
          {Math.round(battery.percent)}%
        </span>
      }
      aside={
        battery.watts != null ? (
          <span
            title={battery.state === 'charging' ? 'Charge rate' : 'Drain rate'}
            className="text-[11px] text-island-muted tabular-nums"
          >
            {sign}
            {formatWatts(battery.watts)}
          </span>
        ) : null
      }
    >
      <div className="space-y-1">
        <div className="flex items-baseline justify-between gap-2 text-[11px]">
          <span className="flex items-center gap-1.5 text-island-text">
            <Icon size={12} aria-hidden />
            <span data-testid="battery-percent" className="tabular-nums">
              {Math.round(battery.percent)}%
            </span>
          </span>
          <span data-testid="battery-summary" className="text-island-muted tabular-nums">
            {summary(battery)}
          </span>
        </div>
        <div className="h-1 overflow-hidden rounded-full bg-island-track">
          <div
            className="h-full rounded-full transition-[width] duration-500"
            style={{ width: `${battery.percent}%`, background: barColor(battery) }}
          />
        </div>
      </div>
    </Card>
  );
}
