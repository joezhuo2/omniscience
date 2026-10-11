import { describe, expect, it } from 'vitest';
import type { KillReport, ProcessGroup } from '../types/telemetry';
import { killFailure, visibleProcesses } from './processes';

const g = (key: string, cpu: number, mem: number): ProcessGroup => ({
  key,
  name: `${key}.exe`,
  pids: [1],
  cpu_pct: cpu,
  mem_bytes: mem,
});

describe('visibleProcesses', () => {
  const groups = [g('a', 1, 300), g('b', 9, 100), g('c', 5, 200)];

  it('sorts by cpu or ram', () => {
    expect(visibleProcesses(groups, new Set(), 'cpu').map((p) => p.key)).toEqual(['b', 'c', 'a']);
    expect(visibleProcesses(groups, new Set(), 'ram').map((p) => p.key)).toEqual(['a', 'c', 'b']);
  });

  it('drops hidden rows and caps at 8', () => {
    expect(visibleProcesses(groups, new Set(['b']), 'cpu').map((p) => p.key)).toEqual(['c', 'a']);
    const many = Array.from({ length: 12 }, (_, i) => g(`p${i}`, i, i));
    expect(visibleProcesses(many, new Set(), 'cpu')).toHaveLength(8);
  });

  it('does not mutate its input', () => {
    const copy = [...groups];
    visibleProcesses(groups, new Set(), 'ram');
    expect(groups).toEqual(copy);
  });
});

describe('killFailure', () => {
  const report = (...outcomes: KillReport['results'][number]['outcome'][]): KillReport => ({
    key: 'x',
    results: outcomes.map((outcome, i) => ({ pid: i + 1, outcome, message: null })),
  });

  it('is null when every process ended or was already gone', () => {
    expect(killFailure(report('killed', 'not_found'), 'x.exe')).toBeNull();
  });

  it('explains protected and access-denied failures', () => {
    expect(killFailure(report('protected'), 'explorer.exe')).toMatch(/protected/);
    expect(killFailure(report('killed', 'access_denied'), 'svc.exe')).toMatch(/administrator/);
  });

  it('falls back to the backend message', () => {
    const r: KillReport = {
      key: 'x',
      results: [{ pid: 1, outcome: 'failed', message: 'Windows error 31' }],
    };
    expect(killFailure(r, 'x.exe')).toBe("Couldn't end x.exe: Windows error 31");
  });
});
