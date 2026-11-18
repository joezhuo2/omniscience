import { invoke } from '../../lib/ipc';
import { ChevronDown, ChevronRight, MoreHorizontal, Pin, Search, X } from 'lucide-react';
import { useCallback, useContext, useEffect, useRef, useState } from 'react';
import { formatBytes, formatRateRounded } from '../../lib/format';
import {
  CONFIRM_MS,
  collapsedProcesses,
  killFailure,
  mainPid,
  processRate,
  readPinned,
  savePinned,
  visibleProcesses,
  type ProcessSort,
} from '../../lib/processes';
import { useTelemetryStore } from '../../store';
import { useToastStore } from '../../toasts';
import type { KillReport, ProcessGroup } from '../../types/telemetry';
import { Card } from './Card';
import { ProcessMenu, type MenuTarget } from './ProcessMenu';
import { InfoTabVisible } from './infoTabContext';
import { Segmented } from '../Segmented';
import { ScrollArea } from '../ScrollArea';
import { Skeleton } from './Skeleton';
import { Tip } from '../Tip';

function SortToggle({ sort, onChange }: { sort: ProcessSort; onChange: (s: ProcessSort) => void }) {
  return (
    <Segmented
      role="tab"
      label="Sort processes"
      className="uppercase"
      items={(['cpu', 'ram', 'gpu', 'disk', 'network'] as const).map((v) => ({
        value: v,
        label: v,
      }))}
      value={sort}
      onChange={onChange}
    />
  );
}

type OpenMenu = (target: MenuTarget, anchor: DOMRect, opener: HTMLElement | null) => void;

function MoreButton({ label, onOpen }: { label: string; onOpen: (el: HTMLElement) => void }) {
  return (
    <button
      aria-label={label}
      aria-haspopup="menu"
      className="flex h-5 w-5 items-center justify-center rounded text-island-muted hover:bg-island-track/40 hover:text-island-text"
      onClick={(event) => onOpen(event.currentTarget)}
    >
      <MoreHorizontal size={12} aria-hidden />
    </button>
  );
}

function Row({
  p,
  sort,
  armed,
  pinned,
  expanded,
  onKill,
  onToggleExpand,
  onTogglePin,
  onMenu,
}: {
  p: ProcessGroup;
  sort: ProcessSort;
  armed: boolean;
  pinned: boolean;
  expanded: boolean;
  onKill: (p: ProcessGroup) => void;
  onToggleExpand: (key: string) => void;
  onTogglePin: (key: string) => void;
  onMenu: OpenMenu;
}) {
  const target: MenuTarget = {
    name: p.name,
    pids: p.pids,
    mainPid: mainPid(p),
    pinned,
    onTogglePin: () => onTogglePin(p.key),
  };
  // The GPU view swaps both columns for GPU usage and GPU memory.
  const gpu = sort === 'gpu';
  const throughput = sort === 'disk' || sort === 'network';
  const rate = processRate(sort === 'disk' ? p.disk_bps : p.net_bps);
  return (
    <li className="group text-[11px]">
      <div
        className="flex items-center gap-2 rounded-md px-1.5 py-1 hover:bg-island-track/30"
        onContextMenu={(event) => {
          event.preventDefault();
          onMenu(target, new DOMRect(event.clientX, event.clientY, 0, 0), null);
        }}
      >
        {p.members.length > 1 ? (
          <button
            aria-label={`${expanded ? 'Hide' : 'Show'} ${p.name} processes`}
            aria-expanded={expanded}
            className="-mr-1 flex h-5 w-4 shrink-0 items-center justify-center text-island-muted hover:text-island-text"
            onClick={() => onToggleExpand(p.key)}
          >
            <ChevronRight
              size={11}
              aria-hidden
              className={`transition-transform ${expanded ? 'rotate-90' : ''}`}
            />
          </button>
        ) : (
          <span className="-mr-1 w-4 shrink-0" />
        )}
        <Tip content={p.name}>
          <span className="min-w-0 flex-1 truncate text-island-text">
            {pinned && (
              <Pin size={10} aria-label="Pinned" className="mr-1 inline text-island-muted" />
            )}
            {p.name.replace(/\.exe$/i, '')}
            {p.pids.length > 1 && <span className="ml-1 text-island-muted">×{p.pids.length}</span>}
          </span>
        </Tip>
        <Tip
          content={
            throughput
              ? sort === 'disk'
                ? 'Disk read + write'
                : 'Network send + receive'
              : undefined
          }
        >
          <span
            className={`${throughput ? 'w-20' : 'w-12'} text-right text-island-text tabular-nums`}
          >
            {throughput
              ? rate == null
                ? '—'
                : formatRateRounded(rate)
              : `${(gpu ? p.gpu_pct : p.cpu_pct).toFixed(1)}%`}
          </span>
        </Tip>
        <Tip content={gpu ? 'GPU memory' : 'Memory'}>
          <span className="w-16 text-right text-island-muted tabular-nums">
            {formatBytes(gpu ? p.gpu_mem_bytes : p.mem_bytes)}
          </span>
        </Tip>
        <MoreButton
          label={`More actions for ${p.name}`}
          onOpen={(el) => onMenu(target, el.getBoundingClientRect(), el)}
        />
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
      </div>
      {expanded && (
        <div role="group" aria-label={`${p.name} processes`} className="pb-1 pl-6">
          {p.members.map((m) => (
            <div
              key={m.pid}
              className="flex items-center gap-2 px-1.5 py-0.5 text-[10px] text-island-muted"
              onContextMenu={(event) => {
                event.preventDefault();
                onMenu(
                  { name: `${p.name} (${m.pid})`, pids: [m.pid], mainPid: m.pid },
                  new DOMRect(event.clientX, event.clientY, 0, 0),
                  null,
                );
              }}
            >
              <span className="min-w-0 flex-1 tabular-nums">PID {m.pid}</span>
              <span className="w-12 text-right tabular-nums">{m.cpu_pct.toFixed(1)}%</span>
              <span className="w-16 text-right tabular-nums">{formatBytes(m.mem_bytes)}</span>
              <MoreButton
                label={`More actions for ${p.name} PID ${m.pid}`}
                onOpen={(el) =>
                  onMenu(
                    { name: `${p.name} (${m.pid})`, pids: [m.pid], mainPid: m.pid },
                    el.getBoundingClientRect(),
                    el,
                  )
                }
              />
              <span className="w-5" />
            </div>
          ))}
        </div>
      )}
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
  const [pinned, setPinned] = useState(readPinned);
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(new Set());
  const [menu, setMenu] = useState<{
    target: MenuTarget;
    anchor: DOMRect;
    opener: HTMLElement | null;
  } | null>(null);
  const [listHeight, setListHeight] = useState<number>();
  const measureList = useCallback((el: HTMLUListElement | null) => {
    if (!el) return;
    const observer = new ResizeObserver(() => setListHeight(el.offsetHeight));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  const [query, setQuery] = useState('');
  const [settledQuery, setSettledQuery] = useState('');
  const searchTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const searching = query !== settledQuery;
  const disarm = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [open, setOpen] = useState(true);
  const sampling = useContext(InfoTabVisible) && open;

  useEffect(() => {
    const send = (enabled: boolean) =>
      void invoke('set_process_sampling', { enabled }).catch(() => {});
    send(sampling);
    return () => send(false);
  }, [sampling]);

  useEffect(
    () => () => {
      clearTimeout(disarm.current ?? undefined);
      clearTimeout(searchTimer.current ?? undefined);
    },
    [],
  );

  function onSearch(value: string) {
    setQuery(value);
    setShowAll(false);
    setArmed(null);
    clearTimeout(disarm.current ?? undefined);
    clearTimeout(searchTimer.current ?? undefined);
    if (!value.trim()) {
      setSettledQuery(value);
    } else {
      searchTimer.current = setTimeout(() => setSettledQuery(value), 180);
    }
  }

  function toggleIn(set: ReadonlySet<string>, key: string) {
    const next = new Set(set);
    if (!next.delete(key)) next.add(key);
    return next;
  }

  function onTogglePin(key: string) {
    const next = toggleIn(pinned, key);
    setPinned(next);
    savePinned(next);
  }

  function closeMenu() {
    menu?.opener?.focus();
    setMenu(null);
  }

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

  const all = groups ? visibleProcesses(groups, hidden, sort, undefined, settledQuery, pinned) : [];
  const collapsed = collapsedProcesses(all, pinned);
  const rows = showAll ? all : collapsed;
  return (
    <Card
      section="processes"
      title="Top processes"
      aside={<SortToggle sort={sort} onChange={setSort} />}
      onOpenChange={setOpen}
    >
      <div className="glass-field mb-2 flex items-center gap-1.5 rounded-md px-2 focus-within:ring-1 focus-within:ring-island-muted">
        <Search size={12} aria-hidden className="shrink-0 text-island-muted" />
        <input
          type="search"
          aria-label="Search processes"
          aria-controls="process-results"
          placeholder="Search name or PID"
          value={query}
          onChange={(event) => onSearch(event.target.value)}
          className="min-w-0 flex-1 bg-transparent py-1 text-[11px] text-island-text outline-none placeholder:text-island-muted [&::-webkit-search-cancel-button]:appearance-none"
        />
        {query && (
          <button
            type="button"
            aria-label="Clear process search"
            onClick={() => onSearch('')}
            className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-island-muted hover:text-island-text"
          >
            <X size={12} aria-hidden />
          </button>
        )}
      </div>
      {(sort === 'disk' || sort === 'network') &&
        groups &&
        groups.every((p) => processRate(sort === 'disk' ? p.disk_bps : p.net_bps) == null) && (
          <p role="status" className="mb-2 text-[10px] text-island-muted">
            Per-app throughput is unavailable. On Windows, administrator rights may be required.
          </p>
        )}
      <div id="process-results" aria-busy={!groups || searching}>
        {!groups || searching ? (
          <div className="space-y-1.5">
            {searching && (
              <span role="status" className="sr-only">
                Searching processes
              </span>
            )}
            {Array.from({ length: 5 }, (_, i) => (
              <Skeleton key={i} className="h-4" />
            ))}
          </div>
        ) : (
          <>
            {all.length === 0 && (
              <p role="status" className="py-2 text-center text-[11px] text-island-muted">
                {settledQuery.trim() ? 'No matching processes' : 'No processes available'}
              </p>
            )}
            <ScrollArea
              className={`-mx-1.5 overscroll-none transition-[height] duration-200 ease-out ${showAll ? 'max-h-36 overflow-y-auto' : 'overflow-hidden'}`}
              style={{ height: listHeight && (showAll ? Math.min(listHeight, 144) : listHeight) }}
            >
              <ul ref={measureList} data-testid="process-rows">
                {rows.map((p) => (
                  <Row
                    key={p.key}
                    p={p}
                    sort={sort}
                    armed={armed === p.key}
                    pinned={pinned.has(p.key)}
                    expanded={expanded.has(p.key)}
                    onKill={onKill}
                    onToggleExpand={(key) => setExpanded((e) => toggleIn(e, key))}
                    onTogglePin={onTogglePin}
                    onMenu={(target, anchor, opener) => setMenu({ target, anchor, opener })}
                  />
                ))}
              </ul>
            </ScrollArea>
            {all.length > collapsed.length && (
              <button
                aria-expanded={showAll}
                onClick={() => setShowAll((v) => !v)}
                className="mt-1 flex w-full items-center justify-center gap-1 py-0.5 text-[10px] text-island-muted transition-colors hover:text-island-text"
              >
                {showAll ? 'Show less' : `Show ${all.length - collapsed.length} more`}
                <ChevronDown
                  size={11}
                  aria-hidden
                  className={`transition-transform ${showAll ? 'rotate-180' : ''}`}
                />
              </button>
            )}
          </>
        )}
      </div>
      {menu && <ProcessMenu target={menu.target} anchor={menu.anchor} onClose={closeMenu} />}
    </Card>
  );
}
