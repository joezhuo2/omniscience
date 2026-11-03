import { useStaleness } from '../../hooks/useStaleness';

/** Amber dot shown when telemetry stops arriving. Ticks once a second while mounted. */
export function StaleDot() {
  const { stale } = useStaleness();
  if (!stale) return null;
  return (
    <span
      role="status"
      aria-label="Telemetry stalled"
      title="No data for a few seconds"
      className="h-1.5 w-1.5 rounded-full bg-island-warn"
    />
  );
}
