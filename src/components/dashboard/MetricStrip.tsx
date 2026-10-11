import { formatPercent, formatRate, formatUsedTotal } from '../../lib/format';
import type { MetricKey } from '../../lib/levels';
import { ringSpecs } from '../../lib/metrics';
import { useTelemetryStore } from '../../store';
import type { CoreMetrics } from '../../types/telemetry';
import { Ring } from '../Ring';
import { RingIcon } from '../RingIcon';

function valueText(key: MetricKey, core: CoreMetrics | null | undefined): string {
  if (!core) return '--';
  switch (key) {
    case 'cpu':
      return formatPercent(core.cpu.percent);
    case 'ram':
      return formatUsedTotal(core.ram.used_bytes, core.ram.total_bytes);
    case 'igpu':
      return formatPercent(core.igpu?.percent);
    case 'dgpu':
      return formatPercent(core.dgpu?.percent);
    case 'disk':
      return formatPercent(core.disk?.percent);
    case 'net':
      return `↓${formatRate(core.net.down_bps)} ↑${formatRate(core.net.up_bps)}`;
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
            stroke={4}
            label={r.label}
            icon={<RingIcon metric={r.key} size={15} />}
          />
          <span className="text-[10px] font-medium text-slate-400">{r.label}</span>
          <span
            data-testid={`value-${r.key}`}
            className="w-full truncate text-center text-[10px] leading-tight text-slate-200 tabular-nums"
            title={valueText(r.key, core)}
          >
            {valueText(r.key, core)}
          </span>
        </div>
      ))}
    </div>
  );
}
