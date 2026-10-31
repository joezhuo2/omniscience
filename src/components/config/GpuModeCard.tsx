import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

type Mode = 'auto' | 'med' | 'min' | 'max';
export type GpuStatus = {
  selected: Mode | null;
  current: Mode | null;
  supported: boolean;
  mux_supported: boolean;
  restart_required: boolean;
  error: string | null;
};

const modes: { value: Mode; label: string; description: string }[] = [
  { value: 'auto', label: 'Auto', description: 'iGPU on battery, hybrid graphics when plugged in' },
  { value: 'med', label: 'Med', description: 'Hybrid graphics: both GPUs available' },
  { value: 'min', label: 'Min', description: 'iGPU only: dGPU disabled' },
  {
    value: 'max',
    label: 'Max',
    description: 'dGPU display mode: requires a supported MUX and Windows restart',
  },
];

export function GpuModeCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<GpuStatus>('get_gpu_mode', active);
  return (
    <Card section="gpu-mode" title="GPU mode">
      <div className="grid grid-cols-4 gap-1" role="group" aria-label="GPU mode">
        {modes.map(({ value, label, description }) => (
          <button
            type="button"
            key={value}
            className="config-button"
            title={description}
            aria-pressed={(status?.selected ?? status?.current) === value}
            disabled={
              !status?.supported ||
              busy ||
              status.restart_required ||
              (value === 'max' && !status.mux_supported)
            }
            onClick={() => void apply('set_gpu_mode', { mode: value })}
          >
            {label}
          </button>
        ))}
      </div>
      <p className="mt-2 text-[11px] text-island-muted">
        Auto uses the iGPU on battery and enables the dGPU when plugged in. Med keeps both GPUs
        available; Min disables the dGPU; Max routes the display through the dGPU.
      </p>
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying GPU mode…'
          : !status
            ? 'Reading GPU support…'
            : status.restart_required
              ? 'Restart Windows to finish the GPU mode change. Auto switching is paused until restart.'
              : status.supported
                ? `Firmware mode: ${status.current}. Close dGPU apps before selecting Min.`
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
