import { Globe, Timer, Wifi } from 'lucide-react';
import { useEffect } from 'react';
import { formatBytes, formatLatency, formatMbps, formatPercent } from '../../lib/format';
import { useSettingsStore } from '../../settings';
import { useTelemetryStore } from '../../store';
import { Card } from './Card';
import { Skeleton } from './Skeleton';
import { Num } from '../Num';
import { Tip } from '../Tip';

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
  const ping = target ? (
    <Tip content={`Round trip to ${target}`}>
      <span className="flex shrink-0 items-center gap-1 whitespace-nowrap text-[11px] text-island-muted tabular-nums">
        <Timer size={11} aria-hidden />
        <span data-testid="ping">
          <Num value={detail?.ping_ms} format={formatLatency} />
        </span>
        {detail?.ping_jitter_ms != null && (
          <span data-testid="ping-jitter">±{formatLatency(detail.ping_jitter_ms)}</span>
        )}
        {detail?.ping_loss_pct != null && (
          <span data-testid="ping-loss">{formatPercent(detail.ping_loss_pct)} loss</span>
        )}
      </span>
    </Tip>
  ) : null;
  return (
    <Card section="network" title="Network" summary={ping}>
      {!detail ? (
        <Skeleton className="h-6" />
      ) : (
        <div className="space-y-1 text-[11px]">
          {detail.net_adapters.length === 0 && (
            <div className="flex items-center justify-between gap-2">
              <p className="text-island-muted">No active adapter</p>
              {ping}
            </div>
          )}
          {detail.net_adapters.slice(0, VISIBLE_ADAPTERS).map((a, index) => (
            <div key={a.name} className="flex items-baseline justify-between gap-2">
              <Tip content={a.name}>
                <span className="min-w-0 truncate text-island-text">
                  {a.name}
                  {a.ipv4 && <span className="pl-1.5 text-island-muted">{a.ipv4}</span>}
                </span>
              </Tip>
              {index === 0 && ping}
            </div>
          ))}
          {detail.wifi && (
            <div
              data-testid="wifi"
              className="flex items-center gap-1 text-[10px] text-island-muted"
            >
              <Wifi size={10} aria-hidden />
              <span className="min-w-0 truncate text-island-text">{detail.wifi.ssid}</span>
              <span className="tabular-nums">
                {formatPercent(detail.wifi.signal_pct)}
                {detail.wifi.link_mbps != null && ` · ${formatMbps(detail.wifi.link_mbps)}`}
              </span>
            </div>
          )}
          {(detail.data_today_bytes != null || detail.data_month_bytes != null) && (
            <div className="flex items-center gap-1.5 text-[10px] text-island-muted tabular-nums">
              <span data-testid="data-usage">
                Today {formatBytes(detail.data_today_bytes)} · This month{' '}
                {formatBytes(detail.data_month_bytes)}
              </span>
              {detail.metered === true && (
                <span
                  data-testid="metered"
                  className="shrink-0 rounded-full bg-island-warn/20 px-1.5 text-[9px] text-island-warn"
                >
                  Metered
                </span>
              )}
            </div>
          )}
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
