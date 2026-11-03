import { Card } from '../dashboard/Card';
import { SourceTabs } from './SourceTabs';
import { useSourceTab } from './useSourceTab';
import { useNativeControl } from './useNativeControl';

type Mode = 'efficiency' | 'balanced' | 'performance';
export type PowerStatus = {
  ac: boolean | null;
  ac_mode: Mode | null;
  battery_mode: Mode | null;
  current: Mode | null;
  supported: boolean;
  error: string | null;
};

const modes: { value: Mode; label: string; description: string }[] = [
  { value: 'efficiency', label: 'Efficiency', description: 'Best power efficiency' },
  { value: 'balanced', label: 'Balanced', description: 'Balanced' },
  { value: 'performance', label: 'Performance', description: 'Best performance' },
];

export function PowerModeCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<PowerStatus>('get_power_mode', active);
  const [ac, setAc] = useSourceTab(status?.ac);
  const saved = ac ? status?.ac_mode : status?.battery_mode;
  const selected = saved ?? (status?.ac === ac ? status.current : null);
  return (
    <Card
      section="power-mode"
      title="Power mode"
      aside={
        <SourceTabs label="Power mode profile" ac={ac} activeAc={status?.ac} onChange={setAc} />
      }
    >
      <div className="grid grid-cols-3 gap-1" role="group" aria-label="Power mode">
        {modes.map(({ value, label, description }) => (
          <button
            type="button"
            key={value}
            className="config-button"
            title={description}
            aria-pressed={selected === value}
            disabled={!status?.supported || busy}
            onClick={() =>
              void apply(
                'set_power_mode',
                { ac, mode: value },
                selected
                  ? { label: 'Power mode', command: 'set_power_mode', args: { ac, mode: selected } }
                  : undefined,
              )
            }
          >
            {label}
          </button>
        ))}
      </div>
      <p className="mt-2 text-[11px] text-island-muted">
        Each tab sets the mode used on that power source. The current source applies immediately;
        the other is applied when the power source changes.
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
