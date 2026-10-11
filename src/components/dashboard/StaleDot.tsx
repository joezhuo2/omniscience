import { useEffect, useState } from 'react';
import { useTelemetryStore } from '../../store';

/** No tick for this long means the collector stalled. */
const STALE_MS = 3000;

/** Amber dot shown when telemetry stops arriving. Ticks once a second while mounted. */
export function StaleDot() {
  const lastTickAt = useTelemetryStore((s) => s.lastTickAt);
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);
  if (lastTickAt == null || now - lastTickAt < STALE_MS) return null;
  return (
    <span
      role="status"
      aria-label="Telemetry stalled"
      title="No data for a few seconds"
      className="h-1.5 w-1.5 rounded-full bg-island-warn"
    />
  );
}
