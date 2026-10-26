import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useRef, useState } from 'react';
import { Pill } from './components/Pill';
import { UpdateBanner } from './components/UpdateBanner';
import { Dashboard } from './components/dashboard/Dashboard';
import { SettingsView } from './components/settings/SettingsView';
import { useIsland } from './hooks/useIsland';
import { useTelemetry } from './hooks/useTelemetry';
import { isMounted, isOpen } from './lib/island';
import { OPEN_SETTINGS_EVENT, subscribeSettings, type Settings } from './settings';
import { CHECK_UPDATE_EVENT, useUpdaterStore } from './updater';

/** Matches `--fade-ms`: how long the outgoing view takes to fade before the swap. */
const VIEW_FADE_MS = 140;

/** Window height beyond the island: top gap plus a little room, as in islet. */
const WINDOW_EXTRA = 16;

/** Keeps the native window as tall as the open island, so empty space never blocks clicks. */
function useFitWindow(ref: React.RefObject<HTMLDivElement | null>, active: boolean) {
  useEffect(() => {
    const el = ref.current;
    if (!active || !el) return;
    let last = 0;
    const ro = new ResizeObserver(() => {
      const h = Math.ceil(el.getBoundingClientRect().height) + WINDOW_EXTRA;
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
  const banner = useUpdaterStore((s) => s.banner);
  const checkNow = useUpdaterStore((s) => s.checkNow);
  const [settingsPinned, setSettingsPinned] = useState(false);
  const [view, setView] = useState<'dashboard' | 'settings'>('dashboard');
  const [fading, setFading] = useState(false);
  const fadeTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const switchView = (next: 'dashboard' | 'settings') => {
    clearTimeout(fadeTimer.current);
    setFading(true);
    fadeTimer.current = setTimeout(() => {
      setView(next);
      setFading(false);
    }, VIEW_FADE_MS);
  };
  const { phase, onClosed } = useIsland(settingsPinned, banner != null, () =>
    setSettingsPinned(false),
  );
  const open = isOpen(phase);
  const islandRef = useRef<HTMLDivElement>(null);
  useFitWindow(islandRef, open);

  useEffect(() => subscribeSettings(), []);

  // Tray "Settings…" opens straight into the settings view.
  useEffect(() => {
    const unlisten = listen(OPEN_SETTINGS_EVENT, () => {
      setView('settings');
      setSettingsPinned(true);
    });
    return () => {
      unlisten.then((f) => f()).catch(() => {});
    };
  }, []);

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

  const panel = banner ? (
    <UpdateBanner />
  ) : view === 'settings' ? (
    <SettingsView onBack={() => switchView('dashboard')} />
  ) : (
    <Dashboard onSettings={() => switchView('settings')} />
  );

  return (
    <div
      ref={islandRef}
      data-testid="island"
      data-phase={phase}
      className={`island${open ? ' expanded' : ''}`}
      onTransitionEnd={(e) => {
        if (e.propertyName === 'clip-path' && phase === 'collapsing') {
          setView('dashboard');
          onClosed();
        }
      }}
    >
      <Pill />
      {isMounted(phase) && (
        <div className="panel">
          <div className="panel-scroll view-fade" data-fading={fading}>
            {panel}
          </div>
        </div>
      )}
    </div>
  );
}
