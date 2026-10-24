import { describe, expect, it } from 'vitest';
import {
  formatBytes,
  formatGhz,
  formatLatency,
  formatRpm,
  formatRateRounded,
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

  it('formats used over total in whole GB with no spaces around the slash', () => {
    const gb = 1024 ** 3;
    expect(formatUsedTotal(15.4 * gb, 31.1 * gb)).toBe('15/31 GB');
    expect(formatUsedTotal(15.6 * gb, 31.1 * gb)).toBe('16/31 GB');
  });

  it('formats rounded rates', () => {
    expect(formatRateRounded(0)).toBe('0 B/s');
    expect(formatRateRounded(81920)).toBe('80 KB/s');
    expect(formatRateRounded(1.4 * 1024 * 1024)).toBe('1 MB/s');
    // 1023.6 KB rounds to 1024 KB, which is shown as 1 MB.
    expect(formatRateRounded(1023.6 * 1024)).toBe('1 MB/s');
    expect(formatRateRounded(null)).toBe('--');
  });

  it('formats clock speed in GHz to one decimal', () => {
    expect(formatGhz(3249)).toBe('3.2 GHz');
    expect(formatGhz(2000)).toBe('2.0 GHz');
    expect(formatGhz(null)).toBe('--');
    expect(formatGhz(0)).toBe('--');
  });

  it('formats fan speed', () => {
    expect(formatRpm(2104.4)).toBe('2104 rpm');
    expect(formatRpm(0)).toBe('--');
    expect(formatRpm(null)).toBe('--');
  });

  it('formats latency', () => {
    expect(formatLatency(14.2)).toBe('14 ms');
    expect(formatLatency(0)).toBe('<1 ms');
    expect(formatLatency(null)).toBe('--');
  });
});
