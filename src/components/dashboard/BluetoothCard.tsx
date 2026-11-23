import { Bluetooth } from 'lucide-react';
import { formatPercent } from '../../lib/format';
import { ACCENTS, WARNING } from '../../lib/levels';
import { useTelemetryStore } from '../../store';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

const LOW_PCT = 20;

export function BluetoothCard() {
  const devices = useTelemetryStore((s) => s.snapshot?.detail?.bluetooth);
  if (devices?.length === 0) return null;
  return (
    <Card title="Bluetooth" section="bluetooth">
      {!devices ? (
        <Skeleton className="h-6" />
      ) : (
        <div className="space-y-2">
          {devices.map((d) => {
            const low = d.percent <= LOW_PCT;
            return (
              <div key={d.name} data-testid="bluetooth-device" className="space-y-1">
                <div className="flex items-baseline justify-between gap-2 text-[11px]">
                  <span className="flex min-w-0 items-center gap-1.5 text-island-text">
                    <Bluetooth size={12} aria-hidden className="shrink-0" />
                    <span className="truncate">{d.name}</span>
                  </span>
                  <span
                    data-testid="bluetooth-percent"
                    className="shrink-0 tabular-nums"
                    style={{ color: low ? WARNING : undefined }}
                  >
                    {formatPercent(d.percent)}
                  </span>
                </div>
                <div className="h-1 overflow-hidden rounded-full bg-island-track">
                  <div
                    className="h-full rounded-full transition-[width] duration-500"
                    style={{
                      width: `${Math.min(100, Math.max(0, d.percent))}%`,
                      background: low ? WARNING : ACCENTS.dgpu,
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </Card>
  );
}
