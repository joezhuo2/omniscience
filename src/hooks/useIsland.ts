import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useReducer, useRef } from 'react';
import {
  COLLAPSE_DELAY_MS,
  COLLAPSE_FALLBACK_MS,
  EXPAND_DELAY_MS,
  nextPhase,
  type Phase,
} from '../lib/island';

type Timer = ReturnType<typeof setTimeout> | undefined;

/**
 * Drives the pill <-> dashboard morph from hover: expand after 250 ms of hover, collapse 400 ms
 * after the cursor leaves (re-entering cancels), Esc collapses. Talks to the backend to snap the
 * native window before expanding and shrink it after collapsing.
 */
export function useIsland() {
  const [phase, dispatch] = useReducer(nextPhase, 'collapsed' as Phase);
  const expandTimer = useRef<Timer>(undefined);
  const collapseTimer = useRef<Timer>(undefined);
  const fallbackTimer = useRef<Timer>(undefined);
  const previous = useRef<Phase>(phase);

  // Side effects on phase transitions.
  useEffect(() => {
    const from = previous.current;
    previous.current = phase;
    if (phase === 'expanding' && from === 'collapsed') {
      invoke('set_expanded', { expanded: true })
        .catch(() => {})
        .finally(() => dispatch('resized'));
    }
    if (phase === 'collapsing') {
      clearTimeout(fallbackTimer.current);
      fallbackTimer.current = setTimeout(() => dispatch('closed'), COLLAPSE_FALLBACK_MS);
    } else {
      clearTimeout(fallbackTimer.current);
    }
    if (phase === 'collapsed' && from === 'collapsing') {
      invoke('set_expanded', { expanded: false }).catch(() => {});
    }
  }, [phase]);

  useEffect(
    () => () => {
      clearTimeout(expandTimer.current);
      clearTimeout(collapseTimer.current);
      clearTimeout(fallbackTimer.current);
    },
    [],
  );

  const onPointerEnter = useCallback(() => {
    clearTimeout(collapseTimer.current);
    clearTimeout(expandTimer.current);
    expandTimer.current = setTimeout(() => dispatch('expand'), EXPAND_DELAY_MS);
  }, []);

  const onPointerLeave = useCallback(() => {
    clearTimeout(expandTimer.current);
    clearTimeout(collapseTimer.current);
    collapseTimer.current = setTimeout(() => dispatch('collapse'), COLLAPSE_DELAY_MS);
  }, []);

  const onClosed = useCallback(() => dispatch('closed'), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') dispatch('collapse');
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  return { phase, onPointerEnter, onPointerLeave, onClosed };
}
