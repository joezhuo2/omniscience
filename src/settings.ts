import { invoke, listen } from './lib/ipc';
import { create } from 'zustand';
import { setUnits } from './lib/format';
import type { Settings } from './types/generated/Settings';

export type { Settings };

export const SETTINGS_EVENT = 'omniscience://settings';
export const OPEN_SETTINGS_EVENT = 'omniscience://open-settings';

interface SettingsState {
  settings: Settings | null;
  error: string | null;
  load: () => Promise<void>;
  /** Applies a change; the backend clamps it and the stored value is what it actually applied. */
  update: (patch: Partial<Settings>) => Promise<void>;
}

function applyUnits(s: Settings): Settings {
  setUnits(s.temp_unit, s.net_unit);
  return s;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  error: null,
  load: async () => {
    try {
      set({ settings: applyUnits(await invoke<Settings>('get_settings')), error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
  update: async (patch) => {
    const current = get().settings;
    if (!current) return;
    try {
      const applied = await invoke<Settings>('set_settings', {
        settings: { ...current, ...patch },
      });
      set({ settings: applyUnits(applied), error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));

/** Keeps the store in sync with changes made from the tray menu. */
export function subscribeSettings(): () => void {
  const unlisten = listen<Settings>(SETTINGS_EVENT, (e) =>
    useSettingsStore.setState({ settings: applyUnits(e.payload) }),
  );
  return () => {
    unlisten.then((f) => f()).catch(() => {});
  };
}
