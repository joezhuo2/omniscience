import { ringSpecs } from '../lib/metrics';
import { useTelemetryStore } from '../store';
import { Ring } from './Ring';
import { RingIcon } from './RingIcon';

/** Collapsed view: six rings, no text. */
export function Pill() {
  const core = useTelemetryStore((s) => s.snapshot?.core);
  return (
    <div data-testid="pill" className="flex h-[42px] w-full items-center justify-evenly px-3">
      {ringSpecs(core).map((r) => (
        <Ring
          key={r.key}
          outer={r.outer}
          inner={r.inner}
          size={30}
          stroke={3}
          label={r.label}
          icon={<RingIcon metric={r.key} size={12} />}
        />
      ))}
    </div>
  );
}
