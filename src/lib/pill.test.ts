import { describe, expect, it } from 'vitest';
import { sampleSnapshot } from '../test/fixtures';
import { pillNumber, pillWidth } from './pill';

describe('pillWidth', () => {
  it('matches the backend formula', () => {
    expect(pillWidth(6, false)).toBe(260);
    expect(pillWidth(1, false)).toBe(110);
    expect(pillWidth(6, true)).toBe(404);
    expect(pillWidth(3, true)).toBe(88 + 66 + 16 + 72);
  });
});

describe('pillNumber', () => {
  const core = { ...sampleSnapshot.core, cpu_temp_c: 61.4, dgpu_temp_c: null };
  it('shows rounded percents', () => {
    expect(pillNumber('ram', core, 'percent')).toBe(`${Math.round(core.ram.percent)}%`);
  });
  it('prefers temperature when asked and falls back to percent', () => {
    expect(pillNumber('cpu', core, 'temp')).toBe('61°');
    expect(pillNumber('cpu', core, 'percent')).toBe(`${Math.round(core.cpu.percent)}%`);
    expect(pillNumber('ram', core, 'temp')).toBe(`${Math.round(core.ram.percent)}%`);
  });
  it('shows a dash for missing data', () => {
    expect(pillNumber('cpu', null, 'percent')).toBe('–');
    expect(pillNumber('dgpu', core, 'temp')).toBe('–');
  });
  it('shows a compact download rate for the network', () => {
    const net = { ...core.net, down_bps: 2.5 * 1024 * 1024 };
    expect(pillNumber('net', { ...core, net }, 'percent')).toBe('3M');
  });
});
