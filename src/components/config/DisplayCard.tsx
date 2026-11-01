import { Card } from '../dashboard/Card';
import { useNativeControl } from './useNativeControl';

export type DisplayStatus = {
  device: string | null;
  current: { width: number; height: number; hz: number } | null;
  resolutions: [number, number][];
  rates: number[];
  auto: boolean;
  supported: boolean;
  error: string | null;
};

export function DisplayCard({ active = true }: { active?: boolean }) {
  const { status, busy, error, apply } = useNativeControl<DisplayStatus>(
    'get_display_mode',
    active,
  );
  const disabled = !status?.supported || busy;
  const current = status?.current;
  return (
    <Card section="display" title="Display">
      {status?.device && <p className="mb-2 text-xs text-island-muted">{status.device}</p>}
      <p className="mb-1 text-[11px] text-island-muted">Refresh rate</p>
      <div className="flex flex-wrap gap-1" role="group" aria-label="Refresh rate">
        <button
          type="button"
          className="config-button"
          title="Lowest rate of at least 60 Hz on battery, highest when plugged in"
          aria-pressed={!!status?.auto}
          disabled={disabled}
          onClick={() => void apply('set_display_refresh', { hz: null })}
        >
          Auto
        </button>
        {status?.rates.map((hz) => (
          <button
            type="button"
            key={hz}
            className="config-button"
            aria-pressed={!status.auto && current?.hz === hz}
            disabled={disabled}
            onClick={() => void apply('set_display_refresh', { hz })}
          >
            {hz} Hz
          </button>
        ))}
      </div>
      <p className="mb-1 mt-2 text-[11px] text-island-muted">Resolution</p>
      <div className="flex flex-wrap gap-1" role="group" aria-label="Resolution">
        {status?.resolutions.map(([width, height]) => (
          <button
            type="button"
            key={`${width}x${height}`}
            className="config-button"
            aria-pressed={current?.width === width && current.height === height}
            disabled={disabled}
            onClick={() => void apply('set_display_resolution', { width, height })}
          >
            {width}×{height}
          </button>
        ))}
      </div>
      <p className="mt-2 text-xs" role="status" aria-live="polite">
        {busy
          ? 'Applying display mode…'
          : !status
            ? 'Reading display modes…'
            : current
              ? `Current: ${current.width}×${current.height} at ${current.hz} Hz.`
              : 'Display modes are unavailable.'}
      </p>
      {(error || status?.error) && (
        <p role="alert" className="mt-2 text-xs text-amber-400 break-words">
          {error || status?.error}
        </p>
      )}
    </Card>
  );
}
