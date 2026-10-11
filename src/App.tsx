import { invoke } from '@tauri-apps/api/core';
import { motion } from 'motion/react';
import { useEffect, useRef } from 'react';
import { Pill } from './components/Pill';
import { Dashboard } from './components/dashboard/Dashboard';
import { useIsland } from './hooks/useIsland';
import { useTelemetry } from './hooks/useTelemetry';
import { isOpen } from './lib/island';

const SPRING = { type: 'spring', stiffness: 300, damping: 25 } as const;
const FADE = { duration: 0.18, ease: 'easeOut' } as const;
const PILL_HEIGHT = 42;

/** Keeps the native window as tall as the open dashboard, so empty space never blocks clicks. */
function useFitWindow(ref: React.RefObject<HTMLDivElement | null>, active: boolean) {
  useEffect(() => {
    const el = ref.current;
    if (!active || !el) return;
    let last = 0;
    const ro = new ResizeObserver(() => {
      const h = Math.ceil(el.getBoundingClientRect().height) + 2;
      if (Math.abs(h - last) > 2) {
        last = h;
        invoke('set_overlay_height', { height: h }).catch(() => {});
      }
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [ref, active]);
}

export default function App() {
  useTelemetry();
  const { phase, onPointerEnter, onPointerLeave, onClosed } = useIsland();
  const open = isOpen(phase);
  const contentRef = useRef<HTMLDivElement>(null);
  useFitWindow(contentRef, open);

  return (
    <div className="flex h-full w-full items-start justify-center">
      <motion.div
        data-testid="island"
        data-phase={phase}
        className="glass w-full overflow-hidden"
        initial={false}
        animate={{ height: open ? 'auto' : PILL_HEIGHT, borderRadius: open ? 18 : 21 }}
        transition={SPRING}
        onAnimationComplete={() => {
          if (phase === 'collapsing') onClosed();
        }}
        onPointerEnter={onPointerEnter}
        onPointerLeave={onPointerLeave}
      >
        <div ref={contentRef}>
          {open ? (
            <motion.div
              key="dashboard"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={FADE}
            >
              <Dashboard />
            </motion.div>
          ) : (
            <motion.div
              key="pill"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={FADE}
            >
              <Pill />
            </motion.div>
          )}
        </div>
      </motion.div>
    </div>
  );
}
