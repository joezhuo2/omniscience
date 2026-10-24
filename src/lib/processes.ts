import type { KillReport, ProcessGroup } from '../types/telemetry';

export type ProcessSort = 'cpu' | 'ram' | 'gpu';

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
): ProcessGroup[] {
  return groups
    .filter((g) => !hidden.has(g.key))
    .sort((a, b) => {
      switch (sort) {
        case 'cpu':
          return b.cpu_pct - a.cpu_pct || b.mem_bytes - a.mem_bytes;
        case 'ram':
          return b.mem_bytes - a.mem_bytes || b.cpu_pct - a.cpu_pct;
        case 'gpu':
          return b.gpu_pct - a.gpu_pct || b.gpu_mem_bytes - a.gpu_mem_bytes;
      }
    })
    .slice(0, limit);
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
