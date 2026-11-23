import { BatteryCharging, BatteryMedium, PlugZap } from 'lucide-react';
import { formatDuration, formatWatts, formatWh } from '../../lib/format';
import { useSettingsStore } from '../../settings';
import { useTelemetryStore } from '../../store';
import type { Battery } from '../../types/telemetry';
import { Card } from './Card';
import { Num } from '../Num';
import { ChargeLimitBar } from './ChargeLimitBar';
import { Skeleton } from './Skeleton';
import { Tip } from '../Tip';

/** Below this charge the bar turns amber, below half of it red. */
const LOW_PCT = 20;
const pct = (p: number | null | undefined) => `${Math.round(p ?? 0)}%`;

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
function HealthRow({ battery }: { battery: Battery }) {
  const { design_capacity_mwh, full_charge_capacity_mwh, wear_pct, cycle_count } = battery;
  const capacity =
    design_capacity_mwh != null && full_charge_capacity_mwh != null
      ? `${formatWh(full_charge_capacity_mwh)} / ${formatWh(design_capacity_mwh)}`
      : null;
  if (wear_pct == null && capacity == null && cycle_count == null) return null;
  return (
    <div
      data-testid="battery-health"
      className="flex items-baseline justify-between gap-2 text-[11px] text-island-muted tabular-nums"
    >
      {wear_pct != null && (
        <Tip content="Capacity lost since new">
          <span>Wear {wear_pct.toFixed(1)}%</span>
        </Tip>
      )}
      {capacity != null && (
        <Tip content="Full-charge / design capacity">
          <span>{capacity}</span>
        </Tip>
      )}
      {cycle_count != null && <span>{cycle_count} cycles</span>}
    </div>
  );
}

/** Laptop battery. Not rendered on machines without one. */
export function BatteryCard() {
  const loaded = useTelemetryStore((s) => s.snapshot?.detail != null);
  const battery = useTelemetryStore((s) => s.snapshot?.detail?.battery);
  const configEnabled = useSettingsStore((s) => s.settings?.config_enabled ?? false);
  if (!loaded) {
    return (
      <Card section="battery" title="Battery">
        <Skeleton className="h-6" />
      </Card>
    );
  }
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
          <Num value={battery.percent} format={pct} />
        </span>
      }
      aside={
        battery.watts != null ? (
          <Tip content={battery.state === 'charging' ? 'Charge rate' : 'Drain rate'}>
            <span className="text-[11px] text-island-muted tabular-nums">
              <Num value={battery.watts} format={(w) => `${sign}${formatWatts(w)}`} />
            </span>
          </Tip>
        ) : null
      }
    >
      <div className="space-y-1">
        <div className="flex items-baseline justify-between gap-2 text-[11px]">
          <span className="flex items-center gap-1.5 text-island-text">
            <Icon size={12} aria-hidden />
            <span data-testid="battery-percent" className="tabular-nums">
              <Num value={battery.percent} format={pct} />
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
        <HealthRow battery={battery} />
        {configEnabled && <ChargeLimitBar />}
      </div>
    </Card>
  );
}
