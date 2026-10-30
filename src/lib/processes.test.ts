import { describe, expect, it } from 'vitest';
import type { KillReport, ProcessGroup } from '../types/telemetry';
import { killFailure, visibleProcesses } from './processes';

const g = (key: string, cpu: number, mem: number, gpu = 0, vram = 0): ProcessGroup => ({
  key,
  name: `${key}.exe`,
  pids: [1],
  cpu_pct: cpu,
  mem_bytes: mem,
  gpu_pct: gpu,
  gpu_mem_bytes: vram,
  disk_bps: null,
  net_bps: null,
});

describe('visibleProcesses', () => {
  it('filters names case-insensitively before limiting, and matches child PIDs', () => {
    const many = Array.from({ length: 12 }, (_, i) => g(`app${i}`, i, i));
    many[0]!.name = 'Quiet App.exe';
    many[0]!.pids = [42, 9876];
    const search = (query: string) => visibleProcesses(many, new Set(), 'cpu', 8, query);
    expect(search('  QUIET APP  ').map((p) => p.key)).toEqual(['app0']);
    expect(search('9876').map((p) => p.key)).toEqual(['app0']);
    expect(search('missing')).toEqual([]);
    expect(search('   ')).toEqual(visibleProcesses(many, new Set(), 'cpu'));
    expect(visibleProcesses(many, new Set(['app0']), 'cpu', 8, 'quiet')).toEqual([]);
  });

  it('keeps each sort order within search results', () => {
    const matches = [g('app-a', 9, 1, 2), g('app-b', 1, 9, 1), g('other', 99, 99, 99)];
    const keys = (sort: 'cpu' | 'ram' | 'gpu') =>
      visibleProcesses(matches, new Set(), sort, 8, 'app').map((p) => p.key);
    expect(keys('cpu')).toEqual(['app-a', 'app-b']);
    expect(keys('ram')).toEqual(['app-b', 'app-a']);
    expect(keys('gpu')).toEqual(['app-a', 'app-b']);
  });

  const groups = [g('a', 1, 300), g('b', 9, 100), g('c', 5, 200)];

  it('sorts by cpu or ram', () => {
    expect(visibleProcesses(groups, new Set(), 'cpu').map((p) => p.key)).toEqual(['b', 'c', 'a']);
    expect(visibleProcesses(groups, new Set(), 'ram').map((p) => p.key)).toEqual(['a', 'c', 'b']);
  });

  it('sorts by gpu usage, then gpu memory', () => {
    const gpu = [g('a', 9, 9, 0, 50), g('b', 1, 1, 30, 0), g('c', 5, 5, 0, 200)];
    expect(visibleProcesses(gpu, new Set(), 'gpu').map((p) => p.key)).toEqual(['b', 'c', 'a']);
  });

  it.each(['disk', 'network'] as const)(
    'sorts %s rates before idle and missing values without mutating input',
    (sort) => {
      const field = sort === 'disk' ? 'disk_bps' : 'net_bps';
      const apps = [g('missing', 99, 99), g('idle', 0, 0), g('busy', 0, 0), g('invalid', 0, 0)];
      apps[1]![field] = 0;
      apps[2]![field] = 4096;
      apps[3]![field] = Infinity;
      const before = [...apps];
      expect(visibleProcesses(apps, new Set(), sort).map((p) => p.key)).toEqual([
        'busy',
        'idle',
        'invalid',
        'missing',
      ]);
      expect(visibleProcesses(apps, new Set(['busy']), sort, 1).map((p) => p.key)).toEqual([
        'idle',
      ]);
      expect(visibleProcesses(apps, new Set(), sort, 8, 'missing').map((p) => p.key)).toEqual([
        'missing',
      ]);
      expect(apps).toEqual(before);
    },
  );

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
