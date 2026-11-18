import { GlassCheckbox } from '@crystalflux/checkbox-glass';
import { FeedbackButton } from '@crystalflux/feedback-button';
import { GlassSelect } from '@crystalflux/select-glass';
import { useRef, useState } from 'react';
import { applyAppearance, loadAppearance, saveAppearance, syncAccent } from '../../lib/appearance';
import { useGlass } from '../../lib/glass';
import { invoke } from '../../lib/ipc';
import { RESET_GROUPS, resetPatch } from '../../lib/settingsReset';
import {
  buildSettingsDocument,
  importedSettings,
  parseSettingsDocument,
} from '../../lib/settingsFile';
import { useSettingsStore, type Settings } from '../../settings';
import { useToastStore } from '../../toasts';
import type { ThresholdPair } from '../../types/generated/ThresholdPair';
import { Row, ThresholdInput } from './SettingsView';

const button =
  'rounded border border-island-faint px-2 py-0.5 text-[10px] text-island-text hover:border-island-muted';

const METRICS: { key: keyof Settings['metric_thresholds']; label: string }[] = [
  { key: 'cpu', label: 'CPU' },
  { key: 'ram', label: 'Memory' },
  { key: 'igpu', label: 'Integrated GPU' },
  { key: 'dgpu', label: 'Discrete GPU' },
  { key: 'disk', label: 'Disk' },
  { key: 'net', label: 'Network' },
];

export function MetricThresholdsSettings({
  settings,
  set,
}: {
  settings: Settings;
  set: (patch: Partial<Settings>) => void;
}) {
  const all = settings.metric_thresholds;
  const setPair = (key: keyof typeof all, pair: ThresholdPair | null) =>
    set({ metric_thresholds: { ...all, [key]: pair } });
  return (
    <>
      <Row label="Per-metric thresholds" hint="Overrides the warning and critical limits above.">
        <span />
      </Row>
      {METRICS.map(({ key, label }) => {
        const pair = all[key];
        return (
          <div key={key} className="py-1.5 text-[11px] text-island-text">
            <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
              <GlassCheckbox
                className="text-[11px] text-island-text"
                checked={pair !== null}
                onChange={(on) =>
                  setPair(
                    key,
                    on ? { warn: settings.warn_threshold, crit: settings.crit_threshold } : null,
                  )
                }
                label={
                  <>
                    <span className="sr-only">{label} </span>Custom
                  </>
                }
              />
              {pair && (
                <span className="flex flex-wrap items-center gap-1">
                  <span className="text-island-muted">{label} warning</span>
                  <ThresholdInput
                    key={`${key}-w-${pair.warn}`}
                    label={`${label} warning threshold`}
                    value={pair.warn}
                    max={98}
                    onCommit={(warn) => setPair(key, { ...pair, warn })}
                  />
                  <span className="text-island-muted">critical</span>
                  <ThresholdInput
                    key={`${key}-c-${pair.crit}`}
                    label={`${label} critical threshold`}
                    value={pair.crit}
                    min={2}
                    onCommit={(crit) => setPair(key, { ...pair, crit })}
                  />
                </span>
              )}
            </div>
          </div>
        );
      })}
    </>
  );
}

export function ResetSettings() {
  const [group, setGroup] = useState(RESET_GROUPS[0]!.id);
  const [armed, setArmed] = useState(false);
  const push = useToastStore((s) => s.push);
  const label = RESET_GROUPS.find((g) => g.id === group)?.label ?? '';
  const reset = async () => {
    const { settings, update } = useSettingsStore.getState();
    if (!settings) return;
    try {
      const defaults = await invoke<Settings>('get_default_settings');
      await update(resetPatch(group, defaults, settings));
      const error = useSettingsStore.getState().error;
      push(error ?? `${label} reset to defaults.`);
    } catch (e) {
      push(`Could not reset: ${String(e)}`);
    }
  };
  return (
    <Row label="Reset settings" hint="Puts one group back to its defaults.">
      <span className="flex shrink-0 items-center gap-1.5">
        <GlassSelect
          aria-label="Settings group"
          options={RESET_GROUPS.map((g) => g.label)}
          value={label}
          onChange={(l) => {
            setGroup(RESET_GROUPS.find((g) => g.label === l)?.id ?? group);
            setArmed(false);
          }}
        />
        <button
          className={button}
          onBlur={() => setArmed(false)}
          onClick={() => {
            if (!armed) return setArmed(true);
            setArmed(false);
            void reset();
          }}
        >
          {armed ? 'Confirm reset' : 'Reset'}
        </button>
      </span>
    </Row>
  );
}

export function BackupSettings() {
  const push = useToastStore((s) => s.push);
  const glass = useGlass();
  const file = useRef<HTMLInputElement>(null);

  const exportSettings = async () => {
    const { settings } = useSettingsStore.getState();
    if (!settings) return;
    const doc = buildSettingsDocument(settings, loadAppearance(), {
      on: glass.on,
      opacity: glass.opacity,
    });
    try {
      const path = await invoke<string>('export_settings_file', {
        contents: JSON.stringify(doc, null, 2),
      });
      push(`Saved to ${path}`);
    } catch (e) {
      push(`Could not export settings: ${String(e)}`);
      throw e;
    }
  };

  const importSettings = async (chosen: File) => {
    const { settings, update } = useSettingsStore.getState();
    if (!settings) return;
    let doc;
    try {
      doc = parseSettingsDocument(await chosen.text(), settings);
    } catch (e) {
      push(`Could not import settings: ${e instanceof Error ? e.message : String(e)}`);
      return;
    }
    await update(importedSettings(doc, settings));
    saveAppearance(doc.appearance);
    applyAppearance(doc.appearance);
    void syncAccent();
    glass.setOn(doc.glass.on);
    glass.setOpacity(doc.glass.opacity);
    const error = useSettingsStore.getState().error;
    push(error ? `Settings were not applied: ${error}` : 'Settings imported.');
  };

  return (
    <Row label="Settings file" hint="Export or import settings, appearance and glass options.">
      <div className="flex shrink-0 gap-1.5">
        <FeedbackButton action={exportSettings} done="Saved" className={button}>
          Export settings
        </FeedbackButton>
        <button className={button} onClick={() => file.current?.click()}>
          Import settings
        </button>
        <input
          ref={file}
          type="file"
          accept=".json"
          aria-label="Settings file to import"
          className="hidden"
          onChange={(e) => {
            const chosen = e.target.files?.[0];
            e.target.value = '';
            if (chosen) void importSettings(chosen);
          }}
        />
      </div>
    </Row>
  );
}
