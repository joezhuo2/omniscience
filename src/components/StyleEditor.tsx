import { ColorPicker } from '@crystalflux/color-picker';
import { GlassSelect } from '@crystalflux/select-glass';
import { SmoothRange } from '@crystalflux/smooth-range';
import { ArrowLeft } from 'lucide-react';
import { useState } from 'react';
import { Toggle } from './settings/SettingsView';
import { isEmbedded } from '../lib/ipc';
import {
  ACCENT_KEY,
  COLOR_TOKENS,
  THEME_PRESETS,
  FONT_PRESETS,
  FONT_VAR,
  SIZE_VAR,
  applyAppearance,
  defaultFor,
  fontLabel,
  fontValue,
  loadAppearance,
  presetLabel,
  saveAppearance,
  syncAccent,
  withPreset,
  toHex,
  type Overrides,
} from '../lib/appearance';

export function StyleEditor({ onBack }: { onBack: () => void }) {
  const [overrides, setOverrides] = useState<Overrides>(loadAppearance);

  const commit = (next: Overrides) => {
    setOverrides(next);
    saveAppearance(next);
    applyAppearance(next);
    void syncAccent();
  };

  const set = (name: string, value: string | null) => {
    const next = { ...overrides };
    if (value) next[name] = value;
    else delete next[name];
    commit(next);
  };

  const current = presetLabel(overrides);
  const size = parseInt(overrides[SIZE_VAR] ?? '100', 10);

  return (
    <div className="space-y-2 text-[11px]">
      <header className="flex h-9 items-center gap-2">
        <button
          aria-label="Back"
          onClick={onBack}
          className="rounded-md p-1 text-island-muted hover:bg-island-track/50 hover:text-island-text"
        >
          <ArrowLeft size={14} aria-hidden />
        </button>
        <h1 className="text-xs font-semibold text-island-text">Appearance</h1>
      </header>
      <section className="rounded-xl border border-island-faint bg-island-hover p-3">
        <div className="mb-3 flex items-center justify-between gap-3 text-island-muted">
          Theme
          <GlassSelect
            aria-label="Theme"
            options={[...THEME_PRESETS.map((p) => p.label), ...(current ? [] : ['Custom'])]}
            value={current ?? 'Custom'}
            onChange={(label) => {
              const found = THEME_PRESETS.find((p) => p.label === label);
              if (found) commit(withPreset(overrides, found.overrides));
            }}
          />
        </div>
        {!isEmbedded() && (
          <div className="mb-3 flex items-center justify-between gap-3 text-island-muted">
            Follow Windows accent color
            <Toggle
              label="Follow Windows accent color"
              checked={!!overrides[ACCENT_KEY]}
              onChange={(on) => set(ACCENT_KEY, on ? '1' : null)}
            />
          </div>
        )}
        <div className="grid grid-cols-2 gap-x-3 gap-y-2">
          {COLOR_TOKENS.map(({ name, label }) => (
            <ColorPicker
              key={name}
              label={label}
              value={toHex(overrides[name] ?? defaultFor(name))}
              onChange={(hex) => set(name, hex)}
            />
          ))}
          <div className="font-field flex flex-col gap-1 text-island-muted">
            Font
            <GlassSelect
              aria-label="Font"
              className="font-select"
              options={FONT_PRESETS.map((p) => p.label)}
              value={fontLabel(overrides[FONT_VAR])}
              onChange={(label) => set(FONT_VAR, fontValue(label))}
            />
          </div>
          <label className="flex flex-col gap-1 text-island-muted">
            Size {size}%
            <SmoothRange
              min={90}
              max={120}
              step={5}
              value={size}
              onValue={(v) => set(SIZE_VAR, v === 100 ? null : `${v}%`)}
            />
          </label>
        </div>
        <button
          className="mt-3 rounded-md bg-island-track px-2 py-1 text-island-text hover:bg-island-track/70"
          onClick={() => commit({})}
        >
          Reset
        </button>
      </section>
    </div>
  );
}
