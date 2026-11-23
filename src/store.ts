import { create } from 'zustand';
import type { Snapshot } from './types/telemetry';

interface TelemetryState {
  snapshot: Snapshot | null;
  /** Local time the last tick arrived, for the stale-data indicator. */
  lastTickAt: number | null;
  setSnapshot: (s: Snapshot) => void;
}

/** Detail fields that rarely change: an equal value keeps the old reference, so cards that
 * select them skip the re-render. */
const STABLE = ['battery', 'bluetooth', 'disks', 'power_events', 'wifi', 'fans', 'memory'] as const;

function same(a: unknown, b: unknown) {
  return a === b || JSON.stringify(a) === JSON.stringify(b);
}

export const useTelemetryStore = create<TelemetryState>((set) => ({
  snapshot: null,
  lastTickAt: null,
  setSnapshot: (snapshot) =>
    set((state) => {
      // The backend sends the process list every few ticks; keep the last one (same reference,
      // so the process list does not re-render) in between.
      const prev = state.snapshot?.detail;
      let detail = snapshot.detail;
      if (detail && prev) {
        detail = { ...detail };
        if (detail.processes == null) detail.processes = prev.processes;
        for (const key of STABLE) {
          if (same(detail[key], prev[key])) (detail as Record<string, unknown>)[key] = prev[key];
        }
      }
      return { snapshot: detail ? { ...snapshot, detail } : snapshot, lastTickAt: Date.now() };
    }),
}));
