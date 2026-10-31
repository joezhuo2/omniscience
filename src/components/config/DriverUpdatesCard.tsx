import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card } from '../dashboard/Card';

type OfferedUpdate = { id: string; title: string };
export type ScanStatus = {
  phase: 'idle' | 'checking' | 'complete' | 'failed';
  updates: OfferedUpdate[];
  error: string | null;
};

export function DriverUpdatesCard({ active = true }: { active?: boolean }) {
  const [scan, setScan] = useState<ScanStatus>({ phase: 'idle', updates: [], error: null });
  const [error, setError] = useState('');
  const busy = scan.phase === 'checking';
  const updates = scan.phase === 'complete' && !error ? scan.updates : null;
  useEffect(() => {
    if (!active) return;
    let cancelled = false;
    const refresh = async () => {
      try {
        const next = await invoke<ScanStatus>('get_system_update_status');
        if (!cancelled && next) setScan(next);
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    };
    void refresh();
    const timer = busy ? setInterval(() => void refresh(), 1000) : undefined;
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [active, busy]);
  const check = async () => {
    setError('');
    setScan({ phase: 'checking', updates: [], error: null });
    try {
      setScan(await invoke<ScanStatus>('scan_driver_updates'));
    } catch (e) {
      setError(String(e));
      setScan({ phase: 'failed', updates: [], error: null });
    }
  };
  return (
    <Card
      section="driver-updates"
      summary={
        <span className="text-[11px] text-island-muted">
          {busy ? 'Checking…' : updates ? `${updates.length} offered` : 'Not checked'}
        </span>
      }
      title="Driver & software updates"
      aside={
        <button
          type="button"
          className="config-button"
          disabled={busy}
          onClick={() => void check()}
        >
          {busy ? 'Checking…' : 'Check now'}
        </button>
      }
    >
      <p className="text-[11px] text-island-muted">
        Checks Windows Update, including optional drivers and Microsoft software. Other installed
        apps are not checked.
      </p>
      <div className="mt-2 text-xs" role="status" aria-live="polite">
        {busy && <p>Checking for updates. This can take a few minutes.</p>}
        {updates?.length === 0 && (
          <p>All drivers and software offered by Windows Update are up to date.</p>
        )}
        {updates && updates.length > 0 && (
          <ul className="space-y-2">
            {updates.map((update) => (
              <li key={update.id} className="flex items-start justify-between gap-3">
                <span className="min-w-0 break-words">{update.title}</span>
                <button
                  type="button"
                  className="config-button shrink-0"
                  onClick={() => {
                    void invoke('open_update_download', { id: update.id }).catch((e: unknown) =>
                      setError(String(e)),
                    );
                  }}
                >
                  Download
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
      {(error || scan.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || scan.error}
        </p>
      )}
    </Card>
  );
}
