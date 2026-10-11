import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { motion } from 'motion/react';
import { useEffect, useRef, useState } from 'react';
import { Pill } from './components/Pill';
import { UpdateBanner } from './components/UpdateBanner';
import { Dashboard } from './components/dashboard/Dashboard';
import { SettingsView } from './components/settings/SettingsView';
import { useIsland } from './hooks/useIsland';
import { useTelemetry } from './hooks/useTelemetry';
import { isOpen } from './lib/island';
import { OPEN_SETTINGS_EVENT, subscribeSettings, type Settings } from './settings';
import { CHECK_UPDATE_EVENT, useUpdaterStore } from './updater';

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
  const island = useIsland();
  const { phase, onPointerEnter, onPointerLeave, onClosed } = island;
  const open = isOpen(phase);
  const contentRef = useRef<HTMLDivElement>(null);
  const [view, setView] = useState<'dashboard' | 'settings'>('dashboard');
  const banner = useUpdaterStore((s) => s.banner);
  const checkNow = useUpdaterStore((s) => s.checkNow);
  // The collapsed pill grows a strip for the update banner.
  const tall = open || (banner != null && phase === 'collapsed');
  useFitWindow(contentRef, tall);

  // Back to the bare pill once the banner goes away.
  useEffect(() => {
    if (phase === 'collapsed' && !banner) {
      invoke('set_overlay_height', { height: PILL_HEIGHT }).catch(() => {});
    }
  }, [phase, banner]);

  // Launch-time update check (if enabled) and tray "Check for updates now".
  useEffect(() => {
    invoke<Settings>('get_settings')
      .then((s) => {
        if (s.auto_check_updates) void checkNow(false);
      })
      .catch(() => {});
    const unlisten = listen(CHECK_UPDATE_EVENT, () => void checkNow(true));
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [checkNow]);

  useEffect(() => subscribeSettings(), []);

  // Tray "Settings…" opens the settings view directly.
  const openIsland = island.open;
  useEffect(() => {
    const unlisten = listen(OPEN_SETTINGS_EVENT, () => {
      setView('settings');
      openIsland();
    });
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, [openIsland]);

  return (
    <div className="flex h-full w-full items-start justify-center">
      <motion.div
        data-testid="island"
        data-phase={phase}
        className="glass w-full overflow-hidden"
        initial={false}
        animate={{ height: tall ? 'auto' : PILL_HEIGHT, borderRadius: open ? 18 : 21 }}
        transition={SPRING}
        onAnimationComplete={() => {
          if (phase === 'collapsing') onClosed();
        }}
        onPointerEnter={() => {
          // While the update banner asks Yes/No, hovering must not expand over it.
          if (phase === 'collapsed' && banner) return;
          // Every fresh hover-open starts on the dashboard.
          if (phase === 'collapsed') setView('dashboard');
          onPointerEnter();
        }}
        onPointerLeave={onPointerLeave}
      >
        <div ref={contentRef}>
          {open ? (
            <motion.div
              key={view}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={FADE}
            >
              {view === 'settings' ? (
                <SettingsView onBack={() => setView('dashboard')} />
              ) : (
                <Dashboard onSettings={() => setView('settings')} />
              )}
            </motion.div>
          ) : (
            <motion.div
              key="pill"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={FADE}
            >
              <Pill />
              <UpdateBanner />
            </motion.div>
          )}
        </div>
      </motion.div>
    </div>
  );
}
