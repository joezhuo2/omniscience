import { invoke } from '@tauri-apps/api/core';
import { ArrowLeft, ChevronDown, ChevronUp } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import { acceleratorFromEvent, displayAccelerator } from '../../lib/hotkey';
import { useSettingsStore, type Settings } from '../../settings';

const IS_MAC = typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform);
const INTERVALS = [500, 1000, 2000, 5000];

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex min-h-9 items-center justify-between gap-3 py-1.5 text-[11px] text-island-text">
      <span>{label}</span>
      {children}
    </div>
  );
}

/** iOS-style switch. Sizes are explicit so the knob always stays inside the track. */
const SWITCH = { w: 34, h: 20, knob: 16, pad: 2 };

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
      className={`relative shrink-0 overflow-hidden rounded-full p-0 transition-colors duration-200 ${
        checked ? 'bg-island-ok' : 'bg-island-track'
      }`}
      style={{ width: SWITCH.w, height: SWITCH.h }}
    >
      <span
        aria-hidden
        className="absolute block rounded-full bg-white shadow transition-[left] duration-200 ease-out"
        style={{
          top: SWITCH.pad,
          width: SWITCH.knob,
          height: SWITCH.knob,
          left: checked ? SWITCH.w - SWITCH.knob - SWITCH.pad : SWITCH.pad,
        }}
      />
    </button>
  );
}

/** Number input that commits on blur or Enter, so typing "85" does not apply "8" first. */
/** Free-text field committed on blur or Enter. Remount (via `key`) to reset the draft. */
function TextInput({
  value,
  label,
  placeholder,
  onCommit,
}: {
  value: string;
  label: string;
  placeholder?: string;
  onCommit: (v: string) => void;
}) {
  const [draft, setDraft] = useState(value);
  const commit = () => {
    if (draft.trim() !== value) onCommit(draft.trim());
  };
  return (
    <input
      aria-label={label}
      value={draft}
      placeholder={placeholder}
      spellCheck={false}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => e.key === 'Enter' && commit()}
      className="w-32 rounded border border-island-faint bg-island-hover px-1.5 py-0.5 text-right text-island-text"
    />
  );
}

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
  const step = (delta: number) => {
    const n = Math.min(99, Math.max(1, value + delta));
    if (n !== value) onCommit(n);
  };
  const stepButton =
    'flex h-2.5 w-4 items-center justify-center text-island-muted hover:text-island-text';
  return (
    <span className="flex items-center gap-1">
      <span className="flex flex-col">
        <button aria-label={`Increase ${label}`} className={stepButton} onClick={() => step(1)}>
          <ChevronUp size={10} aria-hidden />
        </button>
        <button aria-label={`Decrease ${label}`} className={stepButton} onClick={() => step(-1)}>
          <ChevronDown size={10} aria-hidden />
        </button>
      </span>
      <input
        aria-label={label}
        type="number"
        min={1}
        max={99}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => e.key === 'Enter' && commit()}
        className="w-12 rounded border border-island-faint bg-island-hover px-1.5 py-0.5 text-right text-island-text tabular-nums"
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
          ? 'border-island-text text-island-text'
          : 'border-island-faint text-island-text hover:border-island-muted'
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
          className="rounded-md p-1 text-island-muted hover:bg-island-track/50 hover:text-island-text"
        >
          <ArrowLeft size={14} aria-hidden />
        </button>
        <h1 className="text-xs font-semibold text-island-text">Settings</h1>
      </header>
      {!settings ? (
        <p className="py-4 text-center text-[11px] text-island-muted">Loading…</p>
      ) : (
        <div className="divide-y divide-island-faint rounded-xl border border-island-faint bg-island-hover px-3">
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
            <div
              role="radiogroup"
              aria-label="Update interval"
              className="flex rounded-md bg-island-track p-0.5 text-[10px]"
            >
              {INTERVALS.map((ms) => {
                const active = settings.poll_interval_ms === ms;
                return (
                  <button
                    key={ms}
                    role="radio"
                    aria-checked={active}
                    onClick={() => set({ poll_interval_ms: ms })}
                    className={`rounded px-2 py-0.5 tabular-nums transition-colors ${
                      active
                        ? 'bg-island-faint text-island-text'
                        : 'text-island-muted hover:text-island-text'
                    }`}
                  >
                    {ms / 1000} s
                  </button>
                );
              })}
            </div>
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
          <Row label="Ping target">
            <TextInput
              key={`ping-${settings.ping_target}`}
              label="Ping target"
              placeholder="Off"
              value={settings.ping_target}
              onCommit={(v) => set({ ping_target: v })}
            />
          </Row>
          <Row label="Show public IP (asks OpenDNS)">
            <Toggle
              label="Show public IP"
              checked={settings.show_public_ip}
              onChange={(v) => set({ show_public_ip: v })}
            />
          </Row>
          <Row label="Position">
            <button
              onClick={() => void invoke('reset_position').catch(() => {})}
              className="rounded border border-island-faint px-2 py-0.5 text-[10px] text-island-text hover:border-island-muted"
            >
              Reset to top center
            </button>
          </Row>
        </div>
      )}
      {error && <p className="pt-2 text-[11px] text-island-crit">{error}</p>}
    </div>
  );
}
