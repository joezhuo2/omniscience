import { formatTemp, formatWatts } from '../../lib/format';
import { ACCENTS, levelColor } from '../../lib/levels';
import { useTelemetryStore } from '../../store';
import type { CoreSample } from '../../types/telemetry';
import { Card } from './Card';
import { Skeleton } from './Skeleton';
import { Tip } from '../Tip';

/** A performance limit below this is normal turbo jitter, not throttling. */
const THROTTLE_MIN_PCT = 5;

/** Cell fill: the level color, more opaque the busier the core. */
function cellStyle(core: CoreSample) {
  const alpha = 0.18 + (Math.min(100, Math.max(0, core.usage_pct)) / 100) * 0.82;
  return {
    backgroundColor: levelColor(core.level, ACCENTS.cpu),
    opacity: alpha,
  };
}

export function CpuHeatmap() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const cores = detail?.cores ?? [];
  const hottest = detail?.temps.reduce<number | null>(
    (max, t) => (max == null || t.celsius > max ? t.celsius : max),
    null,
  );
  const limit = detail?.cpu_limit_pct;
  const throttled =
    limit != null && Number.isFinite(limit) && limit >= THROTTLE_MIN_PCT && limit < 100;
  const cols = cores.length > 16 ? 8 : Math.max(4, Math.ceil(Math.sqrt(cores.length || 16)));
  return (
    <Card
      title="CPU cores"
      section="cpu-cores"
      aside={
        <span className="flex items-center gap-1.5 text-[11px] text-island-muted tabular-nums">
          {throttled && (
            <Tip content="Windows reports reduced available CPU performance from power policy or platform limits; this is not a direct thermal measurement">
              <span className="rounded-full bg-island-crit/20 px-1.5 text-[9px] text-island-crit">
                Throttled −{Math.round(limit)}%
              </span>
            </Tip>
          )}
          {cores.length ? `${cores.length} threads` : ''}
        </span>
      }
    >
      {!detail ? (
        <Skeleton className="h-16" />
      ) : (
        <div
          className="grid gap-1"
          style={{ gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` }}
        >
          {cores.map((c) => (
            <Tip
              key={c.id}
              content={
                <div className="text-[10px] leading-relaxed tabular-nums">
                  <div className="font-semibold">Core {c.id}</div>
                  <div>Usage {c.usage_pct.toFixed(0)}%</div>
                  <div>Clock {c.freq_mhz ? `${c.freq_mhz} MHz` : '--'}</div>
                  {c.watts != null && <div>Power {formatWatts(c.watts)}</div>}
                  <div>Package {formatTemp(hottest)}</div>
                </div>
              }
            >
              <div
                data-testid="core-cell"
                aria-label={`Core ${c.id}: ${Math.round(c.usage_pct)}%`}
                className="relative h-5 rounded-[4px] ring-island-muted hover:ring-1"
              >
                <div className="absolute inset-0 rounded-[4px]" style={cellStyle(c)} />
              </div>
            </Tip>
          ))}
        </div>
      )}
    </Card>
  );
}
