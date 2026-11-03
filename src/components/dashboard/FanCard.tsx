import { Fan } from 'lucide-react';
import { formatRpm } from '../../lib/format';
import { useTelemetryStore } from '../../store';
import { Card } from './Card';

function FanRow({ label, rpm }: { label: string; rpm: number }) {
  return (
    <div className="flex items-baseline justify-between gap-2 text-[11px]">
      <span className="flex items-center gap-1.5 text-island-text">
        <Fan size={12} aria-hidden />
        {label}
      </span>
      <span data-testid={`fan-${label.toLowerCase()}`} className="text-island-muted tabular-nums">
        {rpm > 0 ? formatRpm(rpm) : 'Stopped'}
      </span>
    </div>
  );
}

export function FanCard() {
  const fans = useTelemetryStore((s) => s.snapshot?.detail?.fans);
  if (!fans) return null;
  return (
    <Card title="Fans" section="fans">
      <div className="space-y-1">
        {fans.cpu_rpm != null && <FanRow label="CPU" rpm={fans.cpu_rpm} />}
        {fans.gpu_rpm != null && <FanRow label="GPU" rpm={fans.gpu_rpm} />}
      </div>
    </Card>
  );
}
