import { Settings, Thermometer, Zap } from 'lucide-react';
import type { ReactNode } from 'react';
import { formatTemp, formatWatts } from '../../lib/format';
import { useTelemetryStore } from '../../store';
import { StaleDot } from './StaleDot';

function Badge({ icon, children }: { icon: ReactNode; children: ReactNode }) {
  return (
    <span className="flex items-center gap-1 rounded-full bg-island-hover px-2 py-0.5 text-[10px] text-island-text tabular-nums">
      {icon}
      {children}
    </span>
  );
}

/** Dashboard header. Also the drag handle for moving the overlay. */
export function Header({ onSettings }: { onSettings?: () => void }) {
  const watts = useTelemetryStore((s) => s.snapshot?.core.package_watts);
  const hottest = useTelemetryStore((s) =>
    s.snapshot?.detail?.temps.reduce<number | null>(
      (max, t) => (max == null || t.celsius > max ? t.celsius : max),
      null,
    ),
  );
  return (
    <header data-tauri-drag-region className="flex h-9 items-center gap-2 px-1">
      <span
        data-tauri-drag-region
        className="flex-1 text-xs font-semibold tracking-wide text-island-text"
      >
        Omniscience
      </span>
      <StaleDot />
      <Badge icon={<Zap size={10} aria-hidden className="text-island-warn" />}>
        {formatWatts(watts)}
      </Badge>
      <Badge icon={<Thermometer size={10} aria-hidden className="text-island-crit" />}>
        {formatTemp(hottest)}
      </Badge>
      {onSettings && (
        <button
          aria-label="Settings"
          className="rounded-md p-1 text-island-muted hover:bg-island-track/50 hover:text-island-text"
          onClick={onSettings}
        >
          <Settings size={14} aria-hidden />
        </button>
      )}
    </header>
  );
}
