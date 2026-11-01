import { useState } from 'react';
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
  if (!status?.supported) return null;
  const value = drag ?? status.limit;
  const commit = () => {
    if (drag != null && drag !== status.limit)
      void apply('set_charge_limit', { limit: drag }).then(() => setDrag(null));
    else setDrag(null);
  };
  return (
    <div className="space-y-1">
      <div className="text-[11px] text-island-muted tabular-nums">Charge limit {value}%</div>
      <input
        type="range"
        min={MIN}
        max={100}
        step={5}
        value={value}
        disabled={busy}
        aria-label="Charge limit"
        onChange={(e) => setDrag(Number(e.target.value))}
        onPointerUp={commit}
        onKeyUp={commit}
        onBlur={commit}
        className="block h-1 w-full cursor-pointer appearance-none rounded-full disabled:opacity-50 [&::-webkit-slider-thumb]:size-2.5 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-island-text"
        style={{
          background: `linear-gradient(to right, var(--color-island-muted) ${((value - MIN) / (100 - MIN)) * 100}%, var(--color-island-track) 0)`,
        }}
      />
    </div>
  );
}
