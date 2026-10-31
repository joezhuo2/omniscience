import { useState } from 'react';
import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

export type PowerLimitsStatus = {
  spl: number | null;
  sppt: number | null;
  supported: boolean;
  error: string | null;
};

type Limits = { spl: number; sppt: number };

export function PowerLimitsCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<PowerLimitsStatus>(
    'get_power_limits',
    active,
  );
  const [draft, setDraft] = useState<Limits | null>(null);
  const value: Limits = draft ?? { spl: status?.spl ?? 45, sppt: status?.sppt ?? 65 };
  const disabled = !status?.supported || busy;
  const commit = async () => {
    if (!draft) return;
    await apply('set_power_limits', draft);
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
    <Card section="power-limits" title="Power limits">
      <div className="flex flex-col gap-2">
        {slider('Sustained (SPL)', 'spl')}
        {slider('Boost (SPPT)', 'sppt')}
      </div>
      <div className="mt-2 flex items-center gap-2">
        <button
          type="button"
          className="config-button"
          disabled={disabled || (status?.spl == null && status?.sppt == null)}
          onClick={() => {
            setDraft(null);
            void apply('reset_power_limits', {});
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
                : status.spl == null
                  ? 'Using firmware defaults.'
                  : 'Reset clears the saved limits; firmware defaults apply after a restart or power-mode change.'}
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
