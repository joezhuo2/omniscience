import { formatUptime } from '../../lib/format';
import { useTelemetryStore } from '../../store';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

const RECENT_EVENTS = 5;

export function SystemCard() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const now = useTelemetryStore((s) => s.lastTickAt);
  const boot = detail?.boot_ms;
  const uptime = boot != null && now != null ? formatUptime((now - boot) / 1000) : '--';
  const events = detail?.power_events.slice(0, RECENT_EVENTS) ?? [];
  return (
    <Card
      title="System"
      section="system"
      summary={<span className="text-[11px] text-island-text tabular-nums">{uptime}</span>}
    >
      {!detail ? (
        <Skeleton className="h-6" />
      ) : (
        <div className="space-y-1 text-[11px]">
          <div className="flex items-baseline justify-between gap-2">
            <span className="text-island-text">Uptime</span>
            <span data-testid="uptime" className="text-island-muted tabular-nums">
              {uptime}
            </span>
          </div>
          {boot != null && (
            <div className="flex items-baseline justify-between gap-2">
              <span className="text-island-text">Last boot</span>
              <span data-testid="last-boot" className="text-island-muted tabular-nums">
                {new Date(boot).toLocaleString()}
              </span>
            </div>
          )}
          {events.map((e) => (
            <div
              key={`${e.ts_ms}-${e.kind}`}
              data-testid="power-event"
              className="flex items-baseline justify-between gap-2 text-[10px] text-island-muted tabular-nums"
            >
              <span>{e.kind === 'sleep' ? 'Sleep' : 'Wake'}</span>
              <span>{new Date(e.ts_ms).toLocaleString()}</span>
            </div>
          ))}
        </div>
      )}
    </Card>
  );
}
