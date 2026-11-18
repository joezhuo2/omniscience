import { invoke, isEmbedded } from '../../lib/ipc';
import { GlassCheckbox } from '@crystalflux/checkbox-glass';
import { FeedbackButton } from '@crystalflux/feedback-button';
import { GlassSwitch } from '@crystalflux/switch-glass';
import { SmoothRange } from '@crystalflux/smooth-range';
import { ArrowLeft, ChevronDown, ChevronUp } from 'lucide-react';
import { useEffect, useState, type ReactNode } from 'react';
import { RANGES } from '../../lib/history';
import { acceleratorFromEvent, displayAccelerator } from '../../lib/hotkey';
import { useSettingsStore, type Settings } from '../../settings';
import { useToastStore } from '../../toasts';
import { Toasts } from '../dashboard/Toasts';
import { IslandImageField } from './IslandImageField';
import { Segmented } from '../Segmented';
import { useGlass } from '../../lib/glass';
import { ALL_RINGS } from '../../lib/pill';
import type { PillNumbers } from '../../types/generated/PillNumbers';

const IS_MAC = typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform);
const INTERVALS = [500, 1000, 2000, 5000];
const RING_LABELS: Record<string, string> = {
  cpu: 'CPU',
  ram: 'RAM',
  igpu: 'iGPU',
  dgpu: 'dGPU',
  disk: 'Disk',
  net: 'Network',
};
const PILL_NUMBERS: { value: PillNumbers; label: string }[] = [
  { value: 'off', label: 'Off' },
  { value: 'percent', label: '%' },
  { value: 'temp', label: '°C' },
];

function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex min-h-9 items-center justify-between gap-3 py-1.5 text-[11px] text-island-text">
      <span>
        {label}
        {hint && <span className="block text-[10px] text-island-muted">{hint}</span>}
      </span>
      {children}
    </div>
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
    <GlassSwitch
      className="shrink-0"
      checked={checked}
      onChange={onChange}
      label={<span className="sr-only">{label}</span>}
    />
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
  min = 1,
  max = 99,
  unit = '%',
  step: stepSize = 1,
}: {
  value: number;
  label: string;
  onCommit: (v: number) => void;
  min?: number;
  max?: number;
  unit?: string;
  step?: number;
}) {
  // Remounted (via `key`) when the applied value changes, which resets the draft.
  const [draft, setDraft] = useState(String(value));
  const commit = () => {
    const n = Number(draft);
    if (Number.isFinite(n) && n !== value) onCommit(n);
    else setDraft(String(value));
  };
  const step = (delta: number) => {
    const n = Math.min(max, Math.max(min, value + delta));
    if (n !== value) onCommit(n);
  };
  const stepButton =
    'flex h-2.5 w-4 items-center justify-center text-island-muted hover:text-island-text';
  return (
    <span className="flex items-center gap-1">
      <span className="flex flex-col">
        <button
          aria-label={`Increase ${label}`}
          className={stepButton}
          onClick={() => step(stepSize)}
        >
          <ChevronUp size={10} aria-hidden />
        </button>
        <button
          aria-label={`Decrease ${label}`}
          className={stepButton}
          onClick={() => step(-stepSize)}
        >
          <ChevronDown size={10} aria-hidden />
        </button>
      </span>
      <input
        aria-label={label}
        type="number"
        min={min}
        max={max}
        step={stepSize}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => e.key === 'Enter' && commit()}
        className="w-12 rounded border border-island-faint bg-island-hover px-1.5 py-0.5 text-right text-island-text tabular-nums"
      />
      {unit}
    </span>
  );
}

function PillRings({ value, onChange }: { value: string[]; onChange: (v: string[]) => void }) {
  const hidden = ALL_RINGS.filter((k) => !value.includes(k));
  const rows = [...value, ...hidden];
  const move = (i: number, delta: number) => {
    const next = [...value];
    next.splice(i + delta, 0, ...next.splice(i, 1));
    onChange(next);
  };
  const arrow =
    'rounded p-0.5 text-island-muted hover:text-island-text disabled:opacity-30 disabled:hover:text-island-muted';
  return (
    <ul className="flex flex-col gap-1">
      {rows.map((key, i) => {
        const shown = i < value.length;
        const name = RING_LABELS[key] ?? key;
        return (
          <li key={key} className="flex items-center justify-between gap-3">
            <GlassCheckbox
              className="text-[10px] text-island-text"
              checked={shown}
              disabled={shown && value.length === 1}
              onChange={(on) => onChange(on ? [...value, key] : value.filter((k) => k !== key))}
              label={name}
            />
            <span className="flex">
              <button
                aria-label={`Move ${name} up`}
                className={arrow}
                disabled={!shown || i === 0}
                onClick={() => move(i, -1)}
              >
                <ChevronUp size={12} aria-hidden />
              </button>
              <button
                aria-label={`Move ${name} down`}
                className={arrow}
                disabled={!shown || i === value.length - 1}
                onClick={() => move(i, 1)}
              >
                <ChevronDown size={12} aria-hidden />
              </button>
            </span>
          </li>
        );
      })}
    </ul>
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
  const {
    on: glass,
    opacity: glassOpacity,
    setOn: setGlass,
    setOpacity: setGlassOpacity,
  } = useGlass();
  useEffect(() => {
    if (!settings) void load();
  }, [settings, load]);

  const set = (patch: Partial<Settings>) => void update(patch);
  const standalone = !isEmbedded();

  const push = useToastStore((s) => s.push);
  const [includeTelemetry, setIncludeTelemetry] = useState(false);
  const copyReport = async () => {
    try {
      const report = await invoke<string>('get_diagnostic_report', { includeTelemetry });
      await navigator.clipboard.writeText(report);
      push('Diagnostic report copied to the clipboard.');
    } catch (e) {
      push(`Could not copy the diagnostic report: ${String(e)}`);
      throw e;
    }
  };

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
          <Row label="CPU temp warning above">
            <ThresholdInput
              key={`cpu-warn-${settings.cpu_temp_warn_c}`}
              label="CPU temperature warning"
              value={settings.cpu_temp_warn_c}
              min={30}
              max={109}
              unit="°C"
              onCommit={(v) => set({ cpu_temp_warn_c: v })}
            />
          </Row>
          <Row label="CPU temp critical above">
            <ThresholdInput
              key={`cpu-crit-${settings.cpu_temp_crit_c}`}
              label="CPU temperature critical"
              value={settings.cpu_temp_crit_c}
              min={31}
              max={110}
              unit="°C"
              onCommit={(v) => set({ cpu_temp_crit_c: v })}
            />
          </Row>
          <Row label="Open delay">
            <ThresholdInput
              key={`open-${settings.hover_open_delay_ms}`}
              label="Open delay"
              value={settings.hover_open_delay_ms}
              min={0}
              max={2000}
              step={50}
              unit="ms"
              onCommit={(v) => set({ hover_open_delay_ms: v })}
            />
          </Row>
          <Row label="Close delay">
            <ThresholdInput
              key={`close-${settings.hover_close_delay_ms}`}
              label="Close delay"
              value={settings.hover_close_delay_ms}
              min={0}
              max={2000}
              step={50}
              unit="ms"
              onCommit={(v) => set({ hover_close_delay_ms: v })}
            />
          </Row>
          {standalone && (
            <>
              <Row
                label="Dodge browser tabs"
                hint="Fades the pill and waits a second before opening while a browser's tabs are under the cursor."
              >
                <Toggle
                  label="Dodge browser tabs"
                  checked={settings.dodge_browser_tabs}
                  onChange={(v) => set({ dodge_browser_tabs: v })}
                />
              </Row>
              <Row label="Pill rings">
                <PillRings
                  value={settings.pill_rings}
                  onChange={(pill_rings) => set({ pill_rings })}
                />
              </Row>
              <Row label="Numbers on pill">
                <Segmented
                  label="Numbers on pill"
                  items={PILL_NUMBERS}
                  value={settings.pill_numbers}
                  onChange={(pill_numbers) => set({ pill_numbers })}
                />
              </Row>
            </>
          )}
          <Row label="Update every">
            <Segmented
              label="Update interval"
              items={INTERVALS.map((ms) => ({ value: ms, label: `${ms / 1000} s` }))}
              value={settings.poll_interval_ms}
              onChange={(ms) => set({ poll_interval_ms: ms })}
            />
          </Row>
          <Row label="Keep history for">
            <Segmented
              label="History retention"
              items={RANGES.map(({ label, hours }) => ({ value: hours, label }))}
              value={settings.history_hours}
              onChange={(hours) => set({ history_hours: hours })}
            />
          </Row>
          <Row label="Glass cards" hint="Translucent cards that show the island image through.">
            <Toggle label="Glass cards" checked={glass} onChange={setGlass} />
          </Row>
          {glass && (
            <Row label={`Glass opacity ${glassOpacity}%`}>
              <SmoothRange
                aria-label="Glass opacity"
                className="w-28"
                min={20}
                max={95}
                step={5}
                value={glassOpacity}
                onValue={setGlassOpacity}
              />
            </Row>
          )}
          {standalone && (
            <>
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
            </>
          )}
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
          <Row
            label="Sustained usage alerts"
            hint="Notifies when usage stays above 90% for 30 seconds."
          >
            <Toggle
              label="Sustained usage alerts"
              checked={settings.sustained_alerts}
              onChange={(v) => set({ sustained_alerts: v })}
            />
          </Row>
          <Row
            label="Sensor details"
            hint="Shows each reading's source, age and whether it is a proxy. Problems always show."
          >
            <Toggle
              label="Show sensor details"
              checked={settings.sensor_details}
              onChange={(v) => set({ sensor_details: v })}
            />
          </Row>
          {standalone && (
            <>
              <Row
                label="System config (GPU, power, display)"
                hint="Adds the Config tab, which changes hardware settings."
              >
                <Toggle
                  label="System config"
                  checked={settings.config_enabled}
                  onChange={(v) => set({ config_enabled: v })}
                />
              </Row>
              <Row label="Island image" hint="Background for the island. Blank when none is set.">
                <IslandImageField />
              </Row>
              <Row label="Position">
                <FeedbackButton
                  action={() => invoke('reset_position')}
                  done="Moved"
                  className="rounded border border-island-faint px-2 py-0.5 text-[10px] text-island-text hover:border-island-muted"
                >
                  Reset to top center
                </FeedbackButton>
              </Row>
            </>
          )}
          <Row
            label="Diagnostic report"
            hint="App version, sensor and control availability, recent errors. Telemetry leaves out process names and IP addresses."
          >
            <div className="flex shrink-0 flex-col items-end gap-1.5">
              <GlassCheckbox
                className="text-[10px] text-island-muted"
                checked={includeTelemetry}
                onChange={setIncludeTelemetry}
                label="Include telemetry"
              />
              <FeedbackButton
                action={copyReport}
                done="Copied"
                className="rounded border border-island-faint px-2 py-0.5 text-[10px] text-island-text hover:border-island-muted"
              >
                Copy diagnostic report
              </FeedbackButton>
            </div>
          </Row>
        </div>
      )}
      {error && <p className="pt-2 text-[11px] text-island-crit">{error}</p>}
      <Toasts />
    </div>
  );
}
