import { afterEach, describe, expect, it } from 'vitest';
import {
  formatRate,
  formatRateRounded,
  formatTemp,
  setUnits,
  tempFromDisplay,
  tempSymbol,
  tempValue,
} from './format';
import { pillNumber } from './pill';
import type { CoreMetrics } from '../types/telemetry';

afterEach(() => setUnits('celsius', 'bytes'));

describe('temperature unit', () => {
  it('converts for display and back', () => {
    setUnits('fahrenheit', 'bytes');
    expect(formatTemp(80)).toBe('176°F');
    expect(formatTemp(0)).toBe('32°F');
    expect(tempSymbol()).toBe('°F');
    expect(tempValue(30)).toBe(86);
    expect(tempFromDisplay(176)).toBe(80);
    expect(tempFromDisplay(175)).toBe(79.4);
    expect(tempValue(tempFromDisplay(175))).toBe(175);
  });

  it('leaves Celsius untouched', () => {
    expect(formatTemp(80.4)).toBe('80°C');
    expect(tempValue(80)).toBe(80);
    expect(tempFromDisplay(80)).toBe(80);
  });

  it('follows the unit on the pill', () => {
    setUnits('fahrenheit', 'bytes');
    const core = { cpu_temp_c: 50, cpu: { percent: 10 } } as unknown as CoreMetrics;
    expect(pillNumber('cpu', core, 'temp')).toBe('122°');
  });
});

describe('network unit', () => {
  it('keeps bytes by default', () => {
    expect(formatRate(1536)).toBe('1.5 KB/s');
    expect(formatRateRounded(1_048_576)).toBe('1 MB/s');
  });

  it('shows bits as value times eight in decimal steps', () => {
    setUnits('celsius', 'bits');
    expect(formatRate(0)).toBe('0 b/s');
    expect(formatRate(100)).toBe('800 b/s');
    expect(formatRate(125_000)).toBe('1.0 Mb/s');
    expect(formatRate(12_500_000)).toBe('100 Mb/s');
    expect(formatRateRounded(125_000_000)).toBe('1 Gb/s');
    expect(formatRateRounded(1000)).toBe('8 Kb/s');
    expect(formatRate(null)).toBe('--');
    expect(formatRateRounded(-1)).toBe('--');
  });

  it('keeps the pill rate compact in bits', () => {
    setUnits('celsius', 'bits');
    const core = { net: { down_bps: 125_000, up_bps: 0 } } as unknown as CoreMetrics;
    expect(pillNumber('net', core, 'percent')).toBe('1M');
  });
});
