import { describe, expect, it } from 'vitest';
import { formatSnoozeEnd, isSnoozed, minutesToTime, timeToMinutes } from './alerts';

describe('alerts helpers', () => {
  it('converts minutes to HH:MM and back', () => {
    expect(minutesToTime(0)).toBe('00:00');
    expect(minutesToTime(1439)).toBe('23:59');
    expect(minutesToTime(1350)).toBe('22:30');
    expect(timeToMinutes('22:30')).toBe(1350);
    expect(timeToMinutes('00:00')).toBe(0);
    for (const m of [0, 59, 60, 725, 1439]) expect(timeToMinutes(minutesToTime(m))).toBe(m);
  });

  it('rejects malformed times', () => {
    expect(timeToMinutes('')).toBeNull();
    expect(timeToMinutes('24:00')).toBeNull();
    expect(timeToMinutes('12:60')).toBeNull();
    expect(timeToMinutes('abc')).toBeNull();
  });

  it('detects an active snooze', () => {
    expect(isSnoozed(0, 1000)).toBe(false);
    expect(isSnoozed(500, 1000)).toBe(false);
    expect(isSnoozed(1000, 1000)).toBe(false);
    expect(isSnoozed(1001, 1000)).toBe(true);
  });

  it('formats the snooze end in local time', () => {
    expect(formatSnoozeEnd(new Date(2026, 0, 2, 7, 5).getTime())).toBe('07:05');
  });
});
