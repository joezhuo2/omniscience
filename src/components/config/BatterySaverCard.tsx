import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

export type BatterySaverStatus = {
  on: boolean | null;
  threshold: number | null;
  supported: boolean;
  error: string | null;
};

const thresholds = [0, 100, 90, 80, 70, 60, 50, 40, 30, 20];

export function BatterySaverCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<BatterySaverStatus>(
    'get_battery_saver',
    active,
  );
  const disabled = !status?.supported || busy;
  const threshold = status?.threshold;
  const options =
    threshold == null || thresholds.includes(threshold) ? thresholds : [...thresholds, threshold];
  return (
    <Card
      section="battery-saver"
      title="Battery saver"
      aside={
        status?.on != null && (
          <span className="text-[10px] font-medium uppercase text-island-muted">
            {status.on ? 'On' : 'Off'}
          </span>
        )
      }
    >
      <div className="flex items-center gap-2">
        <label className="flex flex-1 items-center gap-2 text-[11px] text-island-muted">
          Turn on at
          <select
            className="config-input flex-1"
            value={threshold ?? ''}
            disabled={disabled || threshold == null}
            onChange={(e) =>
              void apply(
                'set_battery_saver_threshold',
                { threshold: Number(e.target.value) },
                threshold != null
                  ? {
                      label: 'Battery saver threshold',
                      command: 'set_battery_saver_threshold',
                      args: { threshold },
                    }
                  : undefined,
              )
            }
          >
            {threshold == null && <option value="" />}
            {options.map((value) => (
              <option key={value} value={value}>
                {value ? `${value}%` : 'Off'}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          className="config-button"
          disabled={disabled || status?.on == null}
          onClick={() =>
            void apply(
              'set_battery_saver',
              { on: !status?.on },
              status?.on != null
                ? { label: 'Battery saver', command: 'set_battery_saver', args: { on: status.on } }
                : undefined,
            )
          }
        >
          {status?.on ? 'Turn off' : 'Turn on'}
        </button>
      </div>
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying battery saver…'
          : !status
            ? 'Reading battery saver…'
            : status.supported
              ? `Battery saver is ${status.on ? 'on' : 'off'}.`
              : 'Battery saver control requires Windows.'}
      </p>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
