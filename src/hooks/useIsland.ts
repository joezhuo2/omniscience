import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { COLLAPSE_FALLBACK_MS, EXPAND_DELAY_MS, nextPhase, type Phase } from '../lib/island';

/** Emitted by the backend cursor poll (the collapsed window ignores the mouse). */
export const HOVER_EVENT = 'hover';

type Timer = ReturnType<typeof setTimeout> | undefined;

/**
 * Drives the pill <-> dashboard morph. Opens 250 ms after the cursor reaches the pill and starts
 * closing the moment it leaves; re-entering mid-collapse reverses at once. Esc closes. While `pinned` (tray Settings…) it
 * opens without hover and stays open until hovered and left, or Esc. While `sticky` (update
 * prompt) it stays open regardless, until the prompt goes away.
 */
export function useIsland(pinned: boolean, sticky: boolean, onUnpin: () => void) {
  const [phase, dispatch] = useReducer(nextPhase, 'collapsed' as Phase);
  const [hovered, setHovered] = useState(false);
  const expandTimer = useRef<Timer>(undefined);
  const fallbackTimer = useRef<Timer>(undefined);
  const previous = useRef<Phase>(phase);
  const phaseRef = useRef<Phase>(phase);
  const unpin = useRef(onUnpin);
  const stickyRef = useRef(sticky);
  const hoveredRef = useRef(false);
  useEffect(() => {
    unpin.current = onUnpin;
    stickyRef.current = sticky;
  }, [onUnpin, sticky]);

  // Backend side effects on phase transitions.
  useEffect(() => {
    const from = previous.current;
    previous.current = phase;
    phaseRef.current = phase;
    if (phase === 'expanding' && from === 'collapsed') {
      invoke('set_expanded', { expanded: true })
        .catch(() => {})
        .finally(() => dispatch('resized'));
    }
    clearTimeout(fallbackTimer.current);
    if (phase === 'collapsing') {
      fallbackTimer.current = setTimeout(() => dispatch('closed'), COLLAPSE_FALLBACK_MS);
    }
    if (phase === 'collapsed' && from === 'collapsing') {
      invoke('set_expanded', { expanded: false }).catch(() => {});
    }
  }, [phase]);

  // Pinned or sticky opens right away; when both are released away from the cursor, close.
  const held = pinned || sticky;
  useEffect(() => {
    if (held) {
      dispatch('expand');
    } else if (!hoveredRef.current) {
      dispatch('collapse');
    }
  }, [held]);

  useEffect(() => {
    const unlisten = listen<boolean>(HOVER_EVENT, (e) => {
      const inside = e.payload;
      hoveredRef.current = inside;
      setHovered(inside);
      clearTimeout(expandTimer.current);
      if (inside) {
        // The window is still large mid-collapse, so reverse without the hover delay.
        if (phaseRef.current === 'collapsing') dispatch('expand');
        else expandTimer.current = setTimeout(() => dispatch('expand'), EXPAND_DELAY_MS);
      } else {
        // Leaving after a pinned open releases the pin.
        unpin.current();
        if (!stickyRef.current) dispatch('collapse');
      }
    });
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || stickyRef.current) return;
      unpin.current();
      dispatch('collapse');
    };
    window.addEventListener('keydown', onKey);
    return () => {
      unlisten.then((f) => f()).catch(() => {});
      window.removeEventListener('keydown', onKey);
      clearTimeout(expandTimer.current);
      clearTimeout(fallbackTimer.current);
    };
  }, []);

  const onClosed = useCallback(() => dispatch('closed'), []);

  return { phase, hovered, onClosed };
}
