/**
 * Pill <-> dashboard state machine.
 *
 * - `expanding`: the native window is being snapped to the expanded bounds.
 * - `expanded`: the shape grows to the full panel and content fades in.
 * - `collapsing`: the shape shrinks back to the pill; the window shrinks when that ends.
 */
export type Phase = 'collapsed' | 'expanding' | 'expanded' | 'collapsing';

export type IslandEvent =
  /** Cursor reached the pill, or pinned open. */
  | 'expand'
  /** The native window finished growing. */
  | 'resized'
  /** Cursor left (collapse starts immediately), or Esc. */
  | 'collapse'
  /** The collapse animation finished. */
  | 'closed';

/** Upper bound on the collapse animation (`--shape-ms` plus slack), in case `transitionend` is missed. */
export const COLLAPSE_FALLBACK_MS = 450;

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

/** Whether the shape is open (full panel) for this phase. */
export const isOpen = (phase: Phase) => phase === 'expanded';

/** Whether panel content is mounted (it unmounts once fully collapsed). */
export const isMounted = (phase: Phase) => phase !== 'collapsed';
