import { Zap } from 'lucide-react';
import { formatBytes, formatPercent, formatRpm, formatWatts } from '../../lib/format';
import { ACCENTS } from '../../lib/levels';
import { useTelemetryStore } from '../../store';
import type { GpuDetail } from '../../types/telemetry';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

function memoryText(g: GpuDetail): string {
  const gb = 1024 ** 3;
  if (g.kind === 'discrete' && g.mem_used_bytes != null && g.mem_total_bytes) {
    return `${(g.mem_used_bytes / gb).toFixed(1)} / ${(g.mem_total_bytes / gb).toFixed(1)} GB`;
  }
  return g.mem_used_bytes != null ? `${formatBytes(g.mem_used_bytes)} shared` : '--';
}

function GpuRow({ gpu }: { gpu: GpuDetail }) {
  const accent = gpu.kind === 'discrete' ? ACCENTS.dgpu : ACCENTS.igpu;
  const usage = gpu.usage_pct ?? 0;
  return (
    <div className="space-y-1">
      <div className="flex items-baseline justify-between gap-2 text-[11px]">
        <span className="flex min-w-0 items-center gap-1.5">
          <span className="truncate text-island-text" title={gpu.name}>
            {gpu.name}
          </span>
          {gpu.throttling && (
            <span className="shrink-0 rounded-full bg-island-crit/20 px-1.5 text-[9px] text-island-crit">
              Throttling
            </span>
          )}
        </span>
        <span className="shrink-0 text-island-muted tabular-nums">
          {formatPercent(gpu.usage_pct)} · {memoryText(gpu)}
          {gpu.fan_rpm != null && ` · ${formatRpm(gpu.fan_rpm)}`}
        </span>
      </div>
      <div className="h-1 overflow-hidden rounded-full bg-island-track">
        <div
          className="h-full rounded-full transition-[width] duration-500"
          style={{ width: `${Math.min(100, usage)}%`, background: accent }}
        />
      </div>
    </div>
  );
}

export function GpuPowerCard() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const packageWatts = useTelemetryStore((s) => s.snapshot?.core.package_watts);
  const power = detail?.power;
  const parts = power
    ? (
        [
          ['APU', power.apu],
          ['CPU', power.cpu],
          ['GPU', power.gpu],
          ['NPU', power.npu],
        ] as const
      ).filter(([, w]) => w != null)
    : [];
  return (
    <Card
      title="GPU & Power"
      aside={
        <span className="flex items-center gap-1 text-[11px] text-island-warn tabular-nums">
          <Zap size={11} aria-hidden />
          {formatWatts(packageWatts)}
        </span>
      }
    >
      {!detail ? (
        <div className="space-y-2">
          <Skeleton className="h-6" />
          <Skeleton className="h-6" />
        </div>
      ) : (
        <div className="space-y-2">
          {detail.gpus.length === 0 && (
            <p className="text-[11px] text-island-muted">No GPU found</p>
          )}
          {detail.gpus.map((gpu) => (
            <GpuRow key={gpu.name} gpu={gpu} />
          ))}
          {parts.length > 0 && (
            <div className="flex gap-3 pt-0.5 text-[10px] text-island-muted tabular-nums">
              {parts.map(([label, w]) => (
                <span key={label}>
                  {label} {formatWatts(w)}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </Card>
  );
}
