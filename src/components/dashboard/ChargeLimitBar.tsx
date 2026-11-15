import { SmoothRange } from '@crystalflux/smooth-range';
import { useEffect, useRef, useState } from 'react';
import { useNativeControl } from '../config/useNativeControl';

export interface ChargeLimitStatus {
  limit: number;
  supported: boolean;
  error: string | null;
}

const MIN = 20;

export function ChargeLimitBar() {
  const { status, busy, apply } = useNativeControl<ChargeLimitStatus>('get_charge_limit');
  const [drag, setDrag] = useState<number | null>(null);
  const latestCommit = useRef(() => {});
  const commit = () => {
    if (drag != null && drag !== status?.limit)
      void apply('set_charge_limit', { limit: drag }).then(() => setDrag(null));
    else setDrag(null);
  };
  useEffect(() => {
    latestCommit.current = commit;
  });
  if (!status?.supported) return null;
  const value = drag ?? status.limit;
  return (
    <div className="space-y-1">
      <div className="text-[11px] text-island-muted tabular-nums">Charge limit {value}%</div>
      <SmoothRange
        min={MIN}
        max={100}
        step={5}
        value={value}
        disabled={busy}
        aria-label="Charge limit"
        onValue={setDrag}
        // SmoothRange glides for 300 ms after a click before reporting the final value.
        onPointerUp={() => setTimeout(() => latestCommit.current(), 350)}
        onKeyUp={commit}
        onBlur={commit}
        className="block w-full"
      />
    </div>
  );
}
