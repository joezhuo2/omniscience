import { Zap } from 'lucide-react';
import { formatBytes, formatEnergy, formatPercent, formatRpm, formatWatts } from '../../lib/format';
import { ACCENTS } from '../../lib/levels';
import { useSettingsStore } from '../../settings';
import { useTelemetryStore } from '../../store';
import type { GpuDetail } from '../../types/telemetry';
import { Card } from './Card';
import { Skeleton } from './Skeleton';
import { Num } from '../Num';
import { Tip } from '../Tip';

function memoryText(g: GpuDetail): string {
  const gb = 1024 ** 3;
  if (g.kind === 'discrete' && g.mem_used_bytes != null && g.mem_total_bytes) {
    return `${(g.mem_used_bytes / gb).toFixed(1)} / ${(g.mem_total_bytes / gb).toFixed(1)} GB`;
  }
  return g.mem_used_bytes != null ? `${formatBytes(g.mem_used_bytes)} shared` : '--';
}

function clockText(g: GpuDetail): string {
  return [
    g.core_clock_mhz != null && `Core ${Math.round(g.core_clock_mhz)} MHz`,
    g.mem_clock_mhz != null && `Mem ${Math.round(g.mem_clock_mhz)} MHz`,
    g.encode_pct != null && `Enc ${formatPercent(g.encode_pct)}`,
    g.decode_pct != null && `Dec ${formatPercent(g.decode_pct)}`,
  ]
    .filter(Boolean)
    .join(' · ');
}

function GpuRow({ gpu }: { gpu: GpuDetail }) {
  const accent = gpu.kind === 'discrete' ? ACCENTS.dgpu : ACCENTS.igpu;
  const usage = gpu.usage_pct ?? 0;
  const clocks = clockText(gpu);
  return (
    <div className="space-y-1">
      <div className="flex items-baseline justify-between gap-2 text-[11px]">
        <span className="flex min-w-0 items-center gap-1.5">
          <Tip content={gpu.name}>
            <span className="truncate text-island-text">{gpu.name}</span>
          </Tip>
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
      {clocks && (
        <div data-testid="gpu-clocks" className="text-[10px] text-island-muted tabular-nums">
          {clocks}
        </div>
      )}
    </div>
  );
}

export function GpuPowerCard() {
  const detail = useTelemetryStore((s) => s.snapshot?.detail);
  const packageWatts = useTelemetryStore((s) => s.snapshot?.core.package_watts);
  const power = detail?.power;
  const price = useSettingsStore((s) => s.settings?.energy_price_per_kwh ?? 0);
  const currency = useSettingsStore((s) => s.settings?.energy_currency ?? '$');
  const energy = detail?.energy_today_wh;
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
      section="gpu-power"
      aside={
        <Tip content="Package power">
          <span
            data-testid="package-power"
            className="flex items-center gap-1 text-[11px] text-island-warn tabular-nums"
          >
            <Zap size={11} aria-hidden />
            <Num value={packageWatts} format={formatWatts} />
          </span>
        </Tip>
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
          {energy != null && Number.isFinite(energy) && (
            <div data-testid="energy-today" className="text-[10px] text-island-muted tabular-nums">
              Energy today {formatEnergy(energy)}
              {price > 0 && ` · ${currency}${((energy / 1000) * price).toFixed(2)}`}
            </div>
          )}
        </div>
      )}
    </Card>
  );
}
