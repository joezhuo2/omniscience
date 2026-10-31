import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card } from '../dashboard/Card';

type OfferedUpdate = { id: string; title: string };

export function DriverUpdatesCard() {
  const [updates, setUpdates] = useState<OfferedUpdate[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const check = async () => {
    setBusy(true);
    setError('');
    setUpdates(null);
    try {
      setUpdates(await invoke<OfferedUpdate[]>('scan_driver_updates'));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Card
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
      {error && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error}
        </p>
      )}
    </Card>
  );
}
