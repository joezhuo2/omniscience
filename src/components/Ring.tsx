import type { ReactNode } from 'react';
import { levelColor } from '../lib/levels';
import { ringDash } from '../lib/ring';
import type { Level } from '../types/telemetry';

export interface Arc {
  /** 0-100, or null when the sensor is missing. */
  percent: number | null;
  level: Level;
  accent: string;
}

interface RingProps {
  outer: Arc;
  /** Optional second, concentric arc (e.g. upload inside download). */
  inner?: Arc;
  size: number;
  stroke: number;
  label: string;
  icon?: ReactNode;
}

function ArcCircle({ arc, r, c, stroke }: { arc: Arc; r: number; c: number; stroke: number }) {
  const missing = arc.percent == null;
  const { circumference, offset } = ringDash(arc.percent ?? 0, r);
  return (
    <>
      <circle
        cx={c}
        cy={c}
        r={r}
        fill="none"
        stroke="var(--color-island-track)"
        strokeWidth={stroke}
        strokeDasharray={missing ? '2 3' : undefined}
      />
      {!missing && (
        <circle
          data-testid="ring-arc"
          data-level={arc.level}
          className={`ring-arc${arc.level === 'critical' ? ' ring-critical' : ''}`}
          cx={c}
          cy={c}
          r={r}
          fill="none"
          stroke={levelColor(arc.level, arc.accent)}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${c} ${c})`}
        />
      )}
    </>
  );
}

/** Circular progress ring, optionally with a concentric inner ring and a centered icon. */
export function Ring({ outer, inner, size, stroke, label, icon }: RingProps) {
  const c = size / 2;
  const rOuter = c - stroke / 2;
  const rInner = rOuter - stroke - 1.5;
  const missing = outer.percent == null && (inner == null || inner.percent == null);
  return (
    <div
      role="img"
      aria-label={label}
      className={`relative shrink-0 ${missing ? 'opacity-40' : ''}`}
      style={{ width: size, height: size }}
    >
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden>
        <ArcCircle arc={outer} r={rOuter} c={c} stroke={stroke} />
        {inner && <ArcCircle arc={inner} r={rInner} c={c} stroke={stroke} />}
      </svg>
      {icon && (
        <span className="absolute inset-0 flex items-center justify-center text-island-muted">
          {icon}
        </span>
      )}
    </div>
  );
}
