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
  setSnapshot: (snapshot) => set({ snapshot, lastTickAt: Date.now() }),
}));
