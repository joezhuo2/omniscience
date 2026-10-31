import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** Refresh real device state; never optimistically claim that a hardware write succeeded. */
export function useNativeControl<T>(readCommand: string, active = true) {
  const [status, setStatus] = useState<T | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const writing = useRef(false);
  const generation = useRef(0);
  const refresh = useCallback(async () => {
    if (writing.current) return;
    const request = ++generation.current;
    try {
      const next = await invoke<T>(readCommand);
      if (request === generation.current) {
        setStatus(next);
        setError('');
      }
    } catch (e) {
      if (request === generation.current) setError(String(e));
    }
  }, [readCommand]);
  useEffect(() => {
    if (!active) return;
    const initial = setTimeout(() => void refresh(), 0);
    const timer = setInterval(() => void refresh(), 5000);
    return () => {
      clearTimeout(initial);
      clearInterval(timer);
    };
  }, [active, refresh]);
  const apply = async (command: string, args: Record<string, unknown>) => {
    if (writing.current) return;
    writing.current = true;
    generation.current++;
    setBusy(true);
    setError('');
    try {
      setStatus(await invoke<T>(command, args));
    } catch (e) {
      setError(String(e));
      // A failed multi-step write may have changed part of the device state.
      try {
        setStatus(await invoke<T>(readCommand));
      } catch {
        /* Keep the original error. */
      }
    } finally {
      writing.current = false;
      setBusy(false);
    }
  };
  return { status, busy, error, apply, refresh };
}
