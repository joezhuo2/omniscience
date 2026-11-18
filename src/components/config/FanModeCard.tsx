import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

type Mode = 'silent' | 'performance' | 'turbo';
export type FanModeStatus = {
  mode: Mode | null;
  supported: boolean;
  error: string | null;
};

const modes: { value: Mode; label: string }[] = [
  { value: 'silent', label: 'Silent' },
  { value: 'performance', label: 'Performance' },
  { value: 'turbo', label: 'Turbo' },
];

export function FanModeCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<FanModeStatus>('get_fan_mode', active);
  return (
    <Card section="fan-mode" title="Fan mode">
      <div className="grid grid-cols-3 gap-1" role="group" aria-label="Fan mode">
        {modes.map(({ value, label }) => (
          <button
            key={value}
            type="button"
            className="config-button"
            aria-pressed={status?.mode === value}
            disabled={!status?.supported || busy}
            onClick={() =>
              void apply(
                'set_fan_mode',
                { mode: value },
                status?.mode
                  ? { label: 'Fan mode', command: 'set_fan_mode', args: { mode: status.mode } }
                  : undefined,
              )
            }
          >
            {label}
          </button>
        ))}
      </div>
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying fan mode…'
          : !status
            ? 'Reading fan mode…'
            : status.supported
              ? `Fan mode: ${status.mode ?? 'unknown'}.`
              : 'Requires a supported ASUS ROG laptop and ASUS System Control Interface.'}
      </p>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
