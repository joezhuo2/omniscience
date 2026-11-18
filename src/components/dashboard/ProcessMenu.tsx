import { place, useDismiss } from '@crystalflux/position';
import { GlassSurface } from '@crystalflux/surface-glass';
import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { createPortal } from 'react-dom';
import { invoke } from '../../lib/ipc';
import { CONFIRM_MS, actionFailure } from '../../lib/processes';
import { useToastStore } from '../../toasts';
import type { ActionResult, ProcessDetails, ProcessPriority } from '../../types/telemetry';

export type MenuTarget = {
  name: string;
  pids: number[];
  mainPid: number;
  pinned?: boolean;
  onTogglePin?: () => void;
};

const PRIORITIES: { value: ProcessPriority; label: string }[] = [
  { value: 'idle', label: 'Idle' },
  { value: 'below_normal', label: 'Below normal' },
  { value: 'normal', label: 'Normal' },
  { value: 'above_normal', label: 'Above normal' },
  { value: 'high', label: 'High' },
];

const ITEM =
  'flex w-full items-center gap-2 rounded px-2 py-1 text-left text-[11px] text-island-text hover:bg-island-track/40 focus:bg-island-track/40 focus:outline-none disabled:opacity-40';

export function ProcessMenu({
  target,
  anchor,
  onClose,
}: {
  target: MenuTarget;
  anchor: DOMRect;
  onClose: () => void;
}) {
  const push = useToastStore((s) => s.push);
  const box = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ x: number; y: number }>();
  const [details, setDetails] = useState<ProcessDetails | null>(null);
  const [detailsError, setDetailsError] = useState('');
  const [armed, setArmed] = useState(false);
  const disarm = useRef<ReturnType<typeof setTimeout> | null>(null);

  useDismiss([box], true, onClose);

  useEffect(() => {
    let cancelled = false;
    invoke<ProcessDetails>('process_details', { pid: target.mainPid })
      .then((d) => !cancelled && setDetails(d))
      .catch((e: unknown) => !cancelled && setDetailsError(String(e)));
    return () => {
      cancelled = true;
      clearTimeout(disarm.current ?? undefined);
    };
  }, [target.mainPid]);

  useLayoutEffect(() => {
    const el = box.current;
    if (!el) return;
    const { x, y } = place(
      anchor,
      { width: el.offsetWidth, height: el.offsetHeight },
      { side: 'bottom', align: 'start', gap: 2 },
    );
    setPos({ x, y });
  }, [anchor, details]);

  const placed = pos !== undefined;
  useEffect(() => {
    if (placed) items()[0]?.focus();
  }, [placed]);

  function items() {
    return Array.from(
      box.current?.querySelectorAll<HTMLElement>('[role^="menuitem"]:not(:disabled)') ?? [],
    );
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === 'Tab') return onClose();
    const list = items();
    const at = list.indexOf(document.activeElement as HTMLElement);
    const next =
      event.key === 'ArrowDown'
        ? (at + 1) % list.length
        : event.key === 'ArrowUp'
          ? (at <= 0 ? list.length : at) - 1
          : event.key === 'Home'
            ? 0
            : event.key === 'End'
              ? list.length - 1
              : -1;
    if (next < 0) return;
    event.preventDefault();
    list[next]?.focus();
  }

  async function run(command: string, args: Record<string, unknown>, action: string) {
    onClose();
    try {
      const failure = actionFailure(await invoke<ActionResult[]>(command, args), action);
      if (failure) push(failure);
    } catch (e) {
      push(`${action}: ${String(e)}`);
    }
  }

  function copy(value: string | null, what: string) {
    onClose();
    if (value == null) return;
    navigator.clipboard.writeText(value).then(
      () => push(`${what} copied`),
      () => push(`Couldn't copy ${what.toLowerCase()}`),
    );
  }

  function endTree() {
    if (!armed) {
      setArmed(true);
      disarm.current = setTimeout(() => setArmed(false), CONFIRM_MS);
      return;
    }
    void run('kill_process_tree', { pids: target.pids }, 'End process tree');
  }

  const { pids } = target;
  const suspended = details?.suspended === true;
  return createPortal(
    <div
      ref={box}
      className="fixed z-[1000]"
      style={{ left: pos?.x ?? 0, top: pos?.y ?? 0, visibility: pos ? 'visible' : 'hidden' }}
      onKeyDown={onKeyDown}
    >
      <GlassSurface
        role="menu"
        aria-label={`Actions for ${target.name}`}
        radius={10}
        className="min-w-44 p-1"
      >
        <button
          role="menuitem"
          className={ITEM}
          onClick={() => {
            onClose();
            void invoke('open_process_location', { pid: target.mainPid }).catch((e: unknown) =>
              push(String(e)),
            );
          }}
        >
          Open file location
        </button>
        <button
          role="menuitem"
          className={ITEM}
          disabled={details?.path == null}
          onClick={() => copy(details?.path ?? null, 'Path')}
        >
          Copy path
        </button>
        <button
          role="menuitem"
          className={ITEM}
          disabled={details?.command_line == null}
          onClick={() => copy(details?.command_line ?? null, 'Command line')}
        >
          Copy command line
        </button>
        <div role="group" aria-label="Priority">
          <div aria-hidden className="px-2 pt-1 text-[10px] text-island-muted">
            Priority
          </div>
          {PRIORITIES.map(({ value, label }) => (
            <button
              key={value}
              role="menuitemradio"
              aria-checked={details?.priority === value}
              className={ITEM}
              onClick={() =>
                void run('set_process_priority', { pids, priority: value }, `Priority ${label}`)
              }
            >
              <span aria-hidden className="w-3">
                {details?.priority === value ? '•' : ''}
              </span>
              {label}
            </button>
          ))}
        </div>
        <button
          role="menuitemcheckbox"
          aria-checked={details?.efficiency === true}
          className={ITEM}
          onClick={() =>
            void run(
              'set_process_efficiency',
              { pids, enabled: details?.efficiency !== true },
              'Efficiency mode',
            )
          }
        >
          <span aria-hidden className="w-3">
            {details?.efficiency === true ? '✓' : ''}
          </span>
          Efficiency mode
        </button>
        <button
          role="menuitem"
          className={ITEM}
          onClick={() =>
            void run(
              suspended ? 'resume_processes' : 'suspend_processes',
              { pids },
              suspended ? 'Resume' : 'Suspend',
            )
          }
        >
          {suspended ? 'Resume' : 'Suspend'}
        </button>
        <button
          role="menuitem"
          className={`${ITEM} ${armed ? 'bg-island-crit font-semibold text-white' : 'text-island-crit'}`}
          onClick={endTree}
        >
          {armed ? 'Confirm end process tree?' : 'End process tree'}
        </button>
        {target.onTogglePin && (
          <button
            role="menuitem"
            className={ITEM}
            onClick={() => {
              onClose();
              target.onTogglePin?.();
            }}
          >
            {target.pinned ? 'Unpin' : 'Pin'}
          </button>
        )}
        {detailsError && (
          <p role="status" className="px-2 py-1 text-[10px] text-island-muted">
            Details unavailable
          </p>
        )}
      </GlassSurface>
    </div>,
    document.body,
  );
}
