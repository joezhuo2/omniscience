import { listen } from '@tauri-apps/api/event';
import { useEffect } from 'react';
import { useTelemetryStore } from '../store';
import { TICK_EVENT, type Snapshot } from '../types/telemetry';

/** Subscribes the store to the backend telemetry ticks for the component lifetime. */
export function useTelemetry() {
  const setSnapshot = useTelemetryStore((s) => s.setSnapshot);
  useEffect(() => {
    const unlisten = listen<Snapshot>(TICK_EVENT, (e) => setSnapshot(e.payload));
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [setSnapshot]);
}
