import { Globe, Timer } from 'lucide-react';
import { useEffect } from 'react';
import { formatLatency, formatRate } from '../../lib/format';
import { useSettingsStore } from '../../settings';
import { useTelemetryStore } from '../../store';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

/** Adapters listed; the rest are usually idle virtual switches. */
const VISIBLE_ADAPTERS = 3;

export function NetworkCard() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const loaded = useSettingsStore((s) => s.settings != null);
  const load = useSettingsStore((s) => s.load);
  const target = useSettingsStore((s) => s.settings?.ping_target);
  // The ping target lives in settings, which are otherwise only loaded by the settings view.
  useEffect(() => {
    if (!loaded) void load();
  }, [loaded, load]);
  return (
    <Card
      title="Network"
      aside={
        target ? (
          <span
            title={`Round trip to ${target}`}
            className="flex items-center gap-1 text-[11px] text-island-muted tabular-nums"
          >
            <Timer size={11} aria-hidden />
            <span data-testid="ping">{formatLatency(detail?.ping_ms)}</span>
          </span>
        ) : null
      }
    >
      {!detail ? (
        <Skeleton className="h-6" />
      ) : (
        <div className="space-y-1 text-[11px]">
          {detail.net_adapters.length === 0 && (
            <p className="text-island-muted">No active adapter</p>
          )}
          {detail.net_adapters.slice(0, VISIBLE_ADAPTERS).map((a) => (
            <div key={a.name} className="flex items-baseline justify-between gap-2">
              <span className="min-w-0 truncate text-island-text" title={a.name}>
                {a.name}
                {a.ipv4 && <span className="pl-1.5 text-island-muted">{a.ipv4}</span>}
              </span>
              <span className="shrink-0 text-island-muted tabular-nums">
                ↓ {formatRate(a.down_bps)} · ↑ {formatRate(a.up_bps)}
              </span>
            </div>
          ))}
          {detail.public_ip && (
            <div className="flex items-center gap-1 pt-0.5 text-[10px] text-island-muted">
              <Globe size={10} aria-hidden />
              Public IP <span className="text-island-text tabular-nums">{detail.public_ip}</span>
            </div>
          )}
        </div>
      )}
    </Card>
  );
}
