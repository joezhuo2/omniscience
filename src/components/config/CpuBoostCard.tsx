import { Card } from '../dashboard/Card';
import { SourceTabs } from './SourceTabs';
import { useSourceTab } from './useSourceTab';
import { useNativeControl } from './useNativeControl';

export type CpuBoostStatus = {
  ac: boolean | null;
  plugged: number | null;
  plugged_label: string | null;
  battery: number | null;
  battery_label: string | null;
  plan: string | null;
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
  const [ac, setAc] = useSourceTab(status?.ac);
  const current = ac ? status?.plugged : status?.battery;
  const currentLabel = ac ? status?.plugged_label : status?.battery_label;
  const custom = current != null && current > 2;
  return (
    <Card
      section="cpu-boost"
      title="CPU boost"
      aside={
        <SourceTabs label="CPU boost profile" ac={ac} activeAc={status?.ac} onChange={setAc} />
      }
    >
      <div className="grid grid-cols-3 gap-1" role="group" aria-label="CPU boost">
        {modes.map(({ value, label, description }) => (
          <button
            type="button"
            key={value}
            className="config-button"
            title={description}
            aria-pressed={current === value}
            disabled={!status?.supported || busy}
            onClick={() => void apply('set_cpu_boost', { ac, mode: value })}
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
              ? `${custom ? `${currentLabel}. ` : ''}${status.plan ? `Power plan: ${status.plan}. ` : ''}`
              : 'CPU boost control requires Windows.'}
      </p>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
