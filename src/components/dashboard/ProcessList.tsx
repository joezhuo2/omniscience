import { invoke } from '@tauri-apps/api/core';
import { ChevronDown, X } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { formatBytes } from '../../lib/format';
import {
  COLLAPSED_PROCESSES,
  CONFIRM_MS,
  killFailure,
  visibleProcesses,
  type ProcessSort,
} from '../../lib/processes';
import { useTelemetryStore } from '../../store';
import { useToastStore } from '../../toasts';
import type { KillReport, ProcessGroup } from '../../types/telemetry';
import { Card } from './Card';
import { Skeleton } from './Skeleton';

function SortToggle({ sort, onChange }: { sort: ProcessSort; onChange: (s: ProcessSort) => void }) {
  return (
    <div role="tablist" className="flex rounded-md bg-island-hover p-0.5 text-[10px]">
      {(['cpu', 'ram'] as const).map((s) => (
        <button
          key={s}
          role="tab"
          aria-selected={sort === s}
          className={`rounded px-2 py-0.5 font-medium uppercase transition-colors ${
            sort === s
              ? 'bg-island-track text-island-text'
              : 'text-island-muted hover:text-island-text'
          }`}
          onClick={() => onChange(s)}
        >
          {s}
        </button>
      ))}
    </div>
  );
}

function Row({
  p,
  armed,
  onKill,
}: {
  p: ProcessGroup;
  armed: boolean;
  onKill: (p: ProcessGroup) => void;
}) {
  return (
    <li className="group flex items-center gap-2 rounded-md px-1.5 py-1 text-[11px] hover:bg-island-track/30">
      <span className="min-w-0 flex-1 truncate text-island-text" title={p.name}>
        {p.name.replace(/\.exe$/i, '')}
        {p.pids.length > 1 && <span className="ml-1 text-island-muted">×{p.pids.length}</span>}
      </span>
      <span className="w-12 text-right text-island-text tabular-nums">{p.cpu_pct.toFixed(1)}%</span>
      <span className="w-16 text-right text-island-muted tabular-nums">
        {formatBytes(p.mem_bytes)}
      </span>
      <button
        aria-label={armed ? `Confirm end ${p.name}` : `End ${p.name}`}
        className={`flex h-5 items-center justify-center rounded transition-colors ${
          armed
            ? 'w-11 bg-island-crit text-[10px] font-semibold text-white'
            : 'w-5 text-island-muted hover:bg-island-crit/20 hover:text-island-crit'
        }`}
        onClick={() => onKill(p)}
      >
        {armed ? 'End?' : <X size={12} aria-hidden />}
      </button>
    </li>
  );
}

export function ProcessList() {
  const groups = useTelemetryStore((s) => s.snapshot?.detail?.processes);
  const push = useToastStore((s) => s.push);
  const [sort, setSort] = useState<ProcessSort>('cpu');
  const [armed, setArmed] = useState<string | null>(null);
  const [hidden, setHidden] = useState<ReadonlySet<string>>(new Set());
  const [showAll, setShowAll] = useState(false);
  const disarm = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => () => clearTimeout(disarm.current ?? undefined), []);

  async function onKill(p: ProcessGroup) {
    if (armed !== p.key) {
      setArmed(p.key);
      clearTimeout(disarm.current ?? undefined);
      disarm.current = setTimeout(() => setArmed(null), CONFIRM_MS);
      return;
    }
    clearTimeout(disarm.current ?? undefined);
    setArmed(null);
    setHidden((h) => new Set(h).add(p.key));
    let failure: string | null;
    try {
      const report = await invoke<KillReport>('kill_process_group', { key: p.key });
      failure = killFailure(report, p.name);
    } catch (e) {
      failure = `Couldn't end ${p.name}: ${String(e)}`;
    }
    if (failure) {
      setHidden((h) => {
        const next = new Set(h);
        next.delete(p.key);
        return next;
      });
      push(failure);
    }
  }

  const all = groups ? visibleProcesses(groups, hidden, sort) : [];
  const rows = showAll ? all : all.slice(0, COLLAPSED_PROCESSES);
  return (
    <Card title="Top processes" aside={<SortToggle sort={sort} onChange={setSort} />}>
      {!groups ? (
        <div className="space-y-1.5">
          {Array.from({ length: 5 }, (_, i) => (
            <Skeleton key={i} className="h-4" />
          ))}
        </div>
      ) : (
        <>
          <ul
            data-testid="process-rows"
            className={`-mx-1.5 overscroll-contain ${showAll ? 'max-h-36 overflow-y-auto' : ''}`}
          >
            {rows.map((p) => (
              <Row key={p.key} p={p} armed={armed === p.key} onKill={onKill} />
            ))}
          </ul>
          {all.length > COLLAPSED_PROCESSES && (
            <button
              aria-expanded={showAll}
              onClick={() => setShowAll((v) => !v)}
              className="mt-1 flex w-full items-center justify-center gap-1 rounded-md py-0.5 text-[10px] text-island-muted hover:bg-island-hover hover:text-island-text"
            >
              {showAll ? 'Show less' : `Show ${all.length - COLLAPSED_PROCESSES} more`}
              <ChevronDown
                size={11}
                aria-hidden
                className={`transition-transform ${showAll ? 'rotate-180' : ''}`}
              />
            </button>
          )}
        </>
      )}
    </Card>
  );
}
