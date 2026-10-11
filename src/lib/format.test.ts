import { describe, expect, it } from 'vitest';
import {
  formatBytes,
  formatPercent,
  formatRate,
  formatTemp,
  formatUsedTotal,
  formatWatts,
} from './format';

describe('format', () => {
  it('formats bytes with binary units', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(1024 ** 3 * 5.25)).toBe('5.3 GB');
    expect(formatBytes(1024 * 1024 * 512)).toBe('512 MB');
  });

  it('shows -- for missing or invalid values', () => {
    expect(formatBytes(null)).toBe('--');
    expect(formatBytes(-1)).toBe('--');
    expect(formatRate(undefined)).toBe('--');
    expect(formatPercent(Number.NaN)).toBe('--');
    expect(formatWatts(null)).toBe('-- W');
    expect(formatTemp(null)).toBe('--');
    expect(formatUsedTotal(1, 0)).toBe('--');
  });

  it('formats rates, percents, watts and temperatures', () => {
    expect(formatRate(1.2 * 1024 * 1024)).toBe('1.2 MB/s');
    expect(formatPercent(23.6)).toBe('24%');
    expect(formatWatts(8.22)).toBe('8.2 W');
    expect(formatWatts(45.6)).toBe('46 W');
    expect(formatTemp(47.6)).toBe('48°C');
  });

  it('formats used over total in GB', () => {
    const gb = 1024 ** 3;
    expect(formatUsedTotal(12.4 * gb, 31.1 * gb)).toBe('12.4 / 31.1 GB');
  });
});
