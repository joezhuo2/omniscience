/**
 * Pill <-> dashboard state machine.
 *
 * - `expanding`: the native window is being snapped to the expanded bounds.
 * - `expanded`: content springs open inside the large window.
 * - `collapsing`: content springs closed; the window shrinks when that animation ends.
 */
export type Phase = 'collapsed' | 'expanding' | 'expanded' | 'collapsing';

export type IslandEvent =
  /** Hover intent confirmed (cursor stayed 250 ms). */
  | 'expand'
  /** The native window finished growing. */
  | 'resized'
  /** Cursor left for 400 ms, or Esc. */
  | 'collapse'
  /** The collapse animation finished. */
  | 'closed';

export const EXPAND_DELAY_MS = 250;
export const COLLAPSE_DELAY_MS = 400;
/** Upper bound on the collapse animation, in case its completion callback never fires. */
export const COLLAPSE_FALLBACK_MS = 700;

export function nextPhase(phase: Phase, event: IslandEvent): Phase {
  switch (phase) {
    case 'collapsed':
      return event === 'expand' ? 'expanding' : phase;
    case 'expanding':
      if (event === 'resized') return 'expanded';
      if (event === 'collapse') return 'collapsing';
      return phase;
    case 'expanded':
      return event === 'collapse' ? 'collapsing' : phase;
    case 'collapsing':
      // The window is still large, so re-opening needs no resize.
      if (event === 'expand') return 'expanded';
      if (event === 'closed') return 'collapsed';
      return phase;
  }
}

/** Whether the dashboard content is open (or opening) for this phase. */
export const isOpen = (phase: Phase) => phase === 'expanded';

/** Whether the dashboard is mounted at all (it unmounts once fully collapsed). */
export const isMounted = (phase: Phase) => phase !== 'collapsed';
