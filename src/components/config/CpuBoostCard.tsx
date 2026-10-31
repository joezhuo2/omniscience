import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

export type CpuBoostStatus = {
  current: number | null;
  label: string | null;
  plan: string | null;
  mismatch: boolean;
  supported: boolean;
  error: string | null;
};

const modes = [
  { value: 0, label: 'Off', description: 'Disable processor turbo boost' },
  { value: 1, label: 'On', description: 'Allow processor turbo boost' },
  { value: 2, label: 'Aggressive', description: 'Boost aggressively for maximum performance' },
];

export function CpuBoostCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<CpuBoostStatus>('get_cpu_boost', active);
  const custom = status?.current != null && status.current > 2;
  return (
    <Card section="cpu-boost" title="CPU boost">
      <div className="grid grid-cols-3 gap-1" role="group" aria-label="CPU boost">
        {modes.map(({ value, label, description }) => (
          <button
            type="button"
            key={value}
            className="config-button"
            title={description}
            aria-pressed={status?.current === value}
            disabled={!status?.supported || busy}
            onClick={() => void apply('set_cpu_boost', { mode: value })}
          >
            {label}
          </button>
        ))}
      </div>
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying CPU boost…'
          : !status
            ? 'Reading CPU boost…'
            : status.supported
              ? `${custom ? `${status.label}. ` : ''}${status.plan ? `Power plan: ${status.plan}. ` : ''}Applies to plugged in and battery.`
              : 'CPU boost control requires Windows.'}
      </p>
      {status?.mismatch && (
        <p className="mt-2 text-xs text-amber-400">
          Plugged in and battery use different boost settings in this power plan. Showing the
          plugged-in value; selecting a mode sets both.
        </p>
      )}
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
