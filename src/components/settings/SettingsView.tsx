import { invoke } from '@tauri-apps/api/core';
import { ArrowLeft } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import { acceleratorFromEvent, displayAccelerator } from '../../lib/hotkey';
import { useSettingsStore, type Settings } from '../../settings';

const IS_MAC = typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform);
const INTERVALS = [500, 1000, 2000, 5000];

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex items-center justify-between gap-3 py-1.5 text-[11px] text-slate-300">
      <span>{label}</span>
      {children}
    </label>
  );
}

function Toggle({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      className={`relative h-4 w-7 rounded-full transition-colors ${
        checked ? 'bg-cyan-500' : 'bg-slate-600'
      }`}
    >
      <span
        className={`absolute top-0.5 h-3 w-3 rounded-full bg-white transition-transform ${
          checked ? 'translate-x-3.5' : 'translate-x-0.5'
        }`}
      />
    </button>
  );
}

/** Number input that commits on blur or Enter, so typing "85" does not apply "8" first. */
function ThresholdInput({
  value,
  label,
  onCommit,
}: {
  value: number;
  label: string;
  onCommit: (v: number) => void;
}) {
  // Remounted (via `key`) when the applied value changes, which resets the draft.
  const [draft, setDraft] = useState(String(value));
  const commit = () => {
    const n = Number(draft);
    if (Number.isFinite(n) && n !== value) onCommit(n);
    else setDraft(String(value));
  };
  return (
    <span className="flex items-center gap-1">
      <input
        aria-label={label}
        type="number"
        min={1}
        max={99}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => e.key === 'Enter' && commit()}
        className="w-12 rounded border border-slate-600 bg-slate-900 px-1.5 py-0.5 text-right text-slate-100 tabular-nums"
      />
      %
    </span>
  );
}

function HotkeyRecorder({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const [recording, setRecording] = useState(false);
  useEffect(() => {
    if (!recording) return;
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === 'Escape') return setRecording(false);
      const accel = acceleratorFromEvent(e);
      if (accel) {
        setRecording(false);
        if (accel !== value) onChange(accel);
      }
    };
    window.addEventListener('keydown', onKey, true);
    return () => window.removeEventListener('keydown', onKey, true);
  }, [recording, value, onChange]);
  return (
    <button
      aria-label="Show/hide hotkey"
      onClick={() => setRecording((r) => !r)}
      className={`rounded border px-2 py-0.5 font-mono text-[10px] ${
        recording
          ? 'border-cyan-400 text-cyan-300'
          : 'border-slate-600 text-slate-200 hover:border-slate-400'
      }`}
    >
      {recording ? 'Press keys…' : displayAccelerator(value, IS_MAC)}
    </button>
  );
}

export function SettingsView({ onBack }: { onBack: () => void }) {
  const { settings, error, load, update } = useSettingsStore();
  useEffect(() => {
    if (!settings) void load();
  }, [settings, load]);

  const set = (patch: Partial<Settings>) => void update(patch);

  return (
    <div data-testid="settings" className="px-1 pb-1">
      <header className="flex h-9 items-center gap-2">
        <button
          aria-label="Back"
          onClick={onBack}
          className="rounded-md p-1 text-slate-400 hover:bg-slate-700/50 hover:text-slate-100"
        >
          <ArrowLeft size={14} aria-hidden />
        </button>
        <h1 className="text-xs font-semibold text-slate-200">Settings</h1>
      </header>
      {!settings ? (
        <p className="py-4 text-center text-[11px] text-slate-500">Loading…</p>
      ) : (
        <div className="divide-y divide-slate-700/50 rounded-xl border border-slate-700/40 bg-slate-800/40 px-3">
          <Row label="Warning above">
            <ThresholdInput
              key={`warn-${settings.warn_threshold}`}
              label="Warning threshold"
              value={settings.warn_threshold}
              onCommit={(v) => set({ warn_threshold: v })}
            />
          </Row>
          <Row label="Critical above">
            <ThresholdInput
              key={`crit-${settings.crit_threshold}`}
              label="Critical threshold"
              value={settings.crit_threshold}
              onCommit={(v) => set({ crit_threshold: v })}
            />
          </Row>
          <Row label="Update every">
            <select
              aria-label="Update interval"
              value={settings.poll_interval_ms}
              onChange={(e) => set({ poll_interval_ms: Number(e.target.value) })}
              className="rounded border border-slate-600 bg-slate-900 px-1 py-0.5 text-slate-100"
            >
              {INTERVALS.map((ms) => (
                <option key={ms} value={ms}>
                  {ms / 1000} s
                </option>
              ))}
            </select>
          </Row>
          <Row label="Show/hide hotkey">
            <HotkeyRecorder value={settings.hotkey} onChange={(hotkey) => set({ hotkey })} />
          </Row>
          <Row label="Launch at startup">
            <Toggle
              label="Launch at startup"
              checked={settings.launch_at_startup}
              onChange={(v) => set({ launch_at_startup: v })}
            />
          </Row>
          <Row label="Auto-check for updates">
            <Toggle
              label="Auto-check for updates"
              checked={settings.auto_check_updates}
              onChange={(v) => set({ auto_check_updates: v })}
            />
          </Row>
          <Row label="Hide over fullscreen apps">
            <Toggle
              label="Hide over fullscreen apps"
              checked={settings.hide_over_fullscreen}
              onChange={(v) => set({ hide_over_fullscreen: v })}
            />
          </Row>
          <Row label="Position">
            <button
              onClick={() => void invoke('reset_position').catch(() => {})}
              className="rounded border border-slate-600 px-2 py-0.5 text-[10px] text-slate-200 hover:border-slate-400"
            >
              Reset to top center
            </button>
          </Row>
        </div>
      )}
      {error && <p className="pt-2 text-[11px] text-red-300">{error}</p>}
    </div>
  );
}
