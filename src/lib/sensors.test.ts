import { describe, expect, it } from 'vitest';
import { sampleSnapshot } from '../test/fixtures';
import { SENSORS, sensorStates } from './sensors';

describe('sensorStates', () => {
  it('treats a missing GPU as absent, not a problem', () => {
    const s = sensorStates(sampleSnapshot.core);
    expect(s.dgpu).toBe('absent');
    expect(s.dgpu_temp).toBe('absent');
    expect(s.cpu_temp).toBe('ok');
  });

  it('flags readings that should exist but are null', () => {
    const s = sensorStates({
      ...sampleSnapshot.core,
      cpu_temp_c: null,
      disk: null,
      igpu_temp_c: null,
    });
    expect(s.cpu_temp).toBe('missing');
    expect(s.disk).toBe('missing');
    expect(s.igpu_temp).toBe('missing');
  });

  it('marks the CPU temperature as a proxy', () => {
    expect(SENSORS.cpu_temp.proxy).toBe(true);
    expect(SENSORS.cpu.proxy).toBe(false);
  });
});
