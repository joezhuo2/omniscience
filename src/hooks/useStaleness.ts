import { useEffect, useState } from 'react';
import { useTelemetryStore } from '../store';

/** No tick for this long means the collector stalled. */
const STALE_MS = 3000;

export function useStaleness() {
  const lastTickAt = useTelemetryStore((s) => s.lastTickAt);
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, []);
  const age = lastTickAt == null ? 0 : Math.max(0, now - lastTickAt);
  return { stale: lastTickAt != null && age >= STALE_MS, ageSeconds: Math.floor(age / 1000) };
}
