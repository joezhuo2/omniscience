import type { ActionResult, KillReport, ProcessGroup } from '../types/telemetry';

export type ProcessSort = 'cpu' | 'ram' | 'gpu' | 'disk' | 'network';

/** Keep unavailable throughput distinct from a measured idle app. */
export function processRate(value: number | null | undefined): number | null {
  return value != null && Number.isFinite(value) && value >= 0 ? value : null;
}

/** Rows shown by default, and when "Show more" is on. */
export const COLLAPSED_PROCESSES = 3;
export const VISIBLE_PROCESSES = 8;

/** How long the red "End?" confirmation stays armed. */
export const CONFIRM_MS = 3000;

/** Top groups for the chosen sort, minus rows hidden optimistically while a kill is in flight. */
export function visibleProcesses(
  groups: ProcessGroup[],
  hidden: ReadonlySet<string>,
  sort: ProcessSort,
  limit: number = VISIBLE_PROCESSES,
  query = '',
  pinned: ReadonlySet<string> = new Set(),
): ProcessGroup[] {
  const search = query.trim().toLowerCase();
  const sorted = groups
    .filter(
      (g) =>
        !hidden.has(g.key) &&
        (!search ||
          g.name.toLowerCase().includes(search) ||
          g.pids.some((pid) => String(pid).includes(search))),
    )
    .sort((a, b) => {
      switch (sort) {
        case 'cpu':
          return b.cpu_pct - a.cpu_pct || b.mem_bytes - a.mem_bytes;
        case 'ram':
          return b.mem_bytes - a.mem_bytes || b.cpu_pct - a.cpu_pct;
        case 'gpu':
          return b.gpu_pct - a.gpu_pct || b.gpu_mem_bytes - a.gpu_mem_bytes;
        case 'disk':
        case 'network': {
          const field = sort === 'disk' ? 'disk_bps' : 'net_bps';
          return (
            (processRate(b[field]) ?? -1) - (processRate(a[field]) ?? -1) ||
            a.key.localeCompare(b.key)
          );
        }
      }
    });
  const first = sorted.filter((g) => pinned.has(g.key));
  const rest = sorted.filter((g) => !pinned.has(g.key));
  return [...first, ...rest].slice(0, Math.max(limit, first.length));
}

/** Rows kept when the list is collapsed: the short head plus every pinned group. */
export function collapsedProcesses(
  rows: ProcessGroup[],
  pinned: ReadonlySet<string>,
): ProcessGroup[] {
  return rows.filter((g, i) => i < COLLAPSED_PROCESSES || pinned.has(g.key));
}

const PINNED_KEY = 'omniscience:pinned-processes';

export function readPinned(): Set<string> {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(PINNED_KEY) ?? '[]');
    return new Set(Array.isArray(value) ? value.filter((k) => typeof k === 'string') : []);
  } catch {
    return new Set();
  }
}

export function savePinned(pinned: ReadonlySet<string>): void {
  try {
    localStorage.setItem(PINNED_KEY, JSON.stringify([...pinned]));
  } catch {
    // Pins still work for this session if storage is unavailable.
  }
}

/** The member using the most memory, which the menu inspects for the whole row. */
export function mainPid(p: ProcessGroup): number {
  return (
    p.members.reduce<ProcessGroup['members'][number] | null>(
      (best, m) => (!best || m.mem_bytes > best.mem_bytes ? m : best),
      null,
    )?.pid ??
    p.pids[0] ??
    0
  );
}

const OUTCOME_LABELS = {
  protected: 'protected',
  access_denied: 'access denied',
  not_found: 'not found',
  failed: 'failed',
} as const;

/** Toast text for the results that were not ok, or null when everything succeeded. */
export function actionFailure(results: ActionResult[], action: string): string | null {
  const failed = results.filter((r) => r.outcome !== 'ok');
  if (failed.length === 0) return null;
  const parts = (Object.keys(OUTCOME_LABELS) as (keyof typeof OUTCOME_LABELS)[])
    .map((o) => [failed.filter((r) => r.outcome === o).length, OUTCOME_LABELS[o]] as const)
    .filter(([n]) => n > 0)
    .map(([n, label]) => `${n} ${label}`);
  const message = failed.find((r) => r.message)?.message;
  return `${action}: ${parts.join(', ')}${message ? `. ${message}` : ''}`;
}

/**
 * Why a kill did not fully succeed, or null if every process ended (or had already exited).
 * Used to roll back the optimistic removal and explain it in a toast.
 */
export function killFailure(report: KillReport, name: string): string | null {
  const failed = report.results.filter((r) => r.outcome !== 'killed' && r.outcome !== 'not_found');
  if (failed.length === 0) return null;
  if (failed.some((r) => r.outcome === 'protected'))
    return `${name} is protected and can't be ended`;
  if (failed.some((r) => r.outcome === 'access_denied'))
    return `Can't end ${name}: needs administrator rights`;
  return `Couldn't end ${name}${failed[0]?.message ? `: ${failed[0].message}` : ''}`;
}
