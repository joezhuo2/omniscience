import { useState } from 'react';
import { Card } from '../dashboard/Card';
import { SourceTabs } from './SourceTabs';
import { useSourceTab } from './useSourceTab';
import { useNativeControl } from './useNativeControl';

type Limits = { spl: number; sppt: number };

export type PowerLimitsStatus = {
  ac: boolean | null;
  ac_limits: Limits | null;
  battery_limits: Limits | null;
  supported: boolean;
  error: string | null;
};

export function PowerLimitsCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<PowerLimitsStatus>(
    'get_power_limits',
    active,
  );
  const [draft, setDraft] = useState<Limits | null>(null);
  const [ac, setAc] = useSourceTab(status?.ac);
  const saved = ac ? status?.ac_limits : status?.battery_limits;
  const value: Limits = draft ?? saved ?? { spl: 45, sppt: 65 };
  const disabled = !status?.supported || busy;
  const commit = async () => {
    if (!draft) return;
    await apply('set_power_limits', { ac, ...draft });
    setDraft(null);
  };
  const slider = (label: string, key: keyof Limits) => (
    <label className="block text-xs">
      <span className="flex justify-between">
        <span>{label}</span>
        <span>{value[key]} W</span>
      </span>
      <input
        type="range"
        className="w-full"
        min={5}
        max={80}
        step={1}
        value={value[key]}
        disabled={disabled}
        aria-label={label}
        onChange={(e) => {
          const watts = Number(e.target.value);
          setDraft(
            key === 'spl'
              ? { spl: watts, sppt: Math.max(watts, value.sppt) }
              : { spl: Math.min(watts, value.spl), sppt: watts },
          );
        }}
        onPointerUp={() => void commit()}
        onKeyUp={() => void commit()}
      />
    </label>
  );
  return (
    <Card
      section="power-limits"
      title="Power limits"
      aside={
        <SourceTabs
          label="Power limits profile"
          ac={ac}
          activeAc={status?.ac}
          onChange={(next) => {
            setDraft(null);
            setAc(next);
          }}
        />
      }
    >
      <div className="flex flex-col gap-2">
        {slider('Sustained (SPL)', 'spl')}
        {slider('Boost (SPPT)', 'sppt')}
      </div>
      <div className="mt-2 flex items-center gap-2">
        <button
          type="button"
          className="config-button"
          disabled={disabled || !saved}
          onClick={() => {
            setDraft(null);
            void apply('reset_power_limits', { ac });
          }}
        >
          Reset
        </button>
        <p className="text-xs" role="status" aria-live="polite">
          {busy
            ? 'Applying power limits…'
            : !status
              ? 'Reading power limit support…'
              : !status.supported
                ? 'Requires a supported ASUS ROG laptop and ASUS System Control Interface.'
                : !saved
                  ? 'No saved profile; firmware defaults apply.'
                  : status.ac === ac
                    ? 'Reset clears this profile; firmware defaults apply after a restart or power-mode change.'
                    : 'Saved now and applied when the power source changes.'}
        </p>
      </div>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
