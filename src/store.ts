import { create } from 'zustand';
import type { Snapshot } from './types/telemetry';

interface TelemetryState {
  snapshot: Snapshot | null;
  /** Local time the last tick arrived, for the stale-data indicator. */
  lastTickAt: number | null;
  setSnapshot: (s: Snapshot) => void;
}

export const useTelemetryStore = create<TelemetryState>((set) => ({
  snapshot: null,
  lastTickAt: null,
  setSnapshot: (snapshot) =>
    set((state) => {
      // The backend sends the process list every few ticks; keep the last one (same reference,
      // so the process list does not re-render) in between.
      const detail = snapshot.detail;
      const kept = detail && detail.processes == null ? state.snapshot?.detail?.processes : null;
      return {
        snapshot:
          detail && kept ? { ...snapshot, detail: { ...detail, processes: kept } } : snapshot,
        lastTickAt: Date.now(),
      };
    }),
}));
