import { formatGhz, formatPercent, formatRateRounded, formatUsedTotal } from '../../lib/format';
import type { MetricKey } from '../../lib/levels';
import { RING_STROKE, ringSpecs } from '../../lib/metrics';
import { useTelemetryStore } from '../../store';
import type { CoreMetrics } from '../../types/telemetry';
import { Ring } from '../Ring';
import { RingIcon } from '../RingIcon';

/** One or two value lines under each ring. */
function valueLines(key: MetricKey, core: CoreMetrics | null | undefined): string[] {
  if (!core) return ['--'];
  switch (key) {
    case 'cpu':
      return [formatPercent(core.cpu.percent), formatGhz(core.cpu_mhz)];
    case 'ram':
      return [formatUsedTotal(core.ram.used_bytes, core.ram.total_bytes)];
    case 'igpu':
      return [formatPercent(core.igpu?.percent)];
    case 'dgpu':
      return [formatPercent(core.dgpu?.percent)];
    case 'disk':
      return [formatPercent(core.disk?.percent)];
    case 'net':
      return [
        `↓ ${formatRateRounded(core.net.down_bps)}`,
        `↑ ${formatRateRounded(core.net.up_bps)}`,
      ];
  }
}

/** The six rings again, larger, with labels and values. */
export function MetricStrip() {
  const core = useTelemetryStore((s) => s.snapshot?.core);
  return (
    <div className="grid grid-cols-6 gap-1">
      {ringSpecs(core).map((r) => (
        <div key={r.key} className="flex min-w-0 flex-col items-center gap-1">
          <Ring
            outer={r.outer}
            inner={r.inner}
            size={42}
            stroke={r.inner ? RING_STROKE.stripPair : RING_STROKE.strip}
            label={r.label}
            icon={<RingIcon metric={r.key} size={14} />}
          />
          <span className="text-[10px] font-medium text-island-muted">{r.label}</span>
          <span
            data-testid={`value-${r.key}`}
            className="flex w-full flex-col items-center text-[10px] leading-tight text-island-text tabular-nums"
          >
            {valueLines(r.key, core).map((line, i) => (
              <span key={i} className="max-w-full truncate whitespace-nowrap" title={line}>
                {line}
              </span>
            ))}
          </span>
        </div>
      ))}
    </div>
  );
}
