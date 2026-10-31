import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

type Mode = 'auto' | 'efficiency' | 'balanced' | 'performance';
export type PowerStatus = {
  selected: Mode | null;
  current: Exclude<Mode, 'auto'> | null;
  supported: boolean;
  error: string | null;
};

const modes: { value: Mode; label: string; description: string }[] = [
  {
    value: 'auto',
    label: 'Auto',
    description: 'Best power efficiency on battery, best performance when plugged in',
  },
  { value: 'efficiency', label: 'Efficiency', description: 'Best power efficiency' },
  { value: 'balanced', label: 'Balanced', description: 'Balanced' },
  { value: 'performance', label: 'Performance', description: 'Best performance' },
];

export function PowerModeCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<PowerStatus>('get_power_mode', active);
  const selected = status?.selected === 'auto' ? 'auto' : status?.current;
  return (
    <Card section="power-mode" title="Power mode">
      <div className="grid grid-cols-4 gap-1" role="group" aria-label="Power mode">
        {modes.map(({ value, label, description }) => (
          <button
            type="button"
            key={value}
            className="config-button"
            title={description}
            aria-pressed={selected === value}
            disabled={!status?.supported || busy}
            onClick={() => void apply('set_power_mode', { mode: value })}
          >
            {label}
          </button>
        ))}
      </div>
      <p className="mt-2 text-[11px] text-island-muted">
        Auto uses best power efficiency on battery and best performance when plugged in. Windows
        keeps separate modes for battery and plugged in; a choice applies to the current power
        source.
      </p>
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying power mode…'
          : !status
            ? 'Reading power mode…'
            : status.supported
              ? `Windows power mode: ${status.current ?? 'unknown'}.`
              : 'Requires Windows 11 power modes.'}
      </p>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
