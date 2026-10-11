import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { CRITICAL, WARNING } from '../lib/levels';
import { ringDash } from '../lib/ring';
import { Ring } from './Ring';

const accent = '#22D3EE';

describe('ringDash', () => {
  it('maps percent to dash offset and clamps', () => {
    const r = 10;
    const c = 2 * Math.PI * r;
    expect(ringDash(0, r).offset).toBeCloseTo(c);
    expect(ringDash(100, r).offset).toBeCloseTo(0);
    expect(ringDash(25, r).offset).toBeCloseTo(c * 0.75);
    expect(ringDash(150, r).offset).toBeCloseTo(0);
    expect(ringDash(-5, r).offset).toBeCloseTo(c);
  });
});

describe('Ring', () => {
  it.each([
    ['normal', accent],
    ['warning', WARNING],
    ['critical', CRITICAL],
  ] as const)('colors a %s arc %s', (level, color) => {
    render(<Ring outer={{ percent: 50, level, accent }} size={28} stroke={3} label="CPU" />);
    expect(screen.getByTestId('ring-arc')).toHaveAttribute('stroke', color);
  });

  it('pulses only when critical', () => {
    const { rerender } = render(
      <Ring outer={{ percent: 91, level: 'critical', accent }} size={28} stroke={3} label="CPU" />,
    );
    expect(screen.getByTestId('ring-arc')).toHaveClass('ring-critical');
    rerender(
      <Ring outer={{ percent: 71, level: 'warning', accent }} size={28} stroke={3} label="CPU" />,
    );
    expect(screen.getByTestId('ring-arc')).not.toHaveClass('ring-critical');
  });

  it('shows a dimmed dashed track and no arc when the sensor is missing', () => {
    render(
      <Ring outer={{ percent: null, level: 'normal', accent }} size={28} stroke={3} label="dGPU" />,
    );
    expect(screen.queryByTestId('ring-arc')).toBeNull();
    expect(screen.getByRole('img', { name: 'dGPU' })).toHaveClass('opacity-40');
  });

  it('draws an inner concentric arc', () => {
    render(
      <Ring
        outer={{ percent: 40, level: 'normal', accent }}
        inner={{ percent: 10, level: 'normal', accent }}
        size={28}
        stroke={3}
        label="Network"
      />,
    );
    expect(screen.getAllByTestId('ring-arc')).toHaveLength(2);
  });
});
