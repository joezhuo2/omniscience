import { invoke, isEmbedded, listen } from './lib/ipc';
import { useEffect, useRef, useState, type CSSProperties } from 'react';
import { GlassIsland } from '@crystalflux/island-glass';
import { Pill } from './components/Pill';
import { useIslandImage } from './lib/islandImage';
import { ScrollArea } from './components/ScrollArea';
import { useSmoothScroll } from '@crystalflux/smooth-scroll';
import { UpdateBanner } from './components/UpdateBanner';
import { Dashboard } from './components/dashboard/Dashboard';
import { SettingsView } from './components/settings/SettingsView';
import { StyleEditor } from './components/StyleEditor';
import { useIsland } from './hooks/useIsland';
import { useTelemetry } from './hooks/useTelemetry';
import { isMounted, isOpen } from './lib/island';
import {
  OPEN_SETTINGS_EVENT,
  subscribeSettings,
  useSettingsStore,
  type Settings,
} from './settings';
import { CHECK_UPDATE_EVENT, useUpdaterStore } from './updater';
import { OPEN_SYSTEM_UPDATES_EVENT } from './systemUpdates';
import { saveSectionCollapsed } from './lib/sections';

/** Matches `--fade-ms`: how long the outgoing view takes to fade before the swap. */
const VIEW_FADE_MS = 140;

type View = 'dashboard' | 'settings' | 'style';

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

function EmbeddedApp() {
  useTelemetry();
  useSmoothScroll();
  const [view, setView] = useState<View>('dashboard');

  useEffect(() => {
    void useSettingsStore.getState().load();
  }, []);

  // Detail-tier sampling only runs while the host shows this page.
  useEffect(() => {
    const expand = (expanded: boolean) => void invoke('set_expanded', { expanded }).catch(() => {});
    const sync = () => expand(document.visibilityState === 'visible');
    const hide = () => expand(false);
    sync();
    document.addEventListener('visibilitychange', sync);
    window.addEventListener('pagehide', hide);
    return () => {
      document.removeEventListener('visibilitychange', sync);
      window.removeEventListener('pagehide', hide);
      hide();
    };
  }, []);

  return (
    <main data-testid="embedded" className="mx-auto max-w-[1600px] p-4">
      {view === 'dashboard' ? (
        <Dashboard onSettings={() => setView('settings')} onStyle={() => setView('style')} />
      ) : (
        <div className="mx-auto max-w-xl">
          {view === 'settings' ? (
            <SettingsView onBack={() => setView('dashboard')} />
          ) : (
            <StyleEditor onBack={() => setView('dashboard')} />
          )}
        </div>
      )}
    </main>
  );
}

export default function App() {
  return isEmbedded() ? <EmbeddedApp /> : <OverlayApp />;
}

function OverlayApp() {
  useTelemetry();
  const banner = useUpdaterStore((s) => s.banner);
  const checkNow = useUpdaterStore((s) => s.checkNow);
  const [settingsPinned, setSettingsPinned] = useState(false);
  const [systemUpdatesRequest, setSystemUpdatesRequest] = useState(0);
  const [view, setView] = useState<View>('dashboard');
  const [fading, setFading] = useState(false);
  const fadeTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const switchView = (next: View) => {
    clearTimeout(fadeTimer.current);
    setFading(true);
    fadeTimer.current = setTimeout(() => {
      setView(next);
      setFading(false);
    }, VIEW_FADE_MS);
  };
  const [dashboardPinned, setDashboardPinned] = useState(false);
  const { phase, onClosed } = useIsland(
    settingsPinned,
    banner != null,
    () => setSettingsPinned(false),
    dashboardPinned,
  );
  const open = isOpen(phase);
  const image = useIslandImage((s) => s.image);
  const islandRef = useRef<HTMLDivElement>(null);
  useFitWindow(islandRef, open);

  useEffect(() => {
    void useSettingsStore.getState().load();
    return subscribeSettings();
  }, []);

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

  useEffect(() => {
    const unlisten = listen(OPEN_SYSTEM_UPDATES_EVENT, () => {
      clearTimeout(fadeTimer.current);
      setFading(false);
      saveSectionCollapsed('driver-updates', false);
      setSystemUpdatesRequest((request) => request + 1);
      setView('dashboard');
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
  ) : view === 'style' ? (
    <StyleEditor onBack={() => switchView('dashboard')} />
  ) : (
    <Dashboard
      key={systemUpdatesRequest}
      showSystemUpdates={systemUpdatesRequest}
      onSettings={() => switchView('settings')}
      onStyle={() => switchView('style')}
      pinned={dashboardPinned}
      onTogglePin={() => setDashboardPinned((p) => !p)}
    />
  );

  return (
    <div
      ref={islandRef}
      data-testid="island"
      data-phase={phase}
      className={`island-host${open ? ' expanded' : ''}`}
    >
      <GlassIsland
        open={open}
        onOpenChange={() => {}}
        pill={<Pill />}
        radius={22}
        className="island-shell"
        style={image ? ({ '--island-image': `url(${image})` } as CSSProperties) : undefined}
        onTransitionEnd={(e) => {
          if (e.propertyName === '--t' && phase === 'collapsing') {
            setView('dashboard');
            onClosed();
          }
        }}
      >
        {isMounted(phase) && (
          <div className="panel">
            <ScrollArea className="panel-scroll view-fade" data-fading={fading}>
              {panel}
            </ScrollArea>
          </div>
        )}
      </GlassIsland>
    </div>
  );
}
