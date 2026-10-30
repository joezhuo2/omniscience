import { describe, expect, it } from 'vitest';
import { sampleSnapshot } from '../test/fixtures';
import { ringSpecs, temperatureArc } from './metrics';

describe('temperature rings', () => {
  it.each([
    [0, 0],
    [52.5, 50],
    [105, 100],
    [125, 100],
    [-5, 0],
  ])('scales %s°C to %s%%', (temp, percent) => {
    expect(temperatureArc(temp, 'blue')?.percent).toBe(percent);
  });

  it.each([null, undefined, NaN, Infinity])('hides an unavailable reading (%s)', (temp) => {
    expect(temperatureArc(temp, 'blue')).toBeUndefined();
  });

  it('uses warning bands independently of utilization and respects settings', () => {
    expect(temperatureArc(80, 'blue')?.level).toBe('warning');
    expect(temperatureArc(100, 'blue')?.level).toBe('critical');
    expect(temperatureArc(52.5, 'blue', { warn_threshold: 40, crit_threshold: 60 })?.level).toBe(
      'warning',
    );
    expect(temperatureArc(63, 'blue', { warn_threshold: 40, crit_threshold: 60 })?.level).toBe(
      'warning',
    );
  });

  it('keeps the expanded dashboard dGPU slot even when unavailable', () => {
    expect(ringSpecs(undefined).map((r) => r.key)).toContain('dgpu');
    expect(ringSpecs(sampleSnapshot.core).map((r) => r.key)).toContain('dgpu');
    expect(
      ringSpecs({ ...sampleSnapshot.core, dgpu: { percent: 0, level: 'normal' } }).map(
        (r) => r.key,
      ),
    ).toContain('dgpu');
  });

  it('renders live inner temperatures without expanded detail and preserves network rings', () => {
    const rings = ringSpecs(sampleSnapshot.core);
    expect(rings.find((r) => r.key === 'cpu')?.inner?.percent).toBeCloseTo((48 / 105) * 100);
    expect(rings.find((r) => r.key === 'igpu')?.inner?.percent).toBeCloseTo((45 / 105) * 100);
    expect(rings.find((r) => r.key === 'dgpu')?.inner).toBeUndefined();
    expect(rings.find((r) => r.key === 'net')?.inner?.percent).toBe(5);
  });
});
