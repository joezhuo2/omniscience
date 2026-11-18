import { invoke, listen } from '../lib/ipc';
import { useCallback, useEffect, useReducer, useRef, useState } from 'react';
import { COLLAPSE_FALLBACK_MS, nextPhase, type Phase } from '../lib/island';
import { useSettingsStore } from '../settings';

/** Emitted by the backend cursor poll (the collapsed window ignores the mouse). */
export const HOVER_EVENT = 'hover';
export const HOVER_DODGE_EVENT = 'hover-dodge';
export const DODGE_MIN_DELAY_MS = 1000;

type Timer = ReturnType<typeof setTimeout> | undefined;

/**
 * Drives the pill <-> dashboard morph. Opens the moment the cursor reaches the pill and starts
 * closing the moment it leaves; re-entering mid-collapse reverses at once. Esc closes.
 * While `pinned` (tray Settings…) it opens without hover and stays open until hovered and left,
 * or Esc. While `sticky` (update prompt) it stays open regardless, until the prompt goes away.
 */
export function useIsland(pinned: boolean, sticky: boolean, onUnpin: () => void, locked = false) {
  const [phase, dispatch] = useReducer(nextPhase, 'collapsed' as Phase);
  const [hovered, setHovered] = useState(false);
  const fallbackTimer = useRef<Timer>(undefined);
  const previous = useRef<Phase>(phase);
  const unpin = useRef(onUnpin);
  const stickyRef = useRef(sticky);
  const lockedRef = useRef(locked);
  const hoveredRef = useRef(false);
  const phaseRef = useRef<Phase>(phase);
  const dodgeRef = useRef(false);
  const openTimer = useRef<Timer>(undefined);
  const closeTimer = useRef<Timer>(undefined);
  const [dodging, setDodging] = useState(false);
  const openDelay = useSettingsStore((s) => s.settings?.hover_open_delay_ms ?? 0);
  const closeDelay = useSettingsStore((s) => s.settings?.hover_close_delay_ms ?? 0);
  const delays = useRef({ openDelay, closeDelay });
  useEffect(() => {
    unpin.current = onUnpin;
    stickyRef.current = sticky;
    lockedRef.current = locked;
    delays.current = { openDelay, closeDelay };
  }, [onUnpin, sticky, locked, openDelay, closeDelay]);
  useEffect(() => {
    phaseRef.current = phase;
  }, [phase]);

  // Backend side effects on phase transitions.
  useEffect(() => {
    const from = previous.current;
    previous.current = phase;
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
  const held = pinned || sticky || locked;
  useEffect(() => {
    if (held) {
      dispatch('expand');
    } else if (!hoveredRef.current) {
      dispatch('collapse');
    }
  }, [held]);

  useEffect(() => {
    const clearTimers = () => {
      clearTimeout(openTimer.current);
      clearTimeout(closeTimer.current);
    };
    const unlistenDodge = listen<boolean>(HOVER_DODGE_EVENT, (e) => {
      dodgeRef.current = e.payload;
      setDodging(e.payload);
    });
    const unlisten = listen<boolean>(HOVER_EVENT, (e) => {
      const inside = e.payload;
      hoveredRef.current = inside;
      setHovered(inside);
      clearTimers();
      if (inside) {
        const { openDelay } = delays.current;
        const wait = dodgeRef.current ? Math.max(openDelay, DODGE_MIN_DELAY_MS) : openDelay;
        if (phaseRef.current === 'collapsed' && wait > 0) {
          openTimer.current = setTimeout(() => dispatch('expand'), wait);
        } else {
          dispatch('expand');
        }
      } else {
        // Leaving after a pinned open releases the pin.
        unpin.current();
        if (!stickyRef.current && !lockedRef.current) {
          const { closeDelay } = delays.current;
          if (closeDelay > 0) {
            closeTimer.current = setTimeout(() => {
              if (!stickyRef.current && !lockedRef.current) dispatch('collapse');
            }, closeDelay);
          } else {
            dispatch('collapse');
          }
        }
      }
    });
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || e.defaultPrevented || stickyRef.current || lockedRef.current)
        return;
      clearTimers();
      unpin.current();
      dispatch('collapse');
    };
    window.addEventListener('keydown', onKey);
    return () => {
      unlisten.then((f) => f()).catch(() => {});
      unlistenDodge.then((f) => f()).catch(() => {});
      window.removeEventListener('keydown', onKey);
      clearTimers();
      clearTimeout(fallbackTimer.current);
    };
  }, []);

  const onClosed = useCallback(() => dispatch('closed'), []);

  return { phase, hovered, dodging, onClosed };
}
