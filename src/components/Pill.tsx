import { RING_STROKE, ringSpecs } from '../lib/metrics';
import { useTelemetryStore } from '../store';
import { Ring } from './Ring';

/** The pill row: six small rings, no text. Stays at the top of the island when it expands. */
export function Pill() {
  const core = useTelemetryStore((s) => s.snapshot?.core);
  return (
    <div
      data-testid="pill"
      className="mx-auto flex items-center justify-center gap-2"
      style={{ width: 'var(--pill-w)', height: 'var(--pill-h)' }}
    >
      {ringSpecs(core).map((r) => (
        <Ring
          key={r.key}
          outer={r.outer}
          inner={r.inner}
          size={22}
          stroke={r.inner ? RING_STROKE.pillPair : RING_STROKE.pill}
          label={r.label}
        />
      ))}
    </div>
  );
}
