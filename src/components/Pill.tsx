import { useEffect } from 'react';
import { RING_STROKE, ringSpecs } from '../lib/metrics';
import { useTelemetryStore } from '../store';
import { useSettingsStore } from '../settings';
import { ALL_RINGS, pillNumber, pillWidth } from '../lib/pill';
import { Ring } from './Ring';

/**
 * The pill row: six small rings, no text. Stays at the top of the island when it expands, and is
 * the drag handle for moving the overlay ("deep" so the rings inside also start a drag).
 */
export function Pill() {
  const core = useTelemetryStore((s) => s.snapshot?.core);
  const settings = useSettingsStore((s) => s.settings);
  const order = settings?.pill_rings ?? ALL_RINGS;
  const numbers = settings?.pill_numbers ?? 'off';
  const width = pillWidth(order.length, numbers !== 'off');
  useEffect(() => {
    document.documentElement.style.setProperty('--pill-w', `${width}px`);
  }, [width]);
  const specs = ringSpecs(core, settings);
  const rings = order
    .flatMap((key) => specs.filter((r) => r.key === key))
    .filter((r) => !core || r.key !== 'dgpu' || core.dgpu != null);
  return (
    <div
      data-testid="pill"
      data-tauri-drag-region="deep"
      className="mx-auto flex items-center justify-center gap-2"
      style={{ width: 'var(--pill-w)', height: 'var(--pill-h)' }}
    >
      {rings.map((r) => (
        <span key={r.key} className="flex items-center">
          <Ring
            outer={r.outer}
            inner={r.inner}
            size={22}
            stroke={r.inner ? RING_STROKE.pillPair : RING_STROKE.pill}
            label={r.label}
          />
          {numbers !== 'off' && (
            <span
              data-testid="pill-number"
              className="w-6 text-center text-[9px] tabular-nums text-island-muted"
            >
              {pillNumber(r.key, core, numbers)}
            </span>
          )}
        </span>
      ))}
    </div>
  );
}
